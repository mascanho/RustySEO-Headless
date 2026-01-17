use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};

use crate::models::App;

/// Helper function to create the table header row with consistent styling
fn create_table_header(header_titles: &[&str], accent_color: Color) -> Row<'static> {
    Row::new(header_titles.iter().map(|h| {
        Cell::from(format!(" {} ", h)).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(accent_color)
                .bg(Color::Rgb(30, 30, 45)),
        )
    }))
    .height(1)
}

/// Helper function to style content based on column type and value
fn style_cell_content(
    content: &mut String,
    cell_style: &mut Style,
    j: usize,
    c: &str,
    is_selected: bool,
) {
    // Handle text truncation for long content columns
    if j == 1 || j == 2 || j == 4 || j == 6 || j == 8 {
        // This would need access to horizontal_scroll, so keeping in main function
        // For modularity, this could be passed as parameter
    }

    // Apply column-specific styling
    match j {
        10 => style_status_column(content, cell_style, is_selected), // Status
        11 => style_mobile_column(content),                          // Mobile
        3 => style_length_column(cell_style, c, 60, is_selected),    // Title length
        5 => style_length_column(cell_style, c, 160, is_selected),   // Desc length
        13 => style_indexability_column(content, cell_style, is_selected), // Indexability
        _ => {}                                                      // No special styling
    }
}

/// Helper function to style the status column based on HTTP status codes
fn style_status_column(content: &mut String, cell_style: &mut Style, is_selected: bool) {
    match content.as_str() {
        c if c.contains("200") => {
            *content = "200".to_string();
            if !is_selected {
                *cell_style = cell_style.fg(Color::Green);
            }
        }
        c if c.contains("404") => {
            *content = "404".to_string();
            if !is_selected {
                *cell_style = cell_style.fg(Color::Red);
            }
        }
        c if c.contains("301") || c.contains("302") => {
            if !is_selected {
                *cell_style = cell_style.fg(Color::Blue);
            }
        }
        c if c.contains("500") => {
            *content = "500".to_string();
            if !is_selected {
                *cell_style = cell_style.fg(Color::Yellow);
            }
        }
        c if c.contains("403") => {
            *content = "403".to_string();
            if !is_selected {
                *cell_style = cell_style.fg(Color::Magenta);
            }
        }
        c if c.contains("503") => {
            *content = format!("🚧 {}", c);
            if !is_selected {
                *cell_style = cell_style.fg(Color::LightRed);
            }
        }
        _ => {}
    }
}

/// Helper function to style the mobile column
fn style_mobile_column(content: &mut String) {
    *content = if *content == "true" {
        "Yes".to_string()
    } else {
        "No".to_string()
    };
}

/// Helper function to style length columns with color coding
fn style_length_column(cell_style: &mut Style, c: &str, optimal_max: usize, is_selected: bool) {
    if let Ok(len) = c.parse::<usize>() {
        if len > optimal_max && !is_selected {
            *cell_style = cell_style.fg(Color::Red);
        } else if len < optimal_max && !is_selected {
            *cell_style = cell_style.fg(Color::Green);
        }
    }
}

/// Helper function to style the indexability column
fn style_indexability_column(content: &mut String, cell_style: &mut Style, is_selected: bool) {
    if content.contains("noindex") {
        *content = "Non-indexable".to_string();
        if !is_selected {
            *cell_style = cell_style.fg(Color::Red);
        }
    } else {
        *content = "Indexable".to_string();
        if !is_selected {
            *cell_style = cell_style.fg(Color::Green);
        }
    }
}

/// Helper function to pad content for alignment in fixed-width columns
fn pad_content(content: String, width: usize) -> String {
    let l = content.len();
    if l < width {
        let left_pad = (width - l) / 2;
        let right_pad = width - l - left_pad;
        format!(
            "{}{}{}",
            " ".repeat(left_pad),
            content,
            " ".repeat(right_pad)
        )
    } else {
        content
    }
}

/// Renders the CSS tab with the same table structure as the Dashboard.
/// This tab displays SEO audit data in a tabular format, focusing on CSS-related metrics.
/// The table includes columns for ID, URL, Title, Title Length, Description, Description Length,
/// H1, H1 Length, H2, H2 Length, Status, Mobile, Language, and Indexability.
/// This implementation is a direct copy of the Dashboard table to maintain consistency,
/// with modularized helper functions for better code organization.
pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    // Store the current table area for reference in other parts of the UI
    app.table_rect = Some(area);

    // Define accent and border colors for consistent theming
    let accent_color = Color::Rgb(80, 140, 255);
    let border_color = Color::Rgb(40, 45, 60);

    // Ensure we have filtered data available; if not, initialize with full data if search query is empty
    if app.filtered_table_data.is_empty()
        && !app.table_data.is_empty()
        && app.search_query.is_empty()
    {
        app.filtered_table_data = app.table_data.clone();
    }

    // Define the header titles for each column in the table
    // These correspond to SEO metrics and page information
    let header_titles = [
        "ID",        // Sequential identifier for each row
        "URL",       // The crawled page URL
        "Title",     // Page title text
        "Len",       // Title length in characters
        "Desc",      // Meta description text
        "Len",       // Description length in characters
        "H1",        // H1 heading text
        "Len",       // H1 length in characters
        "H2",        // H2 heading text
        "Len",       // H2 length in characters
        "Status",    // HTTP status code
        "Mobile",    // Mobile-friendliness indicator
        "Lang",      // Detected language
        "Indexable", // Indexability status
    ];

    // Create the table header row using the helper function
    let header = create_table_header(&header_titles, accent_color);

    // Create table rows for each item in the filtered data
    let rows = app.filtered_table_data.iter().enumerate().map(|(i, data)| {
        // Check if this row is currently selected by the user
        let is_selected = app.table_state.selected() == Some(i);

        // Determine row background color based on row index (alternating rows)
        let mut row_style = if i % 2 == 0 {
            Style::default().bg(Color::Rgb(20, 20, 30)) // Even rows: darker background
        } else {
            Style::default().bg(Color::Rgb(25, 25, 40)) // Odd rows: slightly lighter background
        };

        // If row is selected, override styling with selection colors
        if is_selected {
            row_style = row_style
                .fg(Color::White) // White text when selected
                .bg(accent_color) // Accent color background when selected
                .add_modifier(Modifier::BOLD); // Bold text when selected
        }

        // Calculate the starting index for pagination
        let start = app.current_page * app.page_size;
        let full_idx = start + i;

        // Prepare the data to be displayed in each column
        // This maps the raw data array to specific columns
        let displayed_data = vec![
            (full_idx + 1).to_string(), // ID: 1-based sequential number
            data[1].clone(),            // URL: from data index 1
            data[2].clone(),            // Title: from data index 2
            data[3].clone(),            // Title Len: from data index 3
            data[6].clone(),            // Desc: from data index 6
            data[7].clone(),            // Desc Len: from data index 7
            data[4].clone(),            // H1: from data index 4
            data[5].clone(),            // H1 Len: from data index 5
            data[8].clone(),            // H2: from data index 8
            data[9].clone(),            // H2 Len: from data index 9
            data[10].clone(),           // Status: from data index 10
            data[11].clone(),           // Mobile: from data index 11
            data[12].clone(),           // Language: from data index 12
            data[13].clone(),           // Indexability: from data index 13
        ];

        // Process each cell's content and styling
        let cells = displayed_data.iter().enumerate().map(|(j, c)| {
            // Initialize content variable for text processing
            let mut content = if j == 1 || j == 2 || j == 4 || j == 6 || j == 8 {
                // For columns that contain potentially long text (URL, Title, H1, Desc, H2)
                let content_str = c.as_str();
                let char_count = content_str.chars().count();

                // Truncate long content and add ellipsis if necessary
                if char_count > 60 {
                    // Calculate scroll position to show relevant part of text
                    let start = app.horizontal_scroll.min(char_count.saturating_sub(50));
                    let end = (start + 60).min(char_count);
                    let sliced: String =
                        content_str.chars().skip(start).take(end - start).collect();

                    // Add ellipsis prefix if we're not showing from the beginning
                    if start > 0 {
                        format!("…{}", sliced)
                    } else {
                        sliced
                    }
                } else {
                    content_str.to_string()
                }
            } else {
                // For other columns, use content as-is
                c.as_str().to_string()
            };

            // Initialize cell style (may be modified based on content)
            let mut cell_style = Style::default();

            // Apply special styling using helper functions
            style_cell_content(&mut content, &mut cell_style, j, c, is_selected);

            // Apply padding to specific columns for alignment
            let content = if j == 3 || j == 5 || j == 7 || j == 9 || j == 10 || j == 11 {
                // Columns that need fixed width: lengths and status columns
                let width = match j {
                    3 | 5 => 5,          // Length columns: 5 characters wide
                    7 | 9 => 7,          // H1/H2 length columns: 7 characters wide
                    10 | 11 => 8,        // Status and Mobile columns: 8 characters wide
                    _ => unreachable!(), // Should not reach here
                };
                pad_content(content, width)
            } else {
                content // No padding needed for other columns
            };

            // Create the cell with processed content and styling
            Cell::from(content).style(cell_style)
        });

        // Create the row with processed cells and row styling
        Row::new(cells).style(row_style).height(1)
    });

    // Calculate dynamic width for ID column based on total number of items
    let max_id_width = app.full_filtered_table_data.len().to_string().len().max(2) as u16 + 2;

    // Define column width constraints for the table layout
    let widths = [
        Constraint::Length(max_id_width), // ID column: dynamic width
        Constraint::Min(55),              // URL column: minimum 55 characters
        Constraint::Length(20),           // Title column: fixed 20 characters
        Constraint::Length(5),            // Title Len: fixed 5 characters
        Constraint::Length(20),           // Desc column: fixed 20 characters
        Constraint::Length(5),            // Desc Len: fixed 5 characters
        Constraint::Length(20),           // H1 column: fixed 20 characters
        Constraint::Length(7),            // H1 Len: fixed 7 characters
        Constraint::Length(15),           // H2 column: fixed 15 characters
        Constraint::Length(7),            // H2 Len: fixed 7 characters
        Constraint::Length(8),            // Status column: fixed 8 characters
        Constraint::Length(8),            // Mobile column: fixed 8 characters
        Constraint::Length(6),            // Lang column: fixed 6 characters
        Constraint::Min(8),               // Indexable column: minimum 8 characters
    ];

    // Calculate total number of pages for pagination
    let total_pages = (app.full_filtered_table_data.len() + app.page_size - 1) / app.page_size;

    // Create scroll indicator text if horizontal scrolling is active
    let scroll_indicator = if app.horizontal_scroll > 0 {
        format!(" [Scroll: {}] ", app.horizontal_scroll)
    } else {
        String::new()
    };

    // Build the table widget with all components
    let table = Table::new(rows, widths)
        .header(header) // Set the header row
        .block(
            Block::default()
                .borders(Borders::ALL) // Add borders around the table
                .title(Span::styled(
                    format!(
                        " 🎨 CSS Analysis Dashboard ({}) ",
                        app.full_filtered_table_data.len()
                    ),
                    Style::default()
                        .fg(Color::Yellow) // Yellow title color
                        .add_modifier(Modifier::BOLD), // Bold title
                ))
                .title_bottom(
                    Line::from(Span::styled(
                        format!(
                            " Page {} of {} {} ",
                            app.current_page + 1, // Current page (1-based)
                            total_pages,          // Total pages
                            scroll_indicator      // Scroll position indicator
                        ),
                        Style::default().fg(Color::DarkGray).italic(), // Gray, italic bottom text
                    ))
                    .alignment(Alignment::Right), // Right-align the page info
                )
                .border_style(Style::default().fg(border_color)), // Border color
        )
        .column_spacing(1) // Space between columns
        .style(Style::default().bg(Color::Rgb(15, 15, 25))); // Dark background for table

    // Render the table widget to the frame
    f.render_stateful_widget(table, area, &mut app.table_state);

    // Render floating search bar if search is active
    if app.show_search {
        // Calculate position for search bar (bottom right of table area)
        let search_area = Rect {
            x: area.x + area.width.saturating_sub(40), // 40 characters from right edge
            y: area.y + area.height.saturating_sub(3), // 3 lines from bottom edge
            width: 38,                                 // 38 characters wide
            height: 3,                                 // 3 lines high
        };

        // Create the search bar block with styling
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)) // Yellow border for search
            .bg(Color::Rgb(25, 25, 40)) // Dark background
            .title(Span::styled(
                " Fuzzy Search ",                        // Search title
                Style::default().fg(Color::Cyan).bold(), // Cyan, bold title
            ));

        // Create search input display text
        let search_text = format!("> {}", app.search_query);
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(Style::default().fg(Color::White)); // White text

        // Clear the area and render the search bar
        f.render_widget(Clear, search_area);
        f.render_widget(search_paragraph, search_area);
    }
}
