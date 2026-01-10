use rand::seq::SliceRandom;
use rand::thread_rng;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

use crate::crawler::helpers::{html_parser::extract_page_elements, user_agents::user_agents};

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
}

#[derive(Debug, Clone)]
pub struct CrawlEngine {
    client: Client,
    visited: HashSet<String>,
    max_pages: usize,
    user_agents: Vec<String>,
}

impl CrawlEngine {
    pub async fn new() -> Self {
        let user_agents_vec = match user_agents() {
            Ok(agents) if !agents.is_empty() => agents,
            _ => vec!["RustySEO/1.0 (+https://rustyseo.com)".to_string()],
        };

        let default_ua = user_agents_vec[0].clone();

        Self {
            client: Client::builder()
                .user_agent(default_ua)
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            visited: HashSet::new(),
            max_pages: 200, // Safety limit
            user_agents: user_agents_vec,
        }
    }

    pub async fn crawl(&mut self, start_url: &str, headless: bool) -> Vec<PageData> {
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

            if let Ok(data) = self.fetch_page(&current_url, results.len() + 1).await {
                // Find links for next crawl
                if let Ok(html) = self.get_html(&current_url).await {
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

        // THE USER GOES HEADLESS AS SUCH WE PRINT THE RESULTS TO THE CONSOLE
        if headless {
            // print results
            for result in &results {
                println!("Page ID: {}", result.id);
                println!("URL: {}", result.url);
                println!("Status: {}", result.status);
                println!("Title: {}", result.title);
                println!();
            }
        }

        results
    }

    async fn get_html(&self, url: &str) -> Result<String, String> {
        let user_agent = self.pick_random_user_agent();
        match self
            .client
            .get(url)
            .header("User-Agent", user_agent)
            .send()
            .await
        {
            Ok(resp) => match resp.text().await {
                Ok(text) => Ok(text),
                Err(_) => Err("Failed to read response text".to_string()),
            },
            Err(_) => Err("Failed to send request".to_string()),
        }
    }

    pub async fn fetch_page(&self, url_str: &str, id: usize) -> Result<PageData, String> {
        let user_agent = self.pick_random_user_agent();
        let response = match self
            .client
            .get(url_str)
            .header("User-Agent", user_agent)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(_) => return Err("Failed to send request".to_string()),
        };

        let status = format!(
            "{} {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("")
        );

        let html_content = match response.text().await {
            Ok(text) => text,
            Err(_) => return Err("Failed to read response text".to_string()),
        };
        let document = Html::parse_document(&html_content);

        let elements = extract_page_elements(&document);

        Ok(PageData {
            id,
            url: url_str.to_string(),
            title_len: elements.title.len(),
            title: elements.title,
            h1_len: elements.h1.len(),
            h1: elements.h1,
            h2_len: elements.h2.len(),
            h2: elements.h2,
            description_len: elements.description.len(),
            description: elements.description,
            status,
            mobile: elements.mobile,
            language: elements.language,
            indexability: elements.indexability,
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

    fn pick_random_user_agent(&self) -> String {
        let mut rng = thread_rng();
        if self.user_agents.is_empty() {
            "RustySEO/1.0 (+https://rustyseo.com)".to_string()
        } else {
            use rand::Rng;
            let idx = rng.gen_range(0..self.user_agents.len());
            self.user_agents[idx].clone()
        }
    }
}
