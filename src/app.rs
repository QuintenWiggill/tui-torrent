use crate::aria2_client::TorrentStatus;
use crate::torrent_search::TorrentSearchResult;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::io;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Search,
    Results,
    Searching,
}

#[derive(Debug)]
pub struct App {
    pub mode: AppMode,
    pub search_query: Input,
    pub search_results: Vec<TorrentSearchResult>,
    pub active_downloads: Vec<TorrentStatus>,
    pub selected_index: usize,
    pub should_quit: bool,
    pub search_in_progress: bool,
    pub status_message: String,
    pub selected_category: Option<String>,
    pub download_requested: bool,
    pub loading_frame: usize,
    pub search_progress: String,
}

impl App {
    pub fn new() -> Self {
        App {
            mode: AppMode::Normal,
            search_query: Input::default(),
            search_results: Vec::new(),
            active_downloads: Vec::new(),
            selected_index: 0,
            should_quit: false,
            search_in_progress: false,
            status_message: "Starting up...".to_string(),
            selected_category: None,
            download_requested: false,
            loading_frame: 0,
            search_progress: String::new(),
        }
    }

    pub fn start_search(&mut self) {
        self.mode = AppMode::Searching;
        self.search_in_progress = true;
        self.status_message = "Initializing search across multiple sources...".to_string();
        self.search_progress = "Starting search...".to_string();
        self.search_results.clear();
        self.selected_index = 0;
        self.loading_frame = 0;
    }

    pub fn finish_search(&mut self, results: Vec<TorrentSearchResult>) {
        self.search_in_progress = false;
        self.search_results = results;
        self.mode = AppMode::Results;
        self.status_message = format!("Found {} results", self.search_results.len());
        self.selected_index = 0;
    }

    pub fn search_error(&mut self, error: String) {
        self.search_in_progress = false;
        self.mode = AppMode::Normal;
        self.status_message = format!("Search failed: {}", error);
    }

    pub fn update_loading_animation(&mut self) {
        if self.search_in_progress {
            self.loading_frame = (self.loading_frame + 1) % 8;

            // Update search progress message with different states
            let progress_messages = [
                "Connecting to YTS movie database...",
                "Searching YTS for movies...",
                "Connecting to PirateBay API...",
                "Searching PirateBay torrents...",
                "Checking 1337x mirrors...",
                "Searching 1337x database...",
                "Sorting results by seeders...",
                "Finalizing search results...",
            ];

            let message_index = (self.loading_frame / 4) % progress_messages.len();
            self.search_progress = progress_messages[message_index].to_string();
        }
    }

    pub fn get_loading_indicator(&self) -> &'static str {
        const LOADING_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
        LOADING_FRAMES[self.loading_frame]
    }

    pub fn handle_input(&mut self) -> io::Result<bool> {
        if event::poll(std::time::Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                match self.mode {
                    AppMode::Normal => self.handle_normal_mode(key),
                    AppMode::Search => self.handle_search_mode(key),
                    AppMode::Results => self.handle_results_mode(key),
                    AppMode::Searching => self.handle_searching_mode(key),
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('s') => {
                self.mode = AppMode::Search;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.active_downloads.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.active_downloads.len();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.active_downloads.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.active_downloads.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
            }
            _ => {}
        }
    }

    fn handle_search_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.search_query.reset();
            }
            KeyCode::Enter => {
                if !self.search_query.value().is_empty() {
                    self.start_search();
                }
            }
            _ => {
                self.search_query.handle_event(&Event::Key(key));
            }
        }
    }

    fn handle_results_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.search_results.clear();
                self.selected_index = 0;
            }
            KeyCode::Enter => {
                if !self.search_results.is_empty() {
                    self.download_requested = true;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.search_results.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.search_results.len();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.search_results.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.search_results.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
            }
            _ => {}
        }
    }

    fn handle_searching_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_in_progress = false;
                self.mode = AppMode::Normal;
                self.status_message = "Search cancelled".to_string();
            }
            _ => {
                // Ignore other keys while searching
            }
        }
    }
}
