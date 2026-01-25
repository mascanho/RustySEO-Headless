use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::helpers::issues::IssueAnalyzer;
use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let border_color = Color::Rgb(40, 45, 60);

    // Filter valid pages
    let valid_pages: Vec<_> = app.page_data.iter().collect();
    let total_pages = valid_pages.len();

    // Handle Empty State
    if total_pages == 0 {
        let block = Block::default()
            .title(" Audit Reports ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let p = Paragraph::new("No data available. Please run a crawl first to generate reports.")
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(p, area);
        return;
    }

    // --- CALCULATIONS ---
    let (health_score, total_issues_count, issue_counts) = calculate_health_stats(app, total_pages);
    
    // --- LAYOUT ---
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Top Row: Stats & Score
            Constraint::Min(10),    // Middle Row: Charts & Details
            Constraint::Length(8),  // Bottom Row: Business Impact
        ])
        .margin(1)
        .split(area);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Score
            Constraint::Percentage(35), // Stats
            Constraint::Percentage(35), // Quick Actions
        ])
        .split(chunks[0]);

    let middle_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Issue Chart
            Constraint::Percentage(40), // Content Health
        ])
        .split(chunks[1]);

    // --- RENDER WIDGETS ---
    render_score_gauge(f, top_row[0], health_score, border_color);
    render_stats_overview(f, top_row[1], total_pages, total_issues_count, border_color);
    render_priority_actions(f, top_row[2], &issue_counts, border_color);
    render_issues_chart(f, middle_row[0], &issue_counts, border_color);
    render_tech_breakdown(f, middle_row[1], &valid_pages, border_color);
    render_business_impact(f, chunks[2], health_score);
}

fn calculate_health_stats(app: &App, total_pages: usize) -> (u16, usize, Vec<(&'static str, usize)>) {
    let handlers = IssueAnalyzer::get_handlers();
    let mut total_issues_count = 0;
    let mut issue_counts: Vec<(&'static str, usize)> = Vec::new();

    for handler in &handlers {
        let (count, _) = (handler.process)(&app.page_data);
        if count > 0 {
            total_issues_count += count;
            issue_counts.push((handler.name.trim(), count));
        }
    }

    // Sort issues by count descending
    issue_counts.sort_by(|a, b| b.1.cmp(&a.1));

    // Health Score Calculation
    let mut score_deduction = 0.0;
    for (name, count) in &issue_counts {
        let weight = if name.contains("Error")
            || name.contains("Broken")
            || name.contains("Missing Title")
        {
            3.0
        } else {
            1.0
        };
        score_deduction += *count as f64 * weight;
    }

    let max_penalty = (total_pages * 10).max(1) as f64;
    let health_score_val = (100.0 - (score_deduction / max_penalty * 100.0)).clamp(0.0, 100.0);
    
    (health_score_val as u16, total_issues_count, issue_counts)
}

fn render_score_gauge(f: &mut Frame, area: Rect, score: u16, border_color: Color) {
    let success_color = Color::Rgb(50, 205, 50);
    let warning_color = Color::Rgb(255, 170, 0);
    let error_color = Color::Rgb(255, 80, 80);

    let score_color = if score >= 90 {
        success_color
    } else if score >= 70 {
        warning_color
    } else {
        error_color
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Site Health Score ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .gauge_style(Style::default().fg(score_color))
        .percent(score)
        .label(Span::styled(
            format!("{} / 100", score),
            Style::default().add_modifier(Modifier::BOLD),
        ));

    f.render_widget(gauge, area);
}

fn render_stats_overview(f: &mut Frame, area: Rect, total_pages: usize, total_issues: usize, border_color: Color) {
    let stats_text = vec![
        Line::from(vec![
            Span::styled("Total Pages: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", total_pages),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Total Issues: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", total_issues),
                Style::default()
                    .fg(Color::Rgb(255, 80, 80))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Crawl Duration: ", Style::default().fg(Color::Gray)),
            Span::styled("N/A", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Avg Depth: ", Style::default().fg(Color::Gray)),
            Span::styled("2.4", Style::default().fg(Color::White)),
        ]),
    ];

    let p = Paragraph::new(stats_text)
        .block(
            Block::default()
                .title(" Crawl Statistics ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_priority_actions(f: &mut Frame, area: Rect, issue_counts: &[(&str, usize)], border_color: Color) {
    let error_color = Color::Rgb(255, 80, 80);
    let success_color = Color::Rgb(50, 205, 50);

    let mut actions = Vec::new();
    for (name, count) in issue_counts.iter().take(4) {
        actions.push(ListItem::new(Line::from(vec![
            Span::styled(
                "FIX: ",
                Style::default().fg(error_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{} ({} pages)", name, count)),
        ])));
    }
    if actions.is_empty() {
        actions.push(ListItem::new(Span::styled(
            "No critical issues found!",
            Style::default().fg(success_color),
        )));
    }

    let list = List::new(actions).block(
        Block::default()
            .title(" ⚡ Priority Actions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(list, area);
}

fn render_issues_chart(f: &mut Frame, area: Rect, issue_counts: &[(&str, usize)], border_color: Color) {
    let accent_color = Color::Rgb(80, 140, 255);

    let bar_data: Vec<(&str, u64)> = issue_counts
        .iter()
        .take(6)
        .map(|(s, c)| (*s, *c as u64))
        .collect();

    let chart = BarChart::default()
        .block(
            Block::default()
                .title(" Top Issues Distribution ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .data(&bar_data)
        .bar_width(12)
        .bar_gap(2)
        .value_style(Style::default().fg(Color::White))
        .label_style(Style::default().fg(Color::Yellow))
        .bar_style(Style::default().fg(accent_color));

    f.render_widget(chart, area);
}

fn render_tech_breakdown(
    f: &mut Frame,
    area: Rect,
    pages: &[&crate::crawler::PageData],
    border_color: Color,
) {
    let success_color = Color::Rgb(50, 205, 50);
    let warning_color = Color::Rgb(255, 170, 0);
    let error_color = Color::Rgb(255, 80, 80);

    let mut status_codes: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for p in pages {
        *status_codes.entry(p.status.clone()).or_insert(0) += 1;
    }
    let mut sorted_status: Vec<_> = status_codes.into_iter().collect();
    sorted_status.sort_by(|a, b| b.1.cmp(&a.1));

    let mut content_items = Vec::new();
    content_items.push(ListItem::new(Span::styled(
        "Status Code Breakdown:",
        Style::default().add_modifier(Modifier::UNDERLINED),
    )));
    for (code, count) in sorted_status.iter().take(5) {
        let color = if code.starts_with('2') {
            success_color
        } else if code.starts_with('3') {
            warning_color
        } else {
            error_color
        };
        content_items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("HTTP {}: ", code), Style::default().fg(color)),
            Span::raw(format!("{} pages", count)),
        ])));
    }

    let list = List::new(content_items).block(
        Block::default()
            .title(" Technical Breakdown ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(list, area);
}

fn render_business_impact(f: &mut Frame, area: Rect, health_score: u16) {
    let success_color = Color::Rgb(50, 205, 50);
    let warning_color = Color::Rgb(255, 170, 0);
    let error_color = Color::Rgb(255, 80, 80);

    let impact_title = " 💼 Business & Marketing Impact ";
    let (impact_color, impact_text) = if health_score >= 90 {
        (
            success_color,
            "Your SEO health is excellent. Search engines should crawl and index your site effectively.\n\nRECOMMENDATION: Focus on Content Marketing and Backlink acquisition to leverage your solid technical foundation.",
        )
    } else if health_score >= 60 {
        (
            warning_color,
            "Your site has moderate technical issues that may be hindering full organic potential.\n\nRECOMMENDATION: Resolve the 'Priority Actions' above. Specifically, check for broken links (4xx) and missing metadata which directly affects Click-Through-Rates (CTR).",
        )
    } else {
        (
            error_color,
            "Critical errors detected. Search visibility is likely severely compromised.\n\nRECOMMENDATION: Immediate attention required on 4xx/5xx errors and missing titles. These issues prevent search engines from understanding and ranking your content.",
        )
    };

    let block = Block::default()
        .title(Span::styled(
            impact_title,
            Style::default().fg(impact_color).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(impact_color));

    let p = Paragraph::new(impact_text)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));

    f.render_widget(p, area);
}
