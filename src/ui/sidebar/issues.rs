use crate::models::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Row, Table},
    Frame,
};

static ACCENT_COLOUR: Color = Color::Rgb(80, 140, 255);

pub fn render(f: &mut Frame, app: &mut App, area: Rect, content_block: Block) {
    let header =
        Row::new(vec![" Issues", " Urls", " % of"]).style(Style::default().fg(Color::Yellow));

    // Recompute issues data from page data to ensure accuracy
    let total_pages = app.page_data.len();
    // 404 Errors
    let (count_404, _urls_404) = {
        let mut urls: Vec<String> = Vec::new();
        for p in &app.page_data {
            if p.status == "404" || p.status.starts_with("4") {
                urls.push(p.url.clone());
            }
        }
        (urls.len(), urls)
    };
    // Page Titles > 60 chars
    let (count_long_titles, _urls_long_titles) = {
        let mut urls: Vec<String> = Vec::new();
        for p in &app.page_data {
            if p.title_len > 60 {
                urls.push(format!("{} ({} chars)", p.url, p.title_len));
            }
        }
        (urls.len(), urls)
    };
    // Page Titles < 30 chars
    let (count_short_titles, _urls_short_titles) = {
        let mut urls: Vec<String> = Vec::new();
        for p in &app.page_data {
            if p.title_len < 30 {
                urls.push(format!("{} ({})", p.url, p.title_len));
            }
        }
        (urls.len(), urls)
    };
    // Missing Alt Text
    let (count_missing_alt, _urls_missing_alt) = {
        let mut urls: Vec<String> = Vec::new();
        for p in &app.page_data {
            let missing = p
                .images
                .iter()
                .filter(|img| img.alt.trim().is_empty())
                .count();
            if missing > 0 {
                urls.push(format!("{} ({} images)", p.url, missing));
            }
        }
        (urls.len(), urls)
    };
    // Percentages
    let percent_404 = if total_pages > 0 {
        (count_404 * 100) / total_pages
    } else {
        0
    };
    let percent_long_titles = if total_pages > 0 {
        (count_long_titles * 100) / total_pages
    } else {
        0
    };
    let percent_short_titles = if total_pages > 0 {
        (count_short_titles * 100) / total_pages
    } else {
        0
    };
    let percent_missing_alt = if total_pages > 0 {
        (count_missing_alt * 100) / total_pages
    } else {
        0
    };

    // Build rows from recomputed data
    let rows: Vec<Row> = vec![
        Row::new(vec![
            "404 Errors".to_string(),
            count_404.to_string(),
            format!("{}%", percent_404),
        ]),
        Row::new(vec![
            "Page Titles > 60 chars".to_string(),
            count_long_titles.to_string(),
            format!("{}%", percent_long_titles),
        ]),
        Row::new(vec![
            "Page Titles < 30 chars".to_string(),
            count_short_titles.to_string(),
            format!("{}%", percent_short_titles),
        ]),
        Row::new(vec![
            "Missing Alt Text".to_string(),
            count_missing_alt.to_string(),
            format!("{}%", percent_missing_alt),
        ]),
        Row::new(vec![
            "Slow Load".to_string(),
            "0".to_string(),
            "0%".to_string(),
        ]),
    ];

    let table = Table::new(
        rows,
        vec![
            Constraint::Percentage(70), // Issue column takes 1/3
            Constraint::Min(8),         // Urls column
            Constraint::Min(6),         // % of column
        ],
    )
    .header(header)
    .block(content_block.title(Span::styled("  ", Style::default().fg(Color::Yellow))))
    .style(Style::default().fg(Color::White))
    // .highlight_symbol("👉 ")
    .row_highlight_style(
        Style::default()
            .fg(Color::White)
            .bg(ACCENT_COLOUR)
            .add_modifier(ratatui::style::Modifier::BOLD),
    );

    let mut table_state = app.issues_table_state.clone();
    f.render_stateful_widget(table, area, &mut table_state);
    app.issues_table_state = table_state;
}
