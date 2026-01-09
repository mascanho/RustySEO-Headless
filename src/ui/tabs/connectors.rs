use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" API Connectors ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let connector_items: Vec<ListItem> = app
        .connectors_data
        .iter()
        .map(|(name, status)| {
            let status_text = if *status {
                "[CONNECTED]"
            } else {
                "[DISCONNECTED]"
            };
            let status_color = if *status { Color::Green } else { Color::Red };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<30}", name), Style::default()),
                Span::styled(
                    status_text,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
        })
        .collect();

    let list = List::new(connector_items).block(block);
    f.render_widget(list, area);
}
