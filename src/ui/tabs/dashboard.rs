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
        "ID", "URL", "Title", "Len", "H1", "H1 Len", "H2", "H2 Len", "Status",
    ];

    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Rgb(15, 15, 25))
                .bg(Color::Rgb(200, 200, 200)),
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
            row_style = row_style.fg(Color::Black).bg(accent_color).add_modifier(Modifier::BOLD);
        }

        let displayed_data = vec![
            &data[0], // ID
            &data[1], // URL
            &data[2], // Title
            &data[3], // Len
            &data[4], // H1
            &data[5], // H1 Len
            &data[6], // H2
            &data[7], // H2 Len
            &data[8], // Status
        ];

        let cells = displayed_data.iter().enumerate().map(|(j, c)| {
            let mut content = if j == 1 || j == 2 || j == 4 || j == 6 {
                let content = c.as_str();
                if content.len() > 20 {
                    let start = app.horizontal_scroll.min(content.len().saturating_sub(20));
                    let end = (start + 20).min(content.len());
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
            
            if j == 8 { // Status column
                if content.contains("200") {
                    content = format!("✅ {}", content);
                    if !is_selected { cell_style = cell_style.fg(Color::Green); }
                } else if content.contains("404") {
                    content = format!("❌ {}", content);
                    if !is_selected { cell_style = cell_style.fg(Color::Red); }
                } else if content.contains("301") || content.contains("302") {
                    content = format!("➡️  {}", content);
                    if !is_selected { cell_style = cell_style.fg(Color::Blue); }
                }
            }

            Cell::from(content).style(cell_style)
        });

        Row::new(cells).style(row_style).height(1)
    });

    let widths = [
        Constraint::Length(4),
        Constraint::Length(25),
        Constraint::Length(20),
        Constraint::Length(5),
        Constraint::Length(20),
        Constraint::Length(7),
        Constraint::Length(15),
        Constraint::Length(7),
        Constraint::Min(15),
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
                .title(Span::styled(format!(" 📊 SEO Audit Dashboard{} ", scroll_indicator), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
                .border_style(Style::default().fg(border_color))
        )
        .column_spacing(2)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)))
        .highlight_symbol(Span::styled(" ➔ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

    f.render_stateful_widget(table, area, &mut app.table_state);
}

