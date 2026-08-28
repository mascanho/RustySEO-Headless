use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::models::App;

/// Renders the Content tab with independent filtering and scrolling from the Dashboard.
/// This allows for content-specific views and future customizations.
pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    // Split off an N-Grams detail panel for the currently selected page.
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(area);
    let area = panels[0];
    let ngrams_area = panels[1];

    // The content table (not the ngrams panel) is what mouse wheel scrolling
    // and click handling target for this tab.
    app.table_rect = Some(area);

    // Ensure we have filtered data if it was just initialized
    if app.content_filtered_table_data.is_empty()
        && !app.table_data.is_empty()
        && app.content_search_query.is_empty()
    {
        app.content_filtered_table_data = app.table_data.clone();
        app.content_full_filtered_table_data = app.table_data.clone();
    }

    let header_titles = [
        "ID",
        "URL",
        "Word Count",
        "KW 1",
        "KW 2",
        "KW 3",
        "KW 4",
        "KW 5",
        "KW 6",
        "KW 7",
        "KW 8",
        "KW 9",
        "KW 10",
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

    let rows = app
        .content_filtered_table_data
        .iter()
        .enumerate()
        .map(|(i, data)| {
            let is_selected = app.content_table_state.selected() == Some(i);

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

            let start = app.content_current_page * app.content_page_size;
            let full_idx = start + i;
            let mut displayed_data = vec![
                (full_idx + 1).to_string(), // Sequential ID
                data[1].clone(),            // URL
                data[18].clone(),           // Word Count
            ];

            // Add Top 10 Keywords (Indices 35 to 44)
            for j in 35..45 {
                if let Some(kw) = data.get(j) {
                    displayed_data.push(kw.clone());
                } else {
                    displayed_data.push(String::new());
                }
            }

            let cells = displayed_data.iter().enumerate().map(|(j, c)| {
                let content = if j == 1 {
                    // URL
                    let content = c.as_str();
                    let char_count = content.chars().count();
                    if char_count > 60 {
                        let start = app
                            .content_horizontal_scroll
                            .min(char_count.saturating_sub(50));
                        let end = (start + 60).min(char_count);
                        let sliced: String =
                            content.chars().skip(start).take(end - start).collect();
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

                if j == 2 {
                    // Word count column
                    if let Ok(count) = content.trim().parse::<usize>() {
                        if count > 1000 {
                            cell_style = cell_style.fg(Color::Green).bold();
                        } else if count < 200 {
                            cell_style = cell_style.fg(Color::Red);
                        }
                    }
                }

                if j >= 3 {
                    // Keywords
                    cell_style = cell_style.fg(Color::Cyan);
                }

                let cell_content = if j == 2 {
                    // Word count column - center the text
                    Cell::from(Line::from(content).alignment(Alignment::Center)).style(cell_style)
                } else {
                    Cell::from(content).style(cell_style)
                };

                cell_content
            });

            Row::new(cells).style(row_style).height(1)
        });

    let max_id_width = app
        .content_full_filtered_table_data
        .len()
        .to_string()
        .len()
        .max(2) as u16
        + 2;
    let mut widths = vec![
        Constraint::Length(max_id_width), // ID
        Constraint::Min(40),              // URL
        Constraint::Length(12),           // Word Count
    ];

    // Add 10 constraints for keywords
    for _ in 0..10 {
        widths.push(Constraint::Length(20));
    }

    let total_pages = (app.content_full_filtered_table_data.len() + app.content_page_size - 1)
        / app.content_page_size;
    let scroll_indicator = if app.content_horizontal_scroll > 0 {
        format!(" [Scroll: {}] ", app.content_horizontal_scroll)
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
                        " Content Audit ({}) ",
                        app.content_full_filtered_table_data.len()
                    ),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .title_bottom(
                    Line::from(Span::styled(
                        format!(
                            " Page {} of {} {} ",
                            app.content_current_page + 1,
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

    f.render_stateful_widget(table, area, &mut app.content_table_state);

    // Floating Search Bar at bottom right
    if app.show_content_search {
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

        let search_text = format!("> {}", app.content_search_query);
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(Style::default().fg(Color::White));

        f.render_widget(Clear, search_area);
        f.render_widget(search_paragraph, search_area);
    }

    let side_panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(ngrams_area);

    render_ngrams_panel(f, app, side_panels[0], accent_color, border_color);
    render_duplicate_content_panel(f, app, side_panels[1], accent_color, border_color);
}

/// Right-hand panel showing 1/2/3/4-word phrase frequencies for whichever page
/// row is currently selected in the Content Audit table. N-grams are extracted
/// once per page at crawl time (bounded to the top 15 phrases per length, the
/// same shape as the existing top-10 keyword extraction), so this panel is just
/// a lookup - no work happens on render or on selection change.
fn render_ngrams_panel(
    f: &mut Frame,
    app: &App,
    area: Rect,
    accent_color: Color,
    border_color: Color,
) {
    let selected_summary = app
        .content_table_state
        .selected()
        .and_then(|i| app.content_filtered_table_data.get(i))
        .and_then(|row| row.first())
        .and_then(|id_str| id_str.parse::<usize>().ok())
        .and_then(|id| app.page_summaries.get(id.saturating_sub(1)));

    let header = Row::new(["N", "Phrase", "Count"].iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color)
                .bg(Color::Rgb(30, 30, 45)),
        )
    }))
    .height(1);

    let mut rows: Vec<Row> = Vec::new();
    if let Some(summary) = selected_summary {
        let groups: [(&str, &Vec<(String, usize)>); 4] = [
            ("1", &summary.ngrams.unigrams),
            ("2", &summary.ngrams.bigrams),
            ("3", &summary.ngrams.trigrams),
            ("4", &summary.ngrams.quadgrams),
        ];

        for (i, (n, phrases)) in groups.iter().enumerate() {
            let row_bg = if i % 2 == 0 {
                Color::Rgb(20, 20, 30)
            } else {
                Color::Rgb(25, 25, 40)
            };
            for (phrase, count) in phrases.iter().take(6) {
                rows.push(
                    Row::new(vec![
                        Cell::from(n.to_string()),
                        Cell::from(phrase.clone()),
                        Cell::from(count.to_string()),
                    ])
                    .style(Style::default().bg(row_bg)),
                );
            }
        }
    }

    let title = if selected_summary.is_some() {
        " N-Grams (selected page) "
    } else {
        " N-Grams (select a page) "
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(7),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                title,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ))
            .border_style(Style::default().fg(border_color)),
    )
    .column_spacing(1)
    .style(Style::default().bg(Color::Rgb(15, 15, 25)));

    f.render_widget(table, area);
}

/// Bottom-right panel: other crawled pages whose content is near-identical to
/// the currently selected page, per the SimHash fingerprint computed for
/// every page at crawl time (see `crawler::helpers::simhash`). A distance of
/// 0 means byte-identical body text; anything at or under the near-duplicate
/// threshold still reads as "basically the same content" (e.g. a templated
/// page with only a price or name swapped).
fn render_duplicate_content_panel(
    f: &mut Frame,
    app: &App,
    area: Rect,
    accent_color: Color,
    border_color: Color,
) {
    let selected_id = app
        .content_table_state
        .selected()
        .and_then(|i| app.content_filtered_table_data.get(i))
        .and_then(|row| row.first())
        .and_then(|id_str| id_str.parse::<usize>().ok());

    let header = Row::new(["Similar Page", "Match"].iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color)
                .bg(Color::Rgb(30, 30, 45)),
        )
    }))
    .height(1);

    let mut rows: Vec<Row> = Vec::new();
    if let Some(id) = selected_id {
        for pair in &app.duplicate_pairs {
            let other_id = if pair.id_a == id {
                Some(pair.id_b)
            } else if pair.id_b == id {
                Some(pair.id_a)
            } else {
                None
            };
            let Some(other_id) = other_id else { continue };
            let Some(other) = app.page_summaries.get(other_id.saturating_sub(1)) else {
                continue;
            };

            let label = if pair.distance == 0 {
                "Exact".to_string()
            } else {
                let similarity = 100.0 * (64 - pair.distance) as f64 / 64.0;
                format!("{:.0}% similar", similarity)
            };
            rows.push(Row::new(vec![
                Cell::from(other.url.clone()),
                Cell::from(label),
            ]));
        }
    }

    let title = if rows.is_empty() {
        " Duplicate Content (none found for this page) "
    } else {
        " Duplicate Content "
    };

    let table = Table::new(rows, [Constraint::Min(20), Constraint::Length(14)])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    title,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(border_color)),
        )
        .column_spacing(1)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)));

    f.render_widget(table, area);
}
