use crate::models::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

// --- Data Structures ---

#[derive(Default)]
struct SeoMetrics {
    // Core Stats
    total_pages: usize,
    total_size: usize,
    total_words: usize,

    // HTTP Status
    status_2xx: usize,
    status_3xx: usize,
    status_4xx: usize,
    status_5xx: usize,

    // SEO Fundamentals
    pages_with_title: usize,
    optimal_titles: usize, // 30-60 chars
    short_titles: usize,   // <30
    long_titles: usize,    // >60

    pages_with_desc: usize,
    optimal_descriptions: usize, // 120-160 chars
    short_descriptions: usize,   // <120
    long_descriptions: usize,    // >160

    pages_with_h1: usize,
    multiple_h1: usize,
    missing_h1: usize,

    // Indexability & Mobile
    indexable: usize,
    noindex: usize,
    mobile_friendly: usize,

    // Content Analysis
    total_h1: usize,
    total_h2: usize,
    total_h3: usize,
    total_h4: usize,
    total_h5: usize,
    total_h6: usize,

    // Links
    total_internal_links: usize,
    total_external_links: usize,
    pages_with_canonicals: usize,
    pages_with_alternates: usize,

    // Images
    total_images: usize,
    images_with_alt: usize,
    images_missing_alt: usize,

    // Resources
    total_css_files: usize,
    total_js_files: usize,
    pages_with_inline_css: usize,
    pages_with_inline_js: usize,

    // Performance (CWV)
    desktop_score_sum: f64,
    desktop_samples: usize,
    mobile_score_sum: f64,
    mobile_samples: usize,

    // Schema & Structured Data
    pages_with_schema: usize,
    total_schema_objects: usize,

    // Language
    pages_with_lang: usize,
}

// --- Main Render Function ---

pub fn render(f: &mut Frame, app: &mut App, area: Rect, content_block: Block, accent_color: Color) {
    let metrics = collect_metrics(&app.page_data);

    let block = content_block.title(Span::styled(
        " SEO OVERVIEW ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Main layout - split into scrollable sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Header Stats
            Constraint::Length(8),  // HTTP Status Health
            Constraint::Length(11), // SEO Fundamentals
            Constraint::Length(8),  // Content Structure
            Constraint::Min(0),     // Images, Resources & Performance
        ])
        .margin(1)
        .split(inner_area);

    render_header_stats(f, chunks[0], &metrics, accent_color);
    render_http_health(f, chunks[1], &metrics);
    render_seo_fundamentals(f, chunks[2], &metrics);
    render_content_structure(f, chunks[3], &metrics);
    render_technical_details(f, chunks[4], &metrics);
}

// --- Collection Function ---

fn collect_metrics(pages: &[crate::crawler::PageData]) -> SeoMetrics {
    let mut m = SeoMetrics::default();

    m.total_pages = pages.len();

    for page in pages {
        m.total_size += page.size;
        m.total_words += page.word_count.unwrap_or(0);

        // HTTP Status
        if page.status.starts_with('2') {
            m.status_2xx += 1;
        } else if page.status.starts_with('3') {
            m.status_3xx += 1;
        } else if page.status.starts_with('4') {
            m.status_4xx += 1;
        } else if page.status.starts_with('5') {
            m.status_5xx += 1;
        }

        // Title Analysis
        if !page.title.is_empty() {
            m.pages_with_title += 1;
            if page.title_len < 30 {
                m.short_titles += 1;
            } else if page.title_len <= 60 {
                m.optimal_titles += 1;
            } else {
                m.long_titles += 1;
            }
        }

        // Description Analysis
        if !page.description.is_empty() {
            m.pages_with_desc += 1;
            if page.description_len < 120 {
                m.short_descriptions += 1;
            } else if page.description_len <= 160 {
                m.optimal_descriptions += 1;
            } else {
                m.long_descriptions += 1;
            }
        }

        // H1 Analysis
        let h1_count = page.headings.iter().filter(|(tag, _)| tag == "h1").count();
        if h1_count > 0 {
            m.pages_with_h1 += 1;
            if h1_count > 1 {
                m.multiple_h1 += 1;
            }
        } else {
            m.missing_h1 += 1;
        }

        // Indexability
        let indexability_lower = page.indexability.to_lowercase();
        if indexability_lower.contains("noindex") {
            m.noindex += 1;
        } else {
            m.indexable += 1;
        }

        // Mobile
        if page.mobile {
            m.mobile_friendly += 1;
        }

        // Heading counts
        for (tag, _) in &page.headings {
            match tag.as_str() {
                "h1" => m.total_h1 += 1,
                "h2" => m.total_h2 += 1,
                "h3" => m.total_h3 += 1,
                "h4" => m.total_h4 += 1,
                "h5" => m.total_h5 += 1,
                "h6" => m.total_h6 += 1,
                _ => {}
            }
        }

        // Links
        for link in &page.anchor_links {
            if link.href.starts_with("http://") || link.href.starts_with("https://") {
                m.total_external_links += 1;
            } else {
                m.total_internal_links += 1;
            }
        }

        // Canonicals & Alternates
        if !page.canonicals.is_empty() {
            for (rel, _, _) in &page.canonicals {
                if rel == "canonical" {
                    m.pages_with_canonicals += 1;
                    break;
                }
            }
            for (rel, _, _) in &page.canonicals {
                if rel == "alternate" {
                    m.pages_with_alternates += 1;
                    break;
                }
            }
        }

        // Images
        m.total_images += page.images.len();
        for img in &page.images {
            if !img.alt.trim().is_empty() {
                m.images_with_alt += 1;
            } else {
                m.images_missing_alt += 1;
            }
        }

        // Resources
        if let Some(css) = &page.css {
            m.total_css_files += css.external_css_count;
            if css.inline_css_size_bytes.unwrap_or(0) > 0 {
                m.pages_with_inline_css += 1;
            }
        }
        if let Some(js) = &page.javascript {
            m.total_js_files += js.external_js_count;
            if js.inline_js_size_bytes.unwrap_or(0) > 0 {
                m.pages_with_inline_js += 1;
            }
        }

        // Performance
        if let Some(desktop) = &page.cwv_desktop {
            if let Ok(score) = desktop.performance_score.parse::<f64>() {
                m.desktop_score_sum += score;
                m.desktop_samples += 1;
            }
        }
        if let Some(mobile) = &page.cwv_mobile {
            if let Ok(score) = mobile.performance_score.parse::<f64>() {
                m.mobile_score_sum += score;
                m.mobile_samples += 1;
            }
        }

        // Schema
        if !page.schema.is_empty() {
            m.pages_with_schema += 1;
            m.total_schema_objects += page.schema.len();
        }

        // Language
        if !page.language.is_empty() {
            m.pages_with_lang += 1;
        }
    }

    m
}

// --- Render Functions ---

fn render_header_stats(f: &mut Frame, area: Rect, m: &SeoMetrics, accent: Color) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let cards = [
        ("PAGES", m.total_pages.to_string(), Color::Cyan),
        ("SIZE", format_size(m.total_size), Color::Yellow),
        ("WORDS", format_number(m.total_words), Color::Magenta),
        (
            "LINKS",
            format_number(m.total_internal_links + m.total_external_links),
            accent,
        ),
    ];

    for (i, (label, value, color)) in cards.iter().enumerate() {
        let text = vec![
            Line::from(Span::styled(*label, Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled(
                value.clone(),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            )),
        ];

        let p = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(40, 40, 50))),
            )
            .alignment(Alignment::Center);

        f.render_widget(p, layout[i]);
    }
}

fn render_http_health(f: &mut Frame, area: Rect, m: &SeoMetrics) {
    let block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
        .title(Span::styled(
            " HTTP STATUS ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .margin(1)
        .split(inner);

    let cards = [
        ("2xx", m.status_2xx, Color::Green, "Success"),
        ("3xx", m.status_3xx, Color::Cyan, "Redirect"),
        ("4xx", m.status_4xx, Color::Yellow, "Client"),
        ("5xx", m.status_5xx, Color::Red, "Server"),
    ];

    for (i, (code, count, color, label)) in cards.iter().enumerate() {
        let percentage = if m.total_pages > 0 {
            (count * 100) / m.total_pages
        } else {
            0
        };

        let text = vec![
            Line::from(Span::styled(
                *code,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                count.to_string(),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("{}%", percentage),
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                *label,
                Style::default().fg(Color::Rgb(80, 80, 90)),
            )),
        ];

        let p = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(40, 40, 50))),
            )
            .alignment(Alignment::Center);

        f.render_widget(p, layout[i]);
    }
}

fn render_seo_fundamentals(f: &mut Frame, area: Rect, m: &SeoMetrics) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
        .title(Span::styled(
            " SEO FUNDAMENTALS ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = vec![
        create_metric_row(
            "Title Tags",
            m.pages_with_title,
            m.total_pages,
            Some((m.optimal_titles, "optimal")),
        ),
        create_metric_row(
            "Meta Desc",
            m.pages_with_desc,
            m.total_pages,
            Some((m.optimal_descriptions, "optimal")),
        ),
        create_metric_row(
            "H1 Tags",
            m.pages_with_h1,
            m.total_pages,
            Some((m.missing_h1, "missing")),
        ),
        create_metric_row(
            "Indexable",
            m.indexable,
            m.total_pages,
            Some((m.noindex, "noindex")),
        ),
        create_metric_row("Mobile", m.mobile_friendly, m.total_pages, None),
        create_metric_row("Canonical", m.pages_with_canonicals, m.total_pages, None),
        create_metric_row("Lang Attr", m.pages_with_lang, m.total_pages, None),
    ];

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(25),
            Constraint::Percentage(40),
        ],
    )
    .column_spacing(1);

    f.render_widget(table, inner);
}

fn render_content_structure(f: &mut Frame, area: Rect, m: &SeoMetrics) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
        .title(Span::styled(
            " CONTENT STRUCTURE ",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Left: Headings
    let heading_rows = vec![
        Row::new(vec![
            Cell::from("H1"),
            Cell::from(m.total_h1.to_string()).style(Style::default().fg(Color::Cyan)),
        ]),
        Row::new(vec![
            Cell::from("H2"),
            Cell::from(m.total_h2.to_string()).style(Style::default().fg(Color::Cyan)),
        ]),
        Row::new(vec![
            Cell::from("H3"),
            Cell::from(m.total_h3.to_string()).style(Style::default().fg(Color::Cyan)),
        ]),
        Row::new(vec![
            Cell::from("H4"),
            Cell::from(m.total_h4.to_string()).style(Style::default().fg(Color::DarkGray)),
        ]),
        Row::new(vec![
            Cell::from("H5"),
            Cell::from(m.total_h5.to_string()).style(Style::default().fg(Color::DarkGray)),
        ]),
        Row::new(vec![
            Cell::from("H6"),
            Cell::from(m.total_h6.to_string()).style(Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let heading_table = Table::new(
        heading_rows,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .block(Block::default().borders(Borders::RIGHT).title(" Headings "))
    .column_spacing(1);

    f.render_widget(heading_table, layout[0]);

    // Right: Links & Images
    let link_rows = vec![
        Row::new(vec![
            Cell::from("Internal"),
            Cell::from(format_number(m.total_internal_links))
                .style(Style::default().fg(Color::Green)),
        ]),
        Row::new(vec![
            Cell::from("External"),
            Cell::from(format_number(m.total_external_links))
                .style(Style::default().fg(Color::Yellow)),
        ]),
        Row::new(vec![
            Cell::from("Images"),
            Cell::from(m.total_images.to_string()).style(Style::default().fg(Color::Magenta)),
        ]),
        Row::new(vec![
            Cell::from("Alt Text"),
            Cell::from(format!("{}/{}", m.images_with_alt, m.total_images)).style(
                if m.images_missing_alt > 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
        ]),
        Row::new(vec![
            Cell::from("Schema"),
            Cell::from(format!("{} pages", m.pages_with_schema))
                .style(Style::default().fg(Color::Cyan)),
        ]),
    ];

    let link_table = Table::new(
        link_rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(Block::default().title(" Links & Media "))
    .column_spacing(1);

    f.render_widget(link_table, layout[1]);
}

fn render_technical_details(f: &mut Frame, area: Rect, m: &SeoMetrics) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: Resources
    let res_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
        .title(Span::styled(
            " RESOURCES ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let res_text = vec![
        Line::from(vec![
            Span::raw("CSS Files:    "),
            Span::styled(
                format!("{}", m.total_css_files),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Inline CSS:   "),
            Span::styled(
                format!("{} pages", m.pages_with_inline_css),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("JS Files:     "),
            Span::styled(
                format!("{}", m.total_js_files),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(vec![
            Span::raw("Inline JS:    "),
            Span::styled(
                format!("{} pages", m.pages_with_inline_js),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(res_text)
            .block(res_block)
            .wrap(Wrap { trim: true }),
        chunks[0],
    );

    // Right: Performance
    let perf_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
        .title(Span::styled(
            " PERFORMANCE ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));

    let avg_desktop = if m.desktop_samples > 0 {
        m.desktop_score_sum / m.desktop_samples as f64
    } else {
        0.0
    };
    let avg_mobile = if m.mobile_samples > 0 {
        m.mobile_score_sum / m.mobile_samples as f64
    } else {
        0.0
    };

    let perf_text = vec![
        Line::from(vec![
            Span::raw("Desktop CWV:  "),
            score_span(avg_desktop * 100.0),
        ]),
        Line::from(vec![
            Span::raw("Samples:      "),
            Span::styled(
                format!("{}", m.desktop_samples),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Mobile CWV:   "),
            score_span(avg_mobile * 100.0),
        ]),
        Line::from(vec![
            Span::raw("Samples:      "),
            Span::styled(
                format!("{}", m.mobile_samples),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(perf_text)
            .block(perf_block)
            .wrap(Wrap { trim: true }),
        chunks[1],
    );
}

// --- Helper Functions ---

fn create_metric_row<'a>(
    label: &'a str,
    value: usize,
    total: usize,
    extra: Option<(usize, &'a str)>,
) -> Row<'a> {
    let percentage = if total > 0 { (value * 100) / total } else { 0 };
    let color = if percentage >= 90 {
        Color::Green
    } else if percentage >= 70 {
        Color::Yellow
    } else {
        Color::Red
    };

    let extra_text = if let Some((count, desc)) = extra {
        format!(" ({} {})", count, desc)
    } else {
        String::new()
    };

    Row::new(vec![
        Cell::from(label),
        Cell::from(format!("{}/{}", value, total)).style(Style::default().fg(color)),
        Cell::from(format!("{}%{}", percentage, extra_text))
            .style(Style::default().fg(Color::DarkGray)),
    ])
}

fn format_size(bytes: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.1}GB", b / GB)
    } else if b >= MB {
        format!("{:.1}MB", b / MB)
    } else if b >= KB {
        format!("{:.1}KB", b / KB)
    } else {
        format!("{}B", bytes)
    }
}

fn format_number(num: usize) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}

fn score_span(score: f64) -> Span<'static> {
    let (color, text) = if score >= 90.0 {
        (Color::Green, format!("{:.0}", score))
    } else if score >= 50.0 {
        (Color::Yellow, format!("{:.0}", score))
    } else if score > 0.0 {
        (Color::Red, format!("{:.0}", score))
    } else {
        (Color::DarkGray, "N/A".to_string())
    };

    Span::styled(
        text,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}
