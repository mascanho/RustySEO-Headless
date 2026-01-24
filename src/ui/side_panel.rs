use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Clear, Tabs},
    Frame,
};

use crate::models::App;
use crate::ui::sidebar::{actions, bookmarks, issues, settings, summary, tree_view};

/// Standard colors for consistency
const ACCENT_COLOR: Color = Color::Rgb(80, 140, 255);
const BORDER_COLOR: Color = Color::Rgb(40, 45, 60);

pub fn render(f: &mut Frame, app: &mut App) {
    if !app.sidebar_visible {
        return;
    }

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

    let sidebar_titles = vec![
        "General",
        "Issues",
        "Settings",
        "Filter",
        "Act",
        "Bookmarks",
        "Tree",
    ];
    let sidebar_tabs = Tabs::new(sidebar_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    " SIDEBAR ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(BORDER_COLOR))
                .bg(Color::Rgb(15, 15, 25)),
        )
        .select(app.sidebar_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(ACCENT_COLOR)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .divider(Span::styled(" | ", Style::default().fg(BORDER_COLOR)));

    f.render_widget(sidebar_tabs, sidebar_tab_area);

    // Sidebar Content based on tab
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR))
        .bg(Color::Rgb(15, 15, 25));

    match app.sidebar_tab {
        0 => summary::render(f, app, sidebar_content_area, content_block, ACCENT_COLOR),
        1 => issues::render(f, app, sidebar_content_area, content_block),
        2 => settings::render(
            f,
            app,
            sidebar_content_area,
            content_block,
            ACCENT_COLOR,
            BORDER_COLOR,
        ),
        3 => actions::render(f, app, sidebar_content_area, content_block),
        4 => bookmarks::render(
            f,
            app,
            sidebar_content_area,
            content_block,
            ACCENT_COLOR,
            BORDER_COLOR,
        ),
        5 => tree_view::render(f, app, sidebar_content_area, content_block),
        _ => {}
    }
}
