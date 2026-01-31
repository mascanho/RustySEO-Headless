use crate::models::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Sparkline, Table, Wrap},
    Frame,
};
use tui_piechart::{PieChart, PieSlice};

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

    // Calculated for Visuals
    health_score: u16,
    word_count_distribution: Vec<u64>,
    page_size_distribution: Vec<u64>,
}

// --- Main Render Function ---

pub fn render(f: &mut Frame, app: &mut App, area: Rect, content_block: Block, accent_color: Color) {
    let metrics = collect_metrics(app);

    let block = content_block.title(Span::styled(
        " SEO OVERVIEW ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
// ... (rest of render is same)
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(12),
            Constraint::Length(11),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .margin(1)
        .split(inner_area);

    render_header_stats(f, chunks[0], &metrics, accent_color);
    render_charts_area(f, chunks[1], &metrics);
    render_seo_fundamentals(f, chunks[2], &metrics);
    render_content_structure(f, chunks[3], &metrics);
    render_technical_details(f, chunks[4], &metrics);
}

fn collect_metrics(app: &App) -> SeoMetrics {
    let mut m = SeoMetrics::default();
    let pages = &app.page_summaries;

    m.total_pages = pages.len();
    m.total_css_files = app.css_urls_table_data.len();
    m.total_js_files = app.js_urls_table_data.len();

    for page in pages {
        m.total_size += page.size;
        m.total_words += page.word_count;

        if page.status.starts_with('2') {
            m.status_2xx += 1;
        } else if page.status.starts_with('3') {
            m.status_3xx += 1;
        } else if page.status.starts_with('4') {
            m.status_4xx += 1;
        } else if page.status.starts_with('5') {
            m.status_5xx += 1;
        }

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

        if page.h1_count > 0 {
            m.pages_with_h1 += 1;
            if page.h1_count > 1 {
                m.multiple_h1 += 1;
            }
        } else {
            m.missing_h1 += 1;
        }

        let indexability_lower = page.indexability.to_lowercase();
        if indexability_lower.contains("noindex") {
            m.noindex += 1;
        } else {
            m.indexable += 1;
        }

        if page.mobile {
            m.mobile_friendly += 1;
        }

        m.total_h1 += page.h1_count;
        m.total_h2 += page.h2_count;
        m.total_h3 += page.h3_count;
        m.total_h4 += page.h4_count;
        m.total_h5 += page.h5_count;
        m.total_h6 += page.h6_count;

        m.total_internal_links += page.internal_link_count;
        m.total_external_links += page.external_link_count;

        if page.is_canonical {
            m.pages_with_canonicals += 1;
        }

        m.total_images += page.images_count;
        m.images_missing_alt += page.images_missing_alt;
        m.images_with_alt += page.images_count.saturating_sub(page.images_missing_alt);

        if page.has_schema {
            m.pages_with_schema += 1;
            m.total_schema_objects += page.schema_count;
        }

        if !page.language.is_empty() {
            m.pages_with_lang += 1;
        }

        if let Some(score) = page.cwv_performance_desktop {
            m.desktop_score_sum += score;
            m.desktop_samples += 1;
        }
        if let Some(score) = page.cwv_performance_mobile {
            m.mobile_score_sum += score;
            m.mobile_samples += 1;
        }

        m.word_count_distribution.push(page.word_count as u64);
        m.page_size_distribution.push(page.size as u64);
    }

    if m.total_pages > 0 {
        let error_rate = (m.status_4xx + m.status_5xx) as f64 / m.total_pages as f64;
        let warning_rate = (m.status_3xx) as f64 / m.total_pages as f64;
        let missing_meta_rate = (m.total_pages.saturating_sub(m.pages_with_title) + m.total_pages.saturating_sub(m.pages_with_desc) + m.missing_h1) as f64 / (3.0 * m.total_pages as f64);

        let mut score = 100.0;
        score -= error_rate * 40.0;
        score -= warning_rate * 10.0;
        score -= missing_meta_rate * 20.0;
        m.health_score = score.clamp(0.0, 100.0) as u16;
    } else {
        m.health_score = 100;
    }

    m
}

// --- Render Functions ---

const ACCENT_COLOR: Color = Color::Rgb(80, 140, 255);

fn render_header_stats(f: &mut Frame, area: Rect, m: &SeoMetrics, accent: Color) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(4)])
        .split(area);

    // 1. Health Score Gauge
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(ACCENT_COLOR).bg(Color::Rgb(30, 30, 40)))
        .ratio(m.health_score as f64 / 100.0)
        .label(Span::styled(
            format!(" Site Health: {}% ", m.health_score),
            Style::default().fg(Color::White).bold(),
        ));
    f.render_widget(gauge, chunks[0]);

    // 2. Key Stats Cards
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

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
                    .border_style(Style::default().fg(Color::Rgb(60, 60, 75))),
            )
            .alignment(Alignment::Center);

        f.render_widget(p, layout[i]);
    }
}

fn render_charts_area(f: &mut Frame, area: Rect, m: &SeoMetrics) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left: HTTP Status Pie Chart
    render_http_status_chart(f, layout[0], m);

    // Right: Distributions
    render_distributions(f, layout[1], m);
}

fn render_http_status_chart(f: &mut Frame, area: Rect, m: &SeoMetrics) {
    let label_2xx = format!("2xx ({})", m.status_2xx);
    let label_3xx = format!("3xx ({})", m.status_3xx);
    let label_4xx = format!("4xx ({})", m.status_4xx);
    let label_5xx = format!("5xx ({})", m.status_5xx);

    let slices = vec![
        PieSlice::new(&label_2xx, m.status_2xx as f64, Color::Green),
        PieSlice::new(&label_3xx, m.status_3xx as f64, Color::Cyan),
        PieSlice::new(&label_4xx, m.status_4xx as f64, Color::Yellow),
        PieSlice::new(&label_5xx, m.status_5xx as f64, Color::Red),
    ];

    let pie_chart = PieChart::new(slices)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
                .title(" Status Codes "),
        )
        .show_legend(true)
        .show_percentages(true);

    f.render_widget(pie_chart, area);
}

fn render_distributions(f: &mut Frame, area: Rect, m: &SeoMetrics) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Word Count Sparkline
    let wc_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
        .title(" Word Count Trend ");

    let wc_data = if m.word_count_distribution.len() > 100 {
        &m.word_count_distribution[m.word_count_distribution.len() - 100..]
    } else {
        &m.word_count_distribution
    };

    let wc_sparkline = Sparkline::default()
        .block(wc_block)
        .data(wc_data)
        .style(Style::default().fg(Color::Magenta));

    f.render_widget(wc_sparkline, chunks[0]);

    // Page Size Sparkline
    let ps_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
        .title(" Page Size Trend ");

    let ps_data = if m.page_size_distribution.len() > 100 {
        &m.page_size_distribution[m.page_size_distribution.len() - 100..]
    } else {
        &m.page_size_distribution
    };

    let ps_sparkline = Sparkline::default()
        .block(ps_block)
        .data(ps_data)
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(ps_sparkline, chunks[1]);
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
