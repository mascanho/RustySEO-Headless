use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Cell, Row, Table, TableState},
};

pub fn render(
    f: &mut Frame,
    images: &[(String, String)],
    horizontal_scroll: usize,
    table_state: &mut TableState,
    area: Rect,
    block: Block,
) {
    let accent_color = Color::Rgb(80, 140, 255);

    let header_titles = ["#", "Image Src", "Alt Text"];

    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color)
                .bg(Color::Rgb(30, 30, 45)),
        )
    }))
    .height(1);

    let rows = images.iter().enumerate().map(|(i, (src, alt))| {
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

        let displayed_data = [(i + 1).to_string(), src.clone(), alt.clone()];

        let cells = displayed_data.iter().enumerate().map(|(j, c)| {
            let content = if j == 1 {
                // Image Src column
                let content = c.as_str();
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
            } else {
                c.as_str().to_string()
            };

            let padded_content = if j == 0 {
                // Index
                let w = 4;
                let l = content.len();
                if l < w {
                    let left_pad = (w - l) / 2;
                    let right_pad = w - l - left_pad;
                    format!(
                        "{}{}{}",
                        " ".repeat(left_pad),
                        content,
                        " ".repeat(right_pad)
                    )
                } else {
                    content
                }
            } else {
                content
            };

            Cell::from(padded_content)
        });

        Row::new(cells).style(row_style).height(1)
    });

    let widths = [
        Constraint::Length(4), // #
        Constraint::Min(50),   // Image Src
        Constraint::Min(30),   // Alt Text
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
                    format!(" 🖼️  Images ({}) {} ", images.len(), scroll_indicator),
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
