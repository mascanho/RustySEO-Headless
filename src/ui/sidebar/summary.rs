use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
};
use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect, content_block: Block, accent_color: Color) {
    let total_pages = app.page_data.len();
    let mut title_stats = (0, 0, 0); // <30, 30-60, >60
    let mut desc_stats = (0, 0, 0); // <120, 120-160, >160
    let mut status_counts = std::collections::HashMap::new();
    let mut mobile_yes = 0;
    let mut mobile_no = 0;
    let mut indexable_yes = 0;
    let mut indexable_no = 0;
    let mut heading_counts = std::collections::HashMap::new();
    let mut total_headings = 0;

    for page in &app.page_data {
        if page.title_len < 30 { title_stats.0 += 1; }
        else if page.title_len <= 60 { title_stats.1 += 1; }
        else { title_stats.2 += 1; }

        if page.description_len < 120 { desc_stats.0 += 1; }
        else if page.description_len <= 160 { desc_stats.1 += 1; }
        else { desc_stats.2 += 1; }

        *status_counts.entry(page.status.clone()).or_insert(0) += 1;
        if page.mobile { mobile_yes += 1; } else { mobile_no += 1; }
        if page.indexability.to_lowercase().contains("noindex") { indexable_no += 1; } else { indexable_yes += 1; }
        for (tag, _) in &page.headings {
            *heading_counts.entry(tag.clone()).or_insert(0) += 1;
            total_headings += 1;
        }
    }

    let mut items = vec![
        ListItem::new(""),
        ListItem::new(Line::from(vec![
            Span::styled("Total Pages: ", Style::default().fg(accent_color)),
            Span::raw(total_pages.to_string()),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(Span::styled(
            "PAGE TITLES",
            Style::default().add_modifier(Modifier::UNDERLINED).fg(Color::Cyan),
        ))),
        ListItem::new(Line::from(vec![
            Span::styled("  < 30 chars:  ", Style::default().fg(accent_color)),
            Span::raw(title_stats.0.to_string()),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  30-60 chars: ", Style::default().fg(accent_color)),
            Span::raw(title_stats.1.to_string()),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  > 60 chars:  ", Style::default().fg(accent_color)),
            Span::raw(title_stats.2.to_string()),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(Span::styled(
            "META DESCRIPTIONS",
            Style::default().add_modifier(Modifier::UNDERLINED).fg(Color::Cyan),
        ))),
        ListItem::new(Line::from(vec![
            Span::styled("  < 120 chars:  ", Style::default().fg(accent_color)),
            Span::raw(desc_stats.0.to_string()),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  120-160 chars: ", Style::default().fg(accent_color)),
            Span::raw(desc_stats.1.to_string()),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  > 160 chars:  ", Style::default().fg(accent_color)),
            Span::raw(desc_stats.2.to_string()),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(Span::styled(
            "STATUS CODES",
            Style::default().add_modifier(Modifier::UNDERLINED).fg(Color::Cyan),
        ))),
    ];

    let mut status_keys: Vec<_> = status_counts.keys().collect();
    status_keys.sort();
    for status in status_keys {
        let count = status_counts.get(status).unwrap();
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {}: ", status), Style::default().fg(accent_color)),
            Span::raw(count.to_string()),
        ])));
    }

    items.extend(vec![
        ListItem::new(""),
        ListItem::new(Line::from(Span::styled(
            "TECHNICAL",
            Style::default().add_modifier(Modifier::UNDERLINED).fg(Color::Cyan),
        ))),
        ListItem::new(Line::from(vec![
            Span::styled("  Mobile Friendly: ", Style::default().fg(accent_color)),
            Span::raw(format!("Yes: {}, No: {}", mobile_yes, mobile_no)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  Indexable:      ", Style::default().fg(accent_color)),
            Span::raw(format!("Yes: {}, No: {}", indexable_yes, indexable_no)),
        ])),
        ListItem::new(""),
        ListItem::new(Line::from(Span::styled(
            "HEADINGS",
            Style::default().add_modifier(Modifier::UNDERLINED).fg(Color::Cyan),
        ))),
        ListItem::new(Line::from(vec![
            Span::styled("  Total Headings: ", Style::default().fg(accent_color)),
            Span::raw(total_headings.to_string()),
        ])),
    ]);

    let mut heading_keys: Vec<_> = heading_counts.keys().collect();
    heading_keys.sort();
    for tag in heading_keys {
        let count = heading_counts.get(tag).unwrap();
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("  {}: ", tag.to_uppercase()),
                Style::default().fg(accent_color),
            ),
            Span::raw(count.to_string()),
        ])));
    }

    let list = List::new(items).block(content_block.title(Span::styled(
        " Crawl Summary ",
        Style::default().fg(Color::Yellow),
    )));
    f.render_widget(list, area);
}
