use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs, Paragraph},
    Frame,
};

use crate::app::{App, AppState};

pub mod tabs;
pub mod side_panel;
pub mod footer;
pub mod modals;

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Main layout: Input (3 if active) + Navigation (3) + Content Area (Min 0) + Footer (3)
    let input_height = if app.input_mode { 3 } else { 0 };
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(input_height),
            Constraint::Length(3), // Navigation
            Constraint::Min(0),    // Content Area
            Constraint::Length(3), // Footer
        ])
        .split(size);

    let input_area = main_layout[0];
    let tab_area = main_layout[1];
    let content_area = main_layout[2];
    let footer_area = main_layout[3];

    app.tab_rect = Some(tab_area);

    // Render Input Bar ONLY when in input mode
    if app.input_mode {
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(" Command / URL Input (Press Esc to Cancel) ")
            .border_style(Style::default().fg(Color::Blue));

        let input_p = Paragraph::new(app.input.as_str()).block(input_block);
        f.render_widget(input_p, input_area);

        // Make the cursor visible
        f.set_cursor(
            input_area.x + app.cursor_position as u16 + 1,
            input_area.y + 1,
        );
    }

    // Render Navigation Tabs
    let titles = vec!["Crawl", "Logs", "Connectors", "Dashboard", "Reports", "Chat"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Navigation "))
        .select(app.get_state_index())
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, tab_area);

    // Render Tab Content
    match app.current_state {
        AppState::Crawl => tabs::crawl::render(f, app, content_area),
        AppState::Logs => tabs::logs::render(f, app, content_area),
        AppState::Connectors => tabs::connectors::render(f, app, content_area),
        AppState::Dashboard => tabs::dashboard::render(f, app, content_area),
        AppState::Reports => tabs::reports::render(f, app, content_area),
        AppState::Chat => tabs::chat::render(f, app, content_area),
    }

    // Render Footer
    footer::render(f, app, footer_area);

    // Render Modals (Side Panel, Help)
    side_panel::render(f, app);

    if app.show_details {
        modals::details::render(f, app);
    }

    if app.show_help {
        render_help_modal(f);
    }
}

fn render_help_modal(f: &mut Frame) {
    let area = f.size();
    let help_area = centered_rect(70, 70, area);
    let block = Block::default()
        .title(" Help / Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
        .bg(Color::Black);

    let help_text = vec![
        Line::from(vec![Span::styled("Navigation:", Style::default().add_modifier(Modifier::BOLD).fg(Color::LightBlue))]),
        Line::from("  h / ←  : Previous Tab / Close Sidebar"),
        Line::from("  l / →  : Next Tab / Open Sidebar"),
        Line::from("  k / ↑  : Scroll Up / Prev Sidebar Item"),
        Line::from("  j / ↓  : Scroll Down / Next Sidebar Item"),
        Line::from("  Tab    : Cycle Active Window"),
        Line::from(""),
        Line::from(vec![Span::styled("Shortcuts:", Style::default().add_modifier(Modifier::BOLD).fg(Color::LightBlue))]),
        Line::from("  Ctrl+i : Focus Input Bar"),
        Line::from("  Enter  : Show Row Details (Dashboard) / Submit Input"),
        Line::from("  ?      : Toggle Help"),
        Line::from("  Esc    : Close Modals / Reset View"),
        Line::from("  q      : Quit"),
        Line::from(""),
        Line::from(vec![Span::styled("Tools:", Style::default().add_modifier(Modifier::BOLD).fg(Color::LightBlue))]),
        Line::from("  s: Settings | f: Filters | i: Stats | a: Actions"),
    ];

    let p = Paragraph::new(help_text).block(block);


    f.render_widget(ratatui::widgets::Clear, help_area);
    f.render_widget(p, help_area);
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
