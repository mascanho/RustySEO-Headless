use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

use crate::{app::AppState, models::App};

pub mod footer;
pub mod modals;
pub mod side_panel;
pub mod tabs;

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Define main colors
    let bg_color = Color::Rgb(15, 15, 25);
    
    // Render full background first to ensure consistency
    f.render_widget(Block::default().bg(bg_color), size);

    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

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
            .title(vec![
                Span::styled(" 🔍 Command / URL Input ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" (Esc to Cancel) ", Style::default().fg(Color::Gray)),
            ])
            .border_style(Style::default().fg(accent_color));

        let input_p = Paragraph::new(app.input.as_str()).block(input_block);
        f.render_widget(input_p, input_area);

        // Make the cursor visible
        f.set_cursor(
            input_area.x + app.cursor_position as u16 + 1,
            input_area.y + 1,
        );
    }

    // Render Navigation Tabs
    let titles = vec![
        "🕸️ Crawl",
        "📄 Logs",
        "🔌 Connectors",
        "📊 Dashboard",
        "📈 Reports",
        "💬 Chat",
    ];
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🚀 ATALAIA SEO ")
                .border_style(Style::default().fg(border_color))
        )
        .select(app.get_state_index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .divider(Span::styled(" | ", Style::default().fg(border_color)));

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
    let help_area = centered_rect(60, 60, area);
    
    let accent_color = Color::Rgb(80, 140, 255);
    
    let block = Block::default()
        .title(Span::styled(" ⌨️  Help / Shortcuts ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD),
        )
        .bg(Color::Rgb(20, 20, 30));

    let help_text = vec![
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(vec![Span::styled("  h / l   ", Style::default().fg(Color::Cyan)), Span::raw(": Prev/Next Tab / Toggle Sidebar")]),
        Line::from(vec![Span::styled("  k / j   ", Style::default().fg(Color::Cyan)), Span::raw(": Prev/Next Sidebar Item")]),
        Line::from(vec![Span::styled("  ↑ / ↓   ", Style::default().fg(Color::Cyan)), Span::raw(": Vertical Scroll (Dashboard/Logs)")]),
        Line::from(vec![Span::styled("  ← / →   ", Style::default().fg(Color::Cyan)), Span::raw(": Horizontal Scroll (Dashboard)")]),
        Line::from(vec![Span::styled("  Tab     ", Style::default().fg(Color::Cyan)), Span::raw(": Cycle Active Window")]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Shortcuts",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(vec![Span::styled("  Ctrl+i  ", Style::default().fg(Color::Cyan)), Span::raw(": Focus Input Bar")]),
        Line::from(vec![Span::styled("  Enter   ", Style::default().fg(Color::Cyan)), Span::raw(": Show Row Details / Submit Input")]),
        Line::from(vec![Span::styled("  ?       ", Style::default().fg(Color::Cyan)), Span::raw(": Toggle Help")]),
        Line::from(vec![Span::styled("  Esc     ", Style::default().fg(Color::Cyan)), Span::raw(": Close Modals / Reset View")]),
        Line::from(vec![Span::styled("  q       ", Style::default().fg(Color::Red)), Span::raw(": Quit")]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Quick Tools",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(vec![
            Span::styled("  s", Style::default().fg(Color::Cyan)), Span::raw(": Settings | "),
            Span::styled("f", Style::default().fg(Color::Cyan)), Span::raw(": Filters  | "),
            Span::styled("i", Style::default().fg(Color::Cyan)), Span::raw(": Stats    | "),
            Span::styled("a", Style::default().fg(Color::Cyan)), Span::raw(": Actions"),
        ]),
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

