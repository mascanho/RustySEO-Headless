use scraper::Html;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::crawler::helpers::keywords::extract_body_text;

/// Hamming distance at or below this (out of 64 bits) flags two pages as
/// near-duplicate content.
///
/// Two *unrelated* 64-bit fingerprints land at distance 32 on average (std
/// dev ~4, from the binomial bit-flip distribution), so anything comfortably
/// below that is not coincidence. 10 sits ~5.5 std devs out - false positives
/// from pure chance are essentially impossible - while being far more
/// forgiving than the textbook "3 bits" figure, which assumes long documents
/// (the original SimHash paper's target: multi-KB web pages). On real
/// page-length text (a few hundred to ~1500 words), even a single differing
/// paragraph - not a full rewrite, just genuinely different content - easily
/// pushes distance past 3, which is why that threshold was under-flagging
/// obvious near-duplicates.
pub const NEAR_DUPLICATE_THRESHOLD: u32 = 10;

/// 64-bit SimHash fingerprint of a page's visible body text, built from
/// word-bigram shingles (falls back to unigrams for very short pages).
/// Near-duplicate pages - e.g. the same template with a handful of words
/// swapped - produce fingerprints that differ in only a few bits, so
/// duplicates can be found by Hamming distance instead of requiring an exact
/// text match.
pub fn compute_fingerprint(html: &Html) -> u64 {
    let body_text = extract_body_text(html);
    let words: Vec<String> = body_text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect();

    if words.is_empty() {
        return 0;
    }

    let shingle_size = 2.min(words.len());
    let mut weights = [0i32; 64];
    for window in words.windows(shingle_size) {
        let shingle = window.join(" ");
        let mut hasher = DefaultHasher::new();
        shingle.hash(&mut hasher);
        let hash = hasher.finish();
        for (bit, weight) in weights.iter_mut().enumerate() {
            if (hash >> bit) & 1 == 1 {
                *weight += 1;
            } else {
                *weight -= 1;
            }
        }
    }

    let mut fingerprint: u64 = 0;
    for (bit, weight) in weights.iter().enumerate() {
        if *weight > 0 {
            fingerprint |= 1 << bit;
        }
    }
    fingerprint
}

pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_text_has_zero_distance() {
        let html_a = Html::parse_document(
            "<html><body><p>the quick brown fox jumps over the lazy dog</p></body></html>",
        );
        let html_b = Html::parse_document(
            "<html><body><p>the quick brown fox jumps over the lazy dog</p></body></html>",
        );
        assert_eq!(
            hamming_distance(compute_fingerprint(&html_a), compute_fingerprint(&html_b)),
            0
        );
    }

    #[test]
    fn very_different_text_has_high_distance() {
        let html_a = Html::parse_document(
            "<html><body><p>the quick brown fox jumps over the lazy dog repeatedly every single morning without fail</p></body></html>",
        );
        let html_b = Html::parse_document(
            "<html><body><p>stock markets fell sharply today amid fears of rising interest rates across the globe</p></body></html>",
        );
        assert!(hamming_distance(compute_fingerprint(&html_a), compute_fingerprint(&html_b)) > NEAR_DUPLICATE_THRESHOLD);
    }

    #[test]
    fn near_duplicate_text_has_low_distance() {
        // Two words swapped ("COLOR") in a shared boilerplate template - the
        // shape of a real templated product-page duplicate. SimHash's
        // near-duplicate guarantee is statistical: it needs enough shared
        // shingles for a handful of differing ones to average out, so this
        // uses a realistic full-page word count (boilerplate + nav/footer
        // text easily reaches this on a real site) rather than one sentence,
        // where a single word change dominates the vote and the guarantee
        // doesn't hold.
        let changed = "Buy the COLOR widget today for only twenty nine dollars with free \
            shipping. Every COLOR widget ships within one business day.";
        let shared_boilerplate = "This site uses cookies to improve your browsing experience and \
            provide personalized content and advertisements. By continuing to browse this website \
            you agree to our use of cookies as described in our privacy policy and terms of service. \
            Our customer support team is available around the clock to help answer any questions you \
            might have about your order or account. We offer a wide selection of products across many \
            categories including electronics home goods clothing and outdoor equipment. Sign up for \
            our newsletter today to receive exclusive discounts and be the first to hear about new \
            arrivals and special promotions. Free returns are available within thirty days of purchase \
            for any item that does not meet your expectations no questions asked. Follow us on social \
            media for daily updates styling tips and behind the scenes looks at our design process."
            .repeat(4);

        let template = |color: &str| {
            format!(
                "<html><body><p>{} {}</p></body></html>",
                changed.replace("COLOR", color),
                shared_boilerplate
            )
        };
        let html_a = Html::parse_document(&template("red"));
        let html_b = Html::parse_document(&template("blue"));
        assert!(hamming_distance(compute_fingerprint(&html_a), compute_fingerprint(&html_b)) <= NEAR_DUPLICATE_THRESHOLD);
    }
}
