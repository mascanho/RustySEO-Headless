use crate::models::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
};

pub fn render(
    f: &mut Frame,
    app: &mut App,
    area: Rect,
    content_block: Block,
    accent_color: Color,
    border_color: Color,
) {
    let main_block = content_block;
    let inner_area = main_block.inner(area);
    f.render_widget(main_block, area);

    // Split into sections: tabs, content, input
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(inner_area);

    let tabs_area = chunks[0];
    let content_area = chunks[1];
    let input_area = chunks[2];

    // Render tab selector
    let titles = vec!["📚 Bookmarks", "⏰ Recent"];
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .border_style(Style::new().fg(Color::Rgb(50, 50, 50))),
        )
        .style(Style::default().fg(Color::Rgb(80, 80, 80)))
        .highlight_style(
            Style::default()
                // .fg(accent_color)
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .select(app.bookmark_subview);

    f.render_widget(tabs, tabs_area);

    // Render content based on selected tab
    if app.bookmark_subview == 0 {
        render_bookmarks_list(f, app, content_area, accent_color);
    } else {
        render_recent_crawls_list(f, app, content_area, accent_color);
    }

    // Render input/footer area
    if app.bookmark_subview == 0 {
        render_bookmarks_input(f, app, input_area, border_color);
    } else {
        render_recent_crawls_footer(f, input_area, border_color);
    }
}

fn render_bookmarks_list(f: &mut Frame, app: &mut App, area: Rect, accent_color: Color) {
    if app.bookmarks.is_empty() {
        let empty_msg =
            Paragraph::new("No bookmarks saved yet.\n\nAdd bookmarks using the input field below.")
                .style(
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                );
        f.render_widget(empty_msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .bookmarks
        .iter()
        .enumerate()
        .map(|(i, url)| {
            let is_selected = i == app.bookmark_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(accent_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:2}. ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("🌐 ", Style::default().fg(Color::Yellow)),
                Span::styled(url, style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                // .borders(Borders::ALL)
                .border_style(Style::default().fg(accent_color)),
        )
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));

    f.render_widget(list, area);
}

fn render_recent_crawls_list(f: &mut Frame, app: &mut App, area: Rect, accent_color: Color) {
    let recent_urls = app.get_recent_crawled_urls();

    // Adjust index if out of bounds
    if !recent_urls.is_empty() && app.last_crawled_index >= recent_urls.len() {
        app.last_crawled_index = 0;
    }

    if recent_urls.is_empty() {
        let empty_msg =
            Paragraph::new("No recent crawls found.\n\nStart crawling to see recent URLs here.")
                .style(
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                );
        f.render_widget(empty_msg, area);
        return;
    }

    let items: Vec<ListItem> = recent_urls
        .into_iter()
        .enumerate()
        .map(|(i, url)| {
            let is_selected = i == app.last_crawled_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(accent_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:2}. ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("🔍 ", Style::default().fg(Color::Green)),
                Span::styled(url, style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Recent Crawls ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));

    f.render_widget(list, area);
}

fn render_bookmarks_input(f: &mut Frame, app: &mut App, input_area: Rect, border_color: Color) {
    let input_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            " ➕ Add: Enter | Delete: D | Navigate: ↑↓ | ",
            Style::default().fg(Color::Yellow),
        ));

    let input_p = Paragraph::new(app.bookmark_input.as_str())
        .block(input_block)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)));

    f.render_widget(input_p, input_area);

    if app.sidebar_visible && app.sidebar_tab == 4 {
        // Position cursor inside the input block, accounting for the top border and title
        f.set_cursor_position((input_area.x + app.bookmark_cursor as u16, input_area.y + 1));
    }
}

fn render_recent_crawls_footer(f: &mut Frame, input_area: Rect, border_color: Color) {
    let footer_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            " 🚀 Crawl: Enter | Navigate: ↑↓ | Toggle: h | Back: Esc ",
            Style::default().fg(Color::Green),
        ));

    let footer_p = Paragraph::new("")
        .block(footer_block)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)));

    f.render_widget(footer_p, input_area);
}
