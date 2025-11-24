use lt_rs::{alerts::TorrentState, torrent_handle::TorrentHandle};

#[derive(Debug, Clone)]
pub struct AnawtTorrentStatus {
    pub state: TorrentState,
    pub progress: f64,
}

pub struct TorrentEntry {
    pub handle: Option<TorrentHandle>,
    pub status: tokio::sync::watch::Sender<AnawtTorrentStatus>,
}
