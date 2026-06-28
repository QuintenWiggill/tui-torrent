pub mod api;
pub mod app;
pub mod aria2_client;
pub mod aria2_manager;
pub mod ascii_art;
pub mod error;
pub mod torrent_search;
pub mod tui;
pub mod utils;

use app::{App, AppMode};
use aria2_manager::Aria2Manager;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tokio::time::{Duration, Instant};
use torrent_search::TorrentSearchEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏴‍☠️ Starting TUI Torrent...");

    let mut aria2_manager = Aria2Manager::new();

    let aria2_available = match aria2_manager.ensure_aria2_running().await {
        Ok(()) => {
            if let Ok(version) = aria2_manager.get_version().await {
                println!("📡 Connected to aria2 version: {}", version);
            } else {
                println!("📡 Connected to aria2");
            }
            println!(
                "📁 Downloads will be saved to: {}",
                aria2_manager.get_download_dir()
            );
            true
        }
        Err(e) => {
            eprintln!("⚠️  Warning: {}", e);
            eprintln!("💡 Downloads will not work without aria2. Install it with:");
            eprintln!("   macOS: brew install aria2");
            eprintln!("   Ubuntu: sudo apt install aria2");
            eprintln!();
            eprintln!("🔄 Continuing anyway... (search will still work)");
            false
        }
    };

    println!("🚀 Starting TUI interface...");

    // Setup terminal
    terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    // Create terminal once and reuse
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Create app state
    let mut app = App::new();
    app.aria2_installed = aria2_available;
    let search_engine = TorrentSearchEngine::new();

    // Track if we've already rendered the initial searching frame
    let mut initial_search_frame_rendered = false;

    // Update status based on aria2 availability
    if aria2_available {
        let download_dir = aria2_manager.get_download_dir();
        let short_path = if download_dir.len() > 40 {
            format!("...{}", &download_dir[download_dir.len() - 37..])
        } else {
            download_dir
        };
        app.status_message = format!("Ready - Downloads: {} - Press 's' to search", short_path);
    } else {
        app.status_message = "Warning - Search only (aria2 is not available/installed)".to_string();
    }

    // Main loop
    let tick_rate = Duration::from_millis(100);
    let loading_tick_rate = Duration::from_millis(150); // Faster animation during search
    let mut last_tick = Instant::now();
    let mut last_update = Instant::now();

    // Do an initial render
    tui::render_ui(&mut terminal, &app)?;

    loop {
        let mut needs_render = false;

        if app.handle_input()? {
            needs_render = true;
        }

        if app.should_quit {
            break;
        }

        // Perform search (blocking) but ensure searching frame displayed first
        if app.mode == AppMode::Searching && app.search_in_progress {
            if !initial_search_frame_rendered {
                // Immediate render to show user the searching state before network calls
                tui::render_ui(&mut terminal, &app)?;
                initial_search_frame_rendered = true;
            } else {
                // Execute the search now
                let query = app.search_query.value().to_string();
                let category = app.selected_category.clone();
                match search_engine
                    .search_torrents(&query, category.as_deref())
                    .await
                {
                    Ok(results) => app.finish_search(results),
                    Err(e) => app.search_error(e.to_string()),
                }
                initial_search_frame_rendered = false; // reset for next time
                needs_render = true;
            }
        } else {
            initial_search_frame_rendered = false; // reset if we leave searching mode
        }

        // Handle torrent download request
        if app.download_requested && !app.search_results.is_empty() {
            if !aria2_available {
                app.status_message = "Cannot download: aria2 is not available/installed".to_string();
            } else if let Some(selected) = app.search_results.get(app.selected_index) {
                match torrent_search::add_torrent(&selected.magnet_link).await {
                    Ok(gid) => {
                        app.status_message =
                            format!("Added torrent: {} (GID: {})", selected.name, gid);
                        app.mode = AppMode::Normal;
                    }
                    Err(e) => {
                        app.status_message = format!("Failed to add torrent: {}", e);
                    }
                }
            }
            app.download_requested = false;
            needs_render = true;
        }

        // Handle pause request
        if let Some(gid) = app.pause_requested.take() {
            if aria2_available {
                match aria2_client::pause_download(&gid).await {
                    Ok(Ok(_)) => {
                        app.status_message = format!("Paused download (GID: {})", gid);
                        if let Ok(downloads) = aria2_client::get_all_downloads().await {
                            app.active_downloads = downloads;
                        }
                    }
                    Ok(Err(err)) => {
                        app.status_message = format!("Failed to pause: {}", err);
                    }
                    Err(e) => {
                        app.status_message = format!("RPC error: {}", e);
                    }
                }
            } else {
                app.status_message = "Cannot pause: aria2 is not available".to_string();
            }
            needs_render = true;
        }

        // Handle resume request
        if let Some(gid) = app.resume_requested.take() {
            if aria2_available {
                match aria2_client::resume_download(&gid).await {
                    Ok(Ok(_)) => {
                        app.status_message = format!("Resumed download (GID: {})", gid);
                        if let Ok(downloads) = aria2_client::get_all_downloads().await {
                            app.active_downloads = downloads;
                        }
                    }
                    Ok(Err(err)) => {
                        app.status_message = format!("Failed to resume: {}", err);
                    }
                    Err(e) => {
                        app.status_message = format!("RPC error: {}", e);
                    }
                }
            } else {
                app.status_message = "Cannot resume: aria2 is not available".to_string();
            }
            needs_render = true;
        }

        // Update downloads list every 2 seconds if aria2 is available
        if aria2_available && last_update.elapsed() >= Duration::from_secs(2) {
            match aria2_client::get_all_downloads().await {
                Ok(downloads) => {
                    app.active_downloads = downloads;
                }
                Err(e) => {
                    app.status_message = format!("Error updating downloads: {}", e);
                }
            }
            last_update = Instant::now();
            needs_render = true;
        }

        // Update loading animation and render UI
        let current_tick_rate = if app.search_in_progress {
            loading_tick_rate
        } else {
            tick_rate
        };
        if last_tick.elapsed() >= current_tick_rate {
            app.update_loading_animation();
            last_tick = Instant::now();
            needs_render = true; // Timer ticked, we might have animation updates
        }

        if needs_render {
            tui::render_ui(&mut terminal, &app)?;
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    // Clean up aria2 process
    aria2_manager.stop();

    Ok(())
}
