use std::{
    collections::{HashMap, VecDeque},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use lt_rs::{
    add_torrent_params::AddTorrentParams,
    alerts::{
        AddTorrentAlert, Alert, SaveResumeDataAlert, SaveResumeDataFailedAlert, StateChangedAlert,
        StateUpdateAlert, TorrentAlert, TorrentFinishedAlert, TorrentState,
    },
    info_hash::InfoHash,
    session::LtSession,
    settings_pack::SettingsPack,
    torrent_handle::{ResumeDataFlags, StatusFlags, TorrentHandle},
    torrent_status::TorrentStatus,
};
use rclite::Arc;
use tokio::sync::{
    mpsc,
    oneshot::{self},
    watch,
};
use tracing::{error, info};

use crate::torrent_entry::{AnawtTorrentStatus, TorrentEntry};

struct TorrentClientInner {
    session: LtSession,
    torrents: HashMap<InfoHash, TorrentEntry>,
    pending_added_torrents: VecDeque<oneshot::Sender<()>>,

    saving_sender: Option<oneshot::Sender<Result<(), ()>>>,
    pending_saves: u32,
    save_path: String,
}

#[derive(Debug)]
enum ClientMessage {
    AddTorrent(AddTorrentParams),
    GetState(InfoHash, oneshot::Sender<Option<AnawtTorrentStatus>>),
    SubscribeTorrent(
        InfoHash,
        oneshot::Sender<Option<watch::Receiver<AnawtTorrentStatus>>>,
    ),
    Save(oneshot::Sender<Result<(), ()>>),
}

/// Torrent client that communicates with the main libtorrent thread
///
/// ### Note
/// Cheap clone
#[derive(Debug, Clone)]
pub struct TorrentClient {
    tx: mpsc::Sender<ClientMessage>,
    _handle_ref: Arc<()>, // Keeps loop alive until client is fully dropped
}

impl TorrentClient {
    pub fn create(settings: SettingsPack) -> TorrentClient {
        let (tx, mut rx) = mpsc::channel(100);

        let _handle_ref = Arc::new(());
        let _handle = _handle_ref.clone();

        tokio::spawn(async move {
            let mut client = TorrentClientInner::new(&settings, "./data/torrents".to_string());

            client.load_torrents();

            let mut last_update = Instant::now();

            info!("Torrent client started");
            loop {
                // Stops the loop when the client is dropped
                if _handle.strong_count() == 1 {
                    break;
                }

                if last_update.elapsed() > Duration::from_millis(500) {
                    client.post_torrent_updates(StatusFlags::all());
                    last_update = Instant::now();
                }

                while let Ok(msg) = rx.try_recv() {
                    match msg {
                        ClientMessage::AddTorrent(ref params) => client.add_torrent(params),
                        ClientMessage::GetState(info_hash, tx) => {
                            if let Some(entry) = client.torrents.get(&info_hash) {
                                tx.send(Some(entry.status.borrow().clone())).unwrap();
                            } else {
                                tx.send(None).unwrap();
                            }
                        }
                        ClientMessage::SubscribeTorrent(info_hash, tx) => {
                            tx.send(client.subscribe_torrent(info_hash)).unwrap();
                        }
                        ClientMessage::Save(tx) => {
                            if client.pending_saves > 0 {
                                tx.send(Err(())).unwrap();
                                continue;
                            }
                            client.saving_sender = Some(tx);
                            client.save_resume_data();
                        }
                    }
                }

                client.process_alerts();
                tokio::task::yield_now().await;
            }

            info!("Torrent client stopped");
        });

        TorrentClient { _handle_ref, tx }
    }

    pub async fn add_magnet(&self, magnet: &str, path: &str) -> Result<InfoHash, ()> {
        let mut params = AddTorrentParams::parse_magnet_uri(magnet);
        params.set_path(path);
        let info_hash = params.get_info_hash();

        self.tx
            .send(ClientMessage::AddTorrent(params))
            .await
            .unwrap();

        Ok(info_hash)
    }

    pub async fn save(&self) -> Result<(), ()> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(ClientMessage::Save(tx)).await.unwrap();
        rx.await.unwrap()
    }

    pub async fn get_status(&self, info_hash: InfoHash) -> Option<AnawtTorrentStatus> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ClientMessage::GetState(info_hash, tx))
            .await
            .unwrap();

        match rx.await {
            Ok(status) => status,
            Err(_) => None,
        }
    }

    pub async fn subscribe_torrent(
        &self,
        info_hash: InfoHash,
    ) -> Option<watch::Receiver<AnawtTorrentStatus>> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ClientMessage::SubscribeTorrent(info_hash, tx))
            .await
            .unwrap();

        rx.await.unwrap()
    }
}

impl TorrentClientInner {
    pub fn new(settings: &SettingsPack, save_path: String) -> TorrentClientInner {
        if !Path::new(&save_path).exists() {
            fs::create_dir_all(&save_path).unwrap();
        }

        TorrentClientInner {
            session: LtSession::new_with_settings(settings),
            torrents: HashMap::new(),
            pending_added_torrents: VecDeque::new(),

            saving_sender: None,
            pending_saves: 0,
            save_path,
        }
    }

    pub fn add_torrent(&mut self, params: &AddTorrentParams) {
        self.session.async_add_torrent(&params);

        let info_hash = params.get_info_hash();

        info!("Added torrent: {}", info_hash.as_base64());

        let (status, _) = tokio::sync::watch::channel(AnawtTorrentStatus {
            state: TorrentState::CheckingFiles,
            progress: 0.0,
        });

        self.torrents.insert(
            info_hash,
            TorrentEntry {
                handle: None,
                status,
            },
        );
    }

    pub fn save_resume_data(&mut self) {
        self.pending_saves = self.torrents.len() as u32;
        for entry in self.torrents.values() {
            if let Some(handle) = &entry.handle {
                handle.save_resume_data(ResumeDataFlags::SaveInfoDict);
            }
        }
    }

    pub fn load_torrents(&mut self) {
        let save_path = PathBuf::from(&self.save_path);
        let dir = fs::read_dir(save_path).unwrap();

        for entry in dir {
            let entry = entry.unwrap();
            let mut file = File::open(entry.path()).unwrap();
            let mut buffer = vec![];
            file.read_to_end(&mut buffer).unwrap();

            let params = AddTorrentParams::load_resume_data(&buffer);
            info!("Loading: {}", entry.path().display(),);
            self.add_torrent(&params);
        }
    }

    pub fn subscribe_torrent(
        &mut self,
        hash: InfoHash,
    ) -> Option<watch::Receiver<AnawtTorrentStatus>> {
        if let Some(entry) = self.torrents.get_mut(&hash) {
            Some(entry.status.subscribe())
        } else {
            None
        }
    }

    fn process_alerts(&mut self) {
        self.session.pop_alerts();

        // SAFETY: This is safe because alerts will drop before the next pop
        let alerts = unsafe { self.session.take_alerts() };

        for alert in alerts.iter() {
            match alert {
                Alert::TorrentAlert(alert) => self.handle_torrent_alerts(alert),
                Alert::StateUpdate(alert) => self.handle_state_update(alert),
                _ => (),
            }
        }
    }

    fn post_torrent_updates(&mut self, flags: StatusFlags) {
        self.session.post_torrent_updates(flags);
    }

    fn handle_state_update(&mut self, alert: &StateUpdateAlert) {
        let status = alert.status();
        for status in status.iter() {
            let info_hash = status.handle().info_hashes();
            if let Some(entry) = self.torrents.get_mut(&info_hash) {
                entry.status.send_if_modified(|s| {
                    if s.state == status.state() && s.progress == status.progress() {
                        return false;
                    }
                    s.state = status.state();
                    s.progress = status.progress();
                    true
                });
            }
        }
    }

    // ╔===========================================================================╗
    // ║                              Torrent Alerts                               ║
    // ╚===========================================================================╝

    fn handle_torrent_alerts(&mut self, alert: &TorrentAlert) {
        match alert {
            TorrentAlert::TorrentFinished(alert) => self.handle_torrent_finished(alert),
            TorrentAlert::AddTorrent(alert) => self.handle_add_torrent(alert),
            TorrentAlert::StateChanged(alert) => self.handle_state_changed(alert),
            TorrentAlert::SaveResumeData(alert) => self.handle_save_resume_data(alert),
            TorrentAlert::SaveResumeDataFailed(alert) => self.handle_save_resume_data_failed(alert),
            _ => (),
        };
    }

    fn handle_torrent_finished(&mut self, alert: &TorrentFinishedAlert) {
        // info!(
        //     "Finished torrent: {}",
        //     alert.handle().info_hash().as_base64()
        // );
    }

    fn handle_add_torrent(&mut self, alert: &AddTorrentAlert) {
        let error = alert.error();
        if !error.is_ok() {
            error!("Error adding torrent: {:?}", error);
        }

        let handle = alert.handle();

        if let Some(entry) = self.torrents.get_mut(&handle.info_hashes()) {
            entry.handle = Some(handle);
        }

        if let Some(tx) = self.pending_added_torrents.pop_front() {
            tx.send(()).unwrap();
        }
    }

    fn handle_state_changed(&mut self, alert: &StateChangedAlert) {
        // if let Some(entry) = self.torrents.get_mut(&handle.info_hash()) {
        //     entry.status.state = state;
        // }
    }

    fn handle_save_resume_data(&mut self, alert: &SaveResumeDataAlert) {
        self.pending_saves -= 1;
        // TODO: Get and save params
        let params = alert.params();

        let buf = params.write_resume_data_buf();

        info!("Saving: ");

        fs::write(
            self.save_path.clone() + "/" + &params.get_info_hash().as_base64(),
            buf,
        )
        .unwrap();

        if self.pending_saves == 0 {
            self.saving_sender.take().unwrap().send(Ok(())).unwrap();
        }
    }

    fn handle_save_resume_data_failed(&mut self, alert: &SaveResumeDataFailedAlert) {
        self.pending_saves -= 1;
        error!("Error saving resume data: {:?}", alert.error());
    }

    // ===========================  End Torrent Alerts  ============================
}
