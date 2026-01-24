use crate::crawler::helpers::html_parser::PageData;

/// Issue analysis functions for detecting website issues from crawled data
pub struct IssueAnalyzer;

impl IssueAnalyzer {
    /// Analyze crawled data to detect 404 errors and return (count, urls)
    pub fn analyze_404_errors(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls_404 = Vec::new();

        for page in page_data {
            if page.status == "404" || page.status.starts_with("4") {
                urls_404.push(page.url.clone());
            }
        }

        let count = urls_404.len();
        (count, urls_404)
    }

    /// Analyze crawled data to detect pages with titles > 60 characters and return (count, urls)
    pub fn analyze_long_titles(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls_long_titles = Vec::new();

        for page in page_data {
            if page.title_len > 60 {
                urls_long_titles.push(format!("{} ({} chars)", page.url, page.title_len));
            }
        }

        let count = urls_long_titles.len();
        (count, urls_long_titles)
    }

    /// Analyze crawled data to detect images with missing alt text and return (count, urls)
    pub fn analyze_missing_alt_text(page_data: &[PageData]) -> (usize, Vec<String>) {
        let mut urls_missing_alt = Vec::new();

        for page in page_data {
            let mut missing_count = 0;
            for image in &page.images {
                if image.alt.trim().is_empty() {
                    missing_count += 1;
                }
            }

            if missing_count > 0 {
                urls_missing_alt.push(format!("{} ({} images)", page.url, missing_count));
            }
        }

        let count = urls_missing_alt.len();
        (count, urls_missing_alt)
    }

    /// Get real URLs for a specific issue type
    pub fn get_urls_for_issue(page_data: &[PageData], issue_type: &str) -> Vec<String> {
        match issue_type {
            "404 Errors" => Self::analyze_404_errors(page_data).1,
            "Page Titles > 60 chars" => Self::analyze_long_titles(page_data).1,
            "Missing Alt Text" => Self::analyze_missing_alt_text(page_data).1,
            _ => vec![],
        }
    }

    /// Generate issues table data with real crawled data analysis
    pub fn generate_issues_table_data(page_data: &[PageData]) -> Vec<Vec<String>> {
        let total_pages = page_data.len();

        if total_pages == 0 {
            // Return default values if no data
            return vec![
                vec!["404 Errors".to_string(), "0".to_string(), "0%".to_string()],
                vec![
                    "Page Titles > 60 chars".to_string(),
                    "0".to_string(),
                    "0%".to_string(),
                ],
                vec![
                    "Missing Alt Text".to_string(),
                    "0".to_string(),
                    "0%".to_string(),
                ],
                vec!["Slow Load".to_string(), "0".to_string(), "0%".to_string()],
            ];
        }

        // Get real issue counts and URLs
        let (count_404, _urls_404) = Self::analyze_404_errors(page_data);
        let (count_long_titles, _urls_long_titles) = Self::analyze_long_titles(page_data);
        let (count_missing_alt, _urls_missing_alt) = Self::analyze_missing_alt_text(page_data);

        // Calculate percentages
        let percent_404 = if total_pages > 0 {
            (count_404 * 100) / total_pages
        } else {
            0
        };
        let percent_long_titles = if total_pages > 0 {
            (count_long_titles * 100) / total_pages
        } else {
            0
        };
        let percent_missing_alt = if total_pages > 0 {
            (count_missing_alt * 100) / total_pages
        } else {
            0
        };

        // Return issues table with real data
        vec![
            vec![
                "404 Errors".to_string(),
                count_404.to_string(),
                format!("{}%", percent_404),
            ],
            vec![
                "Page Titles > 60 chars".to_string(),
                count_long_titles.to_string(),
                format!("{}%", percent_long_titles),
            ],
            vec![
                "Missing Alt Text".to_string(),
                count_missing_alt.to_string(),
                format!("{}%", percent_missing_alt),
            ],
            vec!["Slow Load".to_string(), "0".to_string(), "0%".to_string()], // Placeholder for future implementation
        ]
    }
}
