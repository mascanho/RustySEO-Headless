use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, List, ListItem},
};

pub fn render(f: &mut Frame, headers: &[String], area: Rect, block: Block) {
    let items = if headers.is_empty() {
        vec![ListItem::new(Span::raw("No headers captured."))]
    } else {
        headers
            .iter()
            .map(|h| ListItem::new(h.clone()))
            .collect::<Vec<_>>()
    };

    let list = List::new(items)
        .block(block.title(Span::styled(
            " HTTP Response Headers ",
            Style::default().fg(Color::Yellow),
        )))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}
