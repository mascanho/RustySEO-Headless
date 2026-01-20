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

pub mod ai;
pub mod app;
pub mod cli;
pub mod crawler;
pub mod db;
pub mod logging;
pub mod models;
pub mod settings;
pub mod ui;

use crate::{
    app::AppState, cli::Cli, crawler::CrawlEngine, models::App, settings::utils::open::edit_file,
    ui::ui,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Logging initialization moved to after TUI setup to capture logs in app
    let log_rx = logging::init();

    // SPAWN SOME IO TASKS TO NOT SLOW DOWN THE UI
    tokio::task::spawn(async move {
        // CREATE THE RECENT CRAWLS FILE IF IT DOES NOT EXIST
        settings::utils::create::create_recent_crawls_file().await;

        // Create the file for the settings
        settings::utils::create::create_settings_file().await;

        // Init database
        db::init_db();
    });

    // Conditionally render the UI based on the args passed
    let cli = Cli::parse();

    if !cli.url.is_empty() {
        // Handle the actions here
        let crawler = CrawlEngine::new().await;
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
        let settings = crate::models::AppSettings::load();
        app.settings = Some(settings);
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
                    if app.show_search {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.show_search = false;
                                app.apply_filter();
                            }
                            KeyCode::Char(c) => {
                                app.search_query.push(c);
                                app.last_search_time = Some(std::time::Instant::now());
                            }
                            KeyCode::Backspace => {
                                app.search_query.pop();
                                app.last_search_time = Some(std::time::Instant::now());
                            }
                            _ => {}
                        }
                    } else if app.show_internal_search {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.show_internal_search = false;
                                app.apply_internal_filter();
                            }
                            KeyCode::Char(c) => {
                                app.internal_search_query.push(c);
                                app.last_search_time = Some(std::time::Instant::now());
                            }
                            KeyCode::Backspace => {
                                app.internal_search_query.pop();
                                app.last_search_time = Some(std::time::Instant::now());
                            }
                            _ => {}
                        }
                    } else if app.show_js_urls_search {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.show_js_urls_search = false;
                                app.apply_js_urls_filter();
                            }
                            KeyCode::Char(c) => {
                                app.js_urls_search_query.push(c);
                                app.last_search_time = Some(std::time::Instant::now());
                            }
                            KeyCode::Backspace => {
                                app.js_urls_search_query.pop();
                                app.last_search_time = Some(std::time::Instant::now());
                            }
                            _ => {}
                        }
                    } else if app.show_css_urls_search {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.show_css_urls_search = false;
                                app.apply_css_urls_filter();
                            }
                            KeyCode::Char(c) => {
                                app.css_urls_search_query.push(c);
                                app.last_search_time = Some(std::time::Instant::now());
                            }
                            KeyCode::Backspace => {
                                app.css_urls_search_query.pop();
                                app.last_search_time = Some(std::time::Instant::now());
                            }
                            _ => {}
                        }
                    } else if app.show_content_search {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.show_content_search = false;
                                app.apply_content_filter();
                            }
                            KeyCode::Char(c) => {
                                app.content_search_query.push(c);
                                app.last_search_time = Some(std::time::Instant::now());
                            }
                            KeyCode::Backspace => {
                                app.content_search_query.pop();
                                app.last_search_time = Some(std::time::Instant::now());
                            }
                            _ => {}
                        }
                    } else if app.show_js_pages_modal {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.close_js_pages_modal(),
                            KeyCode::Char('k') | KeyCode::Up => {
                                let len = app.js_pages_list.len();
                                if len > 0 {
                                    let i = match app.js_pages_state.selected() {
                                        Some(i) => {
                                            if i == 0 {
                                                len - 1
                                            } else {
                                                i - 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    app.js_pages_state.select(Some(i));
                                }
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                let len = app.js_pages_list.len();
                                if len > 0 {
                                    let i = match app.js_pages_state.selected() {
                                        Some(i) => {
                                            if i >= len - 1 {
                                                0
                                            } else {
                                                i + 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    app.js_pages_state.select(Some(i));
                                }
                            }
                            _ => {}
                        }
                    } else if app.show_css_pages_modal {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.close_css_pages_modal(),
                            KeyCode::Char('k') | KeyCode::Up => {
                                let len = app.css_pages_list.len();
                                if len > 0 {
                                    let i = match app.css_pages_state.selected() {
                                        Some(i) => {
                                            if i == 0 {
                                                len - 1
                                            } else {
                                                i - 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    app.css_pages_state.select(Some(i));
                                }
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                let len = app.css_pages_list.len();
                                if len > 0 {
                                    let i = match app.css_pages_state.selected() {
                                        Some(i) => {
                                            if i >= len - 1 {
                                                0
                                            } else {
                                                i + 1
                                            }
                                        }
                                        None => 0,
                                    };
                                    app.css_pages_state.select(Some(i));
                                }
                            }
                            _ => {}
                        }
                    } else if app.show_log_search {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.show_log_search = false;
                                app.apply_log_filter();
                            }
                            KeyCode::Char(c) => {
                                app.log_search_query.push(c);
                                app.last_log_search_time = Some(std::time::Instant::now());
                            }
                            KeyCode::Backspace => {
                                app.log_search_query.pop();
                                app.last_log_search_time = Some(std::time::Instant::now());
                            }
                            _ => {}
                        }
                    } else if app.input_mode {
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
                                } else if app.current_state == AppState::Content {
                                    app.scroll_content_left()
                                } else {
                                    app.move_cursor_left()
                                }
                            }
                            KeyCode::Right => {
                                if app.current_state == AppState::Dashboard {
                                    app.scroll_right(50)
                                } else if app.current_state == AppState::Content {
                                    app.scroll_content_right(50)
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

                        // MODAL PRIORITY 1: AI Chat Modal
                        if app.show_ai_modal {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => app.show_ai_modal = false,
                                KeyCode::Enter => app.submit_ai_message(),
                                KeyCode::Backspace => {
                                    app.ai_input.pop();
                                }
                                KeyCode::Char(c) => {
                                    app.ai_input.push(c);
                                }
                                _ => {}
                            }
                            continue;
                        }

                        // MODAL PRIORITY 2: Details Modal
                        if app.show_details {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => app.show_details = false,
                                KeyCode::Char('h') => app.previous_detail_tab(),
                                // KeyCode::Char('l') | KeyCode::Right => app.next_detail_tab(),
                                KeyCode::Tab => app.next_detail_tab(),
                                KeyCode::BackTab => app.previous_detail_tab(),
                                KeyCode::Left => app.previous_detail_tab(),
                                KeyCode::Right => app.next_detail_tab(),
                                KeyCode::Char('k') => {
                                    if app.detail_tab == 3
                                        || app.detail_tab == 4
                                        || app.detail_tab == 5
                                        || app.detail_tab == 8
                                    {
                                        let len = app.get_current_detail_len();
                                        app.previous_detail_row(len);
                                    } else if [1, 2, 6, 7].contains(&app.detail_tab) {
                                        if app.detail_scroll > 0 {
                                            app.detail_scroll = app.detail_scroll.saturating_sub(1);
                                        }
                                    } else {
                                        app.previous_row();
                                    }
                                }
                                KeyCode::Char('j') => {
                                    if app.detail_tab == 3
                                        || app.detail_tab == 4
                                        || app.detail_tab == 5
                                        || app.detail_tab == 8
                                    {
                                        let len = app.get_current_detail_len();
                                        app.next_detail_row(len);
                                    } else if [1, 2, 6, 7].contains(&app.detail_tab) {
                                        app.detail_scroll += 1;
                                    } else {
                                        app.next_row();
                                    }
                                }
                                KeyCode::Up => {
                                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                                        // Shift+Up for navigating content in detail tab
                                        if app.detail_tab == 3
                                            || app.detail_tab == 4
                                            || app.detail_tab == 5
                                            || app.detail_tab == 8
                                        {
                                            let len = app.get_current_detail_len();
                                            app.previous_detail_row(len);
                                        } else if [0, 1, 2, 6, 7].contains(&app.detail_tab) {
                                            if app.detail_scroll > 0 {
                                                app.detail_scroll =
                                                    app.detail_scroll.saturating_sub(1);
                                            }
                                        }
                                    } else if app.current_state == AppState::Dashboard {
                                        app.previous_row();
                                    }
                                }
                                KeyCode::Down => {
                                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                                        // Shift+Down for navigating content in detail tab
                                        if app.detail_tab == 3
                                            || app.detail_tab == 4
                                            || app.detail_tab == 5
                                            || app.detail_tab == 8
                                        {
                                            let len = app.get_current_detail_len();
                                            app.next_detail_row(len);
                                        } else if [0, 1, 2, 6, 7].contains(&app.detail_tab) {
                                            app.detail_scroll += 1;
                                        }
                                    } else if app.current_state == AppState::Dashboard {
                                        app.next_row();
                                    }
                                }
                                _ => {}
                            }
                            continue;
                        }

                        // MODAL PRIORITY 2.5: Dashboard Menu
                        if app.show_dashboard_menu {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    app.show_dashboard_menu = false
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    app.previous_dashboard_menu_item()
                                }
                                KeyCode::Char('j') | KeyCode::Down => {
                                    app.next_dashboard_menu_item()
                                }
                                KeyCode::Enter => app.execute_dashboard_menu_action(),
                                _ => {}
                            }
                            continue;
                        }

                        // MODAL PRIORITY 2.8: Logs Console
                        if app.show_logs {
                            match key.code {
                                KeyCode::Char('s')
                                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    app.show_log_search = true;
                                }
                                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('L') => {
                                    app.show_logs = false
                                }
                                KeyCode::Char('k') | KeyCode::Up => app.previous_log(),
                                KeyCode::Char('j') | KeyCode::Down => app.next_log(),
                                KeyCode::Char('t') => app.logs_state.select(Some(0)),
                                KeyCode::Char('G') => {
                                    if !app.logs_data.is_empty() {
                                        app.logs_state.select(Some(app.logs_data.len() - 1));
                                    }
                                }
                                KeyCode::Char('[') => app.decrease_logs_height(),
                                KeyCode::Char(']') => app.increase_logs_height(),
                                _ => {}
                            }
                            continue;
                        }

                        // MODAL PRIORITY 3: Sidebar
                        if app.sidebar_visible {
                            if app.sidebar_tab == 1 {
                                if key.code == KeyCode::Char('E') {
                                    edit_file();
                                }
                            }

                            if app.sidebar_tab == 4 {
                                match key.code {
                                    KeyCode::Left => {
                                        if app.bookmark_input.is_empty() {
                                            if app.bookmark_subview > 0 {
                                                app.bookmark_subview -= 1;
                                            }
                                        } else {
                                            app.move_bookmark_cursor_left();
                                        }
                                    }
                                    KeyCode::Right => {
                                        if app.bookmark_input.is_empty() {
                                            app.bookmark_subview = (app.bookmark_subview + 1) % 2;
                                        } else {
                                            app.move_bookmark_cursor_right();
                                        }
                                    }
                                    KeyCode::Enter => {
                                        if app.bookmark_subview == 0 {
                                            // Bookmarks view
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
                                        } else {
                                            // Last crawled view
                                            let recent_urls = app.get_recent_crawled_urls();
                                            if let Some(url) =
                                                recent_urls.get(app.last_crawled_index)
                                            {
                                                app.input_url = url.to_string();
                                                app.start_crawl();
                                            }
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
                                        if app.bookmark_subview == 0
                                            && c == 'D'
                                            && app.bookmark_input.is_empty()
                                        {
                                            app.remove_selected_bookmark();
                                        } else if app.bookmark_subview == 0 {
                                            app.enter_bookmark_char(c);
                                        }
                                    }
                                    KeyCode::Backspace => {
                                        if app.bookmark_subview == 0 {
                                            app.delete_bookmark_char();
                                        }
                                    }

                                    KeyCode::Up => {
                                        if app.bookmark_subview == 0 {
                                            app.previous_bookmark();
                                        } else {
                                            app.previous_last_crawled();
                                        }
                                    }
                                    KeyCode::Down => {
                                        if app.bookmark_subview == 0 {
                                            app.next_bookmark();
                                        } else {
                                            app.next_last_crawled();
                                        }
                                    }
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
                                    // KeyCode::Char('l') | KeyCode::Right => app.next_state(),
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
                            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                if app.show_logs {
                                    app.show_log_search = true;
                                } else if app.current_state == AppState::Dashboard {
                                    app.show_search = true;
                                } else if app.current_state == AppState::Internal {
                                    app.show_internal_search = true;
                                } else if app.current_state == AppState::Css {
                                    app.show_css_urls_search = true;
                                } else if app.current_state == AppState::Javascript {
                                    app.show_js_urls_search = true;
                                } else if app.current_state == AppState::Content {
                                    app.show_content_search = true;
                                }
                            }

                            // Tab/BackTab always cycle main states if no modal
                            KeyCode::Tab => app.next_state(),
                            KeyCode::Backspace => app.previous_state(),

                            // Vim Navigation
                            // KeyCode::Char('h') | KeyCode::Left => app.previous_state(),
                            // KeyCode::Char('l') | KeyCode::Right => {
                            //     if !app.sidebar_visible {
                            //         app.sidebar_visible = true;
                            //     } else {
                            //         app.next_state();
                            //     }
                            // }
                            KeyCode::Char('k') | KeyCode::Up => match app.current_state {
                                AppState::Dashboard | AppState::CoreWebVitals => app.previous_row(),
                                AppState::Content => app.previous_content_row(),
                                AppState::Internal => app.previous_internal_row(),
                                AppState::Javascript => {
                                    let selected = app.js_urls_table_state.selected().unwrap_or(0);
                                    if selected > 0 {
                                        app.js_urls_table_state.select(Some(selected - 1));
                                    } else if app.js_urls_current_page > 0 {
                                        app.js_urls_current_page -= 1;
                                        app.apply_js_urls_pagination();
                                        app.js_urls_table_state.select(Some(
                                            app.js_urls_filtered_table_data.len().saturating_sub(1),
                                        ));
                                    }
                                }
                                AppState::Css => {
                                    let selected = app.css_urls_table_state.selected().unwrap_or(0);
                                    if selected > 0 {
                                        app.css_urls_table_state.select(Some(selected - 1));
                                    } else if app.css_urls_current_page > 0 {
                                        app.css_urls_current_page -= 1;
                                        app.apply_css_urls_pagination();
                                        app.css_urls_table_state.select(Some(
                                            app.css_urls_filtered_table_data
                                                .len()
                                                .saturating_sub(1),
                                        ));
                                    }
                                }
                                _ => {}
                            },
                            KeyCode::Char('j') | KeyCode::Down => match app.current_state {
                                AppState::Dashboard | AppState::CoreWebVitals => app.next_row(),
                                AppState::Content => app.next_content_row(),
                                AppState::Internal => app.next_internal_row(),
                                AppState::Javascript => {
                                    let len = app.js_urls_filtered_table_data.len();
                                    let selected = app.js_urls_table_state.selected().unwrap_or(0);
                                    if selected < len.saturating_sub(1) {
                                        app.js_urls_table_state.select(Some(selected + 1));
                                    } else {
                                        let total_pages =
                                            (app.js_urls_full_filtered_table_data.len()
                                                + app.js_urls_page_size
                                                - 1)
                                                / app.js_urls_page_size;
                                        if app.js_urls_current_page + 1 < total_pages {
                                            app.js_urls_current_page += 1;
                                            app.apply_js_urls_pagination();
                                            app.js_urls_table_state.select(Some(0));
                                        }
                                    }
                                }
                                AppState::Css => {
                                    let len = app.css_urls_filtered_table_data.len();
                                    let selected = app.css_urls_table_state.selected().unwrap_or(0);
                                    if selected < len.saturating_sub(1) {
                                        app.css_urls_table_state.select(Some(selected + 1));
                                    } else {
                                        let total_pages =
                                            (app.css_urls_full_filtered_table_data.len()
                                                + app.css_urls_page_size
                                                - 1)
                                                / app.css_urls_page_size;
                                        if app.css_urls_current_page + 1 < total_pages {
                                            app.css_urls_current_page += 1;
                                            app.apply_css_urls_pagination();
                                            app.css_urls_table_state.select(Some(0));
                                        }
                                    }
                                }
                                _ => {}
                            },

                            // Advanced Vim jumps
                            KeyCode::Char('t') => {
                                // Jump to top
                                match app.current_state {
                                    AppState::Dashboard | AppState::CoreWebVitals => {
                                        app.table_state.select(Some(0))
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Char('G') => {
                                // Jump to bottom
                                match app.current_state {
                                    AppState::Dashboard | AppState::CoreWebVitals => {
                                        if !app.table_data.is_empty() {
                                            app.table_state.select(Some(app.table_data.len() - 1));
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            KeyCode::Enter => {
                                if app.current_state == AppState::Dashboard
                                    || app.current_state == AppState::CoreWebVitals
                                {
                                    app.validate_table_state();
                                    if let Some(selected) = app.table_state.selected() {
                                        if selected < app.table_data.len()
                                            && selected < app.page_data.len()
                                        {
                                            app.show_details = true;
                                        }
                                    }
                                } else if app.current_state == AppState::Javascript {
                                    if let Some(selected) = app.js_urls_table_state.selected() {
                                        if let Some(js_url) =
                                            app.js_urls_filtered_table_data.get(selected)
                                        {
                                            app.show_js_pages_for_url(js_url.url.clone());
                                        }
                                    }
                                } else if app.current_state == AppState::Css {
                                    if let Some(selected) = app.css_urls_table_state.selected() {
                                        if let Some(css_url) =
                                            app.css_urls_filtered_table_data.get(selected)
                                        {
                                            app.show_css_pages_for_url(css_url.url.clone());
                                        }
                                    }
                                }
                            }

                            KeyCode::Char('m') => {
                                if app.current_state == AppState::Dashboard {
                                    app.validate_table_state();
                                    if let Some(selected) = app.table_state.selected() {
                                        if selected < app.table_data.len()
                                            && selected < app.page_data.len()
                                        {
                                            app.show_dashboard_menu = true;
                                        } else if selected == app.table_data.len() {
                                            app.show_dashboard_menu = true;
                                        }
                                    }
                                }
                            }

                            KeyCode::Char(']') => match app.current_state {
                                AppState::Dashboard | AppState::CoreWebVitals => app.next_page(),
                                AppState::Content => app.next_content_page(),
                                AppState::Internal => app.next_internal_page(),
                                AppState::Javascript => {
                                    let total_pages = (app.js_urls_full_filtered_table_data.len()
                                        + app.js_urls_page_size
                                        - 1)
                                        / app.js_urls_page_size;
                                    if app.js_urls_current_page + 1 < total_pages {
                                        app.js_urls_current_page += 1;
                                        app.apply_js_urls_pagination();
                                    }
                                }
                                _ => {}
                            },
                            KeyCode::Char('[') => match app.current_state {
                                AppState::Dashboard | AppState::CoreWebVitals => {
                                    app.previous_page()
                                }
                                AppState::Content => app.previous_content_page(),
                                AppState::Internal => app.previous_internal_page(),
                                AppState::Javascript => {
                                    if app.js_urls_current_page > 0 {
                                        app.js_urls_current_page -= 1;
                                        app.apply_js_urls_pagination();
                                    }
                                }
                                _ => {}
                            },

                            // Quick jumps
                            KeyCode::Char('g') => app.set_sidebar_tab(0),
                            KeyCode::Char('s') => app.set_sidebar_tab(1),
                            KeyCode::Char('e') | KeyCode::Char('E') => {
                                if app.sidebar_visible && app.sidebar_tab == 1 {
                                    app.open_settings_file();
                                }
                            }
                            KeyCode::Char('f') => app.set_sidebar_tab(2),
                            KeyCode::Char('a') => app.set_sidebar_tab(3),
                            KeyCode::Char('A') => app.toggle_ai_modal(),
                            KeyCode::Char('b') | KeyCode::Char('+') => app.set_sidebar_tab(4),
                            KeyCode::Char('L') => app.toggle_logs(),
                            // Number jumps
                            KeyCode::Char('1') => app.current_state = AppState::Dashboard,
                            KeyCode::Char('2') => app.current_state = AppState::Crawl,
                            KeyCode::Char('3') => app.current_state = AppState::Internal,
                            KeyCode::Char('4') => app.current_state = AppState::Redirects,
                            KeyCode::Char('5') => app.current_state = AppState::Images,
                            KeyCode::Char('6') => app.current_state = AppState::Css,
                            KeyCode::Char('7') => app.current_state = AppState::Javascript,
                            KeyCode::Char('8') => app.current_state = AppState::Keywords,
                            KeyCode::Char('9') => app.current_state = AppState::CoreWebVitals,
                            KeyCode::Char('0') => app.current_state = AppState::CustomSearch,
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
                                    let num_s_tabs = 5;
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
                                let num_tabs = 12;
                                let tab_width = tab_rect.width / num_tabs as u16;
                                if tab_width > 0 {
                                    let tab_index = ((mx - tab_rect.x) / tab_width)
                                        .min(num_tabs as u16 - 1)
                                        as usize;
                                    app.current_state = match tab_index {
                                        0 => AppState::Dashboard,
                                        1 => AppState::Crawl,
                                        2 => AppState::Internal,
                                        3 => AppState::Redirects,
                                        4 => AppState::Images,
                                        5 => AppState::Css,
                                        6 => AppState::Javascript,
                                        7 => AppState::Keywords,
                                        8 => AppState::CoreWebVitals,
                                        9 => AppState::CustomSearch,
                                        10 => AppState::Reports,
                                        11 => AppState::Content,
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
                    } else if matches!(
                        mouse.kind,
                        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                    ) {
                        // Handle mouse wheel scrolling on tables
                        if app.current_state == AppState::Dashboard
                            || app.current_state == AppState::CoreWebVitals
                            || app.current_state == AppState::Content
                            || app.current_state == AppState::Internal
                            || app.current_state == AppState::Javascript
                            || app.current_state == AppState::Css
                        {
                            if let Some(rect) = app.table_rect {
                                if mouse.column >= rect.x
                                    && mouse.column < rect.x + rect.width
                                    && mouse.row >= rect.y
                                    && mouse.row < rect.y + rect.height
                                {
                                    match mouse.kind {
                                        MouseEventKind::ScrollUp => {
                                            if app.current_state == AppState::Dashboard
                                                || app.current_state == AppState::CoreWebVitals
                                            {
                                                app.previous_row()
                                            } else if app.current_state == AppState::Content {
                                                app.previous_content_row()
                                            } else if app.current_state == AppState::Internal {
                                                app.previous_internal_row()
                                            } else if app.current_state == AppState::Javascript {
                                                let selected =
                                                    app.js_urls_table_state.selected().unwrap_or(0);
                                                if selected > 0 {
                                                    app.js_urls_table_state
                                                        .select(Some(selected - 1));
                                                }
                                            } else if app.current_state == AppState::Css {
                                                let selected = app
                                                    .css_urls_table_state
                                                    .selected()
                                                    .unwrap_or(0);
                                                if selected > 0 {
                                                    app.css_urls_table_state
                                                        .select(Some(selected - 1));
                                                }
                                            }
                                        }
                                        MouseEventKind::ScrollDown => {
                                            if app.current_state == AppState::Dashboard
                                                || app.current_state == AppState::CoreWebVitals
                                            {
                                                app.next_row()
                                            } else if app.current_state == AppState::Content {
                                                app.next_content_row()
                                            } else if app.current_state == AppState::Internal {
                                                app.next_internal_row()
                                            } else if app.current_state == AppState::Javascript {
                                                let len = app.js_urls_filtered_table_data.len();
                                                let selected =
                                                    app.js_urls_table_state.selected().unwrap_or(0);
                                                if selected < len.saturating_sub(1) {
                                                    app.js_urls_table_state
                                                        .select(Some(selected + 1));
                                                }
                                            } else if app.current_state == AppState::Css {
                                                let len = app.css_urls_filtered_table_data.len();
                                                let selected = app
                                                    .css_urls_table_state
                                                    .selected()
                                                    .unwrap_or(0);
                                                if selected < len.saturating_sub(1) {
                                                    app.css_urls_table_state
                                                        .select(Some(selected + 1));
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
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
