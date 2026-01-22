use scraper::{Html, Selector};
use std::sync::LazyLock;

use crate::crawler::helpers::extractor::text;
use crate::crawler::helpers::image_utils::ImageInfo;
use crate::crawler::helpers::keywords::extract_keywords;
use crate::crawler::helpers::word_count::get_words;
use crate::models::AppSettings;

/// Comprehensive data structure representing a parsed web page
///
/// This struct contains all relevant SEO and content analysis information
/// extracted from an HTML document, including metadata, content metrics,
/// links, images, and structured data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnchorLink {
    pub href: String,
    pub text: String,
    pub rel: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractorInfo {
    pub url: String,
    pub title: String,
    pub description: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CssInfo {
    pub total_size_bytes: Option<usize>,
    pub total_size_formatted: String,
    pub external_css_count: usize,
    pub inline_css_size_bytes: Option<usize>,
    pub inline_css_size_formatted: String,
    pub css_urls: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScriptInfo {
    pub src: Option<String>,
    pub script_type: String,
    pub is_async: bool,
    pub is_defer: bool,
    pub is_module: bool,
    pub integrity: Option<String>,
    pub crossorigin: Option<String>,
    pub inline_size: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JavascriptInfo {
    pub total_size_bytes: Option<usize>,
    pub total_size_formatted: String,
    pub external_js_count: usize,
    pub inline_js_size_bytes: Option<usize>,
    pub inline_js_size_formatted: String,
    pub js_urls: Vec<String>,
    pub scripts: Vec<ScriptInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CwvData {
    pub fcp: String,
    pub lcp: String,
    pub cls: String,
    pub tbt: String,
    pub speed_index: String,
    pub performance_score: String,
}

impl Default for CwvData {
    fn default() -> Self {
        Self {
            fcp: "...".to_string(),
            lcp: "...".to_string(),
            cls: "...".to_string(),
            tbt: "...".to_string(),
            speed_index: "...".to_string(),
            performance_score: "...".to_string(),
        }
    }
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
    pub css: Option<CssInfo>,
    pub javascript: Option<JavascriptInfo>,
    pub keywords: Option<Vec<String>>,
    pub cwv_desktop: Option<CwvData>,
    pub cwv_mobile: Option<CwvData>,
    pub extraction: Option<Vec<String>>,
}

// Define static CSS selectors for common page elements using LazyLock
// This avoids parsing the selector strings on every call, improving performance during crawls
static TITLE_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("title").unwrap());
static H1_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("h1").unwrap());
static H2_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("h2").unwrap());
static DESC_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[name='description']").unwrap());
static VIEWPORT_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[name='viewport']").unwrap());
static LANG_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("html[lang]").unwrap());
static ROBOTS_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[name='robots']").unwrap());
static AFREE_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("a[href]").unwrap());
static IMG_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("img[src]").unwrap());
static HEADING_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("h1,h2,h3,h4,h5,h6").unwrap());
static SCHEMA_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("script[type='application/ld+json']").unwrap());
static CONTENT_TYPE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta[http-equiv=content-type]").unwrap());
static CANONICAL_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("link[rel=canonical], link[rel=alternate]").unwrap());
static BODY_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("body").unwrap());
static STYLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("link[rel='stylesheet'], style").unwrap());
static SCRIPT_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("script").unwrap());

// LAZY LOAD THE SETTINGS WITH THE EXTRACTION AND THE EXTRACTOR
static APP_SETTINGS: LazyLock<AppSettings> = LazyLock::new(|| AppSettings::load());

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
    // Extract basic page metadata
    let title = extract_text_content(document, &TITLE_SELECTOR);
    let h1 = extract_text_content(document, &H1_SELECTOR);
    let h2 = extract_text_content(document, &H2_SELECTOR);
    let description = extract_meta_content(document, &DESC_SELECTOR);

    // Check for mobile viewport meta tag
    let mobile = document.select(&VIEWPORT_SELECTOR).next().is_some();

    // Extract page language from html tag
    let language = document
        .select(&LANG_SELECTOR)
        .next()
        .and_then(|e| e.value().attr("lang"))
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Extract robots meta tag for indexability
    let indexability = document
        .select(&ROBOTS_SELECTOR)
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Extract all links from the page
    let anchor_links: Vec<AnchorLink> = document
        .select(&AFREE_SELECTOR)
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
        .select(&IMG_SELECTOR)
        .map(|e| {
            let src = e.value().attr("src").unwrap().to_string();
            let alt = e.value().attr("alt").unwrap_or("").to_string();

            ImageInfo::new(src, alt)
        })
        .collect();

    // Extract all heading elements (h1-h6) with their levels
    let headings: Vec<(String, String)> = document
        .select(&HEADING_SELECTOR)
        .map(|e| {
            let tag = e.value().name().to_string();
            let text = e.text().collect::<String>();
            (tag, text)
        })
        .collect();

    // Extract structured data (JSON-LD)
    let schema: Vec<String> = document
        .select(&SCHEMA_SELECTOR)
        .map(|e| e.text().collect::<String>())
        .collect();

    // Extract content type from meta tag, default to text/html
    let content_type = document
        .select(&CONTENT_TYPE_SELECTOR)
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "text/html".to_string());

    // Extract canonical and alternate link tags
    let canonicals: Vec<(String, String, Option<String>)> = document
        .select(&CANONICAL_SELECTOR)
        .map(|e| {
            let rel = e.value().attr("rel").unwrap().to_string();
            let href = e.value().attr("href").unwrap().to_string();
            let hreflang = e.value().attr("hreflang").map(|s| s.to_string());
            (rel, href, hreflang)
        })
        .collect();

    // Calculate content size (character count of body text)
    let size = document
        .select(&BODY_SELECTOR)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or_default()
        .len();

    // Calculate word count for content analysis
    let word_count = Some(get_words(document));

    // GET THE CSS INFORMATION THAT IS POSSIBLE TO GRAB FROM THE HTML
    let css = document.select(&STYLE_SELECTOR).fold(
        CssInfo {
            total_size_bytes: Some(0),
            total_size_formatted: "0 B".to_string(),
            external_css_count: 0,
            inline_css_size_bytes: Some(0),
            inline_css_size_formatted: "0 B".to_string(),
            css_urls: Vec::new(),
        },
        |mut acc, e| {
            if e.value().name() == "link" {
                acc.external_css_count += 1;
                // Collect CSS URLs
                if let Some(href) = e.value().attr("href") {
                    acc.css_urls.push(href.to_string());
                }
                // Size estimation for external CSS could be added here
            } else if e.value().name() == "style" {
                let inline_css = e.text().collect::<String>();
                let inline_size = inline_css.len();
                acc.inline_css_size_bytes =
                    Some(acc.inline_css_size_bytes.unwrap_or(0) + inline_size);
                acc.inline_css_size_formatted = format!("{} B", acc.inline_css_size_bytes.unwrap());
            }
            acc
        },
    );

    // Gets all the javascript info from the page
    let javascript = document.select(&SCRIPT_SELECTOR).fold(
        JavascriptInfo {
            total_size_bytes: None,
            total_size_formatted: "".to_string(),
            external_js_count: 0,
            inline_js_size_bytes: None,
            inline_js_size_formatted: "".to_string(),
            js_urls: Vec::new(),
            scripts: Vec::new(),
        },
        |mut acc, e| {
            if e.value().name() == "script" {
                let src = e.value().attr("src").map(|s| s.to_string());
                let script_type = e
                    .value()
                    .attr("type")
                    .unwrap_or("text/javascript")
                    .to_string();
                let is_async = e.value().attr("async").is_some();
                let is_defer = e.value().attr("defer").is_some();
                let is_module = script_type == "module";
                let integrity = e.value().attr("integrity").map(|s| s.to_string());
                let crossorigin = e.value().attr("crossorigin").map(|s| s.to_string());

                let inline_content = if src.is_none() {
                    e.text().collect::<String>()
                } else {
                    String::new()
                };
                let inline_size = inline_content.len();

                if let Some(ref s) = src {
                    acc.external_js_count += 1;
                    acc.js_urls.push(s.clone());
                } else {
                    acc.inline_js_size_bytes =
                        Some(acc.inline_js_size_bytes.unwrap_or(0) + inline_size);
                    acc.inline_js_size_formatted =
                        format!("{} B", acc.inline_js_size_bytes.unwrap());
                }

                acc.scripts.push(ScriptInfo {
                    src,
                    script_type,
                    is_async,
                    is_defer,
                    is_module,
                    integrity,
                    crossorigin,
                    inline_size,
                });
            }
            acc
        },
    );

    // GETS THE KEYWORDS FROM THE CRAWLED PAGE
    let keywords = extract_keywords(&document);

    // GETS THE EXTRACTION FROM THE CRAWLED PAGE

    // IF THE EXTRACTOR IS ACTIVATED THEN WE CALL THE EXTRACTOR
    let extraction = if APP_SETTINGS.crawler.extractor {
        Some(text("text", document))
    } else {
        Some(vec!["empty".to_string()])
    };

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
        css: Some(css),
        javascript: Some(javascript),
        keywords: Some(keywords),
        cwv_desktop: None,
        cwv_mobile: None,
        extraction,
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
