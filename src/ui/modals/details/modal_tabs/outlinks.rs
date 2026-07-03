use std::collections::HashMap;

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Cell, Row, Table, TableState},
};

pub fn render(
    f: &mut Frame,
    anchor_links: &[crate::crawler::helpers::html_parser::AnchorLink],
    url_to_status: &HashMap<String, String>,
    horizontal_scroll: usize,
    table_state: &mut TableState,
    area: Rect,
    block: Block,
) {
    let accent_color = Color::Rgb(80, 140, 255);

    let header_titles = ["#", "Link", "Anchor Text", "Status"];

    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color)
                .bg(Color::Rgb(30, 30, 45)),
        )
    }))
    .height(1);

    let rows = anchor_links.iter().enumerate().map(|(i, link)| {
        let is_selected = table_state.selected() == Some(i);

        let mut row_style = if i % 2 == 0 {
            Style::default().bg(Color::Rgb(20, 20, 30))
        } else {
            Style::default().bg(Color::Rgb(25, 25, 40))
        };

        if is_selected {
            row_style = row_style
                .fg(Color::White)
                .bg(accent_color)
                .add_modifier(Modifier::BOLD);
        }

        let link_display = {
            let content = link.href.as_str();
            if content.len() > 100 {
                let start = horizontal_scroll.min(content.len().saturating_sub(50));
                let end = (start + 100).min(content.len());
                if start > 0 {
                    format!("…{}", &content[start..end])
                } else {
                    content[start..end].to_string()
                }
            } else {
                content.to_string()
            }
        };

        let idx_str = {
            let s = (i + 1).to_string();
            let w = 4;
            let l = s.len();
            if l < w {
                let left_pad = (w - l) / 2;
                let right_pad = w - l - left_pad;
                format!("{}{}{}", " ".repeat(left_pad), s, " ".repeat(right_pad))
            } else {
                s
            }
        };

        let status = url_to_status
            .get(&link.href)
            .and_then(|s| s.split_whitespace().next())
            .unwrap_or("—")
            .to_string();

        let status_color = match status.parse::<u16>().unwrap_or(0) / 100 {
            2 => Color::Green,
            3 => Color::Yellow,
            4 | 5 => Color::Red,
            _ => Color::DarkGray,
        };

        let cells = vec![
            Cell::from(idx_str).style(row_style),
            Cell::from(link_display).style(row_style),
            Cell::from(link.text.clone()).style(row_style),
            Cell::from(format!(" {} ", status)).style(row_style.fg(status_color).add_modifier(Modifier::BOLD)),
        ];

        Row::new(cells).height(1)
    });

    let widths = [
        Constraint::Length(4),  // #
        Constraint::Min(50),    // Link
        Constraint::Min(30),    // Anchor Text
        Constraint::Length(8),  // Status
    ];

    let scroll_indicator = if horizontal_scroll > 0 {
        format!(" [Scroll: {}] ", horizontal_scroll)
    } else {
        String::new()
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            block
                .title(Span::styled(
                    format!(
                        " ↗️  External Links ({}) {} ",
                        anchor_links.len(),
                        scroll_indicator
                    ),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(accent_color)),
        )
        .column_spacing(1)
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));

    f.render_stateful_widget(table, area, table_state);
}
