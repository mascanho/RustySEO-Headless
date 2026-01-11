use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::models::App;
use crate::ui::centered_rect;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let menu_area = centered_rect(25, 35, area);

    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = accent_color; // Blue border for actions menu

    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .bg(Color::Rgb(15, 15, 25))
        .title(Span::styled(
            " 📋 Actions Menu ",
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
        ListItem::new("📋 Copy URL"),
        ListItem::new("🌐 Open URL in Browser"),
        ListItem::new("🔍 Check Keywords"),
        ListItem::new("📊 View SEO Score"),
        ListItem::new("🔗 Extract Links"),
        ListItem::new("📸 Screenshot"),
        ListItem::new("📝 Export Data"),
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

    if selected_idx >= app.table_data.len() || selected_idx >= app.page_data.len() {
        return;
    }

    let row_data = &app.table_data[selected_idx];
    let url = row_data[1].clone(); // URL is at index 1

    match action_index {
        0 => {
            // Copy URL
            match copy_to_clipboard(&url) {
                Ok(_) => {
                    app.logs_data
                        .insert(0, format!("✅ URL copied to clipboard: {}", url));
                }
                Err(e) => {
                    app.logs_data
                        .insert(0, format!("❌ Failed to copy URL to clipboard: {}", e));
                }
            }
        }
        1 => {
            // Open URL in Browser
            open_in_browser(&url);
            app.logs_data
                .insert(0, format!("Opening URL in browser: {}", url));
        }
        2 => {
            // Check Keywords - could open a keywords analysis modal
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

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    // Platform-specific clipboard copying with proper error handling
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        match Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    if stdin.write_all(text.as_bytes()).is_ok() {
                        return child
                            .wait()
                            .map_err(|e| format!("Failed to wait for pbcopy: {}", e))
                            .and_then(|status| {
                                if status.success() {
                                    Ok(())
                                } else {
                                    Err(format!("pbcopy exited with status: {}", status))
                                }
                            });
                    }
                }
                let _ = child.kill();
                Err("Failed to write to pbcopy stdin".to_string())
            }
            Err(e) => Err(format!("Failed to spawn pbcopy: {}", e)),
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        match Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    if stdin.write_all(text.as_bytes()).is_ok() {
                        return child
                            .wait()
                            .map_err(|e| format!("Failed to wait for xclip: {}", e))
                            .and_then(|status| {
                                if status.success() {
                                    Ok(())
                                } else {
                                    Err(format!("xclip exited with status: {}", status))
                                }
                            });
                    }
                }
                let _ = child.kill();
                Err("Failed to write to xclip stdin".to_string())
            }
            Err(e) => Err(format!("Failed to spawn xclip: {}", e)),
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        match Command::new("cmd")
            .args(&["/C", "echo", &text.replace("\"", "\"\"")]) // Escape quotes for Windows
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|echo| echo.wait())
            .and_then(|_| {
                Command::new("clip")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
            }) {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    if stdin.write_all(text.as_bytes()).is_ok() {
                        return child
                            .wait()
                            .map_err(|e| format!("Failed to wait for clip: {}", e))
                            .and_then(|status| {
                                if status.success() {
                                    Ok(())
                                } else {
                                    Err(format!("clip exited with status: {}", status))
                                }
                            });
                    }
                }
                let _ = child.kill();
                Err("Failed to write to clip stdin".to_string())
            }
            Err(e) => Err(format!("Failed to spawn clipboard commands: {}", e)),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Clipboard copying not supported on this platform".to_string())
    }
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
        match Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    if stdin.write_all(text.as_bytes()).is_ok() {
                        return child
                            .wait()
                            .map_err(|e| format!("Failed to wait for clip: {}", e))
                            .and_then(|status| {
                                if status.success() {
                                    Ok(())
                                } else {
                                    Err(format!("clip exited with status: {}", status))
                                }
                            });
                    }
                }
                let _ = child.kill();
                Err("Failed to write to clip stdin".to_string())
            }
            Err(e) => Err(format!("Failed to spawn clip: {}", e)),
        }
    }
}
