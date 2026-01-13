use url::Url;

/// Normalizes a URL to prevent duplicate crawls
/// - Removes fragments (#section)
/// - Removes trailing slashes from paths
/// - Sorts query parameters
/// - Converts to lowercase (for domain only)
pub fn normalize_url(url: &str) -> Option<String> {
    let mut parsed = Url::parse(url).ok()?;
    
    // Remove fragment
    parsed.set_fragment(None);
    
    // Normalize path - remove trailing slash unless it's the root
    let path = parsed.path().to_string();
    if path.len() > 1 && path.ends_with('/') {
        let trimmed = path.trim_end_matches('/');
        parsed.set_path(trimmed);
    }
    
    // Sort query parameters for consistency
    if let Some(query) = parsed.query() {
        let mut params: Vec<_> = query.split('&').collect();
        params.sort_unstable();
        parsed.set_query(Some(&params.join("&")));
    }
    
    Some(parsed.to_string())
}

/// Checks if a URL should be crawled based on file extension and patterns
pub fn should_crawl_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    
    // Skip common non-HTML resources
    // Note: We check the path component, not the full URL, to avoid filtering
    // URLs with query parameters like ?format=json
    let url_path = if let Ok(parsed) = Url::parse(url) {
        parsed.path().to_lowercase()
    } else {
        url_lower.clone()
    };
    
    let skip_extensions = [
        ".jpg", ".jpeg", ".png", ".gif", ".svg", ".webp", ".ico", ".bmp",
        ".pdf", ".zip", ".tar", ".gz", ".rar", ".7z",
        ".mp4", ".mp3", ".avi", ".mov", ".wmv", ".flv", ".webm",
        ".css", ".js", ".woff", ".woff2", ".ttf", ".eot", ".otf",
    ];
    
    for ext in &skip_extensions {
        if url_path.ends_with(ext) {
            return false;
        }
    }
    
    // Skip common patterns
    let skip_patterns = [
        "mailto:",
        "tel:",
        "javascript:",
        "data:",
    ];
    
    for pattern in &skip_patterns {
        if url_lower.starts_with(pattern) {
            return false;
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        assert_eq!(
            normalize_url("https://example.com/page/"),
            Some("https://example.com/page".to_string())
        );
        
        assert_eq!(
            normalize_url("https://example.com/page#section"),
            Some("https://example.com/page".to_string())
        );
        
        assert_eq!(
            normalize_url("https://example.com/"),
            Some("https://example.com/".to_string())
        );
    }

    #[test]
    fn test_should_crawl_url() {
        assert!(should_crawl_url("https://example.com/page"));
        assert!(!should_crawl_url("https://example.com/image.jpg"));
        assert!(!should_crawl_url("mailto:test@example.com"));
        assert!(!should_crawl_url("https://example.com/style.css"));
    }
}
