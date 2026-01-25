use crate::crawler::helpers::html_parser::PageData;

pub struct IssueHandler {
    pub name: &'static str,
    pub process: fn(&[PageData]) -> (usize, Vec<String>),
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
                name: " 301 Errors",
                process: Self::analyse_301_redirects,
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
                name: " Slow Load",
                process: |_| (0, vec![]), // Placeholder
            },
        ]
    }

    // GET THE 301 REDIRECTS STATUS CODES URLS
    pub fn analyse_301_redirects(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();

        for page in page_data {
            if page.status == "301" {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Analyze crawled data to detect 404 errors
    pub fn analyze_404_errors(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.status == "404" || page.status.starts_with("4") {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Analyze page titles > 60 chars
    pub fn analyze_long_titles(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.title_len > 60 {
                urls.push(format!("{} ({} chars)", page.url, page.title_len));
            }
        }
        (urls.len(), urls)
    }

    /// Analyze page titles < 30 chars
    pub fn analyze_short_titles(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.title_len > 0 && page.title_len < 30 {
                urls.push(format!("{} ({} chars)", page.url, page.title_len));
            }
        }
        (urls.len(), urls)
    }

    /// Analyze missing alt text
    pub fn analyze_missing_alt_text(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            let missing = page
                .images
                .iter()
                .filter(|img| img.alt.trim().is_empty())
                .count();
            if missing > 0 {
                urls.push(format!("{} ({} images)", page.url, missing));
            }
        }
        (urls.len(), urls)
    }

    /// Analyze missing H1 tags
    pub fn analyze_missing_h1(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.h1_len == 0 {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Analyze page descriptions > 160 chars
    pub fn analyze_long_descriptions(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.description_len > 160 {
                urls.push(format!("{} ({} chars)", page.url, page.description_len));
            }
        }
        (urls.len(), urls)
    }

    /// Analyze missing page descriptions
    pub fn analyze_missing_descriptions(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.description_len == 0 {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Analyze missing page titles
    pub fn analyze_missing_titles(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls = Vec::new();
        for page in page_data {
            if page.title_len == 0 {
                urls.push(page.url.clone());
            }
        }
        (urls.len(), urls)
    }

    /// Get real URLs for a specific issue type
    pub fn get_urls_for_issue(page_data: &[PageData], issue_type: &str) -> Vec<String> {
        let handlers = Self::get_handlers();
        if let Some(handler) = handlers.iter().find(|h| h.name == issue_type) {
            (handler.process)(page_data).1
        } else {
            vec![]
        }
    }

    /// Generate issues table data
    pub fn generate_issues_table_data(page_data: &[PageData]) -> Vec<Vec<String>> {
        let total_pages = page_data.len();
        let handlers = Self::get_handlers();
        let mut table_data = Vec::new();

        for handler in handlers {
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
}
