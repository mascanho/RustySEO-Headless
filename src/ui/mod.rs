use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
};

use crate::{app::AppState, models::App};

pub mod components;
pub mod footer;
pub mod modals;
pub mod side_panel;
pub mod sidebar;
pub mod tabs;

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Define main colors
    let bg_color = Color::Rgb(15, 15, 25);

    // Render full background first to ensure consistency
    f.render_widget(Block::default().bg(bg_color), size);

    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    // Main layout: Navigation (3) + Content Area (Min 0) + Footer (3)
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Navigation
            Constraint::Min(0),    // Content Area
            Constraint::Length(3), // Footer
        ])
        .split(size);

    let tab_area = main_layout[0];
    let content_area = main_layout[1];
    let footer_area = main_layout[2];

    app.tab_rect = Some(tab_area);

    // Render Navigation Tabs
    let titles = vec![
        "Overview",
        "External",
        "Internal",
        "Redirects",
        "Images",
        "CSS",
        "Javascript",
        "CWV",
        "Content",
        "Files",
        "Custom Extractor",
        "Connectors",
    ];
    let tabs = Tabs::new(titles.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" RustySEO - CLI ")
                .border_style(Style::default().fg(border_color)),
        )
        .select(app.get_state_index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(accent_color)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .divider(Span::styled(" | ", Style::default().fg(border_color)));

    // Compute clickable areas for each tab label, mirroring the Tabs widget layout:
    // 1 col padding on each side of a title + " | " (3 cols) divider between tabs,
    // all inside the bordered block. Truncated titles shrink to the visible width.
    app.tab_hitboxes.clear();
    {
        let inner_x = tab_area.x + 1;
        let end_x = tab_area.x + tab_area.width.saturating_sub(1);
        let mut cursor = inner_x;
        for title in &titles {
            if cursor >= end_x {
                break;
            }
            cursor += 1; // left padding
            if cursor >= end_x {
                break;
            }
            let title_width = title.len().min((end_x - cursor) as usize) as u16;
            let start = cursor;
            cursor += title_width;
            let hitbox_width = if cursor < end_x {
                cursor += 1; // right padding
                title_width + 2
            } else {
                title_width
            };
            if hitbox_width > 0 {
                app.tab_hitboxes.push(Rect {
                    x: start,
                    y: tab_area.y,
                    width: hitbox_width,
                    height: tab_area.height,
                });
            }
            cursor += 3; // " | " divider
        }
    }

    f.render_widget(tabs, tab_area);

    // Render Tab Content
    match app.current_state {
        AppState::Dashboard => tabs::dashboard::render(f, app, content_area),
        AppState::External => tabs::external::render(f, app, content_area),
        AppState::Internal => tabs::internal::render(f, app, content_area),
        AppState::Css => tabs::css::render(f, app, content_area),
        AppState::Javascript => tabs::javascript::render(f, app, content_area),
        AppState::CoreWebVitals => tabs::cwv::render(f, app, content_area),
        AppState::CustomExtractor => tabs::custom_extractor::render(f, app, content_area),
        AppState::Images => tabs::images::render(f, app, content_area),
        AppState::Redirects => tabs::redirects::render(f, app, content_area),

        AppState::Content => tabs::content::render(f, app, content_area),
        AppState::Files => tabs::files::render(f, app, content_area),
        AppState::DataInsights => tabs::data_insights::render(f, app, content_area),
    }

    // Render Footer
    footer::render(f, app, footer_area);

    // Render Modals (Side Panel, Help)
    side_panel::render(f, app);

    if app.show_details {
        modals::details::render(f, app);
    }

    if app.show_dashboard_menu {
        modals::dashboard_menu::render(f, app);
    }

    if app.show_seo_score_modal {
        modals::seo_score::render(f, app);
    }

    if app.show_page_links_modal {
        modals::page_links::render(f, app);
    }

    if app.show_action_result_modal {
        modals::action_result::render(f, app);
    }

    // Render Input Modal when in input mode
    if app.input_mode {
        let modal_area = centered_rect(25, 6, size);

        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(vec![
                Span::styled(
                    " Crawl URL ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " (Esc to Cancel, Enter to crawl ) ",
                    Style::default().fg(Color::Gray),
                ),
            ])
            .border_style(Style::default().fg(accent_color));

        let input_p = Paragraph::new(app.input.as_str())
            .block(input_block)
            .style(Style::default().bg(Color::Rgb(20, 20, 30)));

        f.render_widget(Clear, modal_area);
        f.render_widget(input_p, modal_area);

        // Make the cursor visible in the modal
        f.set_cursor_position((
            modal_area.x + app.cursor_position as u16 + 1,
            modal_area.y + 1,
        ));
    }

    if app.show_help {
        render_help_modal(f);
    }

    if app.show_logs {
        let height = app.logs_height.min(content_area.height);
        let logs_area = Rect::new(
            content_area.x,
            content_area.y + content_area.height.saturating_sub(height),
            content_area.width,
            height,
        );
        f.render_widget(Clear, logs_area);
        tabs::logs::render(f, app, logs_area);
    }

    if app.show_ai_modal {
        modals::ai_chat::render(f, app);
    }

    if app.show_js_pages_modal {
        modals::js_pages::render(f, app);
    }

    if app.show_css_pages_modal {
        modals::css_pages::render(f, app);
    }

    if app.show_issue_urls_modal {
        modals::issue_urls::render(f, app);
    }
}

fn render_help_modal(f: &mut Frame) {
    let area = f.area();
    let help_area = centered_rect(94, 92, area);
    let accent_color = Color::Rgb(80, 140, 255);
    let header_color = Color::Yellow;
    let key_color = Color::Cyan;
    let mod_color = Color::Rgb(255, 170, 0); // Shift / Ctrl combos
    let dim_color = Color::DarkGray;
    let bg_color = Color::Rgb(10, 10, 20);

    fn kv(key: &str, desc: &str, color: Color) -> Line<'static> {
        let width = 13usize;
        let pad = width.saturating_sub(key.chars().count());
        let mut field = String::from(" ");
        field.push_str(key);
        field.extend(std::iter::repeat(' ').take(pad));
        Line::from(vec![
            Span::styled(field, Style::default().fg(color).bold()),
            Span::raw(desc.to_string()),
        ])
    }

    fn hdr(text: &str, bg: Color, fg: Color) -> Line<'static> {
        Line::from(vec![Span::styled(
            format!(" {} ", text),
            Style::default().fg(fg).bg(bg).bold(),
        )])
    }

    fn col_title(text: &str, color: Color) -> Line<'static> {
        Line::from(vec![Span::styled(
            text.to_string(),
            Style::default()
                .fg(color)
                .bold()
                .add_modifier(Modifier::UNDERLINED),
        )])
    }

    let block = Block::default()
        .title(Span::styled(
            " ⌨  RustySEO CLI — Keyboard Shortcuts ",
            Style::default()
                .fg(header_color)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD),
        )
        .bg(bg_color);

    f.render_widget(Clear, help_area);
    f.render_widget(block.clone(), help_area);

    let inner_area = block.inner(help_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // tagline
            Constraint::Length(1), // spacer
            Constraint::Min(0),    // columns
            Constraint::Length(1), // spacer
            Constraint::Length(1), // footer
        ])
        .split(inner_area);

    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            "Everything below is keyboard-first — clicking tabs, rows and the sidebar works too",
            Style::default().fg(dim_color).italic(),
        )]))
        .alignment(Alignment::Center),
        rows[0],
    );

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .margin(1)
        .split(rows[2]);

    // COLUMN 1 — NAVIGATION
    let col1 = vec![
        col_title("🧭 NAVIGATION", accent_color),
        Line::from(""),
        hdr("GLOBAL", header_color, bg_color),
        kv("q", "Quit application", Color::Red),
        kv("?", "Toggle this help", key_color),
        kv("Esc", "Reset / close panel", key_color),
        kv("Ctrl+i", "Open URL input", key_color),
        kv("Shift+D", "Export tab (.xlsx)", mod_color),
        kv("Shift+A", "Toggle AI Copilot", mod_color),
        kv("Shift+L", "Toggle System Logs", mod_color),
        Line::from(""),
        hdr("MAIN TABS", header_color, bg_color),
        kv("Tab", "Next main tab", key_color),
        kv("Backspace", "Previous main tab", key_color),
        kv("1..9, 0", "Overview → Files", key_color),
        kv("e", "Custom Extractor", key_color),
        Line::from(""),
        hdr("TABLE NAVIGATION", header_color, bg_color),
        kv("k/↑  j/↓", "Move row", key_color),
        kv("G", "Jump to bottom", key_color),
        kv("[ / ]", "Prev / next page", key_color),
        kv("Enter", "Open details / assets", key_color),
        kv("m", "Actions menu", key_color),
        Line::from(""),
        hdr("SEARCH & FILTER", header_color, bg_color),
        kv("Ctrl+f", "Search active tab", mod_color),
        kv("Enter/Esc", "Apply & close search", key_color),
        Line::from(""),
        hdr("MOUSE", header_color, bg_color),
        kv("Click", "Switch tabs", key_color),
        kv("Scroll", "Navigate rows/details", key_color),
    ];

    // COLUMN 2 — SIDEBAR
    let col2 = vec![
        col_title("🗂  SIDEBAR", accent_color),
        Line::from(""),
        hdr("QUICK JUMPS", header_color, bg_color),
        kv("g", "General", key_color),
        kv("i", "Issues", key_color),
        kv("b / f", "Bookmarks", key_color),
        kv("t / a", "Tree View", key_color),
        kv("s / +", "Settings", key_color),
        Line::from(""),
        hdr("SIDEBAR CONTROLS", header_color, bg_color),
        kv("Esc/h/←", "Close sidebar", key_color),
        kv("k/↑  j/↓", "Prev / next tab", key_color),
        kv("Tab/Sh+Tab", "Cycle tabs (+Robots/Sitemaps)", mod_color),
        Line::from(""),
        hdr("ISSUES TAB", header_color, bg_color),
        kv("k/↑  j/↓", "Navigate issues", key_color),
        kv("Enter", "Open issue URLs", key_color),
        kv("Shift+E", "Edit settings file", mod_color),
        Line::from(""),
        hdr("BOOKMARKS TAB", header_color, bg_color),
        kv("←/→", "Bookmarks/Recent", key_color),
        kv("↑/↓", "Navigate list", key_color),
        kv("(type)", "Add bookmark URL", key_color),
        kv("Enter", "Crawl / add", key_color),
        kv("Shift+D", "Delete bookmark", mod_color),
        kv("Esc", "Clear / close", key_color),
        Line::from(""),
        hdr("TREE VIEW TAB", header_color, bg_color),
        kv("↑/↓", "Navigate tree", key_color),
        kv("Enter/Space", "Expand / collapse", key_color),
        kv("Shift+E", "Expand all", mod_color),
        kv("Shift+C", "Collapse all", mod_color),
        Line::from(""),
        hdr("ROBOTS & SITEMAPS", header_color, bg_color),
        kv("k/↑  j/↓", "Scroll content", key_color),
        kv("Esc/h/←", "Close sidebar", key_color),
    ];

    // COLUMN 3 — MODALS I
    let col3 = vec![
        col_title("🪟 MODALS · I", accent_color),
        Line::from(""),
        hdr("URL INPUT", header_color, bg_color),
        kv("(type)", "Enter a URL", key_color),
        kv("Enter", "Start crawl", key_color),
        kv("←/→", "Move cursor", key_color),
        kv("Esc", "Cancel", key_color),
        Line::from(""),
        hdr("PAGE DETAILS", header_color, bg_color),
        kv("q/Esc", "Close", key_color),
        kv("←/h  →/Tab", "Prev / next tab", key_color),
        kv("Shift+Tab", "Previous tab", mod_color),
        kv("k/↑  j/↓", "Scroll / rows", key_color),
        kv("Shift+↑/↓", "Navigate tab content", mod_color),
        kv("Scroll", "Mouse wheel supported", key_color),
        Line::from(""),
        hdr("ACTIONS MENU (m)", header_color, bg_color),
        kv("k/↑  j/↓", "Navigate actions", key_color),
        kv("Enter", "Run action", key_color),
        kv("q/Esc", "Close", key_color),
        Line::from(vec![Span::styled(
            "   Copy · Browser · Google",
            Style::default().fg(dim_color),
        )]),
        Line::from(vec![Span::styled(
            "   SEO Score · Extract Links",
            Style::default().fg(dim_color),
        )]),
        Line::from(vec![Span::styled(
            "   Screenshot · Export Data",
            Style::default().fg(dim_color),
        )]),
        Line::from(""),
        hdr("TASK RESULT", header_color, bg_color),
        kv("Enter/q/Esc", "Dismiss", key_color),
    ];

    // COLUMN 4 — MODALS II
    let col4 = vec![
        col_title("🪟 MODALS · II", accent_color),
        Line::from(""),
        hdr("SEO SCORE", header_color, bg_color),
        kv("q/Esc", "Close", key_color),
        Line::from(""),
        hdr("EXTRACT LINKS", header_color, bg_color),
        kv("k/↑  j/↓", "Navigate links", key_color),
        kv("Enter", "Open in browser", key_color),
        kv("q/Esc", "Close", key_color),
        Line::from(""),
        hdr("JS / CSS PAGES", header_color, bg_color),
        kv("k/↑  j/↓", "Pages using asset", key_color),
        kv("q/Esc", "Close", key_color),
        Line::from(""),
        hdr("ISSUE URLS", header_color, bg_color),
        kv("k/↑  j/↓", "Navigate URLs", key_color),
        kv("Enter", "Open in browser", key_color),
        kv("c", "Copy URL", key_color),
        kv("q/Esc", "Close", key_color),
        Line::from(""),
        hdr("SYSTEM LOGS", header_color, bg_color),
        kv("k/j  ↑/↓", "Navigate logs", key_color),
        kv("t / G", "Top / bottom", key_color),
        kv("[ / ]", "Resize console", key_color),
        kv("Ctrl+s", "Search logs", mod_color),
        kv("q/Esc/Sh+L", "Close", key_color),
        Line::from(""),
        hdr("AI COPILOT", header_color, bg_color),
        kv("(type)", "Compose message", key_color),
        kv("Enter", "Send", key_color),
        kv("↑/↓", "Scroll 1 line", key_color),
        kv("PgUp/PgDn", "Scroll 5 lines", key_color),
        kv("q/Esc", "Close", key_color),
    ];

    f.render_widget(
        Paragraph::new(col1).style(Style::default().fg(Color::Gray)),
        cols[0],
    );
    f.render_widget(
        Paragraph::new(col2).style(Style::default().fg(Color::Gray)),
        cols[1],
    );
    f.render_widget(
        Paragraph::new(col3).style(Style::default().fg(Color::Gray)),
        cols[2],
    );
    f.render_widget(
        Paragraph::new(col4).style(Style::default().fg(Color::Gray)),
        cols[3],
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" q ", Style::default().fg(bg_color).bg(Color::Red).bold()),
            Span::raw(" / "),
            Span::styled(" Esc ", Style::default().fg(bg_color).bg(key_color).bold()),
            Span::raw(" / "),
            Span::styled(" ? ", Style::default().fg(bg_color).bg(key_color).bold()),
            Span::raw("  to close this cheat-sheet"),
        ]))
        .alignment(Alignment::Center),
        rows[4],
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
