//! The "Menus" side panel (toggled with Ctrl+M).
//!
//! A second right-hand sidebar, styled like `side_panel.rs`, that mirrors every
//! menu in the RustySEO desktop app (the Tauri build's top menu bar plus its
//! global command shortcuts). It is a navigable *reference* - the CLI can't run
//! the GUI dialogs those entries open, so selecting an entry just shows what it
//! does and the shortcut it uses in the desktop app.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::models::App;

const ACCENT_COLOR: Color = Color::Rgb(80, 140, 255);
const BORDER_COLOR: Color = Color::Rgb(40, 45, 60);
const PANEL_BG: Color = Color::Rgb(15, 15, 25);

/// A single menu entry, matching one `MenubarItem` in the desktop app.
pub struct MenuEntry {
    pub label: &'static str,
    /// Desktop-app shortcut, or "" when the entry has none.
    pub shortcut: &'static str,
    pub description: &'static str,
}

/// A top-level menu (one `MenubarMenu` trigger in the desktop app).
pub struct MenuGroup {
    pub title: &'static str,
    pub entries: &'static [MenuEntry],
}

/// Every RustySEO menu, combined. Order and wording follow the desktop app's
/// `TopMenuBar` component; the trailing "Application" group collects the global
/// commands that live only as shortcuts there (Shortcuts.ts).
pub const MENU_GROUPS: &[MenuGroup] = &[
    MenuGroup {
        title: "File",
        entries: &[
            MenuEntry {
                label: "Open Settings Folder",
                shortcut: "Ctrl+Shift+F",
                description: "Open the RustySEO config folder in the system file manager.",
            },
            MenuEntry {
                label: "Settings (GUI)",
                shortcut: "Ctrl+,",
                description: "Open the settings window (crawler, connectors, extractors, appearance).",
            },
            MenuEntry {
                label: "Exit",
                shortcut: "Ctrl+Q",
                description: "Quit RustySEO.",
            },
        ],
    },
    MenuGroup {
        title: "View",
        entries: &[
            MenuEntry {
                label: "Panels",
                shortcut: "Ctrl+B",
                description: "Toggle the dashboard side panels (shallow crawler only).",
            },
            MenuEntry {
                label: "Dark / Light Mode",
                shortcut: "Ctrl+Shift+L",
                description: "Switch the colour theme between light and dark.",
            },
        ],
    },
    MenuGroup {
        title: "Tasks",
        entries: &[
            MenuEntry {
                label: "New Task",
                shortcut: "Ctrl+T",
                description: "Create a task/to-do for the current URL and strategy.",
            },
            MenuEntry {
                label: "View All Tasks",
                shortcut: "Ctrl+J",
                description: "Open the tasks drawer listing every saved task.",
            },
        ],
    },
    MenuGroup {
        title: "Reports",
        entries: &[
            MenuEntry {
                label: "Generate Crawl Report (PDF)",
                shortcut: "Ctrl+Shift+C",
                description: "Export a full PDF: on-page SEO, technical health, issues, performance and a page inventory.",
            },
            MenuEntry {
                label: "Server Log Report (PDF)",
                shortcut: "Ctrl+Shift+S",
                description: "Export a full server-log PDF: traffic, crawlers, status codes, file types, bandwidth and bot categories.",
            },
        ],
    },
    MenuGroup {
        title: "Tools",
        entries: &[
            MenuEntry {
                label: "Image Converter",
                shortcut: "Ctrl+I",
                description: "Convert and optimise images (WebP/AVIF and friends).",
            },
            MenuEntry {
                label: "Google Ads Simulator",
                shortcut: "Ctrl+G",
                description: "Preview how PPC ads and SERP snippets would render.",
            },
            MenuEntry {
                label: "HTTP Checker",
                shortcut: "Ctrl+U",
                description: "Check status codes and the redirect chain for a single URL.",
            },
            MenuEntry {
                label: "Page Screenshot",
                shortcut: "Ctrl+Shift+P",
                description: "Capture a screenshot of a page.",
            },
            MenuEntry {
                label: "Crawl Diff",
                shortcut: "Ctrl+Shift+D",
                description: "Compare two crawls of the same site (deep crawler only).",
            },
            MenuEntry {
                label: "Log Analyser",
                shortcut: "Ctrl+K",
                description: "Open the server log analyser (Nginx & Apache).",
            },
        ],
    },
    MenuGroup {
        title: "Connectors",
        entries: &[
            MenuEntry {
                label: "Microsoft Clarity",
                shortcut: "",
                description: "Connect a Microsoft Clarity project for behavioural analytics.",
            },
            MenuEntry {
                label: "MS Power BI",
                shortcut: "",
                description: "Connect Microsoft Power BI.",
            },
            MenuEntry {
                label: "PageSpeed Insights",
                shortcut: "",
                description: "Add a Google PageSpeed Insights API key for Core Web Vitals data.",
            },
            MenuEntry {
                label: "Google Analytics",
                shortcut: "",
                description: "Connect a Google Analytics 4 property.",
            },
            MenuEntry {
                label: "Search Console",
                shortcut: "",
                description: "Connect a Google Search Console account and pick a property.",
            },
            MenuEntry {
                label: "Ollama (AI Models)",
                shortcut: "",
                description: "Select a local Ollama model for AI features.",
            },
            MenuEntry {
                label: "Google Gemini",
                shortcut: "",
                description: "Configure the Google Gemini model for AI features.",
            },
        ],
    },
    MenuGroup {
        title: "Crawlers",
        entries: &[
            MenuEntry {
                label: "Shallow Crawler",
                shortcut: "Ctrl+S",
                description: "Single-page / shallow crawl and analysis.",
            },
            MenuEntry {
                label: "Deep Crawler",
                shortcut: "Ctrl+D",
                description: "Full-site deep crawl with no page limit.",
            },
        ],
    },
    MenuGroup {
        title: "Extractors",
        entries: &[MenuEntry {
            label: "Custom Search",
            shortcut: "Ctrl+E",
            description: "Manage custom extraction rules (CSS/XPath/regex). Deep crawler only.",
        }],
    },
    MenuGroup {
        title: "Visualisations",
        entries: &[MenuEntry {
            label: "Crawl Visualisations",
            shortcut: "Ctrl+Shift+V",
            description: "Open the crawl visualisations hub (charts, graphs). Deep crawler only.",
        }],
    },
    MenuGroup {
        title: "Help",
        entries: &[
            MenuEntry {
                label: "Onboarding",
                shortcut: "",
                description: "Replay the first-run onboarding walkthrough.",
            },
            MenuEntry {
                label: "Keyboard Shortcuts",
                shortcut: "",
                description: "Show the keyboard shortcuts reference.",
            },
            MenuEntry {
                label: "Changelog",
                shortcut: "",
                description: "See what changed in this version.",
            },
            MenuEntry {
                label: "About",
                shortcut: "",
                description: "Version, license and credits.",
            },
            MenuEntry {
                label: "Send a Suggestion",
                shortcut: "",
                description: "Send a feature suggestion to the RustySEO team.",
            },
            MenuEntry {
                label: "Report a Bug",
                shortcut: "",
                description: "Open a new issue on the RustySEO GitHub tracker.",
            },
        ],
    },
    MenuGroup {
        title: "Application",
        entries: &[
            MenuEntry {
                label: "Reload",
                shortcut: "Ctrl+R",
                description: "Reload the current view.",
            },
            MenuEntry {
                label: "Clear Cache",
                shortcut: "Ctrl+/",
                description: "Wipe cached crawl/session data without touching the config folder.",
            },
            MenuEntry {
                label: "Full Reset",
                shortcut: "Ctrl+Shift+/",
                description: "Drop the config folders as well, then reload. Destructive.",
            },
        ],
    },
];

/// Total number of selectable entries across every group.
pub fn total_entries() -> usize {
    MENU_GROUPS.iter().map(|g| g.entries.len()).sum()
}

/// Resolve a flat selectable index to its `(group, entry)`.
pub fn entry_at(index: usize) -> Option<(&'static MenuGroup, &'static MenuEntry)> {
    let mut remaining = index;
    for group in MENU_GROUPS {
        if remaining < group.entries.len() {
            return Some((group, &group.entries[remaining]));
        }
        remaining -= group.entries.len();
    }
    None
}

pub fn render(f: &mut Frame, app: &mut App) {
    if !app.menu_panel_visible {
        return;
    }

    let area = f.area();
    let width = (area.width * 38 / 100).max(40).min(area.width);
    let panel_area = Rect {
        x: area.width.saturating_sub(width),
        y: 0,
        width,
        height: area.height,
    };

    f.render_widget(Clear, panel_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(0),    // menu list
            Constraint::Length(7), // selected-entry detail
        ])
        .split(panel_area);

    // Header ---------------------------------------------------------------
    let header = Paragraph::new(Line::from(vec![Span::styled(
        "RustySEO menus  ·  j/k move  ·  Esc closes",
        Style::default().fg(Color::DarkGray),
    )]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                " MENUS ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ))
            .border_style(Style::default().fg(BORDER_COLOR))
            .bg(PANEL_BG),
    );
    f.render_widget(header, chunks[0]);

    // Menu list -----------------------------------------------------------
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR))
        .bg(PANEL_BG);
    let list_inner = list_block.inner(chunks[1]);
    f.render_widget(list_block, chunks[1]);

    let selected = app.menu_panel_selected.min(total_entries().saturating_sub(1));
    let inner_width = list_inner.width as usize;

    let mut lines: Vec<Line> = Vec::new();
    let mut selected_line: usize = 0;
    let mut ordinal: usize = 0;

    for (gi, group) in MENU_GROUPS.iter().enumerate() {
        if gi > 0 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![Span::styled(
            format!("▸ {}", group.title.to_uppercase()),
            Style::default()
                .fg(ACCENT_COLOR)
                .add_modifier(Modifier::BOLD),
        )]));

        for entry in group.entries {
            let is_selected = ordinal == selected;
            if is_selected {
                selected_line = lines.len();
            }

            let marker = if is_selected { "❯ " } else { "  " };
            let shortcut_w = entry.shortcut.chars().count();
            let label_w = marker.chars().count() + entry.label.chars().count();
            let gap = inner_width
                .saturating_sub(label_w + shortcut_w)
                .max(1);

            let label_style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(35, 45, 70))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let mut spans = vec![
                Span::styled(marker, label_style),
                Span::styled(entry.label, label_style),
            ];
            if shortcut_w > 0 {
                spans.push(Span::styled(" ".repeat(gap), label_style));
                spans.push(Span::styled(
                    entry.shortcut,
                    if is_selected {
                        label_style
                    } else {
                        Style::default().fg(Color::Rgb(255, 170, 0))
                    },
                ));
            }
            lines.push(Line::from(spans));
            ordinal += 1;
        }
    }

    let body_height = list_inner.height as usize;
    let mut scroll = app.menu_panel_scroll;
    if selected_line < scroll {
        scroll = selected_line;
    } else if body_height > 0 && selected_line >= scroll + body_height {
        scroll = selected_line + 1 - body_height;
    }
    app.menu_panel_scroll = scroll;

    let list = Paragraph::new(lines)
        .style(Style::default().bg(PANEL_BG))
        .scroll((scroll as u16, 0));
    f.render_widget(list, list_inner);

    // Detail --------------------------------------------------------------
    let (group_title, entry) = entry_at(selected)
        .map(|(g, e)| (g.title, e))
        .unwrap_or(("", &MENU_GROUPS[0].entries[0]));

    let mut detail_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{} › ", group_title),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                entry.label,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            entry.description,
            Style::default().fg(Color::Gray),
        )]),
    ];
    if !entry.shortcut.is_empty() {
        detail_lines.push(Line::from(""));
        detail_lines.push(Line::from(vec![
            Span::styled("Desktop shortcut: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                entry.shortcut,
                Style::default()
                    .fg(Color::Rgb(255, 170, 0))
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    let detail = Paragraph::new(detail_lines)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(PANEL_BG))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    " DETAIL ",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(BORDER_COLOR))
                .bg(PANEL_BG),
        );
    f.render_widget(detail, chunks[2]);
}
