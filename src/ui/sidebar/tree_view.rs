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
        ListItem::new(Line::from(vec![
            Span::styled(
                if _app
                    .settings
                    .as_ref()
                    .map(|s| s.crawler.enable_javascript)
                    .unwrap_or(false)
                {
                    " ⚡ "
                } else {
                    " 💤 "
                },
                Style::default().fg(
                    if _app
                        .settings
                        .as_ref()
                        .map(|s| s.crawler.enable_javascript)
                        .unwrap_or(false)
                    {
                        Color::Green
                    } else {
                        Color::Gray
                    },
                ),
            ),
            Span::styled(
                if _app
                    .settings
                    .as_ref()
                    .map(|s| s.crawler.enable_javascript)
                    .unwrap_or(false)
                {
                    "JS MODE: ON"
                } else {
                    "JS MODE: OFF"
                },
                Style::default().add_modifier(
                    if _app
                        .settings
                        .as_ref()
                        .map(|s| s.crawler.enable_javascript)
                        .unwrap_or(false)
                    {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    },
                ),
            ),
        ])),
    ];
    let list = List::new(items).block(content_block.title(Span::styled(
        " Control Pad ",
        Style::default().fg(Color::Yellow),
    )));
    f.render_widget(list, area);
}
