use reqwest::{blocking::Client, redirect};
use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

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
}

pub struct CrawlEngine {
    client: Client,
    visited: HashSet<String>,
    max_pages: usize,
}

impl CrawlEngine {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("RustySEO/1.0 (+https://rustyseo.com)")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            visited: HashSet::new(),
            max_pages: 50, // Safety limit
        }
    }

    pub fn crawl(&mut self, start_url: &str) -> Vec<PageData> {
        let mut results = Vec::new();
        let mut to_visit = vec![start_url.to_string()];
        let base_url = match Url::parse(start_url) {
            Ok(url) => url,
            Err(_) => return results,
        };

        while let Some(current_url) = to_visit.pop() {
            if self.visited.contains(&current_url) || results.len() >= self.max_pages {
                continue;
            }
            self.visited.insert(current_url.clone());

            if let Ok(data) = self.fetch_page(&current_url, results.len() + 1) {
                // Find links for next crawl
                if let Ok(html) = self.get_html(&current_url) {
                    let links = self.extract_links(&html, &base_url);
                    for link in links {
                        if !self.visited.contains(&link) {
                            to_visit.push(link);
                        }
                    }
                }
                results.push(data);
            }
        }
        results
    }

    fn get_html(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let resp = self.client.get(url).send()?;
        Ok(resp.text()?)
    }

    pub fn fetch_page(
        &self,
        url_str: &str,
        id: usize,
    ) -> Result<PageData, Box<dyn std::error::Error>> {
        let response = self.client.get(url_str).send()?;
        let status = format!(
            "{} {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("")
        );

        let html_content = response.text()?;
        let document = Html::parse_document(&html_content);

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

        Ok(PageData {
            id,
            url: url_str.to_string(),
            title_len: title.len(),
            title,
            h1_len: h1.len(),
            h1,
            h2_len: h2.len(),
            h2,

            description_len: description.len(),
            description,
            status,
        })
    }

    fn extract_links(&self, html: &str, base_url: &Url) -> Vec<String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("a[href]").unwrap();
        let mut links = Vec::new();

        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(absolute_url) = base_url.join(href) {
                    // Stay on same domain
                    if absolute_url.domain() == base_url.domain() {
                        links.push(absolute_url.to_string());
                    }
                }
            }
        }
        links
    }
}
