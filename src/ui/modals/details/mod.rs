use crate::{models::App, ui::centered_rect};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
};

mod modal_tabs;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let detail_area = centered_rect(80, 80, area);

    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .bg(Color::Rgb(15, 15, 25));

    let inner_area = modal_block.inner(detail_area);

    f.render_widget(Clear, detail_area);
    f.render_widget(modal_block, detail_area);

    let selected_idx = app.table_state.selected().unwrap_or(0);
    // Ensure we don't out of bounds if data changed
    if selected_idx >= app.table_data.len() {
        return;
    }
    let row_data = &app.table_data[selected_idx];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    // Render Tabs
    let titles = vec![
        " 📄 General ",
        " 📊 Analysis ",
        " ✅ Checklist ",
        " 🔗 Inlinks ",
        " ↗️  Outlinks ",
        " 🖼️  Images ",
        " 📋 Schema ",
        " 📨 Headers ",
        " 📝 Headings ",
    ];
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Page Details: ID {} ", row_data[0]),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .bg(Color::Rgb(15, 15, 25)),
        )
        .select(app.detail_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .divider(Span::styled(" | ", Style::default().fg(border_color)));

    f.render_widget(tabs, chunks[0]);

    // Render Content based on tab
    let content_block = Block::default().bg(Color::Rgb(20, 20, 30));

    match app.detail_tab {
        0 => modal_tabs::general::render(f, row_data, chunks[1], content_block),
        1 => modal_tabs::analysis::render(f, row_data, chunks[1], content_block),
        2 => modal_tabs::checklist::render(f, row_data, chunks[1], content_block),
        3 => modal_tabs::inlinks::render(f, chunks[1], content_block),
        4 => modal_tabs::outlinks::render(f, chunks[1], content_block),
        5 => modal_tabs::images::render(f, chunks[1], content_block),
        6 => modal_tabs::schema::render(f, chunks[1], content_block),
        7 => modal_tabs::headers::render(f, chunks[1], content_block),
        8 => modal_tabs::headings::render(
            f,
            &app.page_data[selected_idx].headings,
            chunks[1],
            content_block,
        ),
        _ => {}
    }

    // Render Footer
    let footer_block = Block::default()
        .bg(Color::Rgb(15, 15, 25))
        .border_style(Style::default().fg(border_color));
    let footer_text = Paragraph::new(Span::styled(
        " Tab: Next Tab | Shift+Tab: Prev Tab | Q/Esc: Close ",
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::ITALIC),
    ))
    .block(footer_block)
    .alignment(Alignment::Center);
    f.render_widget(footer_text, chunks[2]);
}
