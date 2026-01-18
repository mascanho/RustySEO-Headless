use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::LazyLock;

static BODY_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("body").unwrap());

pub fn extract_keywords(html: &Html) -> Vec<String> {
    // 1. Get text from semantically relevant elements while avoiding duplication
    // We'll use a blacklist of tags to skip instead of a whitelist of tags to include
    // to ensure we get all text nodes in the body that aren't code/metadata.
    let mut body_text = String::new();

    if let Some(body) = html.select(&BODY_SELECTOR).next() {
        for node in body.descendants() {
            if let Some(text) = node.value().as_text() {
                // Check ancestors to avoid script/style content
                let mut is_blacklisted = false;
                let mut parent = node.parent();
                while let Some(p) = parent {
                    if let Some(parent_elem) = p.value().as_element() {
                        let name = parent_elem.name();
                        if name == "script"
                            || name == "style"
                            || name == "noscript"
                            || name == "svg"
                            || name == "canvas"
                        {
                            is_blacklisted = true;
                            break;
                        }
                    }
                    parent = p.parent();
                }

                if !is_blacklisted {
                    body_text.push_str(text);
                    body_text.push(' ');
                }
            }
        }
    }

    // 2. Tokenize and normalize
    let words = body_text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 2) // Skip very short words
        .map(|s| s.to_lowercase());

    // 3. Simple blocklist of common English stop words
    let stop_words = [
        "the", "and", "for", "that", "with", "this", "from", "your", "have", "will", "been",
        "they", "more", "when", "into", "their", "there", "what", "which", "some", "them", "then",
        "just", "than", "were", "well", "only", "about", "could", "also", "would", "very", "every",
        "many", "does", "ever", "most", "even", "such", "than", "here", "there", "where", "when",
        "look", "they", "their", "them", "from", "each", "used", "your", "much", "time", "back",
        "true", "work", "take", "name", "good", "used", "made", "both", "once", "still", "last",
        "long", "find", "down", "come", "than", "away", "find", "come", "than", "down", "once",
        "both",
    ];

    // 4. Count frequencies
    let mut counts = HashMap::new();
    for word in words {
        if !stop_words.contains(&word.as_str()) {
            *counts.entry(word).or_insert(0) += 1;
        }
    }

    // 5. Sort by frequency and take top 10
    let mut sorted_counts: Vec<(String, usize)> = counts.into_iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(&a.1));

    sorted_counts
        .into_iter()
        .take(10)
        .map(|(word, count)| format!("({}) {}", count, word))
        .collect()
}
