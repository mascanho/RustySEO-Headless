use std::fmt::format;

use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

pub fn render(
    f: &mut Frame,
    row_data: &[String],
    canonicals: &[(String, String, Option<String>)],
    scroll: u16,
    area: Rect,
    block: Block,
) {
    let accent_color = Color::Rgb(80, 140, 255);

    let mut content = vec![
        // PAGE INFORMATION Section
        Line::from(Span::styled(
            "PAGE INFORMATION",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "URL: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[1]),
        ]),
        Line::from(vec![
            Span::styled(
                "Title: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[2]),
            Span::styled(
                format!(" ({} chars) ", row_data[3]),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        // META DESCRIPTION Section
        Line::from(vec![
            Span::styled(
                "META DESCRIPTION",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(
                format!(" ({} chars) ", row_data[7]),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        // HEADINGS Section
        Line::from(Span::styled(
            "HEADINGS",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "H1: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[6]),
        ]),
        Line::from(vec![
            Span::styled(
                "H1 Length: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[7]),
            Span::raw(" chars"),
        ]),
        Line::from(vec![
            Span::styled(
                "H2: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[8]),
        ]),
        Line::from(vec![
            Span::styled(
                "H2 Length: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[9]),
            Span::raw(" chars"),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::raw(&row_data[4])),
        Line::from(""),
        // TECHNICAL Section
        Line::from(Span::styled(
            "TECHNICAL DETAILS",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Status Code: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[10]),
        ]),
        Line::from(vec![
            Span::styled(
                "Mobile Friendly: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(if row_data[11] == "true" { "Yes" } else { "No" }),
        ]),
        Line::from(vec![
            Span::styled(
                "Language: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[12]),
        ]),
        Line::from(vec![
            Span::styled(
                "Indexable: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[13]),
        ]),
        Line::from(vec![
            Span::styled(
                "Content Type: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[15]),
        ]),
    ];

    // CANONICAL URLs Section
    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "CANONICAL URLs",
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Cyan)
            .add_modifier(Modifier::UNDERLINED),
    )));
    content.push(Line::from(""));

    if canonicals.is_empty() {
        content.push(Line::from(vec![Span::styled(
            "No canonical URLs found on this page.",
            Style::default().fg(Color::Gray),
        )]));
    } else {
        let mut sorted_canonicals = canonicals.to_vec();
        sorted_canonicals.sort_by(|a, b| {
            let a_can = a.0 == "canonical";
            let b_can = b.0 == "canonical";
            b_can.cmp(&a_can)
        });
        let mut canonical_url = None;
        let mut alternates = Vec::new();
        for (rel, href, hreflang) in sorted_canonicals {
            if rel == "canonical" {
                canonical_url = Some(href);
            } else {
                alternates.push((href, hreflang));
            }
        }
        if let Some(canonical) = canonical_url {
            content.push(Line::from(vec![
                Span::styled(
                    "Canonical: ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(canonical),
            ]));
        }
        if !alternates.is_empty() {
            content.push(Line::from(vec![Span::styled(
                "Alternate Languages:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));
            for (href, hreflang) in alternates {
                if let Some(lang) = hreflang {
                    content.push(Line::from(vec![
                        Span::styled("  • ", Style::default().fg(Color::Cyan)),
                        Span::styled(format!("{}: ", lang), Style::default().fg(Color::Yellow)),
                        Span::raw(href),
                    ]));
                }
            }
        }
    }

    let content = content;

    let paragraph = Paragraph::new(content)
        .block(block.title(Span::styled(
            "General Information ",
            Style::default().fg(Color::Yellow),
        )))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true })
        .scroll((scroll as u16, 0));

    f.render_widget(paragraph, area.inner(Margin::new(1, 0)));
}
