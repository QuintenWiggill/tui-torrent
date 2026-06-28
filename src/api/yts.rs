use crate::torrent_search::TorrentSearchResult;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct YtsClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct YtsResponse {
    status: String,
    data: YtsData,
}

#[derive(Debug, Deserialize)]
struct YtsData {
    #[serde(default)]
    movies: Vec<YtsMovie>,
}

#[derive(Debug, Deserialize)]
struct YtsMovie {
    title: String,
    year: u32,
    #[allow(dead_code)]
    rating: f32,
    #[allow(dead_code)]
    genres: Vec<String>,
    #[allow(dead_code)]
    summary: String,
    #[serde(default)]
    torrents: Vec<YtsTorrent>,
}

#[derive(Debug, Deserialize)]
struct YtsTorrent {
    #[allow(dead_code)]
    url: String,
    hash: String,
    quality: String,
    #[serde(rename = "type")]
    codec: String,
    seeds: u32,
    peers: u32,
    size: String,
    #[allow(dead_code)]
    #[serde(rename = "date_uploaded")]
    date_uploaded: String,
}

impl YtsClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("TUI-Torrent/1.0")
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: "https://yts.gg/api/v2".to_string(),
        }
    }

    pub async fn search(&self, query: &str, limit: Option<u32>) -> Result<Vec<TorrentSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let limit = limit.unwrap_or(20);
        let search_url = format!(
            "{}/list_movies.json?query_term={}&limit={}",
            self.base_url, 
            urlencoding::encode(query),
            limit
        );

        let response = self.client
            .get(&search_url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let yts_response: YtsResponse = response.json().await?;
        
        if yts_response.status != "ok" {
            return Err("YTS API returned error status".into());
        }

        let mut results = Vec::new();

        for movie in yts_response.data.movies {
            for torrent in movie.torrents {
                let magnet_link = format!(
                    "magnet:?xt=urn:btih:{}&dn={}&tr=udp://open.demonii.com:1337/announce&tr=udp://tracker.openbittorrent.com:80&tr=udp://tracker.coppersurfer.tk:6969&tr=udp://glotorrents.pw:6969/announce&tr=udp://tracker.opentrackr.org:1337/announce&tr=udp://torrent.gresille.org:80/announce&tr=udp://p4p.arenabg.com:1337&tr=udp://tracker.leechers-paradise.org:6969",
                    torrent.hash,
                    urlencoding::encode(&format!("{} ({}) [{}] [{}]", movie.title, movie.year, torrent.quality, torrent.codec))
                );

                results.push(TorrentSearchResult {
                    name: format!("{} ({}) [{}] [{}]", movie.title, movie.year, torrent.quality, torrent.codec),
                    size: torrent.size,
                    seeders: torrent.seeds,
                    leechers: torrent.peers,
                    magnet_link,
                    source: "YTS".to_string(),
                });
            }
        }

        // Sort by seeders (descending)
        results.sort_by(|a, b| b.seeders.cmp(&a.seeders));

        Ok(results)
    }
}

impl Default for YtsClient {
    fn default() -> Self {
        Self::new()
    }
}