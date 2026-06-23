use crate::models::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
};

pub fn render(f: &mut Frame, app: &App, area: Rect, block: Block) {
    let title = Span::styled(
        format!(" Sitemaps ({}) ", app.sitemap_urls.len()),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    if app.sitemap_urls.is_empty() {
        let msg = if app.robots_urls_loading {
            " Fetching sitemaps…"
        } else {
            " No sitemap URLs found or crawl not started."
        };

        let p = Paragraph::new(Span::styled(msg, Style::default().fg(Color::DarkGray)))
            .block(block.title(title));
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = app
        .sitemap_urls
        .iter()
        .enumerate()
        .skip(app.sidebar_scroll)
        .map(|(i, url)| {
            let is_xml = url.ends_with(".xml");
            let color = if is_xml {
                Color::Rgb(200, 100, 255)
            } else {
                Color::Rgb(150, 220, 150)
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:>4}  ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(url.as_str(), Style::default().fg(color)),
            ]))
        })
        .collect();

    let list = List::new(items).block(block.title(title));
    f.render_widget(list, area);
}
