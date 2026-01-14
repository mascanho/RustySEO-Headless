use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Clear, Tabs},
};

use crate::models::App;
use crate::ui::sidebar::{actions, bookmarks, filters, settings, summary};

pub fn render(f: &mut Frame, app: &mut App) {
    if !app.sidebar_visible {
        return;
    }

    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    let area = f.area();

    let width = (area.width / 3).max(35).min(area.width);
    let modal_area = Rect {
        x: area.width.saturating_sub(width),
        y: 0,
        width,
        height: area.height,
    };

    f.render_widget(Clear, modal_area);

    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(modal_area);

    let sidebar_tab_area = sidebar_chunks[0];
    let sidebar_content_area = sidebar_chunks[1];

    app.sidebar_tab_rect = Some(sidebar_tab_area);

    let sidebar_titles = vec!["Summary", "Settings", "Filter", "Act", "Bookmarks"];
    let sidebar_tabs = Tabs::new(sidebar_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    " Tools ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(border_color))
                .bg(Color::Rgb(15, 15, 25)),
        )
        .select(app.sidebar_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .divider(Span::styled(" | ", Style::default().fg(border_color)));

    f.render_widget(sidebar_tabs, sidebar_tab_area);

    // Sidebar Content based on tab
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .bg(Color::Rgb(15, 15, 25));

    match app.sidebar_tab {
        0 => summary::render(f, app, sidebar_content_area, content_block, accent_color),
        1 => settings::render(
            f,
            app,
            sidebar_content_area,
            content_block,
            accent_color,
            border_color,
        ),
        2 => filters::render(f, app, sidebar_content_area, content_block),
        3 => actions::render(f, app, sidebar_content_area, content_block),
        4 => bookmarks::render(
            f,
            app,
            sidebar_content_area,
            content_block,
            accent_color,
            border_color,
        ),
        _ => {}
    }
}
