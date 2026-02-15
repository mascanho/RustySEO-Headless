use crate::{models::App, ui::centered_rect};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Tabs},
};

mod modal_tabs;

const ACCENT_COLOR: Color = Color::Rgb(80, 140, 255);
const BORDER_COLOR: Color = Color::Rgb(40, 45, 60);

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

    let page_details = match &app.selected_page_details {
        Some(details) => details,
        None => {
            app.show_details = false;
            return;
        }
    };

    let row_data = &app.filtered_table_data[selected_idx];

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
                    format!(" {} ", row_data[1]),
                    Style::default()
                        .fg(Color::White)
                        .bg(ACCENT_COLOR)
                        .bold()
                        .add_modifier(Modifier::ITALIC)
                        .add_modifier(Modifier::RAPID_BLINK),
                ))
                .bg(Color::Rgb(15, 15, 25))
                .padding(Padding::new(0, 0, 1, 0)),
        )
        .select(app.detail_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .bg(ACCENT_COLOR)
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
            &page_details.canonicals,
            app.detail_scroll,
            chunks[1],
            content_block,
        ),
        1 => modal_tabs::analysis::render(f, row_data, chunks[1], content_block),
        2 => modal_tabs::checklist::render(f, row_data, chunks[1], content_block),
        3 => modal_tabs::inlinks::render(
            f,
            &page_details.anchor_links,
            app.detail_horizontal_scroll,
            &mut app.detail_table_state,
            chunks[1],
            content_block,
        ),
        4 => modal_tabs::outlinks::render(
            f,
            &page_details.outlinks,
            app.detail_horizontal_scroll,
            &mut app.detail_table_state,
            chunks[1],
            content_block,
        ),
        5 => modal_tabs::images::render(
            f,
            &page_details.images,
            app.detail_horizontal_scroll,
            &mut app.detail_table_state,
            chunks[1],
            content_block,
        ),
        6 => {
            let schema_block = Block::default().bg(Color::Rgb(25, 15, 35));
            modal_tabs::schema::render(
                f,
                &page_details.schema.clone(),
                app.detail_scroll,
                chunks[1],
                schema_block,
            );
        }
        7 => modal_tabs::headers::render(
            f,
            &page_details.headers.clone(),
            app.detail_scroll,
            chunks[1],
            content_block,
        ),
        8 => modal_tabs::headings::render(
            f,
            &page_details.headings.clone(),
            app.detail_horizontal_scroll,
            &mut app.detail_table_state,
            chunks[1],
            content_block,
        ),
        _ => {}
    }

    // Render Footers
    let _url = &row_data[1];
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

    let status_line = Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            status,
            Style::default()
                .bg(Color::Rgb(10, 10, 20))
                .fg(status_color)
                .bold(),
        ),
        Span::raw(" "),
    ]);

    let status_p = Paragraph::new(status_line)
        .block(Block::default().bg(Color::Rgb(10, 10, 20)))
        .alignment(Alignment::Right);

    // File Size
    let file_size = page_details.size as u64;

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
        .block(Block::default().bg(Color::Rgb(10, 10, 20))) // Purple
        .alignment(Alignment::Center);

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
