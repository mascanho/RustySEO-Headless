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
    // Logging
    tracing_subscriber::fmt::fmt().without_time().init();

    // Create the file for the settings
    settings::utils::create::create_settings_file().await;

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
                            KeyCode::Esc => {
                                app.input_mode = false;
                            }
                            KeyCode::Char(c) => {
                                app.enter_char(c);
                            }
                            KeyCode::Backspace => {
                                app.delete_char();
                            }
                            KeyCode::Left => {
                                if app.current_state == AppState::Dashboard && !app.input_mode {
                                    app.scroll_left();
                                } else {
                                    app.move_cursor_left();
                                }
                            }
                            KeyCode::Right => {
                                if app.current_state == AppState::Dashboard && !app.input_mode {
                                    app.scroll_right(50);
                                } else {
                                    app.move_cursor_right();
                                }
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Esc => {
                                if app.show_help {
                                    app.show_help = false;
                                } else if app.show_details {
                                    app.show_details = false;
                                } else {
                                    app.reset();
                                }
                            }
                            KeyCode::Char('?') => app.toggle_help(),
                            KeyCode::Enter => {
                                if app.current_state == AppState::Dashboard {
                                    app.show_details = !app.show_details;
                                }
                            }
                            // Trigger Input Mode with Ctrl+I
                            // Note: Some terminals send KeyCode::Tab for Ctrl+I
                            KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.input_mode = true;
                            }
                            // Vim Navigation
                            KeyCode::Char('h') => {
                                if app.sidebar_visible {
                                    app.sidebar_visible = false;
                                } else {
                                    app.previous_state();
                                }
                            }
                            KeyCode::Char('l') => {
                                if !app.sidebar_visible {
                                    app.sidebar_visible = true;
                                } else {
                                    app.next_state();
                                }
                            }
                            KeyCode::Char('k') => {
                                if app.sidebar_visible {
                                    app.sidebar_tab = if app.sidebar_tab == 0 {
                                        3
                                    } else {
                                        app.sidebar_tab - 1
                                    };
                                }
                            }
                            KeyCode::Char('j') => {
                                if app.sidebar_visible {
                                    app.sidebar_tab = (app.sidebar_tab + 1) % 4;
                                } else if app.current_state == AppState::Logs {
                                    app.next_log();
                                }
                            }
                            // Arrow keys for dashboard navigation
                            KeyCode::Up => {
                                if app.current_state == AppState::Dashboard {
                                    app.previous_row();
                                } else if app.current_state == AppState::Logs {
                                    app.previous_log();
                                }
                            }
                            KeyCode::Down => {
                                if app.current_state == AppState::Dashboard {
                                    app.next_row();
                                } else if app.current_state == AppState::Logs {
                                    app.next_log();
                                }
                            }
                            // Quick jumps to sidebar tools
                            KeyCode::Char('s') => app.set_sidebar_tab(0),
                            KeyCode::Char('f') => app.set_sidebar_tab(1),
                            KeyCode::Char('i') => app.set_sidebar_tab(2), // 'i' for info/stats
                            KeyCode::Char('a') => app.set_sidebar_tab(3),
                            // Main tab selection / Sidebar cycling / Detail cycling
                            KeyCode::Tab => {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    app.input_mode = true;
                                } else if app.show_help {
                                    // help doesn't have tabs yet
                                } else if app.show_details {
                                    app.next_detail_tab();
                                } else if app.sidebar_visible {
                                    app.next_sidebar_tab();
                                } else {
                                    app.next_state();
                                }
                            }
                            KeyCode::BackTab => {
                                if app.show_help {
                                    // help doesn't have tabs yet
                                } else if app.show_details {
                                    app.previous_detail_tab();
                                } else if app.sidebar_visible {
                                    app.previous_sidebar_tab();
                                } else {
                                    app.previous_state();
                                }
                            }
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
                                        0 => AppState::Crawl,
                                        1 => AppState::Logs,
                                        2 => AppState::Connectors,
                                        3 => AppState::Dashboard,
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
