use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    app.table_rect = Some(area);
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    // Ensure we have filtered data if it was just initialized
    if app.filtered_table_data.is_empty()
        && !app.table_data.is_empty()
        && app.search_query.is_empty()
    {
        app.filtered_table_data = app.table_data.clone();
    }

    let header_titles = [
        "ID", "URL", "D: Score", "D: FCP", "D: LCP", "D: CLS", "D: TBT", "D: SI", "M: Score",
        "M: FCP", "M: LCP", "M: CLS", "M: TBT", "M: SI",
    ];

    let header = Row::new(header_titles.iter().enumerate().map(|(i, h)| {
        let align = if i == 1 {
            Alignment::Left
        } else {
            Alignment::Center
        };
        Cell::from(Line::from(format!(" {} ", h)).alignment(align)).style(
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
            (full_idx + 1).to_string(), // 0: Sequential ID
            data[1].clone(),            // 1: URL
            data[33].clone(),           // 2: D: Score
            data[34].clone(),           // 3: D: FCP
            data[35].clone(),           // 4: D: LCP
            data[36].clone(),           // 5: D: CLS
            data[37].clone(),           // 6: D: TBT
            data[38].clone(),           // 7: D: SI
            data[39].clone(),           // 8: M: Score
            data[40].clone(),           // 9: M: FCP
            data[41].clone(),           // 10: M: LCP
            data[42].clone(),           // 11: M: CLS
            data[43].clone(),           // 12: M: TBT
            data[44].clone(),           // 13: M: SI
        ];

        let cells = displayed_data.iter().enumerate().map(|(j, c)| {
            let content = if j == 1 {
                // URL
                let content = c.as_str();
                let char_count = content.chars().count();
                if char_count > 80 {
                    let start = app.horizontal_scroll.min(char_count.saturating_sub(30));
                    let end = (start + 80).min(char_count);
                    let sliced: String = content.chars().skip(start).take(end - start).collect();
                    if start > 0 {
                        format!("{}...", sliced)
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

            // Colorize performance scores
            if j == 2 || j == 8 {
                if let Ok(score) = content.parse::<u8>() {
                    if score >= 90 {
                        cell_style = cell_style.fg(Color::Green);
                    } else if score >= 50 {
                        cell_style = cell_style.fg(Color::Yellow);
                    } else {
                        cell_style = cell_style.fg(Color::Red);
                    }
                }
            }

            let align = if j == 1 {
                Alignment::Left
            } else {
                Alignment::Center
            };
            Cell::from(Line::from(content).alignment(align)).style(cell_style)
        });

        Row::new(cells).style(row_style).height(1)
    });

    // Calculate dynamic ID width
    let max_id_width = app.full_filtered_table_data.len().to_string().len().max(2) as u16 + 2;

    let widths = [
        Constraint::Length(max_id_width), // ID
        Constraint::Min(30),              // URL
        Constraint::Length(10),           // D: Score
        Constraint::Length(10),           // D: FCP
        Constraint::Length(10),           // D: LCP
        Constraint::Length(10),           // D: CLS
        Constraint::Length(10),           // D: TBT
        Constraint::Length(10),           // D: SI
        Constraint::Length(10),           // M: Score
        Constraint::Length(10),           // M: FCP
        Constraint::Length(10),           // M: LCP
        Constraint::Length(10),           // M: CLS
        Constraint::Length(10),           // M: TBT
        Constraint::Length(10),           // M: SI
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
                        " Core Web Vitals - Desktop & Mobile ({}) ",
                        app.full_filtered_table_data.len()
                    ),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .title_bottom(
                    Line::from(Span::styled(
                        format!(
                            " Page {} of {} {} ",
                            app.current_page + 1,
                            total_pages,
                            scroll_indicator
                        ),
                        Style::default().fg(Color::DarkGray).italic(),
                    ))
                    .alignment(Alignment::Right),
                )
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
