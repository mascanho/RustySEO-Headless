use scraper::{Html, Selector};
use std::sync::LazyLock;

/// Result of text extraction containing match information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractionResult {
    /// Whether the search text was found in the HTML
    pub found: bool,
    /// The text that was searched for
    pub search_text: String,
    /// Number of occurrences found
    pub count: usize,
    /// Detailed matches with element information
    pub matches: Vec<ExtractionMatch>,
}

/// Detailed information about a single extraction match
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractionMatch {
    /// The HTML element tag where the match was found (e.g., "p", "div", "span")
    pub element: String,
    /// A snippet of text content around the match
    pub snippet: String,
    /// The full text content of the element
    pub full_text: String,
}

// Common text-containing elements to search
static TEXT_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("p, h1, h2, h3, h4, h5, h6, li, td, th, span, a, div, blockquote, figcaption, label, legend, dt, dd, summary, cite, strong, em, b, i, u, mark, small, del, ins, sub, sup, time, code, pre, address").unwrap()
});

/// Searches for a text string in the HTML document's visible text content
///
/// # Arguments
/// * `query` - The text string to search for (case-insensitive)
/// * `html` - The parsed HTML document to search in
///
/// # Returns
/// An `ExtractionResult` containing match information including element details
pub fn search_text(query: &str, html: &Html) -> ExtractionResult {
    if query.is_empty() {
        return ExtractionResult {
            found: false,
            search_text: query.to_string(),
            count: 0,
            matches: vec![],
        };
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();
    let mut seen_texts = std::collections::HashSet::new();

    // Search through all text-containing elements
    for element in html.select(&TEXT_SELECTOR) {
        let text_content: String = element.text().collect();
        let text_trimmed = text_content.trim();

        // Skip empty or already seen content
        if text_trimmed.is_empty() || seen_texts.contains(text_trimmed) {
            continue;
        }

        let text_lower = text_trimmed.to_lowercase();

        // Check if this element contains the search text
        if text_lower.contains(&query_lower) {
            seen_texts.insert(text_trimmed.to_string());

            let tag_name = element.value().name().to_string();

            // Create a snippet with context around the match
            let snippet = create_snippet(text_trimmed, &query_lower, 80);

            matches.push(ExtractionMatch {
                element: tag_name,
                snippet,
                full_text: text_trimmed.to_string(),
            });
        }
    }

    let count = matches.len();

    ExtractionResult {
        found: count > 0,
        search_text: query.to_string(),
        count,
        matches,
    }
}

/// Creates a snippet of text around the matched query
fn create_snippet(text: &str, query_lower: &str, max_len: usize) -> String {
    let text_lower = text.to_lowercase();

    if let Some(pos) = text_lower.find(query_lower) {
        let start = pos.saturating_sub(30);
        let end = (pos + query_lower.len() + 30).min(text.len());

        let mut snippet: String = text.chars().skip(start).take(end - start).collect();

        if start > 0 {
            snippet = format!("...{}", snippet);
        }
        if end < text.len() {
            snippet = format!("{}...", snippet);
        }

        // Truncate if still too long
        if snippet.len() > max_len {
            snippet = format!(
                "{}...",
                snippet.chars().take(max_len - 3).collect::<String>()
            );
        }

        snippet
    } else {
        // Fallback: just truncate the text
        if text.len() > max_len {
            format!("{}...", text.chars().take(max_len - 3).collect::<String>())
        } else {
            text.to_string()
        }
    }
}

/// Legacy function for backwards compatibility - returns a Vec<String> with match info
///
/// # Arguments
/// * `query` - The text string to search for
/// * `html` - The parsed HTML document to search in
///
/// # Returns
/// A vector containing the matched text if found, empty vector otherwise
pub fn text(query: &str, html: &Html) -> Vec<String> {
    let result = search_text(query, html);
    if result.found {
        vec![format!(
            "Found '{}' ({} occurrences)",
            result.search_text, result.count
        )]
    } else if query.is_empty() {
        vec!["No extractor text configured".to_string()]
    } else {
        vec![]
    }
}

/// Full extraction with detailed matches - used by the Custom Search tab
///
/// # Arguments
/// * `query` - The text string to search for
/// * `html` - The parsed HTML document to search in
///
/// # Returns
/// An ExtractionResult with detailed match information
pub fn extract_with_details(query: &str, html: &Html) -> ExtractionResult {
    search_text(query, html)
}
