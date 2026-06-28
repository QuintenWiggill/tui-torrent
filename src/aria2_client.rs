use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Clone, Deserialize)]
pub struct BittorrentInfo {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Bittorrent {
    pub info: Option<BittorrentInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TorrentStatus {
    pub gid: String,
    pub status: String,
    #[serde(rename = "totalLength")]
    pub total_length: String,
    #[serde(rename = "completedLength")]
    pub completed_length: String,
    #[serde(rename = "downloadSpeed")]
    pub download_speed: String,
    #[serde(rename = "infoHash")]
    pub info_hash: Option<String>,
    pub bittorrent: Option<Bittorrent>,
}

pub async fn get_all_downloads() -> Result<Vec<TorrentStatus>, reqwest::Error> {
    let client = reqwest::Client::new();
    
    // 1. Get active downloads
    let payload_active = json!({
        "jsonrpc": "2.0",
        "method": "aria2.tellActive",
        "id": "active"
    });
    
    let res_active = client
        .post("http://localhost:6800/jsonrpc")
        .json(&payload_active)
        .send()
        .await?;
        
    let json_active: serde_json::Value = res_active.json().await?;
    let mut torrents: Vec<TorrentStatus> = serde_json::from_value(json_active["result"].clone()).unwrap_or(vec![]);
    
    // 2. Get waiting/paused downloads (offset: 0, num: 1000)
    let payload_waiting = json!({
        "jsonrpc": "2.0",
        "method": "aria2.tellWaiting",
        "params": [0, 1000],
        "id": "waiting"
    });
    
    let res_waiting = client
        .post("http://localhost:6800/jsonrpc")
        .json(&payload_waiting)
        .send()
        .await?;
        
    let json_waiting: serde_json::Value = res_waiting.json().await?;
    let waiting_torrents: Vec<TorrentStatus> = serde_json::from_value(json_waiting["result"].clone()).unwrap_or(vec![]);
    
    torrents.extend(waiting_torrents);
    Ok(torrents)
}

pub async fn pause_download(gid: &str) -> Result<Result<String, String>, reqwest::Error> {
    let client = reqwest::Client::new();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "aria2.pause",
        "params": [gid],
        "id": "pause"
    });

    let res = client
        .post("http://localhost:6800/jsonrpc")
        .json(&payload)
        .send()
        .await?;

    let json: serde_json::Value = res.json().await?;
    if let Some(err) = json.get("error") {
        let msg = err["message"].as_str().unwrap_or("Unknown error").to_string();
        Ok(Err(msg))
    } else {
        let result = json["result"].as_str().unwrap_or("").to_string();
        Ok(Ok(result))
    }
}

pub async fn resume_download(gid: &str) -> Result<Result<String, String>, reqwest::Error> {
    let client = reqwest::Client::new();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "aria2.unpause",
        "params": [gid],
        "id": "unpause"
    });

    let res = client
        .post("http://localhost:6800/jsonrpc")
        .json(&payload)
        .send()
        .await?;

    let json: serde_json::Value = res.json().await?;
    if let Some(err) = json.get("error") {
        let msg = err["message"].as_str().unwrap_or("Unknown error").to_string();
        Ok(Err(msg))
    } else {
        let result = json["result"].as_str().unwrap_or("").to_string();
        Ok(Ok(result))
    }
}
