# anawt ![License: MIT](https://img.shields.io/badge/license-MIT-blue) [![anawt on crates.io](https://img.shields.io/crates/v/anawt)](https://crates.io/crates/anawt) [![anawt on docs.rs](https://docs.rs/anawt/badge.svg)](https://docs.rs/anawt) [![Source Code Repository](https://img.shields.io/badge/Code-On%20GitHub-blue?logo=GitHub)](https://github.com/pliduino/anawt)

## Anawt

Easy to use torrent client built on top of [libtorrent][__link0] and [tokio][__link1].

### Examples

```rust

use anawt::{TorrentClient, options::AnawtOptions, TorrentState};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let client = TorrentClient::create(AnawtOptions::default());
    let magnet_link = "magnet:?xt=urn:btih:BDBTHK7HMT762DE7RQX7EHPF47TIVID3&dn=nixos-minimal-25.11.650.8bb5646e0bed-x86_64-linux.iso&xl=1584496640";
    let torrent_hash = client.add_magnet(magnet_link, "./downloads").await.unwrap();


    loop {
        // You can also use client.subscribe_torrents() to get torrent updates more efficiently
        let status = client.get_status(torrent_hash).await.unwrap();
        if status.state == TorrentState::Seeding || status.state == TorrentState::Finished {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }
}
```


 [__link0]: https://github.com/arvidn/libtorrent
 [__link1]: https://tokio.rs/
