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

    let area = f.size();
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

    let sidebar_titles = vec![" ⚙  Set ", "  Filter ", " 📊 Stat ", " ⚡ Act "];
    let sidebar_tabs = Tabs::new(sidebar_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(" 🛠️  Tools ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
                .border_style(Style::default().fg(border_color))
                .bg(Color::Rgb(15, 15, 25))
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
            let items = vec![
                ListItem::new(Line::from(vec![Span::styled(" 🧵 Threads: ", Style::default().fg(accent_color)), Span::raw("4")])),
                ListItem::new(Line::from(vec![Span::styled(" ⏱  Timeout: ", Style::default().fg(accent_color)), Span::raw("30s")])),
                ListItem::new(Line::from(vec![Span::styled(" 🤖 UserAgent: ", Style::default().fg(accent_color)), Span::raw("AtalaiaBot")])),
                ListItem::new(Line::from(vec![Span::styled(" ↕  Depth:   ", Style::default().fg(accent_color)), Span::raw("5")])),
            ];
            let list = List::new(items).block(content_block.title(Span::styled(" Crawler Configuration ", Style::default().fg(Color::Yellow))));
            f.render_widget(list, sidebar_content_area);
        }
        1 => {
            let items = vec![
                ListItem::new(Line::from(vec![Span::styled(" [x] ", Style::default().fg(Color::Green)), Span::raw("No-Follow Links")])),
                ListItem::new(Line::from(vec![Span::styled(" [ ] ", Style::default().fg(Color::DarkGray)), Span::raw("No-Index Pages")])),
                ListItem::new(Line::from(vec![Span::styled(" [x] ", Style::default().fg(Color::Green)), Span::raw("Status 200 Only")])),
                ListItem::new(Line::from(vec![Span::styled(" [ ] ", Style::default().fg(Color::DarkGray)), Span::raw("External Domains")])),
            ];
            let list = List::new(items).block(content_block.title(Span::styled(" Scan Filters ", Style::default().fg(Color::Yellow))));
            f.render_widget(list, sidebar_content_area);
        }
        2 => {
            let text = vec![
                Line::from(vec![Span::styled(" Progress: ", Style::default().fg(accent_color)), Span::styled("45%", Style::default().fg(Color::Yellow))]),
                Line::from(vec![Span::styled(" Audited:  ", Style::default().fg(accent_color)), Span::styled("120 ", Style::default().fg(Color::White)), Span::raw("URLs")]),
                Line::from(vec![Span::styled(" Issues:   ", Style::default().fg(accent_color)), Span::styled("2 ", Style::default().fg(Color::Red))]),
            ];
            let p = Paragraph::new(text).block(content_block.title(Span::styled(" Session Stats ", Style::default().fg(Color::Yellow))));
            f.render_widget(p, sidebar_content_area);
        }
        3 => {
            let items = vec![
                ListItem::new(Line::from(vec![Span::styled(" ▶ ", Style::default().fg(Color::Green)), Span::styled("START CRAWL", Style::default().add_modifier(Modifier::BOLD))])),
                ListItem::new(Line::from(vec![Span::styled(" ⏸ ", Style::default().fg(Color::Yellow)), Span::raw("PAUSE")])),
                ListItem::new(Line::from(vec![Span::styled(" ⏹ ", Style::default().fg(Color::Red)), Span::raw("STOP")])),
                ListItem::new(Line::from(vec![Span::styled(" 💾 ", Style::default().fg(Color::Cyan)), Span::raw("EXPORT DATA")])),
            ];
            let list = List::new(items).block(content_block.title(Span::styled(" Control Pad ", Style::default().fg(Color::Yellow))));
            f.render_widget(list, sidebar_content_area);
        }
        _ => {}
    }
}

