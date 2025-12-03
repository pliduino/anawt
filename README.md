# Anawt

Easy to use torrent client built on top of [libtorrent](https://github.com/arvidn/libtorrent) and [tokio](https://tokio.rs/).

## Examples

```rust

use anawt::{TorrentClient, options::AnawtOptions, TorrentState};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let client = TorrentClient::create(AnawtOptions::default());
    let magnet_link = "magnet:?xt=urn:btih:BDBTHK7HMT762DE7RQX7EHPF47TIVID3&dn=nixos-minimal-25.11.650.8bb5646e0bed-x86_64-linux.iso&xl=1584496640";
    let torrent_hash = client.add_torrent(magnet_link, "./downloads").await.unwrap();
    
    
    loop {
        // You can also use client.subscribe_torrents() to get torrent updates more efficiently
        let status = client.get_torrent_status(torrent_hash).await.unwrap();
        if status.state == TorrentState::Seeding || status.state == TorrentState::Finished {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
