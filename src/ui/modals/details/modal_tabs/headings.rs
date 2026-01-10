use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, List, ListItem},
};

pub fn render(f: &mut Frame, headings: &[(String, String)], area: Rect, block: Block) {
    let items = if headings.is_empty() {
        vec![ListItem::new(Span::styled(
            "No headings found on this page.",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        headings
            .iter()
            .filter_map(|(tag, text)| {
                let heading_text = text.trim();
                if heading_text.is_empty() {
                    None
                } else {
                    Some(ListItem::new(format!(
                        "{}: {}",
                        tag.to_uppercase(),
                        heading_text
                    )))
                }
            })
            .collect::<Vec<_>>()
    };

    let list = List::new(items)
        .block(block.title(Span::styled(
            " Headings Overview ",
            Style::default().fg(Color::Yellow),
        )))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}
