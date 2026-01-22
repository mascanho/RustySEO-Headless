use scraper::Html;

/// Result of text extraction containing match information
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// Whether the search text was found in the HTML
    pub found: bool,
    /// The text that was searched for
    pub search_text: String,
    /// Number of occurrences found
    pub count: usize,
}

/// Searches for a text string in the HTML document's visible text content
/// 
/// # Arguments
/// * `query` - The text string to search for (case-insensitive)
/// * `html` - The parsed HTML document to search in
/// 
/// # Returns
/// An `ExtractionResult` containing match information
pub fn search_text(query: &str, html: &Html) -> ExtractionResult {
    if query.is_empty() {
        return ExtractionResult {
            found: false,
            search_text: query.to_string(),
            count: 0,
        };
    }
    
    // Get all visible text content from the HTML
    let all_text = html.root_element().text().collect::<String>();
    let all_text_lower = all_text.to_lowercase();
    let query_lower = query.to_lowercase();
    
    // Count occurrences (case-insensitive)
    let count = all_text_lower.matches(&query_lower).count();
    
    ExtractionResult {
        found: count > 0,
        search_text: query.to_string(),
        count,
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
        vec![format!("Found '{}' ({} occurrences)", result.search_text, result.count)]
    } else if query.is_empty() {
        vec!["No extractor text configured".to_string()]
    } else {
        vec![]
    }
}
