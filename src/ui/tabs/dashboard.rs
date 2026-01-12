use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    let header_titles = [
        "ID",
        "URL",
        "Title",
        "Len",
        "H1",
        "Len",
        "Desc",
        "Len",
        "H2",
        "Len",
        "Status",
        "Mobile",
        "Lang",
        "Indexable",
    ];

    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color)
                .bg(Color::Rgb(30, 30, 45)),
        )
    }))
    .height(1);

    let rows = app.table_data.iter().enumerate().map(|(i, data)| {
        let is_selected = app.table_state.selected() == Some(i);

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

        let displayed_data = vec![
            &data[0],  // ID
            &data[1],  // URL
            &data[2],  // Title
            &data[3],  // Title Len
            &data[4],  // H1
            &data[5],  // H1 Len
            &data[6],  // Desc
            &data[7],  // Desc Len
            &data[8],  // H2
            &data[9],  // H2 Len
            &data[10], // Status
            &data[11], // Mobile
            &data[12], // Language
            &data[13], // Indexability
        ];

        let cells = displayed_data.iter().enumerate().map(|(j, c)| {
            let mut content = if j == 1 || j == 2 || j == 4 || j == 6 || j == 8 {
                // URL, Title, H1, Desc, H2
                let content = c.as_str();
                if content.len() > 100 {
                    let start = app.horizontal_scroll.min(content.len().saturating_sub(50));
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

            let mut cell_style = Style::default();

            if j == 10 {
                // Status column
                match content.as_str() {
                    c if c.contains("200") => {
                        content = format!("200");
                        if !is_selected {
                            cell_style = cell_style.fg(Color::Green);
                        }
                    }
                    c if c.contains("404") => {
                        content = format!("404");
                        if !is_selected {
                            cell_style = cell_style.fg(Color::Red);
                        }
                    }
                    c if c.contains("301") || c.contains("302") => {
                        content = format!("{}", c);
                        if !is_selected {
                            cell_style = cell_style.fg(Color::Blue);
                        }
                    }
                    c if c.contains("500") => {
                        content = format!("500");
                        if !is_selected {
                            cell_style = cell_style.fg(Color::Yellow);
                        }
                    }
                    c if c.contains("403") => {
                        content = format!("403");
                        if !is_selected {
                            cell_style = cell_style.fg(Color::Magenta);
                        }
                    }
                    c if c.contains("503") => {
                        content = format!("🚧 {}", c);
                        if !is_selected {
                            cell_style = cell_style.fg(Color::LightRed);
                        }
                    }
                    _ => {
                        content = format!("{}", c);
                    }
                }
            }

            if j == 11 {
                // Mobile column
                content = if content == "true" {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                };
            }

            let content = if j == 3 || j == 5 || j == 7 || j == 9 || j == 10 || j == 11 {
                let w = match j {
                    3 | 7 => 5,
                    5 | 9 => 7,
                    10 | 11 => 8,
                    _ => unreachable!(),
                };
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

            Cell::from(content).style(cell_style)
        });

        Row::new(cells).style(row_style).height(1)
    });

    let widths = [
        Constraint::Length(4),  // ID
        Constraint::Min(55),    // URL
        Constraint::Length(20), // Title
        Constraint::Length(5),  // Title Len
        Constraint::Length(20), // H1
        Constraint::Length(7),  // H1 Len
        Constraint::Length(20), // Desc
        Constraint::Length(5),  // Desc Len
        Constraint::Length(15), // H2
        Constraint::Length(7),  // H2 Len
        Constraint::Length(8),  // Status
        Constraint::Length(8),  // Mobile
        Constraint::Length(6),  // Lang
        Constraint::Min(8),     // Indexable
    ];

    let scroll_indicator = if app.horizontal_scroll > 0 {
        format!(" [Scroll: {}] ", app.horizontal_scroll)
    } else {
        String::new()
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    format!(" 📊 SEO Audit Dashboard{} ", scroll_indicator),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(border_color)),
        )
        .column_spacing(1)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)));

    f.render_stateful_widget(table, area, &mut app.table_state);
}
