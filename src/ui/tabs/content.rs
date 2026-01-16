use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::models::App;

/// Renders the Content tab with the same table as the Dashboard.
/// This allows for content-specific views and future customizations.
pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    // Ensure we have filtered data if it was just initialized
    if app.filtered_table_data.is_empty()
        && !app.table_data.is_empty()
        && app.search_query.is_empty()
    {
        app.filtered_table_data = app.table_data.clone();
    }

    let header_titles = ["ID", "URL", "Word Count"];

    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color)
                .bg(Color::Rgb(30, 30, 45)),
        )
    }))
    .height(1);

    let rows = app.filtered_table_data.iter().enumerate().map(|(i, data)| {
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

        let start = app.current_page * app.page_size;
        let full_idx = start + i;
        let displayed_data = vec![
            (full_idx + 1).to_string(), // Sequential ID
            data[1].clone(),            // URL
            data[18].clone(),           // Word Count
        ];

        let cells = displayed_data.iter().enumerate().map(|(j, c)| {
            let mut content = if j == 1 || j == 2 || j == 4 || j == 6 || j == 8 {
                // URL, Title, H1, Desc, H2
                let content = c.as_str();
                let char_count = content.chars().count();
                if char_count > 60 {
                    let start = app.horizontal_scroll.min(char_count.saturating_sub(50));
                    let end = (start + 60).min(char_count);
                    let sliced: String = content.chars().skip(start).take(end - start).collect();
                    if start > 0 {
                        format!("…{}", sliced)
                    } else {
                        sliced
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

            if j == 3 {
                if let Ok(len) = c.parse::<usize>() {
                    if len > 60 && !is_selected {
                        cell_style = cell_style.fg(Color::Red);
                    } else if len < 60 && !is_selected {
                        cell_style = cell_style.fg(Color::Green);
                    }
                }
            }

            if j == 5 {
                if let Ok(len) = c.parse::<usize>() {
                    if len > 160 && !is_selected {
                        cell_style = cell_style.fg(Color::Red);
                    } else if len < 160 && !is_selected {
                        cell_style = cell_style.fg(Color::Green);
                    }
                }
            }

            // Indexability column logic
            if j == 13 {
                if content.contains("noindex") {
                    content = "Non-indexable".to_string();
                    if !is_selected {
                        cell_style = cell_style.fg(Color::Red);
                    }
                } else {
                    content = "Indexable".to_string();
                    if !is_selected {
                        cell_style = cell_style.fg(Color::Green);
                    }
                }
            }

            let content = if j == 3 || j == 5 || j == 7 || j == 9 || j == 10 || j == 11 || j == 14 {
                let w = match j {
                    3 | 5 => 5,
                    7 | 9 => 7,
                    10 | 11 => 8,
                    14 => 10,
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
        Constraint::Length(20), // Desc
        Constraint::Length(5),  // Desc Len
        Constraint::Length(20), // H1
        Constraint::Length(7),  // H1 Len
        Constraint::Length(15), // H2
        Constraint::Length(7),  // H2 Len
        Constraint::Length(8),  // Status
        Constraint::Length(8),  // Mobile
        Constraint::Length(6),  // Lang
        Constraint::Min(8),     // Indexable
        Constraint::Length(10), // Canonicals
    ];

    let total_pages = (app.full_filtered_table_data.len() + app.page_size - 1) / app.page_size;
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
                    format!(
                        " 📄 Content Dashboard (Page {}/{}) {} ",
                        app.current_page + 1,
                        total_pages,
                        scroll_indicator
                    ),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(border_color)),
        )
        .column_spacing(1)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)));

    f.render_stateful_widget(table, area, &mut app.table_state);

    // Floating Search Bar at bottom right
    if app.show_search {
        let search_area = Rect {
            x: area.x + area.width.saturating_sub(40),
            y: area.y + area.height.saturating_sub(3),
            width: 38,
            height: 3,
        };

        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .bg(Color::Rgb(25, 25, 40))
            .title(Span::styled(
                " Fuzzy Search ",
                Style::default().fg(Color::Cyan).bold(),
            ));

        let search_text = format!("> {}", app.search_query);
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(Style::default().fg(Color::White));

        f.render_widget(Clear, search_area);
        f.render_widget(search_paragraph, search_area);
    }
}
