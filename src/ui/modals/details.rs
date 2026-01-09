use crate::{models::App, ui::centered_rect};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap},
};

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let detail_area = centered_rect(80, 80, area);

    f.render_widget(Clear, detail_area);

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
        ])
        .split(detail_area);

    // Render Tabs
    let titles = vec![" General ", " SEO Analysis ", " Checklist "];
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Page Details: ID {} ", row_data[0])),
        )
        .select(app.detail_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, chunks[0]);

    // Render Content based on tab
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .bg(Color::Black);

    match app.detail_tab {
        0 => render_general(f, row_data, chunks[1], content_block),
        1 => render_analysis(f, row_data, chunks[1], content_block),
        2 => render_checklist(f, row_data, chunks[1], content_block),
        _ => {}
    }
}

fn render_general(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let info = vec![
        Line::from(vec![
            Span::styled(
                "URL: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Blue),
            ),
            Span::raw(&row_data[1]),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Page Title: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Blue),
            ),
            Span::raw(&row_data[2]),
        ]),
        Line::from(vec![
            Span::styled(
                "H1 Heading: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Blue),
            ),
            Span::raw(&row_data[4]),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Meta Description: ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Blue),
        )]),
        Line::from(vec![Span::raw(&row_data[6])]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Crawl Status: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Blue),
            ),
            Span::raw(&row_data[8]),
        ]),
    ];
    let p = Paragraph::new(info)
        .block(block.title(" General Info "))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_analysis(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let analysis = vec![
        Line::from(vec![Span::styled(
            "Content Analysis:",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Blue),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Title Length: ", Style::default().fg(Color::LightBlue)),
            Span::raw(&row_data[3]),
            Span::raw(" chars"),
        ]),
        Line::from(vec![
            Span::styled("H1 Length:    ", Style::default().fg(Color::LightBlue)),
            Span::raw(&row_data[5]),
            Span::raw(" chars"),
        ]),
        Line::from(vec![
            Span::styled("Desc Length:  ", Style::default().fg(Color::LightBlue)),
            Span::raw(&row_data[7]),
            Span::raw(" chars"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Word Count:   ", Style::default().fg(Color::LightBlue)),
            Span::raw("~842 words (Simulated)"),
        ]),
        Line::from(vec![
            Span::styled("Readability:  ", Style::default().fg(Color::LightBlue)),
            Span::raw("High (Flesch Score: 72)"),
        ]),
    ];
    let p = Paragraph::new(analysis)
        .block(block.title(" SEO Analysis "))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_checklist(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let checklist = vec![
        Line::from(vec![Span::styled(
            "SEO Health Checklist:",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Blue),
        )]),
        Line::from(""),
        Line::from(if row_data[2].len() > 60 {
            " ❌ Title too long (>60 chars)"
        } else {
            " ✅ Title length optimal"
        }),
        Line::from(if row_data[6].len() > 160 {
            " ❌ Meta description too long"
        } else {
            " ✅ Meta description length optimal"
        }),
        Line::from(if row_data[4].is_empty() {
            " ❌ Missing H1 heading"
        } else {
            " ✅ H1 heading present"
        }),
        Line::from(if row_data[8].contains("200") {
            " ✅ HTTP Status OK"
        } else {
            " ❌ HTTP Status Issue"
        }),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Recommendations:",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::LightBlue),
        )]),
        Line::from(" • Ensure the focus keyword is present in the first 100 words."),
        Line::from(" • Add ALT tags to all images (4 missing)."),
    ];
    let p = Paragraph::new(checklist)
        .block(block.title(" Automated Checklist "))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
