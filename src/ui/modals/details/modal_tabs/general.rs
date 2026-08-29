use ratatui::{
    Frame,
    layout::{Constraint, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Cell, Row, Table, TableState},
};

use crate::crawler::PageData;

const ACCENT_COLOR: Color = Color::Rgb(80, 140, 255);

/// One label/value line in the General tab's info table.
struct InfoRow {
    label: String,
    value: String,
    value_style: Style,
}

fn info(label: impl Into<String>, value: impl Into<String>) -> InfoRow {
    let value = value.into();
    let value_style = if value.trim().is_empty() {
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
    } else {
        Style::default().fg(Color::White)
    };
    InfoRow {
        label: label.into(),
        value: if value.trim().is_empty() {
            "-".to_string()
        } else {
            value
        },
        value_style,
    }
}

fn info_styled(
    label: impl Into<String>,
    value: impl Into<String>,
    value_style: Style,
) -> InfoRow {
    InfoRow {
        label: label.into(),
        value: value.into(),
        value_style,
    }
}

fn format_thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}

fn format_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn url_depth(url: &str) -> usize {
    url::Url::parse(url)
        .ok()
        .and_then(|u| {
            u.path_segments()
                .map(|segments| segments.filter(|s| !s.is_empty()).count())
        })
        .unwrap_or(0)
}

fn build_rows(row_data: &[String], page_details: &PageData, external_links_count: usize) -> Vec<InfoRow> {
    let status_code: u16 = row_data
        .get(10)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let status_color = match status_code / 100 {
        1 => Color::Blue,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Red,
        5 => Color::Rgb(255, 0, 255),
        _ => Color::Gray,
    };

    let bool_style = |yes: bool| {
        if yes {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        }
    };

    // No robots meta tag means indexable by default - only an explicit
    // "noindex" makes a page non-indexable, matching the dashboard table.
    let is_indexable = !row_data
        .get(13)
        .map(|s| s.to_lowercase().contains("noindex"))
        .unwrap_or(false);

    let canonical = page_details
        .canonicals
        .iter()
        .find(|(rel, _, _)| rel == "canonical")
        .map(|(_, href, _)| href.clone())
        .unwrap_or_default();

    let keywords: Vec<String> = row_data
        .get(35..45)
        .unwrap_or(&[])
        .iter()
        .filter(|k| !k.trim().is_empty())
        .cloned()
        .collect();
    let keywords_display = keywords.join(", ");

    let mut rows: Vec<InfoRow> = vec![
        info("URL", row_data.get(1).cloned().unwrap_or_default()),
        info("Canonical", canonical),
        info("Title", row_data.get(2).cloned().unwrap_or_default()),
        info("Title Length", row_data.get(3).cloned().unwrap_or_default()),
        info(
            "Meta Description",
            row_data.get(6).cloned().unwrap_or_default(),
        ),
        info(
            "Meta Description Length",
            row_data.get(7).cloned().unwrap_or_default(),
        ),
        info("Heading H1", row_data.get(4).cloned().unwrap_or_default()),
        info(
            "Heading H1 Length",
            row_data.get(5).cloned().unwrap_or_default(),
        ),
        info_styled(
            "Response Code",
            row_data.get(10).cloned().unwrap_or_default(),
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ),
        info_styled(
            "Mobile Optimized",
            if row_data.get(11).map(|s| s.as_str()) == Some("true") {
                "Yes"
            } else {
                "No"
            },
            bool_style(row_data.get(11).map(|s| s.as_str()) == Some("true")),
        ),
        info_styled(
            "Indexable",
            if is_indexable { "Yes" } else { "No" },
            bool_style(is_indexable),
        ),
        info("Content Type", row_data.get(15).cloned().unwrap_or_default()),
        info("Word Count", row_data.get(18).cloned().unwrap_or_default()),
        info(
            "Text Ratio",
            format!("{:.1}%", page_details.text_ratio),
        ),
        info("Top 10 Keywords", keywords_display),
        info(
            "Internal Links",
            page_details.anchor_links.len().to_string(),
        ),
        info("External Links", external_links_count.to_string()),
        info("Images", page_details.images.len().to_string()),
        info(
            "Page Length",
            format!("{} bytes", format_thousands(page_details.size)),
        ),
        info("Page Size", format_size(page_details.size)),
        info(
            "Response Time",
            format!("{:.2} ms", page_details.response_time_ms),
        ),
        info("URL Depth", url_depth(&page_details.url).to_string()),
        info("Language", row_data.get(12).cloned().unwrap_or_default()),
    ];

    if page_details.og_tags.is_empty() {
        rows.push(info("OG Tags", ""));
    } else {
        for (property, content) in &page_details.og_tags {
            rows.push(info(format!("OG: {}", property), content.clone()));
        }
    }

    rows.push(info("Cookies", page_details.cookies.join("; ")));

    rows
}

/// Number of rows the General tab's info table has for `page_details` - used
/// by row-based up/down navigation to know where to wrap around.
pub fn row_count(row_data: &[String], page_details: &PageData, external_links_count: usize) -> usize {
    build_rows(row_data, page_details, external_links_count).len()
}

pub fn render(
    f: &mut Frame,
    row_data: &[String],
    page_details: &PageData,
    external_links_count: usize,
    table_state: &mut TableState,
    area: Rect,
    block: Block,
) {
    let rows = build_rows(row_data, page_details, external_links_count);

    let table_rows: Vec<Row> = rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let is_selected = table_state.selected() == Some(i);

            let mut bg = if i % 2 == 0 {
                Color::Rgb(20, 20, 30)
            } else {
                Color::Rgb(25, 25, 40)
            };
            let mut label_style = Style::default()
                .fg(Color::Rgb(255, 140, 0))
                .add_modifier(Modifier::BOLD);
            let mut value_style = r.value_style;

            if is_selected {
                bg = Color::Rgb(20, 50, 120);
                label_style = Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .add_modifier(Modifier::BOLD);
                value_style = Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .add_modifier(Modifier::BOLD);
            }

            Row::new(vec![
                // The gap between columns lives inside this cell (trailing spaces)
                // rather than in Table::column_spacing, so it's covered by the
                // row's own background instead of showing a bare unstyled seam.
                Cell::from(format!(" {}   ", r.label)).style(label_style.bg(bg)),
                Cell::from(format!(" {} ", r.value)).style(value_style.bg(bg)),
            ])
            .height(1)
        })
        .collect();

    let widths = [Constraint::Length(26), Constraint::Min(20)];

    let table = Table::new(table_rows, widths)
        .block(block)
        .column_spacing(0);

    f.render_stateful_widget(table, area.inner(Margin::new(1, 0)), table_state);
}
