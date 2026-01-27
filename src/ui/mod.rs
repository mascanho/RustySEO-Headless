use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
};

use crate::{app::AppState, models::App};

pub mod components;
pub mod footer;
pub mod modals;
pub mod side_panel;
pub mod sidebar;
pub mod tabs;

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Define main colors
    let bg_color = Color::Rgb(15, 15, 25);

    // Render full background first to ensure consistency
    f.render_widget(Block::default().bg(bg_color), size);

    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    // Main layout: Navigation (3) + Content Area (Min 0) + Footer (3)
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Navigation
            Constraint::Min(0),    // Content Area
            Constraint::Length(3), // Footer
        ])
        .split(size);

    let tab_area = main_layout[0];
    let content_area = main_layout[1];
    let footer_area = main_layout[2];

    app.tab_rect = Some(tab_area);

    // Render Navigation Tabs
    let titles = vec![
        "Overview",
        "Crawl",
        "Internal",
        "Redirects",
        "Images",
        "CSS",
        "Javascript",
        "Keywords",
        "CWV",
        "Custom Extractor",
        "Reports",
        "Content",
        "Files",
    ];
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" RustySEO - CLI ")
                .border_style(Style::default().fg(border_color)),
        )
        .select(app.get_state_index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(accent_color)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .divider(Span::styled(" | ", Style::default().fg(border_color)));

    f.render_widget(tabs, tab_area);

    // Render Tab Content
    match app.current_state {
        AppState::Dashboard => tabs::dashboard::render(f, app, content_area),
        AppState::Crawl => tabs::crawl::render(f, app, content_area),
        AppState::Internal => tabs::internal::render(f, app, content_area),
        AppState::Css => tabs::css::render(f, app, content_area),
        AppState::Javascript => tabs::javascript::render(f, app, content_area),
        AppState::CoreWebVitals => tabs::cwv::render(f, app, content_area),
        AppState::CustomExtractor => tabs::custom_extractor::render(f, app, content_area),
        AppState::Images => tabs::images::render(f, app, content_area),
        AppState::Redirects => tabs::redirects::render(f, app, content_area),
        AppState::Keywords => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" {:?} ", app.current_state))
                .border_style(Style::default().fg(border_color));
            let content = Paragraph::new("Feature coming soon...")
                .block(block)
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(content, content_area);
        }
        AppState::Reports => tabs::reports::render(f, app, content_area),
        AppState::Content => tabs::content::render(f, app, content_area),
        AppState::Files => tabs::files::render(f, app, content_area),
    }

    // Render Footer
    footer::render(f, app, footer_area);

    // Render Modals (Side Panel, Help)
    side_panel::render(f, app);

    if app.show_details {
        modals::details::render(f, app);
    }

    if app.show_dashboard_menu {
        modals::dashboard_menu::render(f, app);
    }

    // SHOW THE OPTIONS MODAL
    if app.options_modal {
        modals::options::render(f, app);
    }

    // Render Input Modal when in input mode
    if app.input_mode {
        let modal_area = centered_rect(25, 6, size);

        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(vec![
                Span::styled(
                    " Crawl URL ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " (Esc to Cancel, Enter to crawl ) ",
                    Style::default().fg(Color::Gray),
                ),
            ])
            .border_style(Style::default().fg(accent_color));

        let input_p = Paragraph::new(app.input.as_str())
            .block(input_block)
            .style(Style::default().bg(Color::Rgb(20, 20, 30)));

        f.render_widget(Clear, modal_area);
        f.render_widget(input_p, modal_area);

        // Make the cursor visible in the modal
        f.set_cursor_position((
            modal_area.x + app.cursor_position as u16 + 1,
            modal_area.y + 1,
        ));
    }

    if app.show_help {
        render_help_modal(f);
    }

    if app.show_logs {
        let area = f.area();
        let height = app.logs_height;
        let logs_area = Rect::new(0, area.height.saturating_sub(height), area.width, height);
        f.render_widget(Clear, logs_area);
        tabs::logs::render(f, app, logs_area);
    }

    if app.show_ai_modal {
        modals::ai_chat::render(f, app);
    }

    if app.show_js_pages_modal {
        modals::js_pages::render(f, app);
    }

    if app.show_css_pages_modal {
        modals::css_pages::render(f, app);
    }

    if app.show_issue_urls_modal {
        modals::issue_urls::render(f, app);
    }
}

fn render_help_modal(f: &mut Frame) {
    let area = f.area();
    let help_area = centered_rect(80, 85, area);
    let accent_color = Color::Rgb(80, 140, 255);
    let header_color = Color::Yellow;
    let key_color = Color::Cyan;

    let block = Block::default()
        .title(Span::styled(
            " RustySEO [CLI] - Shortcut ",
            Style::default()
                .fg(header_color)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD),
        )
        .bg(Color::Rgb(10, 10, 20));

    f.render_widget(Clear, help_area);
    f.render_widget(block.clone(), help_area);

    let inner_area = block.inner(help_area);

    // Split into 3 columns
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .margin(1)
        .split(inner_area);

    // COLUMN 1: GLOBAL NAVIGATION
    let nav_text = vec![
        Line::from(vec![Span::styled(
            "── GLOBAL CONTROLS ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" q       ", Style::default().fg(Color::Red)),
            Span::raw("Quit Application"),
        ]),
        Line::from(vec![
            Span::styled(" ?       ", Style::default().fg(key_color)),
            Span::raw("Toggle Help"),
        ]),
        Line::from(vec![
            Span::styled(" Esc     ", Style::default().fg(key_color)),
            Span::raw("Reset / Close Modals"),
        ]),
        Line::from(vec![
            Span::styled(" A       ", Style::default().fg(key_color)),
            Span::raw("AI Copilot Panel"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── MAIN TABS ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" Tab     ", Style::default().fg(key_color)),
            Span::raw("Next Main Tab"),
        ]),
        Line::from(vec![
            Span::styled(" Sh+Tab  ", Style::default().fg(key_color)),
            Span::raw("Previous Main Tab"),
        ]),
        Line::from(vec![
            Span::styled(" 1-9,0   ", Style::default().fg(key_color)),
            Span::raw("Direct Tab Access"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── SYSTEM TOOLS ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" Ctrl+i  ", Style::default().fg(key_color)),
            Span::raw("Open URL Input"),
        ]),
        Line::from(vec![
            Span::styled(" L       ", Style::default().fg(key_color)),
            Span::raw("Toggle System Logs"),
        ]),
        Line::from(vec![
            Span::styled(" Ctrl+f  ", Style::default().fg(Color::Rgb(255, 170, 0))),
            Span::raw("Search Dashboard/Logs"),
        ]),
        Line::from(vec![
            Span::styled(" [ / ]   ", Style::default().fg(key_color)),
            Span::raw("Previous/Next Page"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── LOGS CONSOLE ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" q/Esc/L ", Style::default().fg(key_color)),
            Span::raw("Close Logs"),
        ]),
        Line::from(vec![
            Span::styled(" k/↑ j/↓ ", Style::default().fg(key_color)),
            Span::raw("Navigate Logs"),
        ]),
        Line::from(vec![
            Span::styled(" t/G     ", Style::default().fg(key_color)),
            Span::raw("Top/Bottom Log"),
        ]),
        Line::from(vec![
            Span::styled(" [ / ]   ", Style::default().fg(key_color)),
            Span::raw("Resize Console"),
        ]),
    ];

    // COLUMN 2: DASHBOARD & MODALS
    let dash_text = vec![
        Line::from(vec![Span::styled(
            "── DASHBOARD TABLE ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" k / ↑   ", Style::default().fg(key_color)),
            Span::raw("Previous Row"),
        ]),
        Line::from(vec![
            Span::styled(" j / ↓   ", Style::default().fg(key_color)),
            Span::raw("Next Row"),
        ]),
        Line::from(vec![
            Span::styled(" t / G   ", Style::default().fg(key_color)),
            Span::raw("Top / Bottom"),
        ]),
        Line::from(vec![
            Span::styled(" Enter   ", Style::default().fg(key_color)),
            Span::raw("View Page Details"),
        ]),
        Line::from(vec![
            Span::styled(" m       ", Style::default().fg(key_color)),
            Span::raw("Actions Context Menu"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── DETAILS MODAL ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" q/Esc   ", Style::default().fg(key_color)),
            Span::raw("Close Details"),
        ]),
        Line::from(vec![
            Span::styled(" h/← Tab ", Style::default().fg(key_color)),
            Span::raw("Previous Tab"),
        ]),
        Line::from(vec![
            Span::styled(" l/→ Sh+Tab", Style::default().fg(key_color)),
            Span::raw("Next Tab"),
        ]),
        Line::from(vec![
            Span::styled(" k/j     ", Style::default().fg(key_color)),
            Span::raw("Navigate Tables"),
        ]),
        Line::from(vec![
            Span::styled(" ↑/↓     ", Style::default().fg(key_color)),
            Span::raw("Scroll Dashboard Table"),
        ]),
        Line::from(vec![
            Span::styled(" Sh+↑/↓  ", Style::default().fg(key_color)),
            Span::raw("Navigate Tab Content"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── SEARCH MODE ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" Enter   ", Style::default().fg(key_color)),
            Span::raw("Apply Filter"),
        ]),
        Line::from(vec![
            Span::styled(" Esc     ", Style::default().fg(key_color)),
            Span::raw("Clear / Close Search"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── AI CHAT MODAL ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" q/Esc   ", Style::default().fg(key_color)),
            Span::raw("Close AI Chat"),
        ]),
        Line::from(vec![
            Span::styled(" Enter   ", Style::default().fg(key_color)),
            Span::raw("Send Message"),
        ]),
        Line::from(vec![
            Span::styled(" Backsp  ", Style::default().fg(key_color)),
            Span::raw("Delete Character"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── DASHBOARD MENU ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" q/Esc   ", Style::default().fg(key_color)),
            Span::raw("Close Menu"),
        ]),
        Line::from(vec![
            Span::styled(" k/↑ j/↓ ", Style::default().fg(key_color)),
            Span::raw("Navigate Items"),
        ]),
        Line::from(vec![
            Span::styled(" Enter   ", Style::default().fg(key_color)),
            Span::raw("Execute Action"),
        ]),
    ];

    // COLUMN 3: SIDEBAR & OTHER
    let sidebar_text = vec![
        Line::from(vec![Span::styled(
            "── SIDEBAR JUMPS ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" g       ", Style::default().fg(key_color)),
            Span::raw("Settings Tab"),
        ]),
        Line::from(vec![
            Span::styled(" s       ", Style::default().fg(key_color)),
            Span::raw("Filters Tab"),
        ]),
        Line::from(vec![
            Span::styled(" f       ", Style::default().fg(key_color)),
            Span::raw("Stats Tab"),
        ]),
        Line::from(vec![
            Span::styled(" a       ", Style::default().fg(key_color)),
            Span::raw("Actions Tab"),
        ]),
        Line::from(vec![
            Span::styled(" b / +   ", Style::default().fg(key_color)),
            Span::raw("Bookmarks Tab"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── SIDEBAR CONTROLS ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" Esc/←/h", Style::default().fg(key_color)),
            Span::raw("Close Sidebar"),
        ]),
        Line::from(vec![
            Span::styled(" k/↑ j/↓ ", Style::default().fg(key_color)),
            Span::raw("Navigate Tabs"),
        ]),
        Line::from(vec![
            Span::styled(" Tab/Sh+Tab", Style::default().fg(key_color)),
            Span::raw("Cycle Tabs"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── BOOKMARK ACTIONS ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" ←/→     ", Style::default().fg(key_color)),
            Span::raw("Navigate Tabs"),
        ]),
        Line::from(vec![
            Span::styled(" ↑/↓     ", Style::default().fg(key_color)),
            Span::raw("Navigate Items"),
        ]),
        Line::from(vec![
            Span::styled(" Enter   ", Style::default().fg(key_color)),
            Span::raw("Add/Crawl Selection"),
        ]),
        Line::from(vec![
            Span::styled(" Esc     ", Style::default().fg(key_color)),
            Span::raw("Clear/Close"),
        ]),
        Line::from(vec![
            Span::styled(" D       ", Style::default().fg(key_color)),
            Span::raw("Delete Bookmark"),
        ]),
        Line::from(vec![
            Span::styled(" Backsp  ", Style::default().fg(key_color)),
            Span::raw("Delete Character"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── SETTINGS ACTIONS ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" E       ", Style::default().fg(key_color)),
            Span::raw("Edit Settings File"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── INPUT MODE ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" Enter   ", Style::default().fg(key_color)),
            Span::raw("Submit URL"),
        ]),
        Line::from(vec![
            Span::styled(" Esc     ", Style::default().fg(key_color)),
            Span::raw("Cancel Input"),
        ]),
        Line::from(vec![
            Span::styled(" ←/→     ", Style::default().fg(key_color)),
            Span::raw("Scroll Text"),
        ]),
        Line::from(vec![
            Span::styled(" Backsp  ", Style::default().fg(key_color)),
            Span::raw("Delete Character"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "── DASHBOARD MENU ──",
            Style::default().fg(header_color).bold(),
        )]),
        Line::from(vec![
            Span::styled(" q/Esc   ", Style::default().fg(key_color)),
            Span::raw("Close Menu"),
        ]),
        Line::from(vec![
            Span::styled(" k/↑ j/↓ ", Style::default().fg(key_color)),
            Span::raw("Navigate Items"),
        ]),
        Line::from(vec![
            Span::styled(" Enter   ", Style::default().fg(key_color)),
            Span::raw("Execute Action"),
        ]),
    ];

    f.render_widget(
        Paragraph::new(nav_text).style(Style::default().fg(Color::Gray)),
        cols[0],
    );
    f.render_widget(
        Paragraph::new(dash_text).style(Style::default().fg(Color::Gray)),
        cols[1],
    );
    f.render_widget(
        Paragraph::new(sidebar_text).style(Style::default().fg(Color::Gray)),
        cols[2],
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
