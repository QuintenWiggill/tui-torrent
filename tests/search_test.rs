use tui_torrent::torrent_search::TorrentSearchEngine;
use tui_torrent::api::{PirateBayClient, YtsClient};

#[tokio::test]
async fn test_real_search() {
    let engine = TorrentSearchEngine::new();
    println!("Searching for 'ubuntu'...");
    match engine.search_torrents("ubuntu", None).await {
        Ok(results) => {
            println!("Found {} total results:", results.len());
            for r in results.iter().take(5) {
                println!("- [{}] {} ({}) - S:{} L:{}", r.source, r.name, r.size, r.seeders, r.leechers);
            }
            assert!(!results.is_empty(), "Results list should not be empty");
        }
        Err(e) => {
            panic!("Search engine failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_piratebay() {
    let client = PirateBayClient::new();
    match client.search("ubuntu", None).await {
        Ok(res) => {
            println!("PirateBay found {} results", res.len());
            assert!(!res.is_empty(), "PirateBay should return results");
        }
        Err(e) => {
            panic!("PirateBay failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_yts() {
    let client = YtsClient::new();
    match client.search("matrix", Some(5)).await {
        Ok(res) => {
            println!("YTS found {} results", res.len());
            assert!(!res.is_empty(), "YTS should return results");
        }
        Err(e) => {
            panic!("YTS failed: {}", e);
        }
    }
}
