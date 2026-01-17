use scraper::{Html, Selector};

use crate::crawler::helpers::image_utils::ImageInfo;
use crate::crawler::helpers::word_count::get_words;

/// Comprehensive data structure representing a parsed web page
///
/// This struct contains all relevant SEO and content analysis information
/// extracted from an HTML document, including metadata, content metrics,
/// links, images, and structured data.
/// Comprehensive data structure representing a link found on a page
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnchorLink {
    pub href: String,
    pub text: String,
    pub rel: String,
}

/// Comprehensive data structure representing a parsed web page
///
/// This struct contains all relevant SEO and content analysis information
/// extracted from an HTML document, including metadata, content metrics,
/// links, images, and structured data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    pub anchor_links: Vec<AnchorLink>,
    pub outlinks: Vec<AnchorLink>,
    pub images: Vec<ImageInfo>,
    pub headings: Vec<(String, String)>,
    pub headers: Vec<String>,
    pub schema: Vec<String>,
    pub content_type: String,
    pub canonicals: Vec<(String, String, Option<String>)>, // rel, href, hreflang
    pub size: usize,
    pub word_count: Option<usize>,
}

/// Extract page elements and metadata from HTML document
///
/// This function parses the HTML document and extracts various SEO-relevant
/// elements like title, headings, meta tags, links, and images.
///
/// # Arguments
/// * `document` - The parsed HTML document to extract elements from
///
/// # Returns
/// A `PageData` struct containing all extracted information
pub fn extract_page_elements(document: &Html) -> PageData {
    // Define CSS selectors for common page elements
    let title_selector = Selector::parse("title").unwrap();
    let h1_selector = Selector::parse("h1").unwrap();
    let h2_selector = Selector::parse("h2").unwrap();
    let desc_selector = Selector::parse("meta[name='description']").unwrap();

    // Extract basic page metadata
    let title = extract_text_content(document, &title_selector);
    let h1 = extract_text_content(document, &h1_selector);
    let h2 = extract_text_content(document, &h2_selector);
    let description = extract_meta_content(document, &desc_selector);

    // Check for mobile viewport meta tag
    let mobile = document
        .select(&Selector::parse("meta[name='viewport']").unwrap())
        .next()
        .is_some();

    // Extract page language from html tag
    let language = document
        .select(&Selector::parse("html[lang]").unwrap())
        .next()
        .and_then(|e| e.value().attr("lang"))
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Extract robots meta tag for indexability
    let indexability = document
        .select(&Selector::parse("meta[name='robots']").unwrap())
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Extract all links from the page
    let anchor_links: Vec<AnchorLink> = document
        .select(&Selector::parse("a[href]").unwrap())
        .map(|e| {
            let href = e.value().attr("href").unwrap().to_string();
            let text = e.text().collect::<String>().trim().to_string();
            let rel = e.value().attr("rel").unwrap_or("").to_string();
            AnchorLink { href, text, rel }
        })
        .collect();

    let outlinks = anchor_links.clone();

    // Extract image information including size estimates
    let images: Vec<ImageInfo> = document
        .select(&Selector::parse("img[src]").unwrap())
        .map(|e| {
            let src = e.value().attr("src").unwrap().to_string();
            let alt = e.value().attr("alt").unwrap_or("").to_string();

            ImageInfo::new(src, alt)
        })
        .collect();

    // Extract all heading elements (h1-h6) with their levels
    let heading_selector = Selector::parse("h1,h2,h3,h4,h5,h6").unwrap();
    let headings: Vec<(String, String)> = document
        .select(&heading_selector)
        .map(|e| {
            let tag = e.value().name().to_string();
            let text = e.text().collect::<String>();
            (tag, text)
        })
        .collect();

    // Extract structured data (JSON-LD)
    let schema_selector = Selector::parse("script[type='application/ld+json']").unwrap();
    let schema: Vec<String> = document
        .select(&schema_selector)
        .map(|e| e.text().collect::<String>())
        .collect();

    // Extract content type from meta tag, default to text/html
    let content_type = document
        .select(&Selector::parse("meta[http-equiv=content-type]").unwrap())
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "text/html".to_string());

    // Extract canonical and alternate link tags
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

    // Calculate content size (character count of body text)
    let size = document
        .select(&Selector::parse("body").unwrap())
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or_default()
        .len();

    // Calculate word count for content analysis
    let word_count = Some(get_words(document));

    // Construct and return the comprehensive page data structure
    PageData {
        id: 0,               // Will be set by calling code
        url: "".to_string(), // Will be set by calling code
        title: title.clone(),
        title_len: title.len(),
        h1: h1.clone(),
        h1_len: h1.len(),
        h2: h2.clone(),
        h2_len: h2.len(),
        description: description.clone(),
        description_len: description.len(),
        status: "".to_string(), // Will be set by calling code
        mobile,
        language,
        indexability,
        anchor_links,
        outlinks,
        images,
        headings,
        headers: vec![], // Reserved for future use
        schema,
        content_type,
        canonicals,
        size,
        word_count,
    }
}

/// Helper function to extract text content from an element
///
/// # Arguments
/// * `document` - The HTML document to search
/// * `selector` - The CSS selector to find the element
///
/// # Returns
/// The text content of the first matching element, or empty string if not found
fn extract_text_content(document: &Html, selector: &Selector) -> String {
    document
        .select(selector)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or_default()
}

/// Helper function to extract content attribute from meta tags
///
/// # Arguments
/// * `document` - The HTML document to search
/// * `selector` - The CSS selector to find the meta element
///
/// # Returns
/// The content attribute value, or empty string if not found
fn extract_meta_content(document: &Html, selector: &Selector) -> String {
    document
        .select(selector)
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or_default()
}
