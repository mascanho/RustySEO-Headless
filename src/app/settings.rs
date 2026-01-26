use crate::models::{App, AppSettings};
use crate::crawler::CrawlEngine;

impl App {
    pub fn open_settings_file(&mut self) {
        let path = crate::models::AppSettings::path();
        #[cfg(target_os = "macos")]
        let cmd = "open";
        #[cfg(not(target_os = "macos"))]
        let cmd = "xdg-open";

        let _ = std::process::Command::new(cmd).arg(path).spawn();

        self.log("Opening settings file...".to_string());
    }

    pub fn start_crawl(&mut self) {
        if self.input_url.is_empty() {
            return;
        }

        self.is_crawling = true;
        self.crawl_progress = 0.0;
        self.table_data.clear();
        self.page_data.clear();
        self.internal_table_data.clear();
        self.css_urls_table_data.clear();
        self.js_urls_table_data.clear();
        self.extractor_table_data.clear();
        self.images_table_data.clear();
        self.files_table_data.clear();
        self.url_to_status.clear();
        self.current_page = 0;
        self.internal_current_page = 0;
        self.css_urls_current_page = 0;
        self.js_urls_current_page = 0;
        self.content_current_page = 0;
        self.extractor_current_page = 0;
        self.images_current_page = 0;
        self.files_current_page = 0;
        
        self.apply_filter();
        self.apply_internal_filter();
        self.apply_css_urls_filter();
        self.apply_js_urls_filter();
        self.apply_extractor_filter();
        self.apply_content_filter();
        self.apply_images_filter();
        self.apply_files_filter();

        let url = self.input_url.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(1000);
        self.crawl_receiver = Some(rx);

        self.log(format!("Starting crawl for: {}", url));

        tokio::spawn(async move {
            let crawler = CrawlEngine::new().await;
            crawler.crawl_concurrently(&url, tx).await;
        });
    }

    pub fn reload_settings_if_changed(&mut self) {
        let mut changed = false;
        if let Some(ref rx) = self.settings_receiver {
            while let Ok(_) = rx.try_recv() {
                changed = true;
            }
        }

        if changed || self.check_settings_mtime() {
            self.settings = Some(AppSettings::load());
            self.log("Settings reloaded".to_string());
        }
    }

    pub fn check_settings_mtime(&mut self) -> bool {
        let path = AppSettings::path();
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(mtime) = metadata.modified() {
                if let Some(last_mtime) = self.last_settings_mtime {
                    if mtime > last_mtime {
                        self.last_settings_mtime = Some(mtime);
                        return true;
                    }
                } else {
                    self.last_settings_mtime = Some(mtime);
                    return false;
                }
            }
        }
        false
    }
}
