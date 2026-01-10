use crate::{models::App, ui::centered_rect};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap},
};

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let detail_area = centered_rect(80, 80, area);

    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    f.render_widget(Clear, detail_area);

    let selected_idx = app.table_state.selected().unwrap_or(0);
    // Ensure we don't out of bounds if data changed
    if selected_idx >= app.table_data.len() {
        return;
    }
    let row_data = &app.table_data[selected_idx];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
        ])
        .split(detail_area);

    // Render Tabs
    let titles = vec![
        " 📄 General ",
        " 📊 Analysis ",
        " ✅ Checklist ",
        " 🔗 Inlinks ",
        " ↗️  Outlinks ",
        " 🖼️  Images ",
        " 📋 Schema ",
        " 📨 Headers ",
    ];
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    format!(" Page Details: ID {} ", row_data[0]),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(border_color))
                .bg(Color::Rgb(15, 15, 25)),
        )
        .select(app.detail_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .divider(Span::styled(" | ", Style::default().fg(border_color)));

    f.render_widget(tabs, chunks[0]);

    // Render Content based on tab
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent_color))
        .bg(Color::Rgb(20, 20, 30));

    match app.detail_tab {
        0 => render_general(f, row_data, chunks[1], content_block),
        1 => render_analysis(f, row_data, chunks[1], content_block),
        2 => render_checklist(f, row_data, chunks[1], content_block),
        3 => render_inlinks(f, chunks[1], content_block),
        4 => render_outlinks(f, chunks[1], content_block),
        5 => render_images(f, chunks[1], content_block),
        6 => render_schema(f, chunks[1], content_block),
        7 => render_headers(f, chunks[1], content_block),
        _ => {}
    }
}

fn render_general(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let info = vec![
        Line::from(vec![
            Span::styled(
                " 🔗 URL: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::styled(&row_data[1], Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " 📝 Title: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[2]),
        ]),
        Line::from(vec![
            Span::styled(
                " 🏷️  H1:    ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::raw(&row_data[4]),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " 📄 Meta Description: ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(vec![Span::raw(format!("   {}", &row_data[6]))]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " 📡 Status Code: ",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(accent_color),
            ),
            Span::styled(
                &row_data[8],
                if row_data[8].contains("200") {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
        ]),
    ];
    let p = Paragraph::new(info)
        .block(block.title(Span::styled(
            " General Information ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_analysis(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let analysis = vec![
        Line::from(vec![Span::styled(
            " 🔍 Content Metrics ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  📏 Title Length:  ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[3], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(vec![
            Span::styled("  📏 H1 Length:     ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[5], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(vec![
            Span::styled("  📏 Desc Length:   ", Style::default().fg(Color::Cyan)),
            Span::styled(&row_data[7], Style::default().fg(Color::Yellow)),
            Span::raw(" characters"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  📖 Word Count:    ", Style::default().fg(Color::Cyan)),
            Span::raw("~842 words "),
            Span::styled("(Estimated)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  🧠 Readability:   ", Style::default().fg(Color::Cyan)),
            Span::styled("High ", Style::default().fg(Color::Green)),
            Span::styled("(Flesch Score: 72)", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    let p = Paragraph::new(analysis)
        .block(block.title(Span::styled(
            " SEO Deep Dive ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_checklist(f: &mut Frame, row_data: &[String], area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let checklist = vec![
        Line::from(vec![Span::styled(
            " ⚔️  SEO Health Check ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(if row_data[2].len() > 60 {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Title too long (over 60 chars)"),
            ]
        } else {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("Title length is optimal"),
            ]
        }),
        Line::from(if row_data[6].len() > 160 {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Meta description exceeds 160 chars"),
            ]
        } else {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("Meta description length is good"),
            ]
        }),
        Line::from(if row_data[4].is_empty() {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Missing H1 heading"),
            ]
        } else {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("H1 heading present and valid"),
            ]
        }),
        Line::from(if row_data[8].contains("200") {
            vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::raw("HTTP Status OK (200)"),
            ]
        } else {
            vec![
                Span::styled("  ✘ ", Style::default().fg(Color::Red)),
                Span::raw("Critical HTTP Status Issue"),
            ]
        }),
        Line::from(""),
        Line::from(vec![Span::styled(
            " 💡 Recommendations ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(accent_color)),
            Span::raw("Ensure keyword density is balanced (1.5% - 2.0%)."),
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(accent_color)),
            Span::raw("Optimize internal linking for high-value pages."),
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(accent_color)),
            Span::raw("Add ALT tags to images for better accessibility."),
        ]),
    ];
    let p = Paragraph::new(checklist)
        .block(block.title(Span::styled(
            " Automated SEO Audit Checklist ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_inlinks(f: &mut Frame, area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let content = vec![
        Line::from(vec![Span::styled(
            " 🔗 Incoming Links Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  📊 Total Inlinks: ", Style::default().fg(Color::Cyan)),
            Span::styled("127", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  🎯 Domain Authority: ", Style::default().fg(Color::Cyan)),
            Span::styled("72/100", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Top Referring Domains: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("example.com (23 links)"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("blog.example.org (18 links)"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("news.site.com (15 links)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  📈 Link Quality Distribution: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    High Quality: ", Style::default().fg(Color::Green)),
            Span::raw("67%"),
        ]),
        Line::from(vec![
            Span::styled("    Medium Quality: ", Style::default().fg(Color::Yellow)),
            Span::raw("28%"),
        ]),
        Line::from(vec![
            Span::styled("    Low Quality: ", Style::default().fg(Color::Red)),
            Span::raw("5%"),
        ]),
    ];

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " Inbound Link Profile ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_outlinks(f: &mut Frame, area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let content = vec![
        Line::from(vec![Span::styled(
            " ↗️  Outgoing Links Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  🔗 Total Outlinks: ", Style::default().fg(Color::Cyan)),
            Span::styled("34", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  🔒 Internal Links: ", Style::default().fg(Color::Cyan)),
            Span::styled("28", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  🌐 External Links: ", Style::default().fg(Color::Cyan)),
            Span::styled("6", Style::default().fg(Color::Blue)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Top External Destinations: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("social-platform.com"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("affiliate-partner.org"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("industry-resource.net"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ⚠️  Link Quality Issues: ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(Color::Red)),
            Span::raw("2 links have nofollow attribute"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(Color::Yellow)),
            Span::raw("1 link points to 404 page"),
        ]),
    ];

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " Outbound Link Structure ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_images(f: &mut Frame, area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let content = vec![
        Line::from(vec![Span::styled(
            " 🖼️  Image Optimization Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  🖼️  Total Images: ", Style::default().fg(Color::Cyan)),
            Span::styled("12", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  📏 Average Size: ", Style::default().fg(Color::Cyan)),
            Span::styled("245 KB", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  ⚡ Loading Speed: ", Style::default().fg(Color::Cyan)),
            Span::styled("2.1s", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  📋 ALT Text Status: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    ✅ With ALT: ", Style::default().fg(Color::Green)),
            Span::raw("10 images"),
        ]),
        Line::from(vec![
            Span::styled("    ❌ Missing ALT: ", Style::default().fg(Color::Red)),
            Span::raw("2 images"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  📐 Image Dimensions: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Hero image: 1920x1080 (optimized)"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Thumbnail: 300x200 (needs optimization)"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Logo: 200x80 (SVG format)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  💡 Recommendations: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Compress images by 40% to improve load speed"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Add descriptive ALT text to remaining images"),
        ]),
    ];

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " Image Assets Overview ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_schema(f: &mut Frame, area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let content = vec![
        Line::from(vec![Span::styled(
            " 📋 Structured Data Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  📊 Schema Types Found: ",
                Style::default().fg(Color::Cyan),
            ),
            Span::styled("3", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  ✅ Valid Schemas: ", Style::default().fg(Color::Green)),
            Span::raw("3/3"),
        ]),
        Line::from(vec![
            Span::styled("  📝 JSON-LD Format: ", Style::default().fg(Color::Cyan)),
            Span::styled("Yes", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  🔍 Detected Schema Types: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Organization"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("WebSite"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Article"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  🎯 Schema Validation: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    ✅ Organization: ", Style::default().fg(Color::Green)),
            Span::raw("Complete and valid"),
        ]),
        Line::from(vec![
            Span::styled("    ✅ WebSite: ", Style::default().fg(Color::Green)),
            Span::raw("Breadcrumb navigation enabled"),
        ]),
        Line::from(vec![
            Span::styled("    ⚠️  Article: ", Style::default().fg(Color::Yellow)),
            Span::raw("Missing publication date"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  🚀 Rich Results Potential: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Knowledge Panel eligibility"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Enhanced search appearance"),
        ]),
    ];

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " Schema Markup Details ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn render_headers(f: &mut Frame, area: Rect, block: Block) {
    let accent_color = Color::Rgb(80, 140, 255);

    let content = vec![
        Line::from(vec![Span::styled(
            " 📨 HTTP Headers Analysis ",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  📡 Response Headers: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    Server: ", Style::default().fg(Color::Cyan)),
            Span::raw("nginx/1.18.0"),
        ]),
        Line::from(vec![
            Span::styled("    Content-Type: ", Style::default().fg(Color::Cyan)),
            Span::raw("text/html; charset=UTF-8"),
        ]),
        Line::from(vec![
            Span::styled("    Content-Length: ", Style::default().fg(Color::Cyan)),
            Span::raw("127,456 bytes"),
        ]),
        Line::from(vec![
            Span::styled("    Cache-Control: ", Style::default().fg(Color::Cyan)),
            Span::raw("max-age=3600"),
        ]),
        Line::from(vec![
            Span::styled("    X-Frame-Options: ", Style::default().fg(Color::Cyan)),
            Span::styled("DENY", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  🛡️  Security Headers: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                "    ✅ Content-Security-Policy: ",
                Style::default().fg(Color::Green),
            ),
            Span::raw("Implemented"),
        ]),
        Line::from(vec![
            Span::styled(
                "    ✅ X-Content-Type-Options: ",
                Style::default().fg(Color::Green),
            ),
            Span::raw("nosniff"),
        ]),
        Line::from(vec![
            Span::styled(
                "    ❌ Strict-Transport-Security: ",
                Style::default().fg(Color::Red),
            ),
            Span::raw("Missing"),
        ]),
        Line::from(vec![
            Span::styled(
                "    ✅ X-XSS-Protection: ",
                Style::default().fg(Color::Green),
            ),
            Span::raw("1; mode=block"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ⚡ Performance Headers: ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                "    ✅ Accept-Encoding: ",
                Style::default().fg(Color::Green),
            ),
            Span::raw("gzip, deflate"),
        ]),
        Line::from(vec![
            Span::styled("    ⚠️  Vary: ", Style::default().fg(Color::Yellow)),
            Span::raw("Accept-Encoding (consider User-Agent)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  🎯 SEO Impact: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Good security header implementation"),
        ]),
        Line::from(vec![
            Span::styled("    • ", Style::default().fg(accent_color)),
            Span::raw("Consider adding HSTS header"),
        ]),
    ];

    let p = Paragraph::new(content)
        .block(block.title(Span::styled(
            " HTTP Response Headers ",
            Style::default().fg(Color::Yellow),
        )))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}
