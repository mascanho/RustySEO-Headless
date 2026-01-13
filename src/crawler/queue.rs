use std::collections::VecDeque;

/// A queue for managing URLs to crawl
/// Uses a VecDeque for efficient FIFO operations (breadth-first crawling)
#[derive(Debug, Clone)]
pub struct CrawlQueue {
    queue: VecDeque<(String, Option<String>)>, // (url, referer)
}

impl CrawlQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Add a URL to the back of the queue (breadth-first)
    pub fn push(&mut self, url: String, referer: Option<String>) {
        self.queue.push_back((url, referer));
    }

    /// Add multiple URLs to the queue
    pub fn push_batch(&mut self, urls: Vec<String>, referer: Option<String>) {
        for url in urls {
            self.queue.push_back((url, referer.clone()));
        }
    }

    /// Take the next URL from the front of the queue
    pub fn pop(&mut self) -> Option<(String, Option<String>)> {
        self.queue.pop_front()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get the current size of the queue
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Clear the queue
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

impl Default for CrawlQueue {
    fn default() -> Self {
        Self::new()
    }
}
