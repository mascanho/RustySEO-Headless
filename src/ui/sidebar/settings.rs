use crate::models::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

pub fn render(
    f: &mut Frame,
    app: &mut App,
    area: Rect,
    content_block: Block,
    accent_color: Color,
    border_color: Color,
) {
    if let Some(settings) = &app.settings {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let main_area = chunks[0];
        let footer_area = chunks[1];

        // Draw the main container block
        f.render_widget(
            content_block.title(Span::styled(
                " CONFIGURATION ENGINE ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            main_area,
        );

        // Split the main area for sections to create a "nested" look
        let inner_area = Rect {
            x: main_area.x + 1,
            y: main_area.y + 1,
            width: main_area.width.saturating_sub(2),
            height: main_area.height.saturating_sub(2),
        };

        // Create a scrollable-like area or just a long layout if it fits
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Engine
                Constraint::Length(5), // Viewport
                Constraint::Length(5), // System
                Constraint::Min(0),    // Internal
            ])
            .split(inner_area);

        // 1. ENGINE SECTION
        let engine_rows = vec![
            Row::new(vec![
                Cell::from("  Max Pages"),
                Cell::from(settings.crawler.max_pages.to_string()),
            ]),
            Row::new(vec![
                Cell::from("  Concurrency"),
                Cell::from(settings.crawler.concurrency.to_string()),
            ]),
            Row::new(vec![
                Cell::from("  Domain Lock"),
                Cell::from(if settings.crawler.stay_on_domain {
                    " PROTECTED "
                } else {
                    " OPEN "
                })
                .style(Style::default().fg(Color::Black).bg(
                    if settings.crawler.stay_on_domain {
                        Color::Green
                    } else {
                        Color::Red
                    },
                )),
            ]),
            Row::new(vec![
                Cell::from("  JavaScript"),
                Cell::from(if settings.crawler.enable_javascript {
                    " ENABLED "
                } else {
                    " DISABLED "
                })
                .style(Style::default().fg(Color::Black).bg(
                    if settings.crawler.enable_javascript {
                        Color::Green
                    } else {
                        Color::Gray
                    },
                )),
            ]),
        ];
        let engine_table = Table::new(
            engine_rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    " 󱐋 ENGINE ",
                    Style::default().fg(Color::Yellow),
                )),
        )
        .style(Style::default().fg(accent_color));
        f.render_widget(engine_table, sections[0]);

        // 2. VIEWPORT SECTION
        let ui_rows = vec![
            Row::new(vec![
                Cell::from("  Theme"),
                Cell::from(format!(" {} ", settings.ui.theme.clone()))
                    .style(Style::default().fg(Color::Black).bg(Color::Cyan)),
            ]),
            Row::new(vec![
                Cell::from("  Refresh"),
                Cell::from(format!(" {}ms ", settings.ui.refresh_rate_ms)),
            ]),
        ];
        let ui_table = Table::new(
            ui_rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    " 🎨 VIEWPORT ",
                    Style::default().fg(Color::Magenta),
                )),
        )
        .style(Style::default().fg(accent_color));
        f.render_widget(ui_table, sections[1]);

        // 3. SYSTEM SECTION
        let sys_rows = vec![
            Row::new(vec![
                Cell::from("  Format"),
                Cell::from(settings.system.export_format.to_uppercase())
                    .style(Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Cell::from("  Logs"),
                Cell::from(settings.system.log_level.to_uppercase())
                    .style(Style::default().fg(Color::Yellow)),
            ]),
        ];
        let sys_table = Table::new(
            sys_rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(" ⚙ SYSTEM ", Style::default().fg(Color::Cyan))),
        )
        .style(Style::default().fg(accent_color));
        f.render_widget(sys_table, sections[2]);

        // 4. CONNECTORS SECTION
        let connectors_rows = vec![
            Row::new(vec![
                Cell::from(" PAGE SPEED ").style(Style::default().fg(Color::Yellow)),
                Cell::from(""),
            ]),
            Row::new(vec![
                Cell::from("  ├─ API Key"),
                Cell::from(if settings.connectors.pagespeed.api_key.is_empty() {
                    "MISSING"
                } else {
                    "********"
                })
                .style(Style::default().fg(
                    if settings.connectors.pagespeed.api_key.is_empty() {
                        Color::Red
                    } else {
                        Color::Green
                    },
                )),
                Cell::from(""),
            ]),
            Row::new(vec![Cell::from(if settings.connectors.pagespeed.status {
                Color::Green
            } else {
                Color::Red
            })]),
            Row::new(vec![
                Cell::from(" SEARCH CONSOLE ").style(Style::default().fg(Color::Yellow)),
                Cell::from(""),
            ]),
            Row::new(vec![
                Cell::from("  ├─ Project"),
                Cell::from(settings.connectors.search_console.project_name.clone())
                    .style(Style::default().fg(Color::Blue)),
            ]),
            Row::new(vec![
                Cell::from("  └─ Status"),
                Cell::from(if settings.connectors.search_console.token.is_empty() {
                    "UNLINKED"
                } else {
                    "CONNECTED"
                })
                .style(Style::default().fg(
                    if settings.connectors.search_console.token.is_empty() {
                        Color::Red
                    } else {
                        Color::Green
                    },
                )),
            ]),
            Row::new(vec![
                Cell::from(" GEMINI AI ").style(Style::default().fg(Color::Yellow)),
                Cell::from(""),
            ]),
            Row::new(vec![
                Cell::from("  ├─ Model"),
                Cell::from(settings.connectors.gemini.model.clone())
                    .style(Style::default().fg(Color::Cyan)),
            ]),
            Row::new(vec![
                Cell::from("  └─ API Key"),
                Cell::from(if settings.connectors.gemini.api_key.is_empty() {
                    "MISSING"
                } else {
                    "********"
                })
                .style(Style::default().fg(
                    if settings.connectors.gemini.api_key.is_empty() {
                        Color::Red
                    } else {
                        Color::Green
                    },
                )),
            ]),
            Row::new(vec![
                Cell::from(" OPENAI ").style(Style::default().fg(Color::Yellow)),
                Cell::from(""),
            ]),
            Row::new(vec![
                Cell::from("  ├─ Model"),
                Cell::from(settings.connectors.openai.model.clone())
                    .style(Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Cell::from("  └─ API Key"),
                Cell::from(if settings.connectors.openai.api_key.is_empty() {
                    "MISSING"
                } else {
                    "********"
                })
                .style(Style::default().fg(
                    if settings.connectors.openai.api_key.is_empty() {
                        Color::Red
                    } else {
                        Color::Green
                    },
                )),
            ]),
            Row::new(vec![
                Cell::from(" SELECTED PROVIDER ").style(Style::default().fg(Color::Yellow)),
                Cell::from(""),
            ]),
            Row::new(vec![
                Cell::from("  ├─ Model"),
                Cell::from(settings.provider.llm.clone()).style(Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Cell::from("  └─ API Key"),
                Cell::from(if settings.connectors.openai.api_key.is_empty() {
                    "MISSING"
                } else {
                    "********"
                })
                .style(Style::default().fg(
                    if settings.connectors.openai.api_key.is_empty() {
                        Color::Red
                    } else {
                        Color::Green
                    },
                )),
            ]),
        ];

        let connectors_table = Table::new(
            connectors_rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    " 🔌 CONNECTORS ",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().fg(accent_color));
        f.render_widget(connectors_table, sections[3]);

        // Footer with Shortcut
        let footer_text = vec![Line::from(vec![
            Span::styled(" ⌨ Shortcut: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                " 'E' ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to open settings.toml"),
        ])];
        let footer = Paragraph::new(footer_text)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().bg(Color::Rgb(15, 15, 25)));
        f.render_widget(footer, footer_area);
    }
}
