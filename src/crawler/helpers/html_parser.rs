use scraper::{Html, Selector};
use url::Url;

use crate::crawler::helpers::word_count::{self, get_words};

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub src: String,
    pub alt: String,
    pub size_bytes: Option<u64>,
    pub size_formatted: String,
}

#[derive(Debug, Clone)]
pub struct PageData {
    pub id: usize,
    pub url: String,
    pub title: String,
    pub title_len: usize,
    pub h1: String,
    pub h1_len: usize,
    pub h2: String,
    pub h2_len: usize,
    pub description: String,
    pub description_len: usize,
    pub status: String,
    pub mobile: bool,
    pub language: String,
    pub indexability: String,
    pub anchor_links: Vec<(String, String)>,
    pub outlinks: Vec<(String, String)>,
    pub images: Vec<ImageInfo>,
    pub headings: Vec<(String, String)>,
    pub headers: Vec<String>,
    pub schema: Vec<String>,
    pub content_type: String,
    pub canonicals: Vec<(String, String, Option<String>)>, // rel, href, hreflang
    pub size: usize,
    pub word_count: Option<usize>,
}

pub fn extract_page_elements(document: &Html) -> PageData {
    let title_selector = Selector::parse("title").unwrap();
    let h1_selector = Selector::parse("h1").unwrap();
    let h2_selector = Selector::parse("h2").unwrap();
    let desc_selector = Selector::parse("meta[name='description']").unwrap();

    let title = document
        .select(&title_selector)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or("".into());

    let h1 = document
        .select(&h1_selector)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or("".into());

    let description = document
        .select(&desc_selector)
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or("".into());

    let h2 = document
        .select(&h2_selector)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or("".into());

    let mobile = document
        .select(&Selector::parse("meta[name='viewport']").unwrap())
        .next()
        .is_some();

    let language = document
        .select(&Selector::parse("html[lang]").unwrap())
        .next()
        .and_then(|e| e.value().attr("lang"))
        .map(|s| s.to_string())
        .unwrap_or("".into());

    let indexability = document
        .select(&Selector::parse("meta[name='robots']").unwrap())
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or("".into());

    let anchor_links: Vec<(String, String)> = document
        .select(&Selector::parse("a[href]").unwrap())
        .map(|e| {
            let href = e.value().attr("href").unwrap().to_string();
            let text = e.text().collect::<String>();
            (href, text)
        })
        .collect();

    let outlinks: Vec<(String, String)> = document
        .select(&Selector::parse("a[href]").unwrap())
        .map(|e| {
            let href = e.value().attr("href").unwrap().to_string();
            let text = e.text().collect::<String>();
            (href, text)
        })
        .collect();

    let images: Vec<ImageInfo> = document
        .select(&Selector::parse("img[src]").unwrap())
        .map(|e| {
            let src = e.value().attr("src").unwrap().to_string();
            let alt = e.value().attr("alt").unwrap_or("").to_string();

            // Try to get image size non-blocking
            let (size_bytes, size_formatted) = get_image_size_fast(&src);

            ImageInfo {
                src,
                alt,
                size_bytes,
                size_formatted,
            }
        })
        .collect();

    let heading_selector = Selector::parse("h1,h2,h3,h4,h5,h6").unwrap();
    let mut headings = Vec::new();
    for element in document.select(&heading_selector) {
        let tag = element.value().name().to_string();
        let text = element.text().collect::<String>();
        headings.push((tag, text));
    }

    let schema_selector = Selector::parse("script[type='application/ld+json']").unwrap();
    let schema: Vec<String> = document
        .select(&schema_selector)
        .map(|e| e.text().collect::<String>())
        .collect();

    let mut content_type = "text/html".to_string();
    if let Some(ct) = document
        .select(&Selector::parse("meta[http-equiv=content-type]").unwrap())
        .next()
        .and_then(|e| e.value().attr("content"))
    {
        content_type = ct.to_string();
    }

    let canonical_selector = Selector::parse("link[rel=canonical], link[rel=alternate]").unwrap();
    let canonicals: Vec<(String, String, Option<String>)> = document
        .select(&canonical_selector)
        .map(|e| {
            let rel = e.value().attr("rel").unwrap().to_string();
            let href = e.value().attr("href").unwrap().to_string();
            let hreflang = e.value().attr("hreflang").map(|s| s.to_string());
            (rel, href, hreflang)
        })
        .collect();

    let size = document
        .select(&Selector::parse("body").unwrap())
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or("".into())
        .len();

    let word_count = Some(get_words(document));

    PageData {
        id: 0,
        url: "".to_string(),
        title: title.clone(),
        title_len: title.len(),
        h1: h1.clone(),
        h1_len: h1.len(),
        h2: h2.clone(),
        h2_len: h2.len(),
        description: description.clone(),
        description_len: description.len(),
        status: "".to_string(),
        mobile,
        language,
        indexability,
        anchor_links,
        outlinks,
        images,
        headings,
        headers: vec![],
        schema,
        content_type,
        canonicals,
        size,
        word_count,
    }
}

/// Fast, non-blocking image size detection
/// Returns (size_bytes, formatted_size) where formatted_size is human-readable
fn get_image_size_fast(src: &str) -> (Option<u64>, String) {
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
fn extract_size_from_url_params(src: &str) -> Option<(Option<u64>, String)> {
    if let Ok(url) = Url::parse(src) {
        let mut width = None;
        let mut height = None;

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

        if let (Some(w), Some(h)) = (width, height) {
            // Rough estimation based on image type and dimensions
            let estimated_bytes = estimate_image_size(w, h, src);
            return Some((
                Some(estimated_bytes),
                format!("~{}KB", estimated_bytes / 1024),
            ));
        }
    }

    None
}

/// Extract size information from filename patterns like "image-800x600.jpg"
fn extract_size_from_filename(src: &str) -> Option<(Option<u64>, String)> {
    // Look for patterns like "800x600", "800_600", "-800x600-" in filename
    let re = regex::Regex::new(r"(\d+)[xX_](\d+)").ok()?;

    if let Some(captures) = re.captures(src) {
        if let (Some(width), Some(height)) = (
            captures.get(1).and_then(|m| m.as_str().parse::<u32>().ok()),
            captures.get(2).and_then(|m| m.as_str().parse::<u32>().ok()),
        ) {
            // Validate reasonable image dimensions
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
fn estimate_image_size(width: u32, height: u32, src: &str) -> u64 {
    let pixels = width as u64 * height as u64;

    // Different compression ratios for different formats
    let bytes_per_pixel = if src.to_lowercase().contains(".png") {
        4.0 // PNG: less compression
    } else if src.to_lowercase().contains(".jpg") || src.to_lowercase().contains(".jpeg") {
        0.5 // JPEG: good compression
    } else if src.to_lowercase().contains(".webp") {
        0.6 // WebP: moderate compression
    } else if src.to_lowercase().contains(".gif") {
        1.0 // GIF: varies, moderate compression
    } else {
        1.0 // Default estimate
    };

    ((pixels as f64 * bytes_per_pixel) / 1024.0).max(1.0) as u64 * 1024 // Ensure at least 1KB
}
