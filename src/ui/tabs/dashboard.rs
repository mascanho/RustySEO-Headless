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
        "H1 Len",
        "H2",
        "H2 Len",
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
                .fg(Color::Black)
                .bg(accent_color)
                .add_modifier(Modifier::BOLD);
        }

        let displayed_data = vec![
            &data[0],  // ID
            &data[1],  // URL
            &data[2],  // Title
            &data[3],  // Len
            &data[4],  // H1
            &data[5],  // H1 Len
            &data[6],  // H2
            &data[7],  // H2 Len
            &data[8],  // Status
            &data[9],  // Mobile
            &data[10], // Language
            &data[11], // Indexability
        ];

        let cells = displayed_data.iter().enumerate().map(|(j, c)| {
            let mut content = if j == 1 || j == 2 || j == 4 || j == 6 {
                // URL, Title, H1, H2
                let content = c.as_str();
                if content.len() > 50 {
                    let start = app.horizontal_scroll.min(content.len().saturating_sub(50));
                    let end = (start + 50).min(content.len());
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

            if j == 8 {
                // Status column
                match content.as_str() {
                    c if c.contains("200") => {
                        content = format!("✅ {}", c);
                        if !is_selected {
                            cell_style = cell_style.fg(Color::Green);
                        }
                    }
                    c if c.contains("404") => {
                        content = format!("❌ {}", c);
                        if !is_selected {
                            cell_style = cell_style.fg(Color::Red);
                        }
                    }
                    c if c.contains("301") || c.contains("302") => {
                        content = format!("➡️  {}", c);
                        if !is_selected {
                            cell_style = cell_style.fg(Color::Blue);
                        }
                    }
                    c if c.contains("500") => {
                        content = format!("⚠️ {}", c);
                        if !is_selected {
                            cell_style = cell_style.fg(Color::Yellow);
                        }
                    }
                    c if c.contains("403") => {
                        content = format!("⛔ {}", c);
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

            Cell::from(content).style(cell_style)
        });

        Row::new(cells).style(row_style).height(1)
    });

    let widths = [
        Constraint::Length(4),
        Constraint::Min(35),
        Constraint::Length(20),
        Constraint::Length(5),
        Constraint::Length(20),
        Constraint::Length(7),
        Constraint::Length(15),
        Constraint::Length(7),
        Constraint::Min(15),
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Min(8),
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
        .column_spacing(2)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)))
        .highlight_symbol(Span::styled(
            " ➔ ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    f.render_stateful_widget(table, area, &mut app.table_state);
}
