use crate::crawler::CrawlMessage;
use crate::models::App;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

impl App {
    pub fn on_tick(&mut self) {
        // 0. Check for robots analysis results
        self.check_robots_results();

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
                        self.queued_urls = queued;
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
        if !results.is_empty() {
            let base_domain = url::Url::parse(&self.input_url)
                .ok()
                .and_then(|u| u.host_str().map(|d| d.to_string()));

            for mut page_data in results {
                let current_id = self.page_summaries.len() + 1;
                page_data.id = current_id;

                // Save to DB and discard large fields
                if let Some(ref conn) = self.db_conn {
                    let _ = crate::db::save_page_data_with_conn(conn, &page_data);
                }

                // Create PageSummary for memory efficiency
                let summary = crate::models::PageSummary {
                    id: current_id,
                    url: page_data.url.clone(),
                    title: page_data.title.clone(),
                    title_len: page_data.title_len,
                    description: page_data.description.clone(),
                    description_len: page_data.description_len,
                    status: page_data.status.clone(),
                    h1_len: page_data.h1_len,
                    h1_count: page_data.headings.iter().filter(|(t, _)| t == "h1").count(),
                    h2_count: page_data.headings.iter().filter(|(t, _)| t == "h2").count(),
                    h3_count: page_data.headings.iter().filter(|(t, _)| t == "h3").count(),
                    h4_count: page_data.headings.iter().filter(|(t, _)| t == "h4").count(),
                    h5_count: page_data.headings.iter().filter(|(t, _)| t == "h5").count(),
                    h6_count: page_data.headings.iter().filter(|(t, _)| t == "h6").count(),
                    has_schema: !page_data.schema.is_empty(),
                    schema_count: page_data.schema.len(),
                    size: page_data.size,
                    word_count: page_data.word_count.unwrap_or(0),
                    internal_link_count: page_data
                        .anchor_links
                        .iter()
                        .filter(|a| {
                            !a.href.starts_with("http")
                                || base_domain.as_ref().map_or(true, |d| a.href.contains(d))
                        })
                        .count(),
                    external_link_count: page_data
                        .anchor_links
                        .iter()
                        .filter(|a| {
                            a.href.starts_with("http")
                                && base_domain.as_ref().map_or(false, |d| !a.href.contains(d))
                        })
                        .count(),
                    images_count: page_data.images.len(),
                    images_missing_alt: page_data
                        .images
                        .iter()
                        .filter(|i| i.alt.trim().is_empty())
                        .count(),
                    is_canonical: !page_data.canonicals.is_empty(),
                    has_png_jpg: page_data.images.iter().any(|i| {
                        i.src.to_lowercase().ends_with(".png")
                            || i.src.to_lowercase().ends_with(".jpg")
                            || i.src.to_lowercase().ends_with(".jpeg")
                    }),
                    mobile: page_data.mobile,
                    indexability: page_data.indexability.clone(),
                    language: page_data.language.clone(),
                    cwv_performance_desktop: page_data
                        .cwv_desktop
                        .as_ref()
                        .and_then(|c| c.performance_score.parse::<f64>().ok()),
                    cwv_performance_mobile: page_data
                        .cwv_mobile
                        .as_ref()
                        .and_then(|c| c.performance_score.parse::<f64>().ok()),
                    has_generic_anchors: page_data
                        .anchor_links
                        .iter()
                        .any(|a| a.text.is_empty() || a.text.to_lowercase().contains("here")),
                };
                self.page_summaries.push(summary);

                let mut row = vec![
                    current_id.to_string(),
                    page_data.url.clone(),
                    page_data.title.clone(),
                    page_data.title_len.to_string(),
                    page_data.h1.clone(),
                    page_data.h1_len.to_string(),
                    page_data.description.clone(),
                    page_data.description_len.to_string(),
                    page_data.h2.clone(),
                    page_data.h2_len.to_string(),
                    page_data.status.clone(),
                    page_data.mobile.to_string(),
                    page_data.language.to_string(),
                    page_data.indexability.to_string(),
                    page_data.anchor_links.len().to_string(),
                    page_data.content_type.clone(),
                    page_data.canonicals.len().to_string(),
                    page_data.size.to_string(),
                    page_data.word_count.unwrap_or(0).to_string(),
                    page_data
                        .css
                        .as_ref()
                        .map_or("0 B".to_string(), |css| css.total_size_formatted.clone()),
                    page_data
                        .css
                        .as_ref()
                        .map_or("0".to_string(), |css| css.external_css_count.to_string()),
                    page_data.css.as_ref().map_or("0 B".to_string(), |css| {
                        css.inline_css_size_formatted.clone()
                    }),
                    page_data
                        .css
                        .as_ref()
                        .and_then(|css| css.css_urls.first())
                        .map_or("inline only".to_string(), |url| url.clone()),
                ];

                // 2b. Add CWV data
                let d = page_data.cwv_desktop.clone().unwrap_or_default();
                row.push(d.performance_score);
                row.push(d.fcp);
                row.push(d.lcp);
                row.push(d.cls);
                row.push(d.tbt);
                row.push(d.speed_index);

                let m = page_data.cwv_mobile.clone().unwrap_or_default();
                row.push(m.performance_score);
                row.push(m.fcp);
                row.push(m.lcp);
                row.push(m.cls);
                row.push(m.tbt);
                row.push(m.speed_index);

                // Add Top 10 Keywords
                for kw in &page_data.keywords {
                    row.push(kw.clone());
                }
                // Fill remaining keyword slots with empty strings if less than 10
                for _ in page_data.keywords.len()..10 {
                    row.push(String::new());
                }

                // Populate internal and external links
                for link in &page_data.outlinks {
                    let normalized_to = crate::crawler::url_normalizer::normalize_url(&link.href)
                        .unwrap_or_else(|| link.href.clone());
                    if normalized_to.starts_with("mailto:") || normalized_to.contains("@") {
                        continue;
                    }

                    let is_internal = base_domain.as_ref().map_or(true, |d| {
                        if let Ok(u) = url::Url::parse(&normalized_to) {
                            u.host_str()
                                .map_or(false, |h| h == d || h.ends_with(&format!(".{}", d)))
                        } else {
                            !normalized_to.contains("://")
                        }
                    });

                    if is_internal {
                        let internal_link = crate::models::InternalLink {
                            id: self.internal_table_data.len() + 1,
                            source: page_data.url.clone(),
                            destination: normalized_to.clone(),
                            anchor: link.text.clone(),
                            rel: link.rel.clone(),
                        };
                        self.internal_table_data.push(internal_link.clone());
                        if self.internal_search_query.is_empty() {
                            self.internal_full_filtered_table_data.push(internal_link);
                        }
                    } else {
                        let external_link = crate::models::ExternalLink {
                            id: self.external_table_data.len() + 1,
                            source: page_data.url.clone(),
                            destination: normalized_to.clone(),
                            anchor: link.text.clone(),
                            rel: link.rel.clone(),
                        };
                        self.external_table_data.push(external_link.clone());
                        if self.external_search_query.is_empty() {
                            self.external_full_filtered_table_data.push(external_link);
                        }
                    }

                    // Collect Files - only flag URLs whose path basename ends in a
                    // recognized downloadable file extension (see
                    // url_normalizer::extract_file_extension). This avoids the domain's
                    // TLD or unrelated dotted path segments (e.g. "/v1.2/blog") being
                    // mistaken for a file extension.
                    if let Some(ext) =
                        crate::crawler::url_normalizer::extract_file_extension(&normalized_to)
                    {
                        if self.seen_files.insert(normalized_to.clone()) {
                            let file_entry = crate::models::FileEntry {
                                id: self.files_table_data.len() + 1,
                                url: normalized_to,
                                filetype: ext.to_uppercase(),
                            };
                            self.files_table_data.push(file_entry.clone());
                            if self.files_search_query.is_empty() {
                                self.files_full_filtered_table_data.push(file_entry);
                            }
                        }
                    }
                }

                // O(1) Unique Aggregates Updates
                if let Some(css_info) = &page_data.css {
                    for css_url in &css_info.css_urls {
                        let normalized = crate::crawler::url_normalizer::normalize_url(css_url)
                            .unwrap_or_else(|| css_url.clone());
                        let count = self.css_counts.entry(normalized.clone()).or_insert(0);
                        *count += 1;
                        if *count == 1 {
                            let entry = crate::models::CssUrl {
                                id: self.css_urls_table_data.len() + 1,
                                url: normalized.clone(),
                                page_count: 1,
                            };
                            self.css_urls_table_data.push(entry.clone());
                            if self.css_urls_search_query.is_empty() {
                                self.css_urls_full_filtered_table_data.push(entry);
                            }
                        } else {
                            // Update count in O(1) by keeping track of IDs? Or just search (it's less frequent now)
                            // Better: if it's already there, we only need to update it sometimes or on-demand.
                            // To keep it simple but fast for now, we'll only do linear search if it's there,
                            // but the count check makes it MUCH less frequent.
                            if let Some(item) = self
                                .css_urls_table_data
                                .iter_mut()
                                .find(|c| c.url == normalized)
                            {
                                item.page_count = *count;
                            }
                        }
                    }
                }

                if let Some(js_info) = &page_data.javascript {
                    for script in &js_info.scripts {
                        if let Some(js_url) = &script.src {
                            let normalized = crate::crawler::url_normalizer::normalize_url(js_url)
                                .unwrap_or_else(|| js_url.clone());
                            let count = self.js_counts.entry(normalized.clone()).or_insert(0);
                            *count += 1;
                            if *count == 1 {
                                let entry = crate::models::JsUrl {
                                    id: self.js_urls_table_state.selected().unwrap_or(0) + 1,
                                    url: normalized.clone(),
                                    script_type: script.script_type.clone(),
                                    is_async: script.is_async,
                                    is_defer: script.is_defer,
                                    page_count: 1,
                                };
                                self.js_urls_table_data.push(entry.clone());
                                if self.js_urls_search_query.is_empty() {
                                    self.js_urls_full_filtered_table_data.push(entry);
                                }
                            } else if let Some(item) = self
                                .js_urls_table_data
                                .iter_mut()
                                .find(|j| j.url == normalized)
                            {
                                item.page_count = *count;
                            }
                        }
                    }
                }

                for image in &page_data.images {
                    let count = self.image_counts.entry(image.src.clone()).or_insert(0);
                    *count += 1;
                    if *count == 1 {
                        let entry = crate::models::ImageTableEntry {
                            id: self.images_table_data.len() + 1,
                            url: image.src.clone(),
                            alt: image.alt.clone(),
                            status: "-".to_string(),
                            size: image.size_formatted.clone(),
                            page_count: 1,
                        };
                        self.images_table_data.push(entry.clone());
                        if self.images_search_query.is_empty() {
                            self.images_full_filtered_table_data.push(entry);
                        }
                    } else if let Some(item) = self
                        .images_table_data
                        .iter_mut()
                        .find(|i| i.url == image.src)
                    {
                        item.page_count = *count;
                    }
                }

                // Redirects & Keywords
                if !page_data.redirect_chain.is_empty() {
                    let entry = crate::models::RedirectEntry {
                        id: self.redirects_table_data.len() + 1,
                        initial_url: page_data.url.clone(),
                        status_code: page_data.status.parse().unwrap_or(0),
                        chain: page_data.redirect_chain.clone(),
                    };
                    self.redirects_table_data.push(entry.clone());
                    if self.redirects_search_query.is_empty() {
                        self.redirects_full_filtered_table_data.push(entry);
                    }

                    // Link Score: every hop (including the originally requested URL)
                    // resolves to the final destination, so inbound links to any of
                    // them can be bypassed straight to the page that was actually crawled.
                    if page_data.requested_url != page_data.url {
                        self.redirect_map
                            .insert(page_data.requested_url.clone(), page_data.url.clone());
                    }
                    for hop in &page_data.redirect_chain {
                        if hop.url != page_data.url {
                            self.redirect_map.insert(hop.url.clone(), page_data.url.clone());
                        }
                    }
                }

                // Link Score: record the canonical target when a page canonicalises
                // to a different URL, so its inbound links can flow to that target.
                if let Some((_, href, _)) = page_data
                    .canonicals
                    .iter()
                    .find(|(_, href, _)| !href.trim().is_empty())
                {
                    let normalized_canonical =
                        crate::crawler::url_normalizer::normalize_url(href)
                            .unwrap_or_else(|| href.clone());
                    if normalized_canonical != page_data.url {
                        self.canonical_map
                            .insert(page_data.url.clone(), normalized_canonical);
                    }
                }

                self.url_to_status
                    .insert(page_data.url.clone(), page_data.status.clone());
                self.table_data.push(row.clone());
                if self.search_query.is_empty() {
                    self.full_filtered_table_data.push(row.clone());
                }
                if self.content_search_query.is_empty() {
                    self.content_full_filtered_table_data.push(row);
                }
            }

            // Apply pagination incrementally
            self.apply_pagination();
            self.apply_internal_pagination();
            // ... (rest of pagination)
            self.apply_external_pagination();
            self.apply_css_urls_pagination();
            self.apply_js_urls_pagination();
            self.apply_extractor_pagination();
            self.apply_content_pagination();
            self.apply_files_pagination();
            self.apply_redirects_pagination();
            self.apply_images_pagination();

            // Rate-limited Issues Update (every 15s)
            static LAST_ISSUES_UPDATE: std::sync::LazyLock<std::sync::Mutex<Instant>> =
                std::sync::LazyLock::new(|| std::sync::Mutex::new(Instant::now()));
            if LAST_ISSUES_UPDATE.lock().unwrap().elapsed() > Duration::from_secs(15) {
                self.update_issues_from_crawled_data();
                *LAST_ISSUES_UPDATE.lock().unwrap() = Instant::now();
            }
        }

        if crawl_finished {
            self.is_crawling = false;
            self.crawl_receiver = None;
            self.crawl_progress = 1.0;
            self.log("SYSTEM - Crawl finished successfully.");
            self.update_issues_from_crawled_data();
            self.log("SYSTEM - Running Crawl Analysis (Link Score)...");
            self.compute_link_scores();
            self.log("SYSTEM - Link Score analysis complete.");

            let check_external_links = self
                .settings
                .as_ref()
                .map(|s| s.crawler.check_external_links)
                .unwrap_or(false);
            if check_external_links {
                self.start_external_link_check();
            }
        }

        // Drain any external link status check results as they stream in.
        if let Some(ref mut rx) = self.external_status_receiver {
            let mut finished = false;
            loop {
                match rx.try_recv() {
                    Ok((url, status)) => {
                        self.url_to_status.insert(url, status);
                    }
                    Err(mpsc::error::TryRecvError::Empty) => break,
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        finished = true;
                        break;
                    }
                }
            }
            if finished {
                self.external_status_receiver = None;
                self.log("SYSTEM - External link check complete.");
            }
        }

        // Search Debouncing (unchanged)
        if let Some(last_time) = self.last_search_time {
            if last_time.elapsed() > Duration::from_millis(500) {
                self.apply_filter();
                self.apply_internal_filter();
                self.apply_external_filter();
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
            if last_time.elapsed() > Duration::from_millis(500) {
                self.apply_log_filter();
                self.last_log_search_time = None;
            }
        }

        // 3. Process logs from tracing
        let mut logs = Vec::new();
        if let Some(ref rx) = self.log_receiver {
            let mut count = 0;
            while let Ok(log) = rx.try_recv() {
                logs.push(log);
                count += 1;
                if count > 50 {
                    break;
                }
            }
        }

        if !logs.is_empty() {
            for log in logs {
                self.log(log);
            }
            self.apply_log_filter();
        }
    }

    pub fn open_details(&mut self, id: usize) {
        if let Some(page_data) = crate::db::load_page_data(id) {
            self.selected_page_details = Some(page_data);
            self.show_details = true;
        }
    }

    pub fn refresh_details_for_current_selection(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            if selected < self.filtered_table_data.len() {
                let row_data = &self.filtered_table_data[selected];
                if let Ok(id) = row_data[0].parse::<usize>() {
                    if let Some(page_data) = crate::db::load_page_data(id) {
                        self.selected_page_details = Some(page_data);
                    }
                }
            }
        }
    }

    pub fn log<S: Into<String>>(&mut self, message: S) {
        let msg = message.into();
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let log_entry = format!("[{}] {}", timestamp, msg);

        self.logs_data.insert(0, log_entry);
        if self.logs_data.len() > 100 {
            self.logs_data.pop();
        }
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
