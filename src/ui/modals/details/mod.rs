use crate::{models::App, ui::centered_rect};
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
    let border_color = accent_color;

    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .bg(Color::Rgb(15, 15, 25));

    let inner_area = modal_block.inner(detail_area);

    f.render_widget(Clear, detail_area);
    f.render_widget(modal_block, detail_area);

    let selected_idx = app.table_state.selected().unwrap_or(0);

    // Always use filtered_table_data as it is synchronized with table_data when no filter is active
    if selected_idx >= app.filtered_table_data.len() {
        app.show_details = false;
        return;
    }

    let row_data = &app.filtered_table_data[selected_idx];

    // The first column (index 0) of the row data contains the original persistent ID
    let original_id = row_data[0].parse::<usize>().unwrap_or(1);
    let page_idx = original_id.saturating_sub(1);

    if page_idx >= app.page_data.len() {
        app.show_details = false;
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    // Render Tabs
    let titles = vec![
        "General",
        "Analysis",
        "Checklist",
        "Inlinks",
        "Outlinks",
        "Images",
        "Schema",
        "Headers",
        "Headings",
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
        .divider(Span::styled("|", Style::default().fg(border_color)));

    f.render_widget(tabs, chunks[0]);

    // Render Content based on tab
    let content_block = Block::default().bg(Color::Rgb(20, 20, 30));
    match app.detail_tab {
        0 => modal_tabs::general::render(
            f,
            row_data,
            &app.page_data[page_idx].canonicals,
            app.detail_scroll,
            chunks[1],
            content_block,
        ),
        1 => modal_tabs::analysis::render(f, row_data, chunks[1], content_block),
        2 => modal_tabs::checklist::render(f, row_data, chunks[1], content_block),
        3 => modal_tabs::inlinks::render(
            f,
            &app.page_data[page_idx].anchor_links,
            app.detail_horizontal_scroll,
            &mut app.detail_table_state,
            chunks[1],
            content_block,
        ),
        4 => modal_tabs::outlinks::render(
            f,
            &app.page_data[page_idx].outlinks,
            app.detail_horizontal_scroll,
            &mut app.detail_table_state,
            chunks[1],
            content_block,
        ),
        5 => modal_tabs::images::render(
            f,
            &app.page_data[page_idx].images,
            app.detail_horizontal_scroll,
            &mut app.detail_table_state,
            chunks[1],
            content_block,
        ),
        6 => {
            let schema_block = Block::default().bg(Color::Rgb(25, 15, 35));
            modal_tabs::schema::render(
                f,
                &app.page_data[page_idx].schema.clone(),
                app.detail_scroll,
                chunks[1],
                schema_block,
            );
        }
        7 => modal_tabs::headers::render(
            f,
            &app.page_data[page_idx].headers.clone(),
            app.detail_scroll,
            chunks[1],
            content_block,
        ),
        8 => modal_tabs::headings::render(
            f,
            &app.page_data[page_idx].headings.clone(),
            app.detail_horizontal_scroll,
            &mut app.detail_table_state,
            chunks[1],
            content_block,
        ),
        _ => {}
    }

    // Render Footers
    let url = &row_data[1];
    let status = &row_data[10];

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

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(25),
            Constraint::Length(20),
        ])
        .split(chunks[2]);

    let url_line = Line::from(vec![
        Span::styled(" 🔗 ", Style::default().fg(Color::Yellow)),
        Span::styled(
            url,
            Style::default()
                .fg(Color::Rgb(180, 150, 255))
                .add_modifier(Modifier::ITALIC),
        ),
    ]);

    let status_line = Line::from(vec![
        Span::styled("⚡ Status: ", Style::default().fg(Color::Yellow)),
        Span::styled(status, Style::default().fg(status_color).bold()),
        Span::raw(" "),
    ]);

    let url_p = Paragraph::new(url_line)
        .block(Block::default().bg(Color::Black))
        .alignment(Alignment::Left);

    let status_p = Paragraph::new(status_line)
        .block(Block::default().bg(Color::Black))
        .alignment(Alignment::Right);

    // File Size
    let file_size = app.page_data[page_idx].size as u64;

    let format_size = |size: u64| -> String {
        if size >= 1024 * 1024 {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        } else if size >= 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{} B", size)
        }
    };

    let file_size_line = Line::from(vec![
        Span::styled("📁 ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format_size(file_size),
            Style::default().fg(Color::White).bold(),
        ),
    ]);

    let file_size_p = Paragraph::new(file_size_line)
        .block(Block::default().bg(Color::Rgb(128, 0, 128))) // Purple
        .alignment(Alignment::Center);

    f.render_widget(url_p, footer_chunks[0]);
    f.render_widget(status_p, footer_chunks[1]);
    f.render_widget(file_size_p, footer_chunks[2]);

    let footer_bottom = Paragraph::new(Span::styled(
        " 💡 Tab: Next | Shift+Tab: Prev | Esc: Close ",
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    ))
    .block(Block::default().bg(Color::Rgb(10, 10, 20)))
    .alignment(Alignment::Center);
    f.render_widget(footer_bottom, chunks[3]);
}
