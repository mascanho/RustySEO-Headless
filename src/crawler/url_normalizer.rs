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
        ".jpg", ".jpeg", ".png", ".gif", ".svg", ".webp", ".ico", ".bmp", ".pdf", ".zip", ".tar",
        ".gz", ".rar", ".7z", ".mp4", ".mp3", ".avi", ".mov", ".wmv", ".flv", ".webm", ".css",
        ".js", ".woff", ".woff2", ".ttf", ".eot", ".otf",
    ];

    for ext in &skip_extensions {
        if url_path.ends_with(ext) {
            return false;
        }
    }

    // Skip common patterns
    let skip_patterns = ["mailto:", "tel:", "javascript:", "data:"];

    for pattern in &skip_patterns {
        if url_lower.starts_with(pattern) {
            return false;
        }
    }

    true
}

/// File extensions considered genuine downloadable files (documents, archives,
/// media, executables, data, fonts). Used to populate the Files tab - an
/// allowlist rather than a blocklist, so an unrecognized/unknown extension is
/// never assumed to be a file.
const FILE_EXTENSIONS: &[&str] = &[
    // Documents
    "pdf", "doc", "docx", "odt", "rtf", "txt", "csv", "tsv", "xls", "xlsx", "ods", "ppt",
    "pptx", "odp", "epub", "pages", "key", "numbers",
    // Archives
    "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso",
    // Media
    "mp3", "mp4", "wav", "avi", "mov", "wmv", "flv", "mkv", "ogg", "ogv", "webm", "m4a", "m4v",
    "flac", "aac",
    // Executables / installers
    "exe", "dmg", "apk", "msi", "deb", "rpm", "bin", "appimage",
    // Data
    "json", "xml", "sql", "db", "sqlite", "yaml", "yml", "log",
    // Fonts
    "woff", "woff2", "ttf", "otf", "eot",
];

/// Extracts a file extension from a URL, but only if it's a recognized
/// downloadable file type. Looks solely at the last segment of the URL
/// *path* (the basename) - never the host or query string - so a bare page
/// like `https://example.com/about` or a versioned route like
/// `https://example.com/v1.2/blog` is never misidentified as a file just
/// because the domain's TLD or an unrelated path segment contains a dot.
pub fn extract_file_extension(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let basename = parsed.path().rsplit('/').next()?;
    if basename.is_empty() {
        return None;
    }

    let (_, ext) = basename.rsplit_once('.')?;
    let ext_lower = ext.to_lowercase();

    if FILE_EXTENSIONS.contains(&ext_lower.as_str()) {
        Some(ext_lower)
    } else {
        None
    }
}

/// Checks if two domains are effectively the same, ignoring 'www.' prefix
pub fn is_same_domain(domain: Option<&str>, base: Option<&str>) -> bool {
    match (domain, base) {
        (Some(d), Some(b)) => {
            let d_clean = d.trim_start_matches("www.");
            let b_clean = b.trim_start_matches("www.");
            d_clean == b_clean
        },
        _ => false,
    }
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

    #[test]
    fn test_extract_file_extension_recognizes_real_files() {
        assert_eq!(
            extract_file_extension("https://example.com/reports/annual.pdf"),
            Some("pdf".to_string())
        );
        assert_eq!(
            extract_file_extension("https://example.com/archive.TAR.GZ"),
            Some("gz".to_string())
        );
    }

    #[test]
    fn test_extract_file_extension_ignores_plain_pages() {
        // No dot anywhere in the path - must not match.
        assert_eq!(extract_file_extension("https://example.com/about-us"), None);
        // Multi-label TLD/domain must not leak into the path check.
        assert_eq!(
            extract_file_extension("https://example.co.uk/blog/post-title"),
            None
        );
        // Bare domain / root path.
        assert_eq!(extract_file_extension("https://example.com/"), None);
        assert_eq!(extract_file_extension("https://example.com"), None);
    }

    #[test]
    fn test_extract_file_extension_ignores_dots_that_are_not_file_extensions() {
        // Versioned route - "2" is not a known file extension.
        assert_eq!(extract_file_extension("https://example.com/v1.2/blog"), None);
        // Unknown/unrecognized extension.
        assert_eq!(
            extract_file_extension("https://example.com/page.xyz"),
            None
        );
    }
}
