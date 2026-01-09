use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Clear},
    Frame,
};
use crate::app::App;

pub fn render(f: &mut Frame, app: &mut App) {
    if !app.sidebar_visible {
        return;
    }

    let area = f.size();
    // Modal area for side panel (left side, covering about 30% width)
    let modal_area = Rect {
        x: 0,
        y: 0,
        width: (area.width / 3).max(35).min(area.width),
        height: area.height,
    };

    f.render_widget(Clear, modal_area); // Clear previous content

    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(modal_area);

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
}
