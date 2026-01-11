use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};
use serde_json;

pub fn render(f: &mut Frame, schema: &[String], scroll: u16, area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let mut content = vec![
        Line::from(vec![Span::styled(
            " 📋 Schema Markup ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
    ];

    if schema.is_empty() {
        content.push(Line::from(vec![Span::styled(
            "No schema markup found on this page.",
            Style::default().fg(Color::Gray),
        )]));
    } else {
        content.push(Line::from(vec![Span::styled(
            format!("Found {} schema(s):", schema.len()),
            Style::default().fg(Color::Cyan),
        )]));
        content.push(Line::from(""));

        for (i, s) in schema.iter().enumerate() {
            content.push(Line::from(vec![Span::styled(
                format!("Schema {}:", i + 1),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            // Try to pretty print JSON
            let lines: Vec<String> = match serde_json::from_str::<serde_json::Value>(s) {
                Ok(json) => {
                    let pretty = serde_json::to_string_pretty(&json).unwrap_or(s.clone());
                    pretty.lines().map(|l| l.to_string()).collect()
                }
                Err(_) => {
                    // If not valid JSON, show raw
                    s.lines().map(|l| l.to_string()).collect()
                }
            };
            for line in lines {
                content.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(line, Style::default().fg(Color::Rgb(180, 120, 255))),
                ]));
            }
            content.push(Line::from(""));
        }
    }

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " Schema Markup Details ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true })
        .scroll((scroll as u16, 0));
    f.render_widget(p, area);
}
