use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};

use crate::app::{App, AppState};

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Define layout constraints based on visibility
    let mut constraints = vec![];

    if app.sidebar_visible {
        constraints.push(Constraint::Length(20)); // Sidebar width
    }
    constraints.push(Constraint::Min(10)); // Main area
    if app.task_panel_visible {
        constraints.push(Constraint::Length(30)); // Task panel width
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(size);

    let mut chunk_idx = 0;

    // Sidebar (Dynamic Panel)
    if app.sidebar_visible {
        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(chunks[chunk_idx]);

        let sidebar_tab_area = sidebar_chunks[0];
        let sidebar_content_area = sidebar_chunks[1];
        
        app.sidebar_tab_rect = Some(sidebar_tab_area);

        let sidebar_titles = vec!["Settings", "Filters", "Stats", "Actions"];
        let sidebar_tabs = Tabs::new(sidebar_titles)
            .block(Block::default().borders(Borders::ALL).title(" Tool Panel "))
            .select(app.sidebar_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(sidebar_tabs, sidebar_tab_area);

        // Sidebar Content based on tab
        let content_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
            
        match app.sidebar_tab {
            0 => {
                let items = vec![
                    ListItem::new("• Threads: 4"),
                    ListItem::new("• Timeout: 30s"),
                    ListItem::new("• User-Agent: AtalaiaBot"),
                    ListItem::new("• Depth Limit: 5"),
                ];
                let list = List::new(items).block(content_block.title(" Crawler Settings "));
                f.render_widget(list, sidebar_content_area);
            }
            1 => {
                let items = vec![
                    ListItem::new("[x] No-Follow"),
                    ListItem::new("[ ] No-Index"),
                    ListItem::new("[x] Status 200 ONLY"),
                    ListItem::new("[ ] External URLs"),
                ];
                let list = List::new(items).block(content_block.title(" Result Filters "));
                f.render_widget(list, sidebar_content_area);
            }
            2 => {
                let p = Paragraph::new("Current Progress: 45%\nPages Searched: 120\nErrors Found: 2")
                    .block(content_block.title(" Live Stats "));
                f.render_widget(p, sidebar_content_area);
            }
            3 => {
                let items = vec![
                    ListItem::new("▶ START CRAWL"),
                    ListItem::new("⏸ PAUSE"),
                    ListItem::new("⏹ STOP"),
                    ListItem::new("💾 EXPORT CSV"),
                ];
                let list = List::new(items).block(content_block.title(" Quick Actions "));
                f.render_widget(list, sidebar_content_area);
            }
            _ => {}
        }
        
        chunk_idx += 1;
    }

    // Main area
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(chunks[chunk_idx]);

    let tab_area = main_layout[0];
    let content_area = main_layout[1];

    app.tab_rect = Some(tab_area);

    let titles = vec!["Crawl", "Logs", "Connectors", "Dashboard", "Reports", "Chat"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Navigation "))
        .select(app.get_state_index())
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, tab_area);

    render_content(f, app, content_area);

    // Task panel
    if app.task_panel_visible {
        chunk_idx += 1;
        let task_block = Block::default()
            .title(" Tasks ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));

        let task_items: Vec<ListItem> = app
            .tasks
            .iter()
            .map(|task| ListItem::new(format!("• {}", task)))
            .collect();

        let task_list = List::new(task_items).block(task_block);

        f.render_widget(task_list, chunks[chunk_idx]);
    }

    if app.show_help {
        render_help_modal(f);
    }
}

fn render_help_modal(f: &mut Frame) {
    let area = f.size();
    let help_area = centered_rect(70, 70, area);
    
    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .bg(Color::Black);

    let text = vec![
        Line::from(vec![Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))]),
        Line::from("  h / l | ← / →  - Change Main Tabs / Toggle Panel"),
        Line::from("  j / k | ↓ / ↑  - Navigate Tool Panel Tabs"),
        Line::from("  1 - 6          - Direct Main Tab Access"),
        Line::from("  Tab / S-Tab    - Cycle Main Tabs"),
        Line::from(""),
        Line::from(vec![Span::styled("Tool Shortcuts", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))]),
        Line::from("  s              - Jump to Crawler Settings"),
        Line::from("  f              - Jump to Result Filters"),
        Line::from("  i              - Jump to Live Stats"),
        Line::from("  a              - Jump to Quick Actions"),
        Line::from(""),
        Line::from(vec![Span::styled("General", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))]),
        Line::from("  ?              - Toggle this Help Menu"),
        Line::from("  Esc            - Reset View / Close Help"),
        Line::from("  q              - Quit Application"),
    ];

    let p = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(ratatui::widgets::Clear, help_area);
    f.render_widget(p, help_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    match app.current_state {
        AppState::Dashboard => render_dashboard(f, app, area),
        AppState::Crawl => {
            let block = Block::default()
                .title(" Crawl ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green));
            
            // Define keyword positions manually for now in this demo
            // In a real app, you might use a more dynamic way to calculate Rects
            let content = vec![
                Line::from(vec![
                    Span::raw("Welcome to "),
                    Span::styled("Atalaia SEO", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[SETTINGS]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("  "),
                    Span::styled("[FILTERS]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("  "),
                    Span::styled("[STATS]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("  "),
                    Span::styled("[ACTIONS]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from("Click a keyword above to open relevant tools."),
                Line::from(""),
                Line::from("Enter URL to crawl:"),
                Line::from("______________________________"),
            ];

            // Store the hit-test regions for keywords
            // This is a bit hacky in TUI because we need to know where the text was rendered
            // For this version, we'll use fixed offsets relative to the 'area'
            app.keyword_rects.clear();
            let base_x = area.x + 1;
            let base_y = area.y + 3; // where the [SETTINGS] line is
            
            app.keyword_rects.push(("settings".to_string(), Rect::new(base_x, base_y, 10, 1)));
            app.keyword_rects.push(("filters".to_string(), Rect::new(base_x + 12, base_y, 9, 1)));
            app.keyword_rects.push(("stats".to_string(), Rect::new(base_x + 23, base_y, 7, 1)));
            app.keyword_rects.push(("actions".to_string(), Rect::new(base_x + 32, base_y, 9, 1)));

            let p = Paragraph::new(content).block(block).wrap(Wrap { trim: true });
            f.render_widget(p, area);
        }
        AppState::Logs => {
            let block = Block::default()
                .title(" System Logs ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            
            let log_items: Vec<ListItem> = app.logs_data.iter().map(|log| {
                let color = if log.contains("ERROR") {
                    Color::Red
                } else if log.contains("DEBUG") {
                    Color::DarkGray
                } else {
                    Color::Green
                };
                ListItem::new(Line::from(Span::styled(log, Style::default().fg(color))))
            }).collect();

            let list = List::new(log_items).block(block);
            f.render_widget(list, area);
        }
        AppState::Connectors => {
            let block = Block::default()
                .title(" API Connectors ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue));
            
            let connector_items: Vec<ListItem> = app.connectors_data.iter().map(|(name, status)| {
                let status_text = if *status { "[CONNECTED]" } else { "[DISCONNECTED]" };
                let status_color = if *status { Color::Green } else { Color::Red };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:<30}", name), Style::default()),
                    Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                ]))
            }).collect();

            let list = List::new(connector_items).block(block);
            f.render_widget(list, area);
        }
        _ => {
            let block = Block::default()
                .title(format!(" {:?} ", app.current_state))
                .borders(Borders::ALL);
            let p = Paragraph::new(format!("Content for {:?} coming soon...", app.current_state))
                .block(block)
                .wrap(Wrap { trim: true });
            f.render_widget(p, area);
        }
    }
}

fn render_dashboard(f: &mut Frame, app: &mut App, area: Rect) {
    let header_titles = ["ID", "Name", "Status", "Date", "Value", "Category", "Notes"];
    
    let header = Row::new(header_titles.iter().map(|h| {
        Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
    }))
    .style(Style::default().bg(Color::Rgb(30, 30, 60)))
    .height(1);

    let rows = app.table_data.iter().map(|data| {
        let cells = data.iter().map(|c| {
            let style = match c.as_str() {
                "Active" => Style::default().fg(Color::Green),
                "Inactive" => Style::default().fg(Color::Red),
                _ => Style::default(),
            };
            Cell::from(c.as_str()).style(style)
        });
        Row::new(cells).height(1).bottom_margin(0)
    });

    // Responsive widths: use a mix of Min and Percentage
    let widths = [
        Constraint::Length(4),
        Constraint::Percentage(20),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Percentage(15),
        Constraint::Min(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" SEO Data Overview "))
        .column_spacing(2)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_widget(table, area);
}
