use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

pub fn render(f: &mut Frame, headers: &[String], area: Rect, block: Block) {
    let content = if headers.is_empty() {
        vec![Line::from(Span::raw("No headers captured."))]
    } else {
        headers
            .iter()
            .map(|h| Line::from(Span::raw(h.clone())))
            .collect::<Vec<_>>()
    };

    let p = Paragraph::new(content).block(block.title(Span::styled(
        "HTTP Response Headers ",
        Style::default().fg(Color::Yellow),
    )));

    f.render_widget(p, area.inner(Margin::new(1, 0)));
}
