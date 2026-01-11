use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
};
use crate::models::App;

pub fn render(f: &mut Frame, _app: &mut App, area: Rect, content_block: Block) {
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
    f.render_widget(list, area);
}
