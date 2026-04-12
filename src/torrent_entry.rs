use lt_rs::{alerts::TorrentState, info_hash::InfoHash, torrent_handle::TorrentHandle};

#[derive(Debug, Clone)]
pub struct AnawtTorrentStatus {
    pub name: String,
    pub download_rate: i32,
    pub upload_rate: i32,
    pub bytes_uploaded: i64,
    pub bytes_downloaded: i64,
    pub total_bytes: i64,
    pub save_path: String,
    pub state: TorrentState,
    pub progress: f64,
}

impl Default for AnawtTorrentStatus {
    fn default() -> Self {
        Self {
            name: Default::default(),
            state: TorrentState::CheckingFiles,
            download_rate: Default::default(),
            upload_rate: Default::default(),
            bytes_uploaded: Default::default(),
            bytes_downloaded: Default::default(),
            total_bytes: Default::default(),
            save_path: Default::default(),
            progress: Default::default(),
        }
    }
}

pub struct TorrentEntry {
    pub info_hash: InfoHash,
    pub handle: Option<TorrentHandle>,
    pub status: tokio::sync::watch::Sender<AnawtTorrentStatus>,
}

impl PartialEq for TorrentEntry {
    fn eq(&self, other: &Self) -> bool {
        self.info_hash == other.info_hash
    }
}

impl Eq for TorrentEntry {}

impl PartialOrd for TorrentEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for TorrentEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.info_hash.cmp(&other.info_hash)
    }
}
