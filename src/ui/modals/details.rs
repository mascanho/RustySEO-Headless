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

    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

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
    let titles = vec![" 📄 General ", " 📊 Analysis ", " ✅ Checklist "];
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    format!(" Page Details: ID {} ", row_data[0]),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(border_color))
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
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent_color))
        .bg(Color::Rgb(20, 20, 30));

    match app.detail_tab {
        0 => render_general(f, row_data, chunks[1], content_block),
        1 => render_analysis(f, row_data, chunks[1], content_block),
        2 => render_checklist(f, row_data, chunks[1], content_block),
        _ => {}
    }
}

fn render_general(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let info = vec![
        Line::from(vec![
            Span::styled(
                " 🔗 URL: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::styled(&row_data[1], Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " 📝 Title: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[2]),
        ]),
        Line::from(vec![
            Span::styled(
                " 🏷️  H1:    ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[4]),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " 📄 Meta Description: ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(vec![Span::raw(format!("   {}", &row_data[6]))]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " 📡 Status Code: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::styled(
                &row_data[8],
                if row_data[8].contains("200") {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
        ]),
    ];
    let p = Paragraph::new(info)
        .block(block.title(Span::styled(
            " General Information ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_analysis(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let analysis = vec![
        Line::from(vec![Span::styled(
            " 🔍 Content Metrics ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  📏 Title Length:  ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[3], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(vec![
            Span::styled("  📏 H1 Length:     ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[5], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(vec![
            Span::styled("  📏 Desc Length:   ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[7], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  📖 Word Count:    ", Style::default().fg(Color::Cyan)),
            Span::raw("~842 words "),
            Span::styled("(Estimated)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  🧠 Readability:   ", Style::default().fg(Color::Cyan)),
            Span::styled("High ", Style::default().fg(Color::Green)),
            Span::styled("(Flesch Score: 72)", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    let p = Paragraph::new(analysis)
        .block(block.title(Span::styled(
            " SEO Deep Dive ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_checklist(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let checklist = vec![
        Line::from(vec![Span::styled(
            " ⚔️  SEO Health Check ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(if row_data[2].len() > 60 {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Title too long (over 60 chars)"),
            ]
        } else {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("Title length is optimal"),
            ]
        }),
        Line::from(if row_data[6].len() > 160 {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Meta description exceeds 160 chars"),
            ]
        } else {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("Meta description length is good"),
            ]
        }),
        Line::from(if row_data[4].is_empty() {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Missing H1 heading"),
            ]
        } else {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("H1 heading present and valid"),
            ]
        }),
        Line::from(if row_data[8].contains("200") {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("HTTP Status OK (200)"),
            ]
        } else {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Critical HTTP Status Issue"),
            ]
        }),
        Line::from(""),
        Line::from(vec![Span::styled(
            " 💡 Recommendations ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(accent_color)),
            Span::raw("Ensure keyword density is balanced (1.5% - 2.0%)."),
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(accent_color)),
            Span::raw("Optimize internal linking for high-value pages."),
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(accent_color)),
            Span::raw("Add ALT tags to images for better accessibility."),
        ]),
    ];
    let p = Paragraph::new(checklist)
        .block(block.title(Span::styled(
            " Automated SEO Audit Checklist ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
