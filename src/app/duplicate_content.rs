//! Near/exact-duplicate content detection, driven by the SimHash fingerprint
//! computed per page in `crawler::helpers::simhash`.

use crate::crawler::helpers::simhash::{hamming_distance, NEAR_DUPLICATE_THRESHOLD};
use crate::models::{App, DuplicatePair, PageSummary};

impl App {
    /// Compares `new_page`'s content fingerprint against every already-crawled
    /// page and records any within `NEAR_DUPLICATE_THRESHOLD` bits as a
    /// duplicate pair. Called once per incoming page, before it's pushed into
    /// `page_summaries` - O(pages crawled so far) per page, which stays cheap
    /// at crawl scale (a few hundred to a few thousand pages), rather than
    /// O(n^2) recomputed every tick.
    ///
    /// A fingerprint of exactly 0 means "no extractable body text" (real text
    /// essentially never hashes to literal zero across all 64 bits), not a
    /// missing value - pages with no unique content are themselves a
    /// duplicate/thin-content signal worth surfacing, so they're compared
    /// like any other fingerprint rather than skipped.
    pub fn detect_duplicate_content(&mut self, new_page: &PageSummary) {
        for existing in &self.page_summaries {
            let distance =
                hamming_distance(existing.content_fingerprint, new_page.content_fingerprint);
            if distance <= NEAR_DUPLICATE_THRESHOLD {
                self.duplicate_pairs.push(DuplicatePair {
                    id_a: existing.id,
                    id_b: new_page.id,
                    distance,
                });
            }
        }
    }
}
