use url::Url;

/// Image information extracted from HTML elements
#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub src: String,
    pub alt: String,
    pub size_bytes: Option<u64>,
    pub size_formatted: String,
}

impl ImageInfo {
    /// Creates a new ImageInfo instance from image element attributes
    pub fn new(src: String, alt: String) -> Self {
        let (size_bytes, size_formatted) = get_image_size_fast(&src);

        Self {
            src,
            alt,
            size_bytes,
            size_formatted,
        }
    }
}

/// Fast, non-blocking image size detection
/// Returns (size_bytes, formatted_size) where formatted_size is human-readable
///
/// This function attempts to estimate image size without making HTTP requests
/// by analyzing URL patterns and filename conventions.
pub fn get_image_size_fast(src: &str) -> (Option<u64>, String) {
    // Skip data URLs and invalid URLs
    if src.starts_with("data:") || src.is_empty() {
        return (None, "Data URI".to_string());
    }

    // Try to extract size from URL parameters first (common CDN pattern)
    if let Some(size_hint) = extract_size_from_url_params(src) {
        return size_hint;
    }

    // Extract size hints from filename patterns
    if let Some(size_hint) = extract_size_from_filename(src) {
        return size_hint;
    }

    // For other images, return Unknown
    (None, "Unknown".to_string())
}

/// Extract size information from common URL patterns like ?width=800&height=600
///
/// Many CDNs and image processing services include dimensions in URL parameters.
/// This function parses those parameters to estimate image size.
fn extract_size_from_url_params(src: &str) -> Option<(Option<u64>, String)> {
    let url = Url::parse(src).ok()?;
    let mut width = None;
    let mut height = None;

    // Parse query parameters looking for dimension information
    for (key, value) in url.query_pairs() {
        match key.to_lowercase().as_str() {
            "w" | "width" => {
                if let Ok(w) = value.parse::<u32>() {
                    width = Some(w);
                }
            }
            "h" | "height" => {
                if let Ok(h) = value.parse::<u32>() {
                    height = Some(h);
                }
            }
            _ => {}
        }
    }

    // If both width and height are found, estimate file size
    if let (Some(w), Some(h)) = (width, height) {
        let estimated_bytes = estimate_image_size(w, h, src);
        return Some((
            Some(estimated_bytes),
            format!("~{}KB", estimated_bytes / 1024),
        ));
    }

    None
}

/// Extract size information from filename patterns like "image-800x600.jpg"
///
/// Many images include dimensions in their filenames. This function uses
/// regex patterns to extract these dimensions and estimate file size.
fn extract_size_from_filename(src: &str) -> Option<(Option<u64>, String)> {
    // Look for patterns like "800x600", "800_600", "-800x600-" in filename
    let re = regex::Regex::new(r"(\d+)[xX_](\d+)").ok()?;

    if let Some(captures) = re.captures(src) {
        if let (Some(width), Some(height)) = (
            captures.get(1).and_then(|m| m.as_str().parse::<u32>().ok()),
            captures.get(2).and_then(|m| m.as_str().parse::<u32>().ok()),
        ) {
            // Validate reasonable image dimensions (between 10px and 10000px)
            if width > 10 && width < 10000 && height > 10 && height < 10000 {
                let estimated_bytes = estimate_image_size(width, height, src);
                return Some((
                    Some(estimated_bytes),
                    format!("~{}KB", estimated_bytes / 1024),
                ));
            }
        }
    }

    None
}

/// Estimate image size based on dimensions and file type
///
/// Uses different compression ratios for different image formats:
/// - PNG: Less compression, larger file sizes
/// - JPEG: Good compression, smaller file sizes  
/// - WebP: Moderate compression, modern format
/// - GIF: Variable compression, depends on content
fn estimate_image_size(width: u32, height: u32, src: &str) -> u64 {
    let pixels = width as u64 * height as u64;

    // Different compression ratios for different formats
    let bytes_per_pixel = if src.to_lowercase().contains(".png") {
        4.0 // PNG: less compression, lossless
    } else if src.to_lowercase().contains(".jpg") || src.to_lowercase().contains(".jpeg") {
        0.5 // JPEG: good compression, lossy
    } else if src.to_lowercase().contains(".webp") {
        0.6 // WebP: moderate compression, modern format
    } else if src.to_lowercase().contains(".gif") {
        1.0 // GIF: varies, moderate compression
    } else {
        1.0 // Default estimate for unknown formats
    };

    // Calculate estimated size and ensure minimum of 1KB
    ((pixels as f64 * bytes_per_pixel) / 1024.0).max(1.0) as u64 * 1024
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_image_size_fast_data_uri() {
        let (size, formatted) = get_image_size_fast(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==",
        );
        assert_eq!(size, None);
        assert_eq!(formatted, "Data URI");
    }

    #[test]
    fn test_get_image_size_fast_empty() {
        let (size, formatted) = get_image_size_fast("");
        assert_eq!(size, None);
        assert_eq!(formatted, "Data URI");
    }

    #[test]
    fn test_extract_size_from_url_params() {
        let result =
            extract_size_from_url_params("https://example.com/image.jpg?width=800&height=600");
        assert!(result.is_some());
        let (size, formatted) = result.unwrap();
        assert!(size.is_some());
        assert!(formatted.contains("KB"));
    }

    #[test]
    fn test_extract_size_from_filename() {
        let result = extract_size_from_filename("https://example.com/image-800x600.jpg");
        assert!(result.is_some());
        let (size, formatted) = result.unwrap();
        assert!(size.is_some());
        assert!(formatted.contains("KB"));
    }

    #[test]
    fn test_estimate_image_size() {
        let size = estimate_image_size(800, 600, "image.jpg");
        assert!(size > 0);

        let png_size = estimate_image_size(800, 600, "image.png");
        let jpg_size = estimate_image_size(800, 600, "image.jpg");
        assert!(png_size > jpg_size); // PNG should be larger than JPEG
    }
}
