//! Link Score: an internal PageRank-style metric on a 1-100 logarithmic scale,
//! computed as part of Crawl Analysis once a crawl finishes.
//!
//! Eligible URLs are internal, crawled, not a redirect, and not canonicalised
//! away, and must be linked to by an AHREF or be the target of a redirect or
//! canonical. Score flows between eligible URLs only, following AHREF links
//! (nofollow links "evaporate" score: counted in the outlink total but pass
//! nothing forward), with redirects/canonicals bypassing straight to their
//! target.

use crate::models::App;
use std::collections::{HashMap, HashSet};

const DAMPING: f64 = 0.85;
const ITERATIONS: usize = 10;

/// Follows redirect/canonical chains to the ultimate target of a URL.
fn resolve_url(
    start: &str,
    redirect_map: &HashMap<String, String>,
    canonical_map: &HashMap<String, String>,
) -> String {
    let mut current = start.to_string();
    let mut seen = HashSet::new();
    loop {
        if !seen.insert(current.clone()) {
            break; // cycle guard
        }
        if let Some(target) = redirect_map.get(&current) {
            current = target.clone();
            continue;
        }
        if let Some(target) = canonical_map.get(&current) {
            current = target.clone();
            continue;
        }
        break;
    }
    current
}

struct Edge {
    from: String,
    to: String,
    nofollow: bool,
}

impl App {
    /// Runs Crawl Analysis' Link Score algorithm over the currently crawled data
    /// and stores the resulting 1-100 scores in `self.link_scores`, keyed by URL.
    pub fn compute_link_scores(&mut self) {
        self.link_scores.clear();

        if self.page_summaries.is_empty() {
            return;
        }

        // Pages the crawler actually resolved to (final, post-redirect identity).
        let crawled_urls: Vec<String> = self.page_summaries.iter().map(|p| p.url.clone()).collect();
        let is_redirect_status: HashMap<&str, bool> = self
            .page_summaries
            .iter()
            .map(|p| (p.url.as_str(), p.status.trim().starts_with('3')))
            .collect();

        // Deduped, non-self-referencing AHREF edges, with destinations resolved
        // through redirects/canonicals to their ultimate target.
        let mut seen_pairs: HashSet<(String, String)> = HashSet::new();
        let mut raw_edges: Vec<Edge> = Vec::new();
        let mut linked_to: HashSet<String> = HashSet::new();

        for link in &self.internal_table_data {
            let from = link.source.clone();
            let to = resolve_url(&link.destination, &self.redirect_map, &self.canonical_map);
            if from == to {
                continue; // non self-referencing
            }
            if !seen_pairs.insert((from.clone(), to.clone())) {
                continue; // unique source/destination pair only
            }
            linked_to.insert(to.clone());
            raw_edges.push(Edge {
                from,
                nofollow: link.rel.to_lowercase().contains("nofollow"),
                to,
            });
        }

        // Redirect and canonical targets are "linked to" even without a direct AHREF.
        for target in self.redirect_map.values() {
            linked_to.insert(resolve_url(target, &self.redirect_map, &self.canonical_map));
        }
        for target in self.canonical_map.values() {
            linked_to.insert(resolve_url(target, &self.redirect_map, &self.canonical_map));
        }

        // Eligible = internal (all crawled URLs are), not a redirect, not
        // canonicalised away, and linked to by something.
        let eligible: HashSet<String> = crawled_urls
            .into_iter()
            .filter(|u| {
                !*is_redirect_status.get(u.as_str()).unwrap_or(&false)
                    && !self.canonical_map.contains_key(u)
                    && linked_to.contains(u)
            })
            .collect();

        let n = eligible.len();
        if n == 0 {
            return;
        }

        // Keep only edges within the eligible subgraph; a source that isn't
        // eligible has no Link Score of its own to distribute.
        let mut outlink_count: HashMap<String, usize> = HashMap::new();
        let mut inbound: HashMap<String, Vec<(String, bool)>> = HashMap::new();
        for edge in &raw_edges {
            if !eligible.contains(&edge.from) || !eligible.contains(&edge.to) {
                continue;
            }
            *outlink_count.entry(edge.from.clone()).or_insert(0) += 1;
            inbound
                .entry(edge.to.clone())
                .or_default()
                .push((edge.from.clone(), edge.nofollow));
        }

        // Iteratively flow Link Score across the eligible subgraph.
        let initial = 1.0 / n as f64;
        let mut scores: HashMap<String, f64> = eligible.iter().map(|u| (u.clone(), initial)).collect();

        for _ in 0..ITERATIONS {
            let mut next_scores = HashMap::with_capacity(n);
            for url in &eligible {
                let mut total_in = 0.0;
                if let Some(sources) = inbound.get(url) {
                    for (source, nofollow) in sources {
                        if *nofollow {
                            continue; // evaporates: counted in outlinks, passes nothing
                        }
                        let source_score = scores.get(source).copied().unwrap_or(0.0);
                        let out_count = *outlink_count.get(source).unwrap_or(&1) as f64;
                        total_in += source_score / out_count;
                    }
                }
                let new_score = ((1.0 - DAMPING) / n as f64) + (DAMPING * total_in);
                next_scores.insert(url.clone(), new_score);
            }
            scores = next_scores;
        }

        // Map raw scores onto a 1-100 logarithmic scale: highest raw score -> 100,
        // lowest -> 1, everything else scaled in between.
        let mut log_scores: HashMap<String, f64> = HashMap::with_capacity(n);
        let mut min_log = f64::MAX;
        let mut max_log = f64::MIN;
        for (url, score) in &scores {
            let l = score.max(f64::MIN_POSITIVE).ln();
            min_log = min_log.min(l);
            max_log = max_log.max(l);
            log_scores.insert(url.clone(), l);
        }

        let range = max_log - min_log;
        let final_scores: HashMap<String, u32> = log_scores
            .into_iter()
            .map(|(url, l)| {
                let normalised = if range > 0.0 { (l - min_log) / range } else { 1.0 };
                let value = 1 + (normalised * 99.0).round() as u32;
                (url, value.min(100))
            })
            .collect();

        self.link_scores = final_scores;
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{App, InternalLink, PageSummary};

    fn page(url: &str, status: &str) -> PageSummary {
        PageSummary {
            id: 0,
            url: url.to_string(),
            title: String::new(),
            title_len: 0,
            description: String::new(),
            description_len: 0,
            status: status.to_string(),
            h1_len: 0,
            h1_count: 0,
            h2_count: 0,
            h3_count: 0,
            h4_count: 0,
            h5_count: 0,
            h6_count: 0,
            has_schema: false,
            schema_count: 0,
            size: 0,
            word_count: 0,
            internal_link_count: 0,
            external_link_count: 0,
            images_count: 0,
            images_missing_alt: 0,
            is_canonical: false,
            has_png_jpg: false,
            mobile: true,
            indexability: "indexable".to_string(),
            language: "en".to_string(),
            cwv_performance_desktop: None,
            cwv_performance_mobile: None,
            has_generic_anchors: false,
            has_noindex_header: false,
            canonical_target: None,
            canonical_count: 0,
        }
    }

    fn link(source: &str, destination: &str, rel: &str) -> InternalLink {
        InternalLink {
            id: 0,
            source: source.to_string(),
            destination: destination.to_string(),
            anchor: String::new(),
            rel: rel.to_string(),
        }
    }

    #[test]
    fn hub_page_scores_highest_and_nofollow_evaporates() {
        let mut app = App::default();

        // home -> a, home -> b ; a -> home ; b -> home (nofollow, evaporates)
        app.page_summaries = vec![page("home", "200"), page("a", "200"), page("b", "200")];
        app.internal_table_data = vec![
            link("home", "a", ""),
            link("home", "b", ""),
            link("a", "home", ""),
            link("b", "home", "nofollow"),
        ];

        app.compute_link_scores();

        assert_eq!(app.link_scores.len(), 3);
        for score in app.link_scores.values() {
            assert!(*score >= 1 && *score <= 100);
        }
        // Home receives from both a and b (evaporated); should be top scored.
        let home = app.link_scores["home"];
        let a = app.link_scores["a"];
        let b = app.link_scores["b"];
        assert!(home >= a && home >= b);
    }

    #[test]
    fn redirect_and_canonical_targets_bypass_source() {
        let mut app = App::default();

        // "old" redirects to "new"; "dup" canonicalises to "new".
        app.page_summaries = vec![page("new", "200")];
        app.internal_table_data = vec![
            link("elsewhere_source", "old", ""),
            link("elsewhere_source", "dup", ""),
        ];
        app.page_summaries.push(page("elsewhere_source", "200"));
        app.redirect_map.insert("old".to_string(), "new".to_string());
        app.canonical_map.insert("dup".to_string(), "new".to_string());

        app.compute_link_scores();

        // "new" should be eligible (linked to via resolved redirect/canonical destinations)
        // while "old"/"dup" never appear as crawled pages at all.
        assert!(app.link_scores.contains_key("new"));
        assert!(!app.link_scores.contains_key("old"));
        assert!(!app.link_scores.contains_key("dup"));
    }

    #[test]
    fn empty_crawl_produces_no_scores() {
        let mut app = App::default();
        app.compute_link_scores();
        assert!(app.link_scores.is_empty());
    }
}
