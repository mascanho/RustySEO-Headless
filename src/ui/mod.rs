use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::app::{App, AppState};

pub mod tabs;
pub mod side_panel;

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Main layout: Top Navigation + Content Area
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size);

    let tab_area = main_layout[0];
    let content_area = main_layout[1];

    app.tab_rect = Some(tab_area);

    // Render Navigation Tabs
    let titles = vec!["Crawl", "Logs", "Connectors", "Dashboard", "Reports", "Chat"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Navigation "))
        .select(app.get_state_index())
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
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

    // Render Modals (Side Panel, Help)
    side_panel::render(f, app);

    if app.show_help {
        render_help_modal(f);
    }
}

fn render_help_modal(f: &mut Frame) {
    let area = f.size();
    let help_area = centered_rect(70, 70, area);
    
    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .bg(Color::Black);

    let text = vec![
        ratatui::text::Line::from(vec![ratatui::text::Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))]),
        ratatui::text::Line::from("  h / l | ← / →  - Change Main Tabs / Toggle Panel"),
        ratatui::text::Line::from("  j / k | ↓ / ↑  - Navigate Tool Panel Tabs"),
        ratatui::text::Line::from("  1 - 6          - Direct Main Tab Access"),
        ratatui::text::Line::from("  Tab / S-Tab    - Cycle Main Tabs"),
        ratatui::text::Line::from(""),
        ratatui::text::Line::from(vec![ratatui::text::Span::styled("Tool Shortcuts", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))]),
        ratatui::text::Line::from("  s              - Jump to Crawler Settings"),
        ratatui::text::Line::from("  f              - Jump to Result Filters"),
        ratatui::text::Line::from("  i              - Jump to Live Stats"),
        ratatui::text::Line::from("  a              - Jump to Quick Actions"),
        ratatui::text::Line::from(""),
        ratatui::text::Line::from(vec![ratatui::text::Span::styled("General", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))]),
        ratatui::text::Line::from("  ?              - Toggle this Help Menu"),
        ratatui::text::Line::from("  Esc            - Reset View / Close Help"),
        ratatui::text::Line::from("  q              - Quit Application"),
    ];

    let p = ratatui::widgets::Paragraph::new(text)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: true });

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
