use scraper::Html;
use std::collections::HashMap;

use crate::crawler::helpers::keywords::extract_body_text;
use crate::models::NgramData;

/// Extracts word n-grams (contiguous 1-4 word phrases) from a page's body text.
///
/// Reuses the same body-text tokenization as `extract_keywords`, so this adds
/// only a few cheap sliding-window passes over an already-tokenized word list -
/// no extra HTML traversal. Output is capped to the top 15 phrases per length,
/// keeping per-page memory bounded regardless of page size.
pub fn extract_ngrams(html: &Html) -> NgramData {
    let body_text = extract_body_text(html);
    let words: Vec<String> = body_text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect();

    let mut data = NgramData::default();
    for n in 1..=4usize {
        if words.len() < n {
            continue;
        }
        let mut counts: HashMap<String, usize> = HashMap::new();
        for window in words.windows(n) {
            *counts.entry(window.join(" ")).or_insert(0) += 1;
        }

        let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        sorted.truncate(15);

        match n {
            1 => data.unigrams = sorted,
            2 => data.bigrams = sorted,
            3 => data.trigrams = sorted,
            _ => data.quadgrams = sorted,
        }
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ngrams() {
        let html = Html::parse_document(
            "<html><body><p>zebra hops along zebra hops along quickly</p></body></html>",
        );
        let ngrams = extract_ngrams(&html);

        // Several phrases tie for the top count here, so assert the count
        // (deterministic) rather than which tied phrase sorts first.
        assert_eq!(ngrams.unigrams[0].1, 2);
        assert_eq!(ngrams.bigrams[0].1, 2);
        assert_eq!(ngrams.trigrams[0].1, 2);
        assert!(ngrams.quadgrams.iter().all(|(_, c)| *c >= 1));

        let zebra_hops = ngrams
            .bigrams
            .iter()
            .find(|(p, _)| p == "zebra hops")
            .expect("bigram 'zebra hops' should be present");
        assert_eq!(zebra_hops.1, 2);
    }

    #[test]
    fn test_extract_ngrams_short_page() {
        let html = Html::parse_document("<html><body><p>hi</p></body></html>");
        let ngrams = extract_ngrams(&html);

        assert_eq!(ngrams.unigrams, vec![("hi".to_string(), 1)]);
        assert!(ngrams.bigrams.is_empty());
        assert!(ngrams.trigrams.is_empty());
        assert!(ngrams.quadgrams.is_empty());
    }
}
