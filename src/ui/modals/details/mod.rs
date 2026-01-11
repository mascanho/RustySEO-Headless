use crate::{models::App, tui_dbg, tui_println, ui::centered_rect};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
};

mod modal_tabs;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let detail_area = centered_rect(60, 70, area);

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
    if selected_idx >= app.table_data.len() || selected_idx >= app.page_data.len() {
        app.show_details = false; // Close modal if data is invalid
        return;
    }
    let row_data = &app.table_data[selected_idx];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // top Footer
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
                    format!(" Page Details"),
                    Style::default()
                        .fg(Color::Yellow)
                        .bold()
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
        3 => modal_tabs::inlinks::render(
            f,
            &app.page_data[selected_idx].anchor_links,
            app.detail_horizontal_scroll,
            &mut app.detail_table_state,
            chunks[1],
            content_block,
        ),
        4 => modal_tabs::outlinks::render(f, chunks[1], content_block),
        5 => modal_tabs::images::render(f, chunks[1], content_block),
        6 => {
            let schema_block = Block::default().bg(Color::Rgb(25, 15, 35));
            modal_tabs::schema::render(
                f,
                &app.page_data[selected_idx].schema.clone(),
                app.detail_scroll,
                chunks[1],
                schema_block,
            );
        }
        7 => modal_tabs::headers::render(
            f,
            &app.page_data[selected_idx].headers.clone(),
            chunks[1],
            content_block,
        ),
        8 => modal_tabs::headings::render(
            f,
            &app.page_data[selected_idx].headings.clone(),
            app.detail_scroll,
            chunks[1],
            content_block,
        ),
        _ => {}
    }

    // Render Footers

    let footer_top = Block::default()
        .bg(Color::Rgb(15, 15, 25))
        .border_style(Style::default().fg(border_color));

    let url = &row_data[1];
    let status = &row_data[10];

    let footer_top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),     // URL on left
            Constraint::Length(15), // Status on right
        ])
        .split(chunks[2]);

    let url_line = Line::from(vec![
        Span::styled("URL: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            url,
            Style::default()
                .fg(Color::Rgb(180, 120, 255))
                .add_modifier(Modifier::ITALIC),
        ),
    ]);
    let url_paragraph = Paragraph::new(url_line)
        .block(footer_top.clone())
        .alignment(Alignment::Left);
    f.render_widget(url_paragraph, footer_top_chunks[0]);

    let status_code = status
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse::<u16>()
        .unwrap_or(0);
    let status_color = match status_code / 100 {
        1 => Color::Blue,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Red,
        5 => Color::Rgb(255, 0, 255), // Magenta
        _ => Color::Gray,
    };
    let status_line = Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Yellow)),
        Span::styled(status, Style::default().fg(status_color)),
    ]);
    let status_paragraph = Paragraph::new(status_line)
        .block(footer_top)
        .alignment(Alignment::Right);
    f.render_widget(status_paragraph, footer_top_chunks[1]);

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
    f.render_widget(footer_text, chunks[3]);
}
