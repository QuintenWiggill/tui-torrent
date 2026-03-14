use crate::types::{DownloadHistoryEntry, DownloadStatus};
use serde_json;
use std::fs;
use std::path::PathBuf;
use std::io::Result as IoResult;

pub struct HistoryManager {
    history_file: PathBuf,
}

impl HistoryManager {
    pub fn new() -> Self {
        let mut history_file = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        history_file.push(".tui_torrent_history.json");
        
        Self { history_file }
    }

    pub fn add_download(&self, entry: DownloadHistoryEntry) -> IoResult<()> {
        let mut history = self.load_history().unwrap_or_default();
        history.push(entry);
        
        // Keep only last 100 entries
        if history.len() > 100 {
            history.drain(0..history.len() - 100);
        }
        
        self.save_history(&history)
    }

    pub fn load_history(&self) -> IoResult<Vec<DownloadHistoryEntry>> {
        if !self.history_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.history_file)?;
        let history: Vec<DownloadHistoryEntry> = serde_json::from_str(&content)
            .unwrap_or_default();
        
        Ok(history)
    }

    pub fn update_download_status(&self, gid: &str, status: DownloadStatus) -> IoResult<()> {
        let mut history = self.load_history().unwrap_or_default();
        
        if let Some(entry) = history.iter_mut().find(|e| e.gid == gid) {
            entry.status = status;
            self.save_history(&history)?;
        }
        
        Ok(())
    }

    pub fn save_history(&self, history: &[DownloadHistoryEntry]) -> IoResult<()> {
        let content = serde_json::to_string_pretty(history)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&self.history_file, content)
    }

    pub fn clear_history(&self) -> IoResult<()> {
        if self.history_file.exists() {
            fs::remove_file(&self.history_file)?;
        }
        Ok(())
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}