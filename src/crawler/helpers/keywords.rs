use scraper::Html;
use std::collections::HashMap;

pub fn extract_keywords(html: &Html) -> Vec<String> {
    // 1. Get all text from body
    let body_text = html
        .select(&scraper::Selector::parse("body").unwrap())
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or_default();

    // 2. Tokenize and normalize
    let words = body_text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 2) // Skip very short words
        .map(|s| s.to_lowercase());

    // 3. Simple blocklist of common English stop words
    let stop_words = [
        "the", "and", "for", "that", "with", "this", "from", "your", "have", "will", "been", "they",
        "more", "when", "into", "their", "there", "what", "which", "some", "them", "then", "just",
        "than", "were", "well", "only", "about", "could", "also", "would", "very", "every", "many",
        "does", "ever", "most", "even", "such", "than", "here", "there", "where", "when", "look",
        "they", "their", "them", "from", "each", "used", "your", "much", "time", "back", "true",
        "work", "take", "name", "good", "used", "made", "both", "once", "still", "last", "long",
        "find", "down", "come", "than", "away", "find", "come", "than", "down", "once", "both",
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
        .map(|(word, _)| word)
        .collect()
}
