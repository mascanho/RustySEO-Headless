use crate::models::App;
use crate::crawler::CrawlMessage;
use tokio::sync::mpsc;

impl App {
    pub fn on_tick(&mut self) {
        // 1. Collect results from background crawler thread
        let mut results = Vec::new();
        let mut crawl_finished = false;
        if let Some(ref mut rx) = self.crawl_receiver {
            loop {
                match rx.try_recv() {
                    Ok(CrawlMessage::Page(data)) => results.push(data),
                    Ok(CrawlMessage::Progress {
                        scanned,
                        queued,
                        processing,
                    }) => {
                        let total = scanned + queued + processing;
                        self.crawl_progress = if total == 0 {
                            0.0
                        } else {
                            (scanned as f64 / total as f64).min(1.0)
                        };
                    }
                    Err(mpsc::error::TryRecvError::Empty) => break,
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        crawl_finished = true;
                        break;
                    }
                }
            }
        }

        // 2. Process collected results
        for data in &results {
            let current_id = self.page_data.len() + 1;
            let mut page_data = data.clone();
            page_data.id = current_id;
            self.page_data.push(page_data);

            let mut row = vec![
                current_id.to_string(),
                data.url.clone(),
                data.title.clone(),
                data.title_len.to_string(),
                data.h1.clone(),
                data.h1_len.to_string(),
                data.description.clone(),
                data.description_len.to_string(),
                data.h2.clone(),
                data.h2_len.to_string(),
                data.status.clone(),
                data.mobile.to_string(),
                data.language.to_string(),
                data.indexability.to_string(),
                data.anchor_links.len().to_string(),
                data.content_type.clone(),
                data.canonicals.len().to_string(),
                data.size.to_string(),
                data.word_count.unwrap_or(0).to_string(),
                data.css
                    .as_ref()
                    .map_or("0 B".to_string(), |css| css.total_size_formatted.clone()),
                data.css
                    .as_ref()
                    .map_or("0".to_string(), |css| css.external_css_count.to_string()),
                data.css.as_ref().map_or("0 B".to_string(), |css| {
                    css.inline_css_size_formatted.clone()
                }),
                data.css
                    .as_ref()
                    .and_then(|css| css.css_urls.first())
                    .map_or("inline only".to_string(), |url| url.clone()),
            ];

            // 2b. Add top 10 keywords to the row
            let mut keywords = data.keywords.clone().unwrap_or_default();
            keywords.resize(10, String::new()); // Ensure we have 10 slots
            for kw in keywords {
                row.push(kw);
            }

            // 2c. Add CWV data
            let d = data.cwv_desktop.clone().unwrap_or_default();
            row.push(d.performance_score);
            row.push(d.fcp);
            row.push(d.lcp);
            row.push(d.cls);
            row.push(d.tbt);
            row.push(d.speed_index);

            let m = data.cwv_mobile.clone().unwrap_or_default();
            row.push(m.performance_score);
            row.push(m.fcp);
            row.push(m.lcp);
            row.push(m.cls);
            row.push(m.tbt);
            row.push(m.speed_index);

            self.table_data.push(row);

            // Populate internal and external links table
            let base_domain = url::Url::parse(&self.input_url)
                .ok()
                .and_then(|u| u.domain().map(|d| d.to_string()));

            for link in &data.anchor_links {
                let normalized_to = crate::crawler::url_normalizer::normalize_url(&link.href)
                    .unwrap_or_else(|| link.href.clone());

                let is_internal = if let Some(ref domain) = base_domain {
                    if let Ok(parsed_to) = url::Url::parse(&normalized_to) {
                        parsed_to.domain() == Some(domain)
                    } else {
                        // If it's a relative URL, normalize_url might have made it absolute if it had base_url
                        // but actually extract_page_elements just gets the href.
                        // Wait, if it's relative, it's usually internal.
                        !normalized_to.contains("://")
                    }
                } else {
                    true
                };

                if is_internal {
                    let internal_link = crate::models::InternalLink {
                        id: self.internal_table_data.len() + 1,
                        source: data.url.clone(),
                        destination: normalized_to.clone(),
                        anchor: link.text.clone(),
                        rel: link.rel.clone(),
                    };
                    self.internal_table_data.push(internal_link);
                } else {
                    let external_link = crate::models::ExternalLink {
                        id: self.external_table_data.len() + 1,
                        source: data.url.clone(),
                        destination: normalized_to.clone(),
                        anchor: link.text.clone(),
                        rel: link.rel.clone(),
                    };
                    self.external_table_data.push(external_link);
                }

                // Collect Files (non-HTML, non-PHP, non-CSS, non-JS)
                let path_part = normalized_to.split('?').next().unwrap_or("").split('#').next().unwrap_or("");
                let last_segment = path_part.split('/').last().unwrap_or("");
                let ext = if last_segment.contains('.') {
                    last_segment.split('.').last().unwrap_or("").to_lowercase()
                } else {
                    String::new()
                };

                if !ext.is_empty()
                    && ext != "html"
                    && ext != "htm"
                    && ext != "php"
                    && ext != "css"
                    && ext != "js"
                    && ext != "aspx"
                    && ext != "asp"
                    && ext != "jsp"
                    && ext != "png"
                    && ext != "jpg"
                    && ext != "jpeg"
                    && ext != "gif"
                    && ext != "svg"
                    && ext != "webp"
                    && ext != "ico"
                    && ext.len() < 10 // Avoid long "extensions" that are likely not files
                {
                    if !self.files_table_data.iter().any(|f| f.url == normalized_to) {
                        self.files_table_data.push(crate::models::FileEntry {
                            id: self.files_table_data.len() + 1,
                            url: normalized_to.clone(),
                            filetype: ext.to_uppercase(),
                        });
                    }
                }
            }

            // Collect CSS URLs for CSS URLs table
            if let Some(css_info) = &data.css {
                for css_url in &css_info.css_urls {
                    // Normalize the CSS URL if possible
                    let normalized_css_url = crate::crawler::url_normalizer::normalize_url(css_url)
                        .unwrap_or_else(|| css_url.clone());

                    // Check if this URL is already in our collection
                    let existing_index = self
                        .css_urls_table_data
                        .iter()
                        .position(|css| css.url == normalized_css_url);

                    if let Some(index) = existing_index {
                        // Increment the page count for existing URL
                        self.css_urls_table_data[index].page_count += 1;
                    } else {
                        // Add new unique CSS URL
                        let css_url_entry = crate::models::CssUrl {
                            id: self.css_urls_table_data.len() + 1,
                            url: normalized_css_url,
                            page_count: 1,
                        };
                        self.css_urls_table_data.push(css_url_entry);
                    }
                }
            }

            // Collect JS URLs for JS URLs table
            if let Some(js_info) = &data.javascript {
                for script in &js_info.scripts {
                    if let Some(js_url) = &script.src {
                        // Normalize the JS URL if possible
                        let normalized_js_url =
                            crate::crawler::url_normalizer::normalize_url(js_url)
                                .unwrap_or_else(|| js_url.clone());

                        // Check if this URL is already in our collection
                        let existing_index = self
                            .js_urls_table_data
                            .iter()
                            .position(|js| js.url == normalized_js_url);

                        if let Some(index) = existing_index {
                            // Increment the page count for existing URL
                            self.js_urls_table_data[index].page_count += 1;
                        } else {
                            // Add new unique JS URL
                            let js_url_entry = crate::models::JsUrl {
                                id: self.js_urls_table_data.len() + 1,
                                url: normalized_js_url,
                                script_type: script.script_type.clone(),
                                is_async: script.is_async,
                                is_defer: script.is_defer,
                                page_count: 1,
                            };
                            self.js_urls_table_data.push(js_url_entry);
                        }
                    }
                }
            }

            // Collect Extraction Results for Custom Search table
            if let Some(extraction) = &data.extraction {
                if extraction.found {
                    for match_item in &extraction.matches {
                        let entry = crate::models::ExtractionTableEntry {
                            id: self.extractor_table_data.len() + 1,
                            url: data.url.clone(),
                            element: match_item.element.clone(),
                            snippet: match_item.snippet.clone(),
                        };
                        self.extractor_table_data.push(entry);
                    }
                }
            }

            // Collect Images for Images table
            for image in &data.images {
                let normalized_img_url = image.src.clone();

                if let Some(existing) = self
                    .images_table_data
                    .iter_mut()
                    .find(|i| i.url == normalized_img_url)
                {
                    existing.page_count += 1;
                } else {
                    self.images_table_data.push(crate::models::ImageTableEntry {
                        id: self.images_table_data.len() + 1,
                        url: normalized_img_url,
                        alt: image.alt.clone(),
                        status: "-".to_string(),
                        size: image.size_formatted.clone(),
                        page_count: 1,
                    });
                }
            }

            // Collect Redirects
            if !data.redirect_chain.is_empty() {
                let final_status = data.status.parse::<u16>().unwrap_or(0);
                self.redirects_table_data.push(crate::models::RedirectEntry {
                    id: self.redirects_table_data.len() + 1,
                    initial_url: data.url.clone(),
                    status_code: final_status,
                    chain: data.redirect_chain.clone(),
                });
            }

            // Collect Keywords
            if let Some(keywords) = &data.keywords {
                let word_count = data.word_count.unwrap_or(0);
                for (i, kw) in keywords.iter().enumerate() {
                    self.keywords_table_data.push(crate::models::KeywordEntry {
                        id: self.keywords_table_data.len() + 1,
                        keyword: kw.clone(),
                        url: data.url.clone(),
                        word_count,
                        relevance: i + 1,
                    });
                }
            }

            self.url_to_status
                .insert(data.url.clone(), data.status.clone());
            self.log(format!("Crawled: {}", data.url));
        }

        if !results.is_empty() {
            self.apply_filter();
            self.apply_internal_filter();
            self.apply_external_filter();
            self.apply_css_urls_filter();
            self.apply_js_urls_filter();
            self.apply_extractor_filter();
            self.apply_content_filter();
            self.apply_files_filter();
            self.apply_redirects_filter();
            self.apply_keywords_filter();
            self.update_issues_from_crawled_data();
        }

        if crawl_finished {
            self.is_crawling = false;
            self.crawl_receiver = None;
            self.crawl_progress = 1.0;
            self.log("SYSTEM - Crawl finished successfully.");

            // Update issues with real crawled data
            self.update_issues_from_crawled_data();
        }

        // Debounce search filtering
        if let Some(last_time) = self.last_search_time {
            if last_time.elapsed() > std::time::Duration::from_millis(300) {
                self.apply_filter();
                self.apply_internal_filter();
                self.apply_css_urls_filter();
                self.apply_js_urls_filter();
                self.apply_extractor_filter();
                self.apply_images_filter();
                self.apply_content_filter();
                self.apply_files_filter();
                self.apply_redirects_filter();
                self.last_search_time = None;
            }
        }
        if let Some(last_time) = self.last_log_search_time {
            if last_time.elapsed() > std::time::Duration::from_millis(300) {
                self.apply_log_filter();
                self.last_log_search_time = None;
            }
        }

        if self.input_url.is_empty() {
            return;
        }

        // 3. Process logs from tracing
        let mut tracing_logs = Vec::new();
        if let Some(ref rx) = self.log_receiver {
            while let Ok(log) = rx.try_recv() {
                tracing_logs.push(log);
            }
        }

        for log in tracing_logs {
            self.log(log);
        }
    }

    pub fn log<S: Into<String>>(&mut self, message: S) {
        let msg = message.into();
        // Check if it already has a timestamp [HH:MM:SS]
        let log_entry = if msg.starts_with('[')
            && msg.get(9..10) == Some("]")
            && msg.get(1..9).map(|s| s.contains(':')).unwrap_or(false)
        {
            msg
        } else {
            let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
            format!("[{}] [SYSTEM] {}", timestamp, msg)
        };

        self.logs_data.insert(0, log_entry);
        if self.logs_data.len() > 100 {
            self.logs_data.pop();
        }
        self.apply_log_filter();
    }

    pub fn increase_logs_height(&mut self) {
        if self.logs_height < 50 {
            self.logs_height += 2;
        }
    }

    pub fn decrease_logs_height(&mut self) {
        if self.logs_height > 5 {
            self.logs_height = self.logs_height.saturating_sub(2);
        }
    }
}
