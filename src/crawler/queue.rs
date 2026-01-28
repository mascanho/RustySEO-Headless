use std::collections::VecDeque;

/// A queue for managing URLs to crawl
/// Uses a VecDeque for efficient FIFO operations (breadth-first crawling)
#[derive(Debug, Clone)]
pub struct CrawlQueue {
    queue: VecDeque<(String, Option<String>)>, // (url, referer)
    max_size: Option<usize>, // Maximum queue size to prevent unbounded growth
}

impl CrawlQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            max_size: Some(50_000), // Default: limit to 50k URLs in queue
        }
    }

    /// Create a new queue with a specific max size
    pub fn with_max_size(max_size: Option<usize>) -> Self {
        Self {
            queue: VecDeque::new(),
            max_size,
        }
    }

    /// Add a URL to the back of the queue (breadth-first)
    /// Returns true if added, false if queue is at capacity
    pub fn push(&mut self, url: String, referer: Option<String>) -> bool {
        if let Some(max) = self.max_size {
            if self.queue.len() >= max {
                return false;
            }
        }
        self.queue.push_back((url, referer));
        true
    }

    /// Add multiple URLs to the queue
    /// Returns the number of URLs actually added (may be less than input if capacity reached)
    pub fn push_batch(&mut self, urls: Vec<String>, referer: Option<String>) -> usize {
        let mut added = 0;
        for url in urls {
            if self.push(url, referer.clone()) {
                added += 1;
            } else {
                break; // Stop if we hit capacity
            }
        }
        added
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

    /// Check if the queue is at capacity
    pub fn is_at_capacity(&self) -> bool {
        if let Some(max) = self.max_size {
            self.queue.len() >= max
        } else {
            false
        }
    }

    /// Get remaining capacity
    pub fn remaining_capacity(&self) -> Option<usize> {
        self.max_size.map(|max| max.saturating_sub(self.queue.len()))
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
