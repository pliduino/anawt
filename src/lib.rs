//! Easy to use torrent client built on top of [libtorrent](https://github.com/arvidn/libtorrent) and [tokio](https://tokio.rs/).
//!
//! ## Examples
//!
//! ```no_run
//!
//! use anawt::{TorrentClient, options::AnawtOptions, TorrentState};
//! use tokio::time::{sleep, Duration};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = TorrentClient::create(AnawtOptions::default());
//!     let magnet_link = "magnet:?xt=urn:btih:BDBTHK7HMT762DE7RQX7EHPF47TIVID3&dn=nixos-minimal-25.11.650.8bb5646e0bed-x86_64-linux.iso&xl=1584496640";
//!     let torrent_hash = client.add_magnet(magnet_link, "./downloads").await.unwrap();
//!
//!
//!     loop {
//!         // You can also use client.subscribe_torrents() to get torrent updates more efficiently
//!         let status = client.get_status(torrent_hash).await.unwrap();
//!         if status.state == TorrentState::Seeding || status.state == TorrentState::Finished {
//!             break;
//!         }
//!         sleep(Duration::from_secs(1)).await;
//!     }
//! }
//! ```
pub use lt_rs::alerts::AlertCategory;
pub use lt_rs::settings_pack::SettingsPack;
pub mod errors;

mod client;
pub mod options;
mod torrent_entry;
pub use client::TorrentClient;
pub use lt_rs::alerts::TorrentState;
pub use lt_rs::info_hash::InfoHash;
pub use torrent_entry::AnawtTorrentStatus;
