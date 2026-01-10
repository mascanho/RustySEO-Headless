use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

pub fn render(f: &mut Frame, headings: &[(String, String)], area: Rect, block: Block) {
    let content = if headings.is_empty() {
        vec![Line::from(Span::raw("No headings found on this page."))]
    } else {
        headings
            .iter()
            .filter_map(|(tag, text)| {
                let heading_text = text.trim();
                if heading_text.is_empty() {
                    None
                } else {
                    Some(Line::from(Span::raw(format!(
                        "{}: {}",
                        tag.to_uppercase(),
                        heading_text
                    ))))
                }
            })
            .collect::<Vec<_>>()
    };

    let p = Paragraph::new(content).block(block.title(Span::styled(
        " Headings Overview ",
        Style::default().fg(Color::Yellow),
    )));

    f.render_widget(p, area);
}
