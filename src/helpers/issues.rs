use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::crawler::helpers::robots;
use crate::models::{InternalLink, PageSummary, RedirectEntry};

// Simple cache for robots.txt results to avoid repeated network requests
#[derive(Clone, Debug)]
struct CachedRobotsResult {
    urls: Vec<String>,
    timestamp: u64,
}

static ROBOTS_CACHE: LazyLock<Arc<Mutex<HashMap<String, CachedRobotsResult>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub struct IssueHandler {
    pub name: &'static str,
    pub process: fn(&[PageSummary]) -> (usize, Vec<String>),
}

/// Issue analysis functions for detecting website issues from crawled data
pub struct IssueAnalyzer;

impl IssueAnalyzer {
    pub fn get_handlers() -> Vec<IssueHandler> {
        vec![
            IssueHandler {
                name: " 404 Errors",
                process: Self::analyze_404_errors,
            },
            IssueHandler {
                name: " 3XX Errors",
                process: Self::analyse_3xx_redirects,
            },
            IssueHandler {
                name: " 5XX Errors",
                process: Self::analyse_5xx_errors,
            },
            IssueHandler {
                name: " Page Titles > 60 chars",
                process: Self::analyze_long_titles,
            },
            IssueHandler {
                name: " Page Titles < 30 chars",
                process: Self::analyze_short_titles,
            },
            IssueHandler {
                name: " Missing Alt Text",
                process: Self::analyze_missing_alt_text,
            },
            IssueHandler {
                name: " Missing H1",
                process: Self::analyze_missing_h1,
            },
            IssueHandler {
                name: " Page Description > 160 chars",
                process: Self::analyze_long_descriptions,
            },
            IssueHandler {
                name: " Missing Page Description",
                process: Self::analyze_missing_descriptions,
            },
            IssueHandler {
                name: " Missing Page Title",
                process: Self::analyze_missing_titles,
            },
            IssueHandler {
                name: " Slow Load (Poor CWV Performance)",
                process: Self::analyse_slow_load,
            },
            IssueHandler {
                name: " Non Canonical",
                process: Self::analyze_non_canonical_urls,
            },
            IssueHandler {
                name: " Canonicalised to Another URL",
                process: Self::analyse_canonicalised_elsewhere,
            },
            IssueHandler {
                name: " Multiple Canonical Tags",
                process: Self::analyse_multiple_canonical_tags,
            },
            IssueHandler {
                name: " Duplicate Content",
                process: Self::analyse_duplicated_content,
            },
            IssueHandler {
                name: " Duplicate Title Tags",
                process: Self::analyse_duplicate_titles,
            },
            IssueHandler {
                name: " Duplicate Meta Descriptions",
                process: Self::analyse_duplicate_descriptions,
            },
            IssueHandler {
                name: " Non Webp/Avif Images",
                process: Self::analyse_urls_with_png_or_jpg,
            },
            IssueHandler {
                name: " Parameterised URLs",
                process: Self::analyse_param_urls,
            },
            IssueHandler {
                name: " Large HTML Pages",
                process: Self::analyse_big_html_pages,
            },
            IssueHandler {
                name: " Thin Content (< 300 words)",
                process: Self::analyse_thin_content,
            },
            IssueHandler {
                name: " Multiple H1 Tags",
                process: Self::analyse_multiple_h1,
            },
            IssueHandler {
                name: " Missing Viewport (Not Mobile Friendly)",
                process: Self::analyse_missing_viewport,
            },
            IssueHandler {
                name: " Missing HTML Lang Attribute",
                process: Self::analyse_missing_lang,
            },
            IssueHandler {
                name: " Noindex (Meta Robots)",
                process: Self::analyse_noindex_meta,
            },
            IssueHandler {
                name: " Noindex (X-Robots-Tag Header)",
                process: Self::analyse_noindex_header,
            },
            IssueHandler {
                name: " Duplicate H1 Tags",
                process: Self::analyse_duplicate_h1,
            },
            IssueHandler {
                name: " Mixed Content (HTTP Resources on HTTPS Page)",
                process: Self::analyse_mixed_content,
            },
            IssueHandler {
                name: " Underscores in URL",
                process: Self::analyse_underscore_urls,
            },
            IssueHandler {
                name: " Uppercase Characters in URL",
                process: Self::analyse_uppercase_urls,
            },
            IssueHandler {
                name: " Non-ASCII Characters in URL",
                process: Self::analyse_non_ascii_urls,
            },
            IssueHandler {
                name: " URL Too Long (> 115 chars)",
                process: Self::analyse_long_urls,
            },
            IssueHandler {
                name: " Generic or Empty Anchors",
                process: Self::analyse_generic_anchors,
            },
            IssueHandler {
                name: " Low Internal Link Count",
                process: Self::analyse_low_internal_link_count,
            },
            IssueHandler {
                name: " Missing Schema",
                process: Self::analyse_missing_schema,
            },
            IssueHandler {
                name: " Non HTTPS",
                process: Self::analyse_non_https,
            },
            // Robots disallow links, Orphan Pages, Redirect Chains, Broken Internal Links,
            // Internal Nofollow Links, Canonical Points to Broken Page, and Redirects to
            // Error are handled separately since they need whole-crawl link-graph data
            // that isn't available per-page - see App::update_issues_from_crawled_data.
        ]
    }

    // GET ALL THE DISALOW LINKS FROM THE ROBOTS
    pub fn analyse_robots_disallow_links(_page_data: &[PageSummary]) -> (usize, Vec<String>) {
        // This function should not be called anymore since we handle robots separately
        // Return empty result as the actual robots analysis is handled asynchronously
        (0, vec![])
    }

    /// Successfully crawled pages that no internal link (from any other crawled page)
    /// points to. Excludes the crawl's own start URL, which is expected to have no
    /// inbound crawled links since it's the seed rather than something discovered.
    pub fn analyse_orphan_pages(
        page_data: &[PageSummary],
        internal_links: &[InternalLink],
        start_url: &str,
    ) -> (usize, Vec<String>) {
        let linked_targets: HashSet<&str> =
            internal_links.iter().map(|l| l.destination.as_str()).collect();

        let mut orphans = Vec::new();
        for page in page_data {
            if !page.status.starts_with('2') {
                continue;
            }
            if page.url == start_url {
                continue;
            }
            if !linked_targets.contains(page.url.as_str()) {
                orphans.push(page.url.clone());
            }
        }
        (orphans.len(), orphans)
    }

    /// Redirects that hop through more than one intermediate URL before reaching
    /// their final destination - each extra hop dilutes link equity and slows loads.
    pub fn analyse_redirect_chains(redirects: &[RedirectEntry]) -> (usize, Vec<String>) {
        let mut chains = Vec::new();
        for entry in redirects {
            if entry.chain.len() <= 1 {
                continue;
            }
            let chain_str = entry
                .chain
                .iter()
                .map(|h| format!("{} ({})", h.url, h.status))
                .collect::<Vec<_>>()
                .join(" -> ");
            chains.push(format!("{}: {}", entry.initial_url, chain_str));
        }
        (chains.len(), chains)
    }

    /// Internal links whose destination resolved to a 4xx/5xx status - broken links
    /// that waste crawl budget and hurt UX.
    pub fn analyse_broken_internal_links(
        internal_links: &[InternalLink],
        url_to_status: &HashMap<String, String>,
    ) -> (usize, Vec<String>) {
        let mut broken = Vec::new();
        for link in internal_links {
            if let Some(status) = url_to_status.get(&link.destination)
                && (status.starts_with('4') || status.starts_with('5'))
            {
                broken.push(format!("{} -> {} ({})", link.source, link.destination, status));
            }
        }

        (broken.len(), broken)
    }

    /// Internal links marked `rel="nofollow"` - usually unintentional on same-site
    /// links, and it wastes crawl budget/link equity that could flow internally.
    pub fn analyse_internal_nofollow_links(internal_links: &[InternalLink]) -> (usize, Vec<String>) {
        let mut flagged = Vec::new();
        for link in internal_links {
            if link.rel.to_lowercase().contains("nofollow") {
                flagged.push(format!("{} -> {}", link.source, link.destination));
            }
        }
        (flagged.len(), flagged)
    }

    /// Pages that canonicalise to another URL which itself returns a 4xx/5xx -
    /// the canonical signal points nowhere useful.
    pub fn analyse_canonical_points_to_broken(
        page_data: &[PageSummary],
        url_to_status: &HashMap<String, String>,
    ) -> (usize, Vec<String>) {
        let mut broken = Vec::new();
        for page in page_data {
            if let Some(target) = &page.canonical_target
                && let Some(status) = url_to_status.get(target)
                && (status.starts_with('4') || status.starts_with('5'))
            {
                broken.push(format!("{} -> {} ({})", page.url, target, status));
            }
        }
        (broken.len(), broken)
    }

    /// Redirects whose final destination is not a 2xx - a redirect loop, or one
    /// that ultimately lands on a broken page.
    pub fn analyse_redirects_to_error(redirects: &[RedirectEntry]) -> (usize, Vec<String>) {
        let mut broken = Vec::new();
        for entry in redirects {
            if !(200..300).contains(&entry.status_code) {
                broken.push(format!("{} (final status: {})", entry.initial_url, entry.status_code));
            }
        }
        (broken.len(), broken)
    }

    // GET THE PAGES THAT ARE NOTRR HTTPS SECURE
    pub fn analyse_non_https(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut non_https = Vec::new();

        for page in page_data {
            if !page.url.starts_with("https://") {
                non_https.push(page.url.clone());
            }
        }

        (non_https.len(), non_https)
    }

    // FLAGG THE ONES WITH MISSING SCHEMA
    pub fn analyse_missing_schema(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut missing_schema = Vec::new();

        for page in page_data {
            if !page.has_schema {
                missing_schema.push(page.url.clone());
            }
        }

        (missing_schema.len(), missing_schema)
    }

    // GET THE PAGES WITH LOW INTERNAL LINK COUNT, EXCLUDING PARAMETERISED URLS
    pub fn analyse_low_internal_link_count(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut low_internal_link_count = Vec::new();

        for page in page_data {
            // Skip error pages and non-HTML content
            if page.status.starts_with("4") || page.status.starts_with("5") {
                continue;
            }

            // Skip parameterized URLs, fragments, and common non-content URLs
            if page.url.contains("?") || page.url.contains("#") {
                continue;
            }

            if page.internal_link_count < 5 {
                low_internal_link_count.push(format!("{}", page.url));
            }
        }

        (low_internal_link_count.len(), low_internal_link_count)
    }

    // GET ALL THE STUFF WITH GENERRIC ANCHORS OR EMPTY
    pub fn analyse_generic_anchors(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut generic_anchors = Vec::new();

        for page in page_data {
            if page.has_generic_anchors {
                generic_anchors.push(page.url.clone());
            }
        }

        (generic_anchors.len(), generic_anchors)
    }

    /// Pages whose Core Web Vitals performance score (desktop or mobile) is below 50,
    /// i.e. Google's "Poor" threshold. Only counted when CWV was actually fetched.
    pub fn analyse_slow_load(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            let desktop_slow = page.cwv_performance_desktop.is_some_and(|s| s < 50.0);
            let mobile_slow = page.cwv_performance_mobile.is_some_and(|s| s < 50.0);
            if !desktop_slow && !mobile_slow {
                continue;
            }
            let detail = match (page.cwv_performance_desktop, page.cwv_performance_mobile) {
                (Some(d), Some(m)) => format!("{} (Desktop: {:.0}, Mobile: {:.0})", page.url, d, m),
                (Some(d), None) => format!("{} (Desktop: {:.0})", page.url, d),
                (None, Some(m)) => format!("{} (Mobile: {:.0})", page.url, m),
                (None, None) => page.url.clone(),
            };
            urls.push(detail);
        }
        (urls.len(), urls)
    }

    /// Pages that canonicalise to a different URL than themselves - worth a manual
    /// check that the consolidation is intentional and not accidentally hiding the page.
    pub fn analyse_canonicalised_elsewhere(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if let Some(target) = &page.canonical_target {
                urls.push(format!("{} -> {}", page.url, target));
            }
        }
        (urls.len(), urls)
    }

    /// Pages with more than one `rel="canonical"` tag - conflicting canonical
    /// signals that search engines may resolve unpredictably.
    pub fn analyse_multiple_canonical_tags(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.canonical_count > 1 {
                urls.push(format!("{} ({} canonical tags)", page.url, page.canonical_count));
            }
        }
        (urls.len(), urls)
    }

    /// Pages sharing the exact same (non-empty) title tag.
    pub fn analyse_duplicate_titles(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut seen: HashMap<&str, &str> = HashMap::new();
        let mut duplicates = Vec::new();
        for page in page_data {
            let title = page.title.trim();
            if title.is_empty() {
                continue;
            }
            if let Some(existing_url) = seen.get(title) {
                duplicates.push(format!("{} [ and ]  {}", existing_url, page.url));
            } else {
                seen.insert(title, &page.url);
            }
        }
        (duplicates.len(), duplicates)
    }

    /// Pages sharing the exact same (non-empty) meta description.
    pub fn analyse_duplicate_descriptions(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut seen: HashMap<&str, &str> = HashMap::new();
        let mut duplicates = Vec::new();
        for page in page_data {
            let description = page.description.trim();
            if description.is_empty() {
                continue;
            }
            if let Some(existing_url) = seen.get(description) {
                duplicates.push(format!("{} [ and ]  {}", existing_url, page.url));
            } else {
                seen.insert(description, &page.url);
            }
        }
        (duplicates.len(), duplicates)
    }

    /// Pages with fewer than 300 words - thin content that's unlikely to rank well.
    pub fn analyse_thin_content(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if !page.status.starts_with('2') {
                continue;
            }
            if page.word_count > 0 && page.word_count < 300 {
                urls.push(format!("{} ({} words)", page.url, page.word_count));
            }
        }
        (urls.len(), urls)
    }

    /// Pages with more than one `<h1>` tag - ambiguous primary heading signal.
    pub fn analyse_multiple_h1(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.h1_count > 1 {
                urls.push(format!("{} ({} H1 tags)", page.url, page.h1_count));
            }
        }
        (urls.len(), urls)
    }

    /// Pages missing a `<meta name="viewport">` tag, so they won't render responsively
    /// on mobile - both a UX and a mobile-first-indexing SEO issue.
    pub fn analyse_missing_viewport(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.status.starts_with('2') && !page.mobile {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Pages missing the `lang` attribute on `<html>`, which hurts screen readers
    /// and lets search engines misjudge the page's language/locale.
    pub fn analyse_missing_lang(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.status.starts_with('2') && page.language.trim().is_empty() {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Pages with a meta robots tag containing `noindex`.
    pub fn analyse_noindex_meta(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.indexability.to_lowercase().contains("noindex") {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Pages blocked from indexing via an `X-Robots-Tag: noindex` response header -
    /// easy to miss since it doesn't show up when viewing page source.
    pub fn analyse_noindex_header(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.has_noindex_header {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Pages sharing the exact same (non-empty) H1 - same signal problem as
    /// duplicate titles, just for the on-page heading instead of the `<title>`.
    pub fn analyse_duplicate_h1(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut seen: HashMap<&str, &str> = HashMap::new();
        let mut duplicates = Vec::new();
        for page in page_data {
            let h1 = page.h1.trim();
            if h1.is_empty() {
                continue;
            }
            if let Some(existing_url) = seen.get(h1) {
                duplicates.push(format!("{} [ and ]  {}", existing_url, page.url));
            } else {
                seen.insert(h1, &page.url);
            }
        }
        (duplicates.len(), duplicates)
    }

    /// HTTPS pages that load an image, stylesheet, or script over plain HTTP -
    /// browsers block or warn on this, and it can break the page for visitors.
    pub fn analyse_mixed_content(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.has_mixed_content {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// URLs containing underscores - search engines treat them as word joiners
    /// rather than separators, unlike hyphens.
    pub fn analyse_underscore_urls(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.url.contains('_') {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// URLs containing uppercase characters - can cause duplicate-content issues
    /// on case-sensitive servers and looks inconsistent in the SERP.
    pub fn analyse_uppercase_urls(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.url.chars().any(|c| c.is_uppercase()) {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// URLs containing non-ASCII characters (unencoded), which can render or copy
    /// inconsistently across browsers, tools, and search engines.
    pub fn analyse_non_ascii_urls(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if !page.url.is_ascii() {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// URLs longer than 115 characters - Google's general guidance threshold
    /// before URLs risk being truncated in the SERP.
    pub fn analyse_long_urls(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            let len = page.url.chars().count();
            if len > 115 {
                urls.push(format!("{} ({} chars)", page.url, len));
            }
        }
        (urls.len(), urls)
    }

    // GETS ALL THE HTML PAGES THAT ARE TOO BIG
    pub fn analyse_big_html_pages(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut big_html_pages = Vec::new();

        for page in page_data {
            if page.size > 500_000 {
                big_html_pages.push(page.url.clone());
            }
        }

        (big_html_pages.len(), big_html_pages)
    }

    // GET ALL THE URLS THAT HAVE PARAMS
    pub fn analyse_param_urls(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut param_urls = Vec::new();

        for page in page_data {
            let has_param = page.url.contains('?')
                || page.url.contains('#')
                || page.url.contains('&')
                || page.url.contains('=');

            if has_param {
                param_urls.push(page.url.clone());
            }
        }

        (param_urls.len(), param_urls)
    }

    // GETS ALL THE URLS that contain PNGs or JPGs
    pub fn analyse_urls_with_png_or_jpg(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut image_urls = Vec::new();

        for page in page_data {
            if page.has_png_jpg {
                image_urls.push(page.url.clone());
            }
        }

        (image_urls.len(), image_urls)
    }

    /// Detects pages that share the same non-empty title and description.
    pub fn analyse_duplicated_content(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut duplicates = Vec::new();
        let mut content_map: HashMap<String, String> = HashMap::new();

        for page in page_data {
            if page.title.is_empty() || page.description.is_empty() {
                continue;
            }

            let key = format!("{}|{}", page.title, page.description);

            if let Some(existing_url) = content_map.get(&key) {
                // Skip query-parameter URLs to avoid false positives
                if page.url.contains("?") || existing_url.contains("?") {
                    continue;
                }

                duplicates.push(format!("{} [ and ]  {}", existing_url, page.url));
            } else {
                content_map.insert(key, page.url.clone());
            }
        }

        (duplicates.len(), duplicates)
    }

    // GET THE URLS THAT ARE NOT CANONICALISED
    pub fn analyze_non_canonical_urls(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();

        for page in page_data {
            if page.url.ends_with(".jpg")
                || page.url.ends_with(".pdf")
                || page.url.ends_with(".png")
                || page.url.ends_with(".svg")
                || page.url.contains("cdn-cgi")
                || page.url.ends_with("exe")
                || page.url.contains("?")
                || page.url.contains("#")
                || page.url.contains("!")
                || page.url.contains(".xml")
            {
                continue;
            } else if !page.is_canonical {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    // GET THE 5XX ERRORS STATUS CODES URLS
    pub fn analyse_5xx_errors(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();

        for page in page_data {
            if page.status.contains('5') {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    // GET THE 301 REDIRECTS STATUS CODES URLS
    pub fn analyse_3xx_redirects(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();

        for page in page_data {
            if page.status.contains('3') {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Analyze crawled data to detect 404 errors
    pub fn analyze_404_errors(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.status == "404" || page.status.starts_with('4') {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Analyze page titles > 60 chars
    pub fn analyze_long_titles(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.title_len > 60 {
                urls.push(format!("{} ({} chars)", page.url, page.title_len));
            }
        }
        (urls.len(), urls)
    }

    /// Analyze page titles < 30 chars
    pub fn analyze_short_titles(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.title_len > 0 && page.title_len < 30 {
                urls.push(format!("{} ({} chars)", page.url, page.title_len));
            }
        }
        (urls.len(), urls)
    }

    /// Analyze missing alt text
    pub fn analyze_missing_alt_text(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.images_missing_alt > 0 {
                urls.push(format!("{} ({} images)", page.url, page.images_missing_alt));
            }
        }
        (urls.len(), urls)
    }

    /// Analyze missing H1 tags
    pub fn analyze_missing_h1(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.h1_len == 0 {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Analyze page descriptions > 160 chars
    pub fn analyze_long_descriptions(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.description_len > 160 {
                urls.push(format!("{} ({} chars)", page.url, page.description_len));
            }
        }
        (urls.len(), urls)
    }

    /// Analyze missing page descriptions
    pub fn analyze_missing_descriptions(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.description_len == 0 {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Analyze missing page titles
    pub fn analyze_missing_titles(page_data: &[PageSummary]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.title_len == 0 {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Get real URLs for a specific issue type
    pub fn get_urls_for_issue(page_data: &[PageSummary], issue_type: &str) -> Vec<String> {
        // Special handling for robots disallow links - perform async analysis
        if issue_type == " Robots Disallow Links" {
            // For now, return a placeholder since we can't call async from sync context
            // This should be handled by the app layer with proper async context
            return vec!["Loading robots.txt analysis...".to_string()];
        }
        
        let handlers = Self::get_handlers();
        if let Some(handler) = handlers.iter().find(|h| h.name == issue_type) {
            (handler.process)(page_data).1
        } else {
            vec![]
        }
    }

    /// Generate issues table data
    pub fn generate_issues_table_data(page_data: &[PageSummary]) -> Vec<Vec<String>> {
        let total_pages = page_data.len();
        let handlers = Self::get_handlers();
        let mut table_data = Vec::new();

        for handler in handlers {
            // Skip robots analysis as it's handled asynchronously
            if handler.name == " Robots Disallow Links" {
                // Skip this - robots count will be handled separately
                continue;
            }
            
            let (count, _) = (handler.process)(page_data);
            let percentage = if total_pages > 0 {
                (count * 100) / total_pages
            } else {
                0
            };
            table_data.push(vec![
                handler.name.to_string(),
                count.to_string(),
                format!("{}%", percentage),
            ]);
        }
        table_data
    }

    /// Generate issues table data, including the whole-crawl checks (robots disallow,
    /// orphan pages, redirect chains, broken internal links) that need link-graph data
    /// beyond a single page's summary.
    #[allow(clippy::too_many_arguments)]
    pub fn generate_issues_table_data_with_robots(
        page_data: &[PageSummary],
        robots_count: usize,
        internal_links: &[InternalLink],
        redirects: &[RedirectEntry],
        url_to_status: &HashMap<String, String>,
        start_url: &str,
    ) -> Vec<Vec<String>> {
        let total_pages = page_data.len();
        let handlers = Self::get_handlers();
        let mut table_data = Vec::new();

        let percentage = |count: usize| {
            if total_pages > 0 {
                (count * 100) / total_pages
            } else {
                0
            }
        };

        for handler in handlers {
            let (count, _) = (handler.process)(page_data);
            table_data.push(vec![
                handler.name.to_string(),
                count.to_string(),
                format!("{}%", percentage(count)),
            ]);
        }

        table_data.push(vec![
            " Robots Disallow Links".to_string(),
            robots_count.to_string(),
            format!("{}%", percentage(robots_count)),
        ]);

        let (orphan_count, _) = Self::analyse_orphan_pages(page_data, internal_links, start_url);
        table_data.push(vec![
            " Orphan Pages".to_string(),
            orphan_count.to_string(),
            format!("{}%", percentage(orphan_count)),
        ]);

        let (chain_count, _) = Self::analyse_redirect_chains(redirects);
        table_data.push(vec![
            " Redirect Chains (> 1 hop)".to_string(),
            chain_count.to_string(),
            format!("{}%", percentage(chain_count)),
        ]);

        let (broken_count, _) = Self::analyse_broken_internal_links(internal_links, url_to_status);
        table_data.push(vec![
            " Broken Internal Links".to_string(),
            broken_count.to_string(),
            format!("{}%", percentage(broken_count)),
        ]);

        let (nofollow_count, _) = Self::analyse_internal_nofollow_links(internal_links);
        table_data.push(vec![
            " Internal Nofollow Links".to_string(),
            nofollow_count.to_string(),
            format!("{}%", percentage(nofollow_count)),
        ]);

        let (broken_canonical_count, _) =
            Self::analyse_canonical_points_to_broken(page_data, url_to_status);
        table_data.push(vec![
            " Canonical Points to Broken Page".to_string(),
            broken_canonical_count.to_string(),
            format!("{}%", percentage(broken_canonical_count)),
        ]);

        let (redirect_error_count, _) = Self::analyse_redirects_to_error(redirects);
        table_data.push(vec![
            " Redirects to Error Page".to_string(),
            redirect_error_count.to_string(),
            format!("{}%", percentage(redirect_error_count)),
        ]);

        table_data
    }

    /// Perform actual robots.txt analysis on-demand (async)
    pub async fn analyze_robots_on_demand(page_data: &[PageSummary]) -> Vec<String> {
        let mut blocked_urls = Vec::new();
        
        if page_data.is_empty() {
            return blocked_urls;
        }
        
        // Get the base domain from the first page to construct robots.txt URL
        if let Some(first_page) = page_data.first() {
            let base_url = first_page.url.split('/').take(3).collect::<Vec<_>>().join("/");
            let robots_url = format!("{}/robots.txt", base_url);
            
            // Check cache first (cache for 30 minutes for on-demand requests)
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            {
                let cache = ROBOTS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(cached) = cache.get(&robots_url) {
                    if current_time - cached.timestamp < 1800 { // 30 minutes cache
                        return cached.urls.clone();
                    }
                }
            }
            
            // Perform async network request with timeout
            match tokio::time::timeout(
                std::time::Duration::from_secs(10),
                robots::extract_robots_blocked_urls(&robots_url)
            ).await {
                Ok(Ok(urls)) => {
                    let filtered_urls: Vec<String> = urls.into_iter()
                        .filter(|url| !url.trim().is_empty() && url.trim() != "/" && url.trim() != "")
                        .collect();
                    
                    // Cache the result
                    {
                        let mut cache = ROBOTS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
                        cache.insert(robots_url.clone(), CachedRobotsResult {
                            urls: filtered_urls.clone(),
                            timestamp: current_time,
                        });
                    }
                    
                    blocked_urls.extend(filtered_urls);
                },
                Ok(Err(_)) => {
                    // Network error, return empty
                },
                Err(_) => {
                    // Timeout, return empty
                }
            }
        }

        blocked_urls
    }
}
