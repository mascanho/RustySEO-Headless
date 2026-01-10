use clap::Parser;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use std::{error::Error, io};

pub mod app;
pub mod cli;
pub mod crawler;
pub mod db;
pub mod logging;
pub mod models;
pub mod settings;
pub mod ui;

use crate::{
    app::AppState,
    cli::Cli,
    crawler::CrawlEngine,
    models::App,
    ui::{tabs::crawl, ui},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Logging initialization moved to after TUI setup to capture logs in app
    let log_rx = logging::init();

    // Create the file for the settings
    settings::utils::create::create_settings_file().await;

    // Init database
    db::init_db();

    // Conditionally render the UI based on the args passed
    let cli = Cli::parse();

    if !cli.url.is_empty() {
        // Handle the actions here
        let mut crawler = CrawlEngine::new().await;
        crawler.crawl(&cli.url, true).await;
        return Ok(());
    } else {
        // In case no arguments are passed then continue rendering the UI for CLI
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // create app and run it
        let mut app = App::new();
        app.log_receiver = Some(log_rx);
        app.bookmarks = db::load_bookmarks();
        let res = run_app(&mut terminal, &mut app);

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err)
        }
        Ok(())
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let tick_rate = std::time::Duration::from_millis(100);

    loop {
        terminal.draw(|f| ui(f, app)).expect("Something went wrong");

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) => {
                    if app.input_mode {
                        match key.code {
                            KeyCode::Enter => {
                                app.input_url = app.input.drain(..).collect();
                                app.input_mode = false;
                                app.reset_cursor();
                                app.start_crawl();
                            }
                            KeyCode::Esc => app.input_mode = false,
                            KeyCode::Char(c) => app.enter_char(c),
                            KeyCode::Backspace => app.delete_char(),
                            KeyCode::Left => {
                                if app.current_state == AppState::Dashboard {
                                    app.scroll_left()
                                } else {
                                    app.move_cursor_left()
                                }
                            }
                            KeyCode::Right => {
                                if app.current_state == AppState::Dashboard {
                                    app.scroll_right(50)
                                } else {
                                    app.move_cursor_right()
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // MODAL PRIORITY 1: Help
                        if app.show_help {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                                    app.show_help = false
                                }
                                _ => {}
                            }
                            continue;
                        }

                        // MODAL PRIORITY 2: Details Modal
                        if app.show_details {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => app.show_details = false,
                                KeyCode::Char('h') | KeyCode::Left => app.previous_detail_tab(),
                                KeyCode::Char('l') | KeyCode::Right => app.next_detail_tab(),
                                KeyCode::Tab => app.next_detail_tab(),
                                KeyCode::BackTab => app.previous_detail_tab(),
                                KeyCode::Char('k') | KeyCode::Up => app.previous_row(),
                                KeyCode::Char('j') | KeyCode::Down => app.next_row(),
                                _ => {}
                            }
                            continue;
                        }

                        // MODAL PRIORITY 3: Sidebar
                        if app.sidebar_visible {
                            if app.sidebar_tab == 4 {
                                match key.code {
                                    KeyCode::Enter => {
                                        if !app.bookmark_input.is_empty() {
                                            crate::db::add_bookmark(&app.bookmark_input);
                                            app.bookmarks = crate::db::load_bookmarks();
                                            app.bookmark_input.clear();
                                            app.bookmark_cursor = 0;
                                        } else if let Some(url) =
                                            app.bookmarks.get(app.bookmark_index)
                                        {
                                            app.input_url = url.to_string();
                                            app.start_crawl();
                                        }
                                    }
                                    KeyCode::Esc => {
                                        if !app.bookmark_input.is_empty() {
                                            app.bookmark_input.clear();
                                            app.bookmark_cursor = 0;
                                        } else {
                                            app.sidebar_visible = false;
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        if c == 'D' && app.bookmark_input.is_empty() {
                                            app.remove_selected_bookmark();
                                        } else {
                                            app.enter_bookmark_char(c);
                                        }
                                    }
                                    KeyCode::Backspace => app.delete_bookmark_char(),
                                    KeyCode::Left => app.move_bookmark_cursor_left(),
                                    KeyCode::Right => app.move_bookmark_cursor_right(),
                                    KeyCode::Up => app.previous_bookmark(),
                                    KeyCode::Down => app.next_bookmark(),
                                    KeyCode::Tab => app.next_sidebar_tab(),
                                    KeyCode::BackTab => app.previous_sidebar_tab(),
                                    _ => {}
                                }
                            } else {
                                match key.code {
                                    KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
                                        app.sidebar_visible = false
                                    }
                                    KeyCode::Char('k') | KeyCode::Up => app.previous_sidebar_tab(),
                                    KeyCode::Char('j') | KeyCode::Down => app.next_sidebar_tab(),
                                    KeyCode::Char('l') | KeyCode::Right => app.next_state(),
                                    KeyCode::Tab => app.next_sidebar_tab(),
                                    KeyCode::BackTab => app.previous_sidebar_tab(),
                                    KeyCode::Enter => {
                                        if app.sidebar_tab == 3 {
                                            // Handle Control Pad actions if needed
                                        }
                                    }
                                    KeyCode::Char('+') => app.set_sidebar_tab(4),
                                    _ => {}
                                }
                            }

                            // Shared Sidebar actions (only if not handled by bookmark input)
                            // We allow 'q' to quit unless we are in the bookmark tab and potentially typing
                            if key.code == KeyCode::Char('q') && app.sidebar_tab != 4 {
                                return Ok(());
                            }
                            if key.code == KeyCode::Char('?') && app.sidebar_tab != 4 {
                                app.toggle_help();
                            }

                            continue;
                        }

                        // GLOBAL NAVIGATION (when no modals are open)
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('?') => app.toggle_help(),
                            KeyCode::Esc => app.reset(),
                            KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.input_mode = true
                            }

                            // Tab/BackTab always cycle main states if no modal
                            KeyCode::Tab => app.next_state(),
                            KeyCode::BackTab => app.previous_state(),

                            // Vim Navigation
                            KeyCode::Char('h') | KeyCode::Left => app.previous_state(),
                            KeyCode::Char('l') | KeyCode::Right => {
                                if !app.sidebar_visible {
                                    app.sidebar_visible = true;
                                } else {
                                    app.next_state();
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => match app.current_state {
                                AppState::Dashboard => app.previous_row(),
                                AppState::Logs => app.previous_log(),
                                _ => {}
                            },
                            KeyCode::Char('j') | KeyCode::Down => match app.current_state {
                                AppState::Dashboard => app.next_row(),
                                AppState::Logs => app.next_log(),
                                _ => {}
                            },

                            // Advanced Vim jumps
                            KeyCode::Char('g') => {
                                // Jump to top (gg)
                                match app.current_state {
                                    AppState::Dashboard => app.table_state.select(Some(0)),
                                    AppState::Logs => app.logs_state.select(Some(0)),
                                    _ => {}
                                }
                            }
                            KeyCode::Char('G') => {
                                // Jump to bottom
                                match app.current_state {
                                    AppState::Dashboard => {
                                        if !app.table_data.is_empty() {
                                            app.table_state.select(Some(app.table_data.len() - 1));
                                        }
                                    }
                                    AppState::Logs => {
                                        if !app.logs_data.is_empty() {
                                            app.logs_state.select(Some(app.logs_data.len() - 1));
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            KeyCode::Enter => {
                                if app.current_state == AppState::Dashboard {
                                    app.show_details = true;
                                }
                            }

                            // Quick jumps
                            KeyCode::Char('s') => app.set_sidebar_tab(0),
                            KeyCode::Char('f') => app.set_sidebar_tab(1),
                            KeyCode::Char('i') => app.set_sidebar_tab(2),
                            KeyCode::Char('a') => app.set_sidebar_tab(3),
                            KeyCode::Char('b') | KeyCode::Char('+') => app.set_sidebar_tab(4),

                            // Number jumps
                            KeyCode::Char('1') => app.current_state = AppState::Crawl,
                            KeyCode::Char('2') => app.current_state = AppState::Logs,
                            KeyCode::Char('3') => app.current_state = AppState::Connectors,
                            KeyCode::Char('4') => app.current_state = AppState::Dashboard,
                            KeyCode::Char('5') => app.current_state = AppState::Reports,
                            KeyCode::Char('6') => app.current_state = AppState::Chat,
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    if matches!(mouse.kind, MouseEventKind::Down(_)) {
                        let mx = mouse.column;
                        let my = mouse.row;

                        // 1. Check sidebar tabs (Modal)
                        if app.sidebar_visible {
                            if let Some(s_rect) = app.sidebar_tab_rect {
                                // If hit the tabs
                                if mx >= s_rect.x
                                    && mx < s_rect.x + s_rect.width
                                    && my >= s_rect.y
                                    && my < s_rect.y + s_rect.height
                                {
                                    let num_s_tabs = 4;
                                    let s_tab_width = s_rect.width / num_s_tabs as u16;
                                    if s_tab_width > 0 {
                                        app.sidebar_tab = ((mx - s_rect.x) / s_tab_width)
                                            .min(num_s_tabs as u16 - 1)
                                            as usize;
                                    }
                                    continue;
                                }

                                // If clicked outside modal (since it's on the right, anything to the left of s_rect.x)
                                if mx < s_rect.x {
                                    app.sidebar_visible = false;
                                    continue;
                                }
                            }
                        }

                        // 2. Check main navigation tabs
                        if let Some(tab_rect) = app.tab_rect {
                            if mx >= tab_rect.x
                                && mx < tab_rect.x + tab_rect.width
                                && my >= tab_rect.y
                                && my < tab_rect.y + tab_rect.height
                            {
                                let num_tabs = 6;
                                let tab_width = tab_rect.width / num_tabs as u16;
                                if tab_width > 0 {
                                    let tab_index = ((mx - tab_rect.x) / tab_width)
                                        .min(num_tabs as u16 - 1)
                                        as usize;
                                    app.current_state = match tab_index {
                                        0 => AppState::Dashboard,
                                        1 => AppState::Logs,
                                        2 => AppState::Connectors,
                                        3 => AppState::Crawl,
                                        4 => AppState::Reports,
                                        5 => AppState::Chat,
                                        _ => app.current_state,
                                    };
                                }
                            }
                        }

                        // 3. Check keywords
                        let mut keyword_clicked = None;
                        for (name, rect) in &app.keyword_rects {
                            if mx >= rect.x
                                && mx < rect.x + rect.width
                                && my >= rect.y
                                && my < rect.y + rect.height
                            {
                                keyword_clicked = Some(name.clone());
                                break;
                            }
                        }

                        if let Some(name) = keyword_clicked {
                            match name.as_str() {
                                "settings" => app.set_sidebar_tab(0),
                                "filters" => app.set_sidebar_tab(1),
                                "stats" => app.set_sidebar_tab(2),
                                "actions" => app.set_sidebar_tab(3),
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Final update for every loop iteration (even without input)
        app.on_tick();
    }
}
