use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{backend::{Backend, CrosstermBackend}, Terminal};
use std::{error::Error, io};

mod app;
mod ui;

use crate::app::{App, AppState};
use crate::ui::ui;

fn main() -> Result<(), Box<dyn Error>> {
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        match event::read()? {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc => {
                    if app.show_help {
                        app.show_help = false;
                    } else {
                        app.reset();
                    }
                }
                KeyCode::Char('?') => app.toggle_help(),
                // Vim Navigation
                KeyCode::Char('h') | KeyCode::Left => {
                    if app.sidebar_visible {
                        app.sidebar_visible = false;
                    } else {
                        app.previous_state();
                    }
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if !app.sidebar_visible {
                         app.sidebar_visible = true;
                    } else {
                        app.next_state();
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if app.sidebar_visible {
                        app.sidebar_tab = if app.sidebar_tab == 0 { 3 } else { app.sidebar_tab - 1 };
                    }
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if app.sidebar_visible {
                        app.sidebar_tab = (app.sidebar_tab + 1) % 4;
                    }
                }
                // Quick jumps to sidebar tools
                KeyCode::Char('s') => app.set_sidebar_tab(0),
                KeyCode::Char('f') => app.set_sidebar_tab(1),
                KeyCode::Char('i') => app.set_sidebar_tab(2), // 'i' for info/stats
                KeyCode::Char('a') => app.set_sidebar_tab(3),
                // Main tab selection
                KeyCode::Tab => app.next_state(),
                KeyCode::BackTab => app.previous_state(),
                KeyCode::Char('1') => app.current_state = AppState::Crawl,
                KeyCode::Char('2') => app.current_state = AppState::Logs,
                KeyCode::Char('3') => app.current_state = AppState::Connectors,
                KeyCode::Char('4') => app.current_state = AppState::Dashboard,
                KeyCode::Char('5') => app.current_state = AppState::Reports,
                KeyCode::Char('6') => app.current_state = AppState::Chat,
                _ => {}
                }
            }
            Event::Mouse(mouse) => {
                if matches!(mouse.kind, MouseEventKind::Down(_)) {
                    let mx = mouse.column;
                    let my = mouse.row;

                    // 1. Check main navigation tabs
                    if let Some(tab_rect) = app.tab_rect {
                        if mx >= tab_rect.x && mx < tab_rect.x + tab_rect.width && my >= tab_rect.y && my < tab_rect.y + tab_rect.height {
                            let num_tabs = 6;
                            let tab_width = tab_rect.width / num_tabs as u16;
                            if tab_width > 0 {
                                let tab_index = ((mx - tab_rect.x) / tab_width).min(num_tabs as u16 - 1) as usize;
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

                    // 2. Check sidebar tabs
                    if app.sidebar_visible {
                        if let Some(s_rect) = app.sidebar_tab_rect {
                            if mx >= s_rect.x && mx < s_rect.x + s_rect.width && my >= s_rect.y && my < s_rect.y + s_rect.height {
                                let num_s_tabs = 4;
                                let s_tab_width = s_rect.width / num_s_tabs as u16;
                                if s_tab_width > 0 {
                                    app.sidebar_tab = ((mx - s_rect.x) / s_tab_width).min(num_s_tabs as u16 - 1) as usize;
                                }
                            }
                        }
                    }

                    // 3. Check keywords
                    let mut keyword_clicked = None;
                    for (name, rect) in &app.keyword_rects {
                        if mx >= rect.x && mx < rect.x + rect.width && my >= rect.y && my < rect.y + rect.height {
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
}
