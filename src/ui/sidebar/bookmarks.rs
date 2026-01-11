use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use crate::models::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect, content_block: Block, accent_color: Color, border_color: Color) {
    let main_block = content_block.title(Span::styled(
        " 📚 Bookmarks ",
        Style::default().fg(Color::Yellow),
    ));
    let inner_area = main_block.inner(area);
    f.render_widget(main_block, area);

    let bookmark_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(inner_area);

    let list_area = bookmark_chunks[0];
    let input_area = bookmark_chunks[1];

    let items: Vec<ListItem> = app
        .bookmarks
        .iter()
        .enumerate()
        .map(|(i, url)| {
            let is_selected = i == app.bookmark_index;
            let mut style = Style::default();
            if is_selected {
                style = style
                    .fg(accent_color)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED);
            }
            ListItem::new(Line::from(vec![
                Span::styled(
                    " 🌐 ",
                    Style::default().fg(if is_selected {
                        accent_color
                    } else {
                        Color::Cyan
                    }),
                ),
                Span::styled(url.as_str(), style),
            ]))
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, list_area);

    let input_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            " ➕ Add Bookmark / D - Delete Selected ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let input_p = Paragraph::new(app.bookmark_input.as_str())
        .block(input_block)
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));

    f.render_widget(input_p, input_area);

    if app.sidebar_visible && app.sidebar_tab == 4 {
        f.set_cursor_position((
            input_area.x + app.bookmark_cursor as u16,
            input_area.y + 1,
        ));
    }
}
