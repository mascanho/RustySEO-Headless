use crate::models::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
};

pub fn render(f: &mut Frame, _app: &mut App, area: Rect, content_block: Block) {
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
    f.render_widget(list, area);
}
