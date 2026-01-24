use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::models::App;
use crate::ui::centered_rect;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let menu_area = centered_rect(25, 35, area);

    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = accent_color; // Blue border for actions menu

    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .bg(Color::Rgb(15, 15, 25))
        .title(Span::styled(
            " Actions Menu ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let inner_area = modal_block.inner(menu_area);
    f.render_widget(Clear, menu_area);
    f.render_widget(modal_block, menu_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Menu items
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    // Menu items
    let items = vec![
        ListItem::new("Export Data From Table"),
        ListItem::new(" Open URL in Browser"),
        ListItem::new(" Open in Google"),
        ListItem::new(" Check Keywords"),
        ListItem::new(" View SEO Score"),
        ListItem::new(" Extract Links"),
        ListItem::new(" Screenshot"),
        ListItem::new(" Export Data"),
    ];

    let mut menu_state = ListState::default();
    menu_state.select(Some(app.dashboard_menu_selection));

    let menu = List::new(items)
        .block(
            Block::default()
                .bg(Color::Rgb(20, 20, 30))
                .borders(Borders::NONE),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(accent_color)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .highlight_symbol("➔ ");

    f.render_stateful_widget(menu, chunks[0], &mut menu_state);

    // Footer
    let footer_text = Paragraph::new(Span::styled(
        " j/k/↑/↓: Navigate | Enter: Select | Esc: Close ",
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::ITALIC),
    ))
    .alignment(Alignment::Center);

    f.render_widget(footer_text, chunks[1]);
}

pub fn handle_action(app: &mut App, action_index: usize) {
    let selected_idx = match app.table_state.selected() {
        Some(idx) => idx,
        None => return,
    };

    // Use filtered_table_data as it matches the visual selection
    if selected_idx >= app.filtered_table_data.len() {
        return;
    }

    let row_data = &app.filtered_table_data[selected_idx];
    let url = row_data[1].clone(); // URL is at index 1

    match action_index {
        0 => {
            // Copy URL - now non-blocking
            copy_to_clipboard(url);
        }
        1 => {
            // Open URL in Browser
            open_in_browser(&url);
            app.logs_data
                .insert(0, format!("Opening URL in browser: {}", url));
        }
        2 => {
            // Check Keywords
            app.logs_data
                .insert(0, format!("Keywords check for: {}", url));
        }
        3 => {
            // View SEO Score
            app.logs_data.insert(0, format!("SEO Score for: {}", url));
        }
        4 => {
            // Extract Links
            app.logs_data
                .insert(0, format!("Extracting links from: {}", url));
        }
        5 => {
            // Screenshot
            app.logs_data
                .insert(0, format!("Taking screenshot of: {}", url));
        }
        6 => {
            // Export Data
            app.logs_data
                .insert(0, format!("Exporting data for: {}", url));
        }
        _ => {}
    }

    // Keep only last 100 logs
    if app.logs_data.len() > 100 {
        app.logs_data.pop();
    }

    app.show_dashboard_menu = false;
}

fn copy_to_clipboard(text: String) {
    std::thread::spawn(move || {
        #[cfg(target_os = "macos")]
        {
            use std::io::Write;
            use std::process::{Command, Stdio};
            match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                Ok(mut child) => {
                    if let Some(mut stdin) = child.stdin.take() {
                        if stdin.write_all(text.as_bytes()).is_ok() {
                            drop(stdin); // Send EOF
                            match child.wait() {
                                Ok(status) if status.success() => {
                                    tracing::info!("✅ URL copied to clipboard: {}", text);
                                }
                                Ok(status) => {
                                    tracing::error!("❌ pbcopy exited with status: {}", status);
                                }
                                Err(e) => {
                                    tracing::error!("❌ Failed to wait for pbcopy: {}", e);
                                }
                            }
                            return;
                        }
                    }
                    let _ = child.kill();
                    tracing::error!("❌ Failed to write to pbcopy stdin");
                }
                Err(e) => tracing::error!("❌ Failed to spawn pbcopy: {}", e),
            }
        }

        #[cfg(target_os = "linux")]
        {
            use std::io::Write;
            use std::process::{Command, Stdio};

            // Try Wayland first (wl-copy)
            let try_wl = Command::new("wl-copy").stdin(Stdio::piped()).spawn();

            if let Ok(mut child) = try_wl {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                    drop(stdin);
                    let _ = child.wait();
                    tracing::info!("✅ URL copied to clipboard (Wayland): {}", text);
                    return;
                }
            }

            // Try xclip
            let try_xclip = Command::new("xclip")
                .args(&["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .spawn();

            if let Ok(mut child) = try_xclip {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                    drop(stdin);
                    let _ = child.wait();
                    tracing::info!("✅ URL copied to clipboard (xclip): {}", text);
                    return;
                }
            }

            // Try xsel
            let try_xsel = Command::new("xsel")
                .args(&["--clipboard", "--input"])
                .stdin(Stdio::piped())
                .spawn();

            if let Ok(mut child) = try_xsel {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                    drop(stdin);
                    let _ = child.wait();
                    tracing::info!("✅ URL copied to clipboard (xsel): {}", text);
                    return;
                }
            }

            tracing::error!(
                "❌ Failed to copy to clipboard: No clipboard tool found (install wl-copy, xclip, or xsel)"
            );
        }

        #[cfg(target_os = "windows")]
        {
            use std::io::Write;
            use std::process::{Command, Stdio};
            match Command::new("clip").stdin(Stdio::piped()).spawn() {
                Ok(mut child) => {
                    if let Some(mut stdin) = child.stdin.take() {
                        if stdin.write_all(text.as_bytes()).is_ok() {
                            drop(stdin);
                            match child.wait() {
                                Ok(status) if status.success() => {
                                    tracing::info!("✅ URL copied to clipboard: {}", text);
                                }
                                Ok(status) => {
                                    tracing::error!("❌ clip exited with status: {}", status);
                                }
                                Err(e) => {
                                    tracing::error!("❌ Failed to wait for clip: {}", e);
                                }
                            }
                            return;
                        }
                    }
                    let _ = child.kill();
                    tracing::error!("❌ Failed to write to clip stdin");
                }
                Err(e) => tracing::error!("❌ Failed to spawn clip: {}", e),
            }
        }
    });
}

fn open_in_browser(url: &str) {
    // Platform-specific browser opening
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("open").arg(url).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let _ = Command::new("xdg-open").arg(url).spawn();
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("cmd").args(&["/C", "start", url]).spawn();
    }
}
