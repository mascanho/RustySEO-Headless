use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Statistics tracker for crawl operations
#[derive(Debug, Clone)]
pub struct CrawlStats {
    pub pages_crawled: Arc<AtomicUsize>,
    pub pages_failed: Arc<AtomicUsize>,
    pub links_discovered: Arc<AtomicUsize>,
    pub links_filtered: Arc<AtomicUsize>,
    pub links_duplicate: Arc<AtomicUsize>,
}

impl CrawlStats {
    pub fn new() -> Self {
        Self {
            pages_crawled: Arc::new(AtomicUsize::new(0)),
            pages_failed: Arc::new(AtomicUsize::new(0)),
            links_discovered: Arc::new(AtomicUsize::new(0)),
            links_filtered: Arc::new(AtomicUsize::new(0)),
            links_duplicate: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn increment_crawled(&self) {
        self.pages_crawled.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_failed(&self) {
        self.pages_failed.fetch_add(1, Ordering::SeqCst);
    }

    pub fn add_discovered(&self, count: usize) {
        self.links_discovered.fetch_add(count, Ordering::SeqCst);
    }

    pub fn add_filtered(&self, count: usize) {
        self.links_filtered.fetch_add(count, Ordering::SeqCst);
    }

    pub fn add_duplicate(&self, count: usize) {
        self.links_duplicate.fetch_add(count, Ordering::SeqCst);
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Crawled: {}, Failed: {}, Links Found: {}, Filtered: {}, Duplicates: {}",
            self.pages_crawled.load(Ordering::SeqCst),
            self.pages_failed.load(Ordering::SeqCst),
            self.links_discovered.load(Ordering::SeqCst),
            self.links_filtered.load(Ordering::SeqCst),
            self.links_duplicate.load(Ordering::SeqCst),
        )
    }
}

impl Default for CrawlStats {
    fn default() -> Self {
        Self::new()
    }
}
