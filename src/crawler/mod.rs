pub mod engine;
pub mod helpers;
pub mod queue;
pub mod sitemap;
pub mod stats;
pub mod url_normalizer;

pub use engine::CrawlEngine;
pub use helpers::html_parser::PageData;

#[derive(Clone)]
pub enum CrawlMessage {
    Page(PageData),
    Progress {
        scanned: usize,
        queued: usize,
        processing: usize,
    },
}
