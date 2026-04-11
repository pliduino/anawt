use lt_rs::{alerts::TorrentState, info_hash::InfoHash, torrent_handle::TorrentHandle};

#[derive(Debug, Clone)]
pub struct AnawtTorrentStatus {
    pub name: String,
    pub state: TorrentState,
    pub progress: f64,
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
