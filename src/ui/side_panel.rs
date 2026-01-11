use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App) {
    if !app.sidebar_visible {
        return;
    }

    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    let area = f.area();

    let width = (area.width / 3).max(35).min(area.width);
    let modal_area = Rect {
        x: area.width.saturating_sub(width),
        y: 0,
        width,
        height: area.height,
    };

    f.render_widget(Clear, modal_area);

    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(modal_area);

    let sidebar_tab_area = sidebar_chunks[0];
    let sidebar_content_area = sidebar_chunks[1];

    app.sidebar_tab_rect = Some(sidebar_tab_area);

    let sidebar_titles = vec![
        " ⚙ Settings ",
        " 📊 Summary ",
        "  Filter ",
        " ⚡ Act ",
        "📚 Bookmarks",
    ];
    let sidebar_tabs = Tabs::new(sidebar_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    " 🛠️  Tools ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(border_color))
                .bg(Color::Rgb(15, 15, 25)),
        )
        .select(app.sidebar_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .divider(Span::styled(" | ", Style::default().fg(border_color)));

    f.render_widget(sidebar_tabs, sidebar_tab_area);

    // Sidebar Content based on tab
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .bg(Color::Rgb(15, 15, 25));

    match app.sidebar_tab {
        0 => {
            // Settings Tab
            let mut items = Vec::new();
            if let Some(settings) = &app.settings {
                // Crawler Section
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(" 🕷️  CRAWLER ENGINE ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ])));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   Max Pages:      ", Style::default().fg(accent_color)),
                    Span::styled(settings.crawler.max_pages.to_string(), Style::default().fg(Color::White)),
                ])));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   Concurrency:    ", Style::default().fg(accent_color)),
                    Span::styled(settings.crawler.concurrency.to_string(), Style::default().fg(Color::White)),
                ])));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   Stay on Domain: ", Style::default().fg(accent_color)),
                    Span::styled(settings.crawler.stay_on_domain.to_string(), Style::default().fg(if settings.crawler.stay_on_domain { Color::Green } else { Color::Red })),
                ])));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   User Agent:     ", Style::default().fg(accent_color)),
                    Span::styled(settings.crawler.user_agent.clone(), Style::default().fg(Color::DarkGray)),
                ])));
                
                items.push(ListItem::new(""));
                
                // UI Section
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(" 🎨 UI / AESTHETICS ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                ])));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   Theme:          ", Style::default().fg(accent_color)),
                    Span::styled(settings.ui.theme.clone(), Style::default().fg(Color::Cyan)),
                ])));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   Refresh Rate:   ", Style::default().fg(accent_color)),
                    Span::styled(format!("{}ms", settings.ui.refresh_rate_ms), Style::default().fg(Color::White)),
                ])));
                
                items.push(ListItem::new(""));
                
                // System Section
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(" 🖥️  SYSTEM / DATA ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ])));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   Database:       ", Style::default().fg(accent_color)),
                    Span::styled(settings.system.database_path.clone(), Style::default().fg(Color::DarkGray)),
                ])));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("   Export Format:  ", Style::default().fg(accent_color)),
                    Span::styled(settings.system.export_format.clone().to_uppercase(), Style::default().fg(Color::Green)),
                ])));
                
                items.push(ListItem::new(""));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(" 📂 Config Path: ", Style::default().fg(Color::DarkGray)),
                ])));
                let path = crate::models::AppSettings::path();
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("   {}", path.display()), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                ])));
            } else {
                items.push(ListItem::new(" No settings loaded. "));
            }

            let list = List::new(items).block(content_block.title(Span::styled(
                " App Configuration ",
                Style::default().fg(Color::Yellow),
            )));
            f.render_widget(list, sidebar_content_area);
        }
        1 => {
            // Compute crawl summary stats (Previously arm 0)
            let total_pages = app.page_data.len();
            let mut title_stats = (0, 0, 0); // <30, 30-60, >60
            let mut desc_stats = (0, 0, 0); // <120, 120-160, >160
            let mut status_counts = std::collections::HashMap::new();
            let mut mobile_yes = 0;
            let mut mobile_no = 0;
            let mut indexable_yes = 0;
            let mut indexable_no = 0;
            let mut heading_counts = std::collections::HashMap::new();
            let mut total_headings = 0;

            for page in &app.page_data {
                // Titles
                if page.title_len < 30 {
                    title_stats.0 += 1;
                } else if page.title_len <= 60 {
                    title_stats.1 += 1;
                } else {
                    title_stats.2 += 1;
                }

                // Descriptions
                if page.description_len < 120 {
                    desc_stats.0 += 1;
                } else if page.description_len <= 160 {
                    desc_stats.1 += 1;
                } else {
                    desc_stats.2 += 1;
                }

                // Status
                *status_counts.entry(page.status.clone()).or_insert(0) += 1;

                // Mobile
                if page.mobile {
                    mobile_yes += 1;
                } else {
                    mobile_no += 1;
                }

                // Indexable
                if page.indexability.to_lowercase().contains("noindex") {
                    indexable_no += 1;
                } else {
                    indexable_yes += 1;
                }

                // Headings
                for (tag, _) in &page.headings {
                    *heading_counts.entry(tag.clone()).or_insert(0) += 1;
                    total_headings += 1;
                }
            }

            let mut items = vec![
                ListItem::new(""),
                ListItem::new(Line::from(vec![
                    Span::styled("Total Pages: ", Style::default().fg(accent_color)),
                    Span::raw(total_pages.to_string()),
                ])),
                ListItem::new(""),
                ListItem::new(Line::from(Span::styled(
                    "PAGE TITLES",
                    Style::default()
                        .add_modifier(Modifier::UNDERLINED)
                        .fg(Color::Cyan),
                ))),
                ListItem::new(Line::from(vec![
                    Span::styled("  < 30 chars: ", Style::default().fg(accent_color)),
                    Span::raw(title_stats.0.to_string()),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  30-60 chars: ", Style::default().fg(accent_color)),
                    Span::raw(title_stats.1.to_string()),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  > 60 chars: ", Style::default().fg(accent_color)),
                    Span::raw(total_headings.to_string()), // Simplified display
                ])),
            ];
            
            // Re-adding titles stats correctly
            items[6] = ListItem::new(Line::from(vec![
                Span::styled("  > 60 chars: ", Style::default().fg(accent_color)),
                Span::raw(title_stats.2.to_string()),
            ]));

            items.extend(vec![
                ListItem::new(""),
                ListItem::new(Line::from(Span::styled(
                    "META DESCRIPTIONS",
                    Style::default()
                        .add_modifier(Modifier::UNDERLINED)
                        .fg(Color::Cyan),
                ))),
                ListItem::new(Line::from(vec![
                    Span::styled("  < 120 chars: ", Style::default().fg(accent_color)),
                    Span::raw(desc_stats.0.to_string()),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  120-160 chars: ", Style::default().fg(accent_color)),
                    Span::raw(desc_stats.1.to_string()),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  > 160 chars: ", Style::default().fg(accent_color)),
                    Span::raw(desc_stats.2.to_string()),
                ])),
                ListItem::new(""),
                ListItem::new(Line::from(Span::styled(
                    "STATUS CODES",
                    Style::default()
                        .add_modifier(Modifier::UNDERLINED)
                        .fg(Color::Cyan),
                ))),
            ]);

            let mut status_keys: Vec<_> = status_counts.keys().collect();
            status_keys.sort();
            for status in status_keys {
                let count = status_counts.get(status).unwrap();
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("  {}: ", status), Style::default().fg(accent_color)),
                    Span::raw(count.to_string()),
                ])));
            }

            items.extend(vec![
                ListItem::new(""),
                ListItem::new(Line::from(Span::styled(
                    "TECHNICAL",
                    Style::default()
                        .add_modifier(Modifier::UNDERLINED)
                        .fg(Color::Cyan),
                ))),
                ListItem::new(Line::from(vec![
                    Span::styled("  Mobile Friendly: ", Style::default().fg(accent_color)),
                    Span::raw(format!("Yes: {}, No: {}", mobile_yes, mobile_no)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled("  Indexable: ", Style::default().fg(accent_color)),
                    Span::raw(format!("Yes: {}, No: {}", indexable_yes, indexable_no)),
                ])),
            ]);

            let list = List::new(items).block(content_block.title(Span::styled(
                " Crawl Summary ",
                Style::default().fg(Color::Yellow),
            )));
            f.render_widget(list, sidebar_content_area);
        }
        2 => {
            // Filters (Previously arm 1)
            let items = vec![
                ListItem::new(Line::from(vec![
                    Span::styled(" [x] ", Style::default().fg(Color::Green)),
                    Span::raw("No-Follow Links"),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled(" [ ] ", Style::default().fg(Color::DarkGray)),
                    Span::raw("No-Index Pages"),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled(" [x] ", Style::default().fg(Color::Green)),
                    Span::raw("Status 200 Only"),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled(" [ ] ", Style::default().fg(Color::DarkGray)),
                    Span::raw("External Domains"),
                ])),
            ];
            let list = List::new(items).block(content_block.title(Span::styled(
                " Scan Filters ",
                Style::default().fg(Color::Yellow),
            )));
            f.render_widget(list, sidebar_content_area);
        }
        3 => {
            // Actions (Previously arm 3)
            let items = vec![
                ListItem::new(Line::from(vec![
                    Span::styled(" ▶ ", Style::default().fg(Color::Green)),
                    Span::styled("START CRAWL", Style::default().add_modifier(Modifier::BOLD)),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled(" ⏸ ", Style::default().fg(Color::Yellow)),
                    Span::raw("PAUSE"),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled(" ⏹ ", Style::default().fg(Color::Red)),
                    Span::raw("STOP"),
                ])),
                ListItem::new(Line::from(vec![
                    Span::styled(" 💾 ", Style::default().fg(Color::Cyan)),
                    Span::raw("EXPORT DATA"),
                ])),
            ];
            let list = List::new(items).block(content_block.title(Span::styled(
                " Control Pad ",
                Style::default().fg(Color::Yellow),
            )));
            f.render_widget(list, sidebar_content_area);
        }

        // Bookmarks
        4 => {
            let main_block = content_block.title(Span::styled(
                " 📚 Bookmarks ",
                Style::default().fg(Color::Yellow),
            ));
            let inner_area = main_block.inner(sidebar_content_area);
            f.render_widget(main_block, sidebar_content_area);

            let bookmark_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(inner_area);

            let list_area = bookmark_chunks[0];
            let input_area = bookmark_chunks[1];

            let items: Vec<ListItem> = app
                .bookmarks
                .iter()
                .enumerate()
                .map(|(i, url)| {
                    let is_selected = i == app.bookmark_index;
                    let mut style = Style::default();
                    if is_selected {
                        style = style
                            .fg(accent_color)
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::REVERSED);
                    }
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            " 🌐 ",
                            Style::default().fg(if is_selected {
                                accent_color
                            } else {
                                Color::Cyan
                            }),
                        ),
                        Span::styled(url.as_str(), style),
                    ]))
                })
                .collect();

            let list = List::new(items);
            f.render_widget(list, list_area);

            // Input for adding new bookmark
            let input_block = Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    " ➕ Add Bookmark / D - Delete Selected ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));

            let input_p = Paragraph::new(app.bookmark_input.as_str())
                .block(input_block)
                .style(Style::default().bg(Color::Rgb(20, 20, 30)));

            f.render_widget(input_p, input_area);

            // Set cursor in input
            if app.sidebar_visible && app.sidebar_tab == 4 {
                f.set_cursor_position((
                    input_area.x + app.bookmark_cursor as u16,
                    input_area.y + 1,
                ));
            }
        }
        _ => {}
    }
}
