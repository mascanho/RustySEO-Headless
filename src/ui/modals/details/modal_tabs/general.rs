use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
    Frame,
};

pub fn render(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let items = vec![
        // PAGE INFORMATION Section
        ListItem::new(Line::from(Span::styled(
            "PAGE INFORMATION",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        ))),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled(
                "URL: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[1]),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "Title: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[2]),
        ])),
        ListItem::new(""),
        // HEADINGS Section
        ListItem::new(Line::from(Span::styled(
            "HEADINGS",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        ))),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled(
                "H1: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[4]),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "H1 Length: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[5]),
            Span::raw(" chars"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "H2: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[8]),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "H2 Length: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[9]),
            Span::raw(" chars"),
        ])),
        ListItem::new(""),
        // DESCRIPTION Section
        ListItem::new(Line::from(Span::styled(
            "META DESCRIPTION",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        ))),
        ListItem::new(""),
        ListItem::new(Span::raw(&row_data[6])),
        ListItem::new(""),
        // TECHNICAL Section
        ListItem::new(Line::from(Span::styled(
            "TECHNICAL DETAILS",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        ))),
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled(
                "Status Code: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[10]),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "Mobile Friendly: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(if row_data[11] == "true" { "Yes" } else { "No" }),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "Language: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[12]),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "Indexable: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[13]),
        ])),
    ];

    let list = List::new(items)
        .block(block.title(Span::styled(
            "General Information ",
            Style::default().fg(Color::Yellow),
        )))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area.inner(Margin::new(1, 0)));
}
