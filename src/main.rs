mod app;
mod logging;
mod tmdb;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use log::info;
use ratatui::prelude::*;
use tokio::sync::mpsc;

use app::{App, AppAction, SearchType, View};
use tmdb::TmdbClient;

#[tokio::main]
async fn main() -> io::Result<()> {
    let log_file = logging::init().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to initialize logging: {}", e);
        std::path::PathBuf::from("hippo.log")
    });

    info!("hippo starting up");
    eprintln!("Log file: {}", log_file.display());

    let tmdb = TmdbClient::from_env().unwrap_or_else(|e| {
        eprintln!("Error: {}. Set TMDB_API_KEY env var.", e);
        std::process::exit(1);
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<AppAction>();
    let mut app = App::new(tmdb, action_tx.clone());
    app.load_home();

    loop {
        app.tick = app.tick.wrapping_add(1);
        terminal.draw(|frame| ui::ui(frame, &app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(&mut app, key.code, key.modifiers);
                }
            }
        }

        while let Ok(action) = action_rx.try_recv() {
            app.handle_action(action);
        }

        if app.should_quit {
            break;
        }
    }

    info!("hippo shutting down");
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return;
        }
        _ => {}
    }

    if app.loading {
        return;
    }

    match app.view {
        View::Home => match code {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            KeyCode::Char('h') | KeyCode::Left => app.move_left(),
            KeyCode::Char('l') | KeyCode::Right => app.move_right(),
            KeyCode::Char('j') | KeyCode::Down => app.move_down(),
            KeyCode::Char('k') | KeyCode::Up => app.move_up(),
            KeyCode::Enter | KeyCode::Char(' ') => app.select_item(),
            KeyCode::Char('/') => {
                app.view = View::Search;
                app.search_input_mode = true;
            }
            _ => {}
        },
        View::Search => {
            if app.search_input_mode {
                match code {
                    KeyCode::Esc => {
                        app.view = View::Home;
                        app.search_query.clear();
                        app.search_results.clear();
                        app.search_input_mode = false;
                    }
                    KeyCode::Enter => {
                        app.search();
                        app.search_input_mode = false;
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                    }
                    _ => {}
                }
            } else {
                match code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Esc => {
                        app.view = View::Home;
                        app.search_query.clear();
                        app.search_results.clear();
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        if app.search_item_idx + 1 < app.search_results.len() {
                            app.search_item_idx += 1;
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if app.search_item_idx > 0 {
                            app.search_item_idx -= 1;
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => app.select_item(),
                    KeyCode::Char('/') => {
                        app.search_input_mode = true;
                        app.search_query.clear();
                    }
                    KeyCode::Tab => {
                        app.search_type = match app.search_type {
                            SearchType::Movie => SearchType::Tv,
                            SearchType::Tv => SearchType::Movie,
                        };
                        if !app.search_query.is_empty() {
                            app.search();
                        }
                    }
                    _ => {}
                }
            }
        }
        View::TvDetail => match code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.go_back(),
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(ref detail) = app.tv_detail {
                    if app.tv_item_idx + 1 < detail.seasons.len() {
                        app.tv_item_idx += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.tv_item_idx > 0 {
                    app.tv_item_idx -= 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => app.select_item(),
            _ => {}
        },
        View::SeasonDetail => match code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => app.go_back(),
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(ref detail) = app.season_detail {
                    if app.season_item_idx + 1 < detail.episodes.len() {
                        app.season_item_idx += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.season_item_idx > 0 {
                    app.season_item_idx -= 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => app.select_item(),
            _ => {}
        },
    }
}
