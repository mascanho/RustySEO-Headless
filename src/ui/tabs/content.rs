use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::models::App;

/// Renders the Content tab with dummy content.
/// This tab is a placeholder for content-related features.
pub fn render(f: &mut Frame, _app: &mut App, area: Rect) {
    // Define border color for consistent styling
    let border_color = Color::Rgb(40, 45, 60);

    // Create the main block with title and borders
    let block = create_content_block(border_color);

    // Get dummy content text
    let content_text = get_dummy_content();

    // Create paragraph widget with content
    let paragraph = create_content_paragraph(content_text, block);

    // Render the widget to the frame
    f.render_widget(paragraph, area);
}

/// Creates the block for the content tab.
///
/// # Arguments
/// * `border_color` - The color for the border
///
/// # Returns
/// A Block widget configured for the content tab
fn create_content_block(border_color: Color) -> Block<'static> {
    Block::default()
        .title(Span::styled(
            " 📄 Content Tab ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
}

/// Generates dummy content for the tab.
///
/// # Returns
/// A string containing placeholder content
fn get_dummy_content() -> &'static str {
    "📄 This is the Content tab.\n\n\
     Here you can view and manage crawled content.\n\
     Features coming soon:\n\
     • Content analysis\n\
     • Text extraction\n\
     • SEO recommendations\n\
     • Content optimization tools"
}

/// Creates a paragraph widget for the content.
///
/// # Arguments
/// * `text` - The text content to display
/// * `block` - The block to wrap the paragraph in
///
/// # Returns
/// A Paragraph widget configured with the content
fn create_content_paragraph(text: &'static str, block: Block<'static>) -> Paragraph<'static> {
    Paragraph::new(text)
        .block(block)
        .style(Style::default().bg(Color::Rgb(15, 15, 25)))
        .wrap(Wrap { trim: true })
}
