use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Row, Table},
};

pub fn render(f: &mut Frame, anchor_links: &[(String, String)], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    // Create header row
    let header = Row::new(vec![
        Span::styled(
            "🔗 Link",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "📝 Anchor Text",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    // Create data rows
    let rows: Vec<Row> = anchor_links
        .iter()
        .enumerate()
        .map(|(i, (href, text))| {
            let link_color = if i % 2 == 0 {
                Color::Rgb(180, 120, 255)
            } else {
                Color::Rgb(120, 180, 255)
            };
            let text_color = if i % 2 == 0 {
                Color::Rgb(255, 180, 120)
            } else {
                Color::Rgb(255, 120, 180)
            };
            Row::new(vec![
                Span::styled(href.clone(), Style::default().fg(link_color)),
                Span::styled(text.clone(), Style::default().fg(text_color)),
            ])
        })
        .collect();

    // Create table
    let table = Table::new(
        rows,
        &[Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .header(header)
    .block(
        block
            .title(Span::styled(
                format!(" 🔗 Outgoing Links ({}) ", anchor_links.len()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(Style::default().fg(accent_color)),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .column_spacing(1);

    f.render_widget(table, area);
}
