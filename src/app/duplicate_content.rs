//! Near/exact-duplicate content detection, driven by the SimHash fingerprint
//! computed per page in `crawler::helpers::simhash`.

use crate::crawler::helpers::simhash::{hamming_distance, NEAR_DUPLICATE_THRESHOLD};
use crate::models::{App, DuplicatePair, PageSummary};

/// Above this many crawled pages the per-page `O(n)` pairwise scan (and thus the
/// `O(n^2)` total) stops running. On a large crawl the scan both stalls the UI
/// thread and lets `duplicate_pairs` grow without bound; past this point the
/// duplicate signal is already well established from the pages seen so far.
const DUP_SCAN_MAX_PAGES: usize = 25_000;

/// Hard ceiling on recorded duplicate pairs. Templated sites can legitimately
/// produce enormous numbers of near-duplicate pairs; keep the table useful
/// without letting it consume gigabytes.
const DUP_PAIRS_MAX: usize = 200_000;

impl App {
    /// Compares `new_page`'s content fingerprint against every already-crawled
    /// page and records any within `NEAR_DUPLICATE_THRESHOLD` bits as a
    /// duplicate pair. Called once per incoming page, before it's pushed into
    /// `page_summaries`.
    ///
    /// This is inherently `O(pages crawled)` per call. It is bounded three ways
    /// so it can't sink a large crawl:
    /// - pages whose body text didn't hash to anything (`content_fingerprint`
    ///   == 0, common when JS rendering is off) are skipped entirely - every
    ///   such page is within 0 bits of every other, which otherwise turns into
    ///   a quadratic explosion of meaningless pairs;
    /// - the scan stops once `page_summaries` exceeds `DUP_SCAN_MAX_PAGES`;
    /// - `duplicate_pairs` is capped at `DUP_PAIRS_MAX`.
    pub fn detect_duplicate_content(&mut self, new_page: &PageSummary) {
        // A zero fingerprint means "no extractable body text", not a real hash.
        // Comparing these produces a distance-0 match against every other
        // text-less page - an O(k^2) blow-up with no analytical value. Thin /
        // empty-content pages are already surfaced by the Issues tab.
        if new_page.content_fingerprint == 0 {
            return;
        }

        if self.page_summaries.len() >= DUP_SCAN_MAX_PAGES {
            return;
        }

        if self.duplicate_pairs.len() >= DUP_PAIRS_MAX {
            return;
        }

        for existing in &self.page_summaries {
            if existing.content_fingerprint == 0 {
                continue;
            }
            let distance =
                hamming_distance(existing.content_fingerprint, new_page.content_fingerprint);
            if distance <= NEAR_DUPLICATE_THRESHOLD {
                self.duplicate_pairs.push(DuplicatePair {
                    id_a: existing.id,
                    id_b: new_page.id,
                    distance,
                });
                if self.duplicate_pairs.len() >= DUP_PAIRS_MAX {
                    break;
                }
            }
        }
    }
}
