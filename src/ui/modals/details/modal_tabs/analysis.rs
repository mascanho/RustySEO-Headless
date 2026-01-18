use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

pub fn render(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Metrics
            Constraint::Min(12),    // Keywords table
        ])
        .split(area.inner(Margin::new(1, 1)));

    let analysis = vec![
        Line::from(vec![Span::styled(
            " Content Metrics ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Title Length:   ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[3], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(vec![
            Span::styled("  H1 Length:      ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[5], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(vec![
            Span::styled("  Desc Length:    ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[7], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Word Count:     ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[18], Style::default().fg(Color::White)),
            Span::raw(" words "),
        ]),
    ];

    let p = Paragraph::new(analysis)
        .block(Block::default())
        .wrap(Wrap { trim: true });
    f.render_widget(p, chunks[0]);

    // Keywords Table
    let mut keyword_rows = Vec::new();
    for i in 0..10 {
        let idx = 23 + i;
        if let Some(kw_str) = row_data.get(idx) {
            if !kw_str.is_empty() {
                // kw_str is "(count) word"
                let parts: Vec<&str> = kw_str.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    let freq = parts[0].trim_matches(|c| c == '(' || c == ')');
                    let word = parts[1];
                    keyword_rows.push(Row::new(vec![
                        Cell::from(format!(" #{} ", i + 1)),
                        Cell::from(format!(" {} ", word)).style(Style::default().fg(Color::Cyan)),
                        Cell::from(format!(" {} ", freq)).style(Style::default().fg(Color::Yellow).bold()),
                    ]));
                }
            }
        }
    }

    let header = Row::new(vec![
        Cell::from(" Rank ").style(Style::default().fg(accent_color).bold()),
        Cell::from(" Keyword ").style(Style::default().fg(accent_color).bold()),
        Cell::from(" Freq ").style(Style::default().fg(accent_color).bold()),
    ])
    .height(1)
    .bottom_margin(1);

    let table = Table::new(
        keyword_rows,
        [
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(40, 45, 60)))
            .title(Span::styled(
                " Top 10 Keywords ",
                Style::default().fg(Color::Yellow).bold(),
            )),
    )
    .column_spacing(1)
    .style(Style::default().bg(Color::Rgb(20, 20, 30)));

    f.render_widget(table, chunks[1]);

    // Outer block
    f.render_widget(block, area);
}
