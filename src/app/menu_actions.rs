//! Real implementations backing the Actions Menu entries that used to be log-only
//! stubs: on-page SEO scoring, screenshot capture, and CSV export.

use crate::models::{SeoScoreBreakdown, SeoScoreFactor};
use headless_chrome::{protocol::cdp::Page::CaptureScreenshotFormatOption, Browser, LaunchOptions};

// Indices into a `table_data` row, matching the order pushed in `App::on_tick`
// (src/app/actions.rs). Rows are always built with this exact shape.
const COL_TITLE: usize = 2;
const COL_TITLE_LEN: usize = 3;
const COL_H1: usize = 4;
const COL_DESCRIPTION: usize = 6;
const COL_DESCRIPTION_LEN: usize = 7;
const COL_STATUS: usize = 10;
const COL_INDEXABILITY: usize = 13;
const COL_WORD_COUNT: usize = 18;

fn get(row: &[String], idx: usize) -> &str {
    row.get(idx).map(|s| s.as_str()).unwrap_or("")
}

/// Build a composite 0-100 on-page SEO score for a single crawled row, mirroring
/// the same thresholds already used to color-code the Overview table (title <=60
/// chars, description <=160 chars, indexable, etc).
pub fn calculate(url: &str, row: &[String], link_score: Option<u32>) -> SeoScoreBreakdown {
    let mut score: u32 = 0;
    let mut factors = Vec::new();

    // Status code - 20 pts for 2xx, partial credit for a redirect, 0 otherwise.
    let status = get(row, COL_STATUS);
    let (status_pts, status_pass, status_detail) = if status.contains("200") {
        (20, true, format!("{} OK", status))
    } else if status.contains("301") || status.contains("302") {
        (10, false, format!("{} redirect", status))
    } else {
        (0, false, format!("{} error", status))
    };
    score += status_pts;
    factors.push(SeoScoreFactor {
        label: "Status Code".to_string(),
        passed: status_pass,
        detail: status_detail,
    });

    // Title - present and within 10-60 chars.
    let title = get(row, COL_TITLE);
    let title_len: usize = get(row, COL_TITLE_LEN).parse().unwrap_or(0);
    let (title_pts, title_pass, title_detail) = if title.trim().is_empty() {
        (0, false, "missing".to_string())
    } else if (10..=60).contains(&title_len) {
        (15, true, format!("{} chars", title_len))
    } else {
        (7, false, format!("{} chars (ideal 10-60)", title_len))
    };
    score += title_pts;
    factors.push(SeoScoreFactor {
        label: "Title".to_string(),
        passed: title_pass,
        detail: title_detail,
    });

    // Meta description - present and within 50-160 chars.
    let description = get(row, COL_DESCRIPTION);
    let description_len: usize = get(row, COL_DESCRIPTION_LEN).parse().unwrap_or(0);
    let (desc_pts, desc_pass, desc_detail) = if description.trim().is_empty() {
        (0, false, "missing".to_string())
    } else if (50..=160).contains(&description_len) {
        (15, true, format!("{} chars", description_len))
    } else {
        (7, false, format!("{} chars (ideal 50-160)", description_len))
    };
    score += desc_pts;
    factors.push(SeoScoreFactor {
        label: "Meta Description".to_string(),
        passed: desc_pass,
        detail: desc_detail,
    });

    // H1 - present.
    let h1 = get(row, COL_H1);
    let (h1_pts, h1_pass, h1_detail) = if h1.trim().is_empty() {
        (0, false, "missing".to_string())
    } else {
        (15, true, "present".to_string())
    };
    score += h1_pts;
    factors.push(SeoScoreFactor {
        label: "H1".to_string(),
        passed: h1_pass,
        detail: h1_detail,
    });

    // Indexability.
    let indexability = get(row, COL_INDEXABILITY);
    let (idx_pts, idx_pass, idx_detail) = if indexability.contains("noindex") {
        (0, false, "noindex".to_string())
    } else {
        (15, true, "indexable".to_string())
    };
    score += idx_pts;
    factors.push(SeoScoreFactor {
        label: "Indexability".to_string(),
        passed: idx_pass,
        detail: idx_detail,
    });

    // Content depth.
    let word_count: usize = get(row, COL_WORD_COUNT).parse().unwrap_or(0);
    let (wc_pts, wc_pass, wc_detail) = if word_count >= 300 {
        (10, true, format!("{} words", word_count))
    } else if word_count >= 150 {
        (5, false, format!("{} words (thin)", word_count))
    } else {
        (0, false, format!("{} words (thin)", word_count))
    };
    score += wc_pts;
    factors.push(SeoScoreFactor {
        label: "Content Length".to_string(),
        passed: wc_pass,
        detail: wc_detail,
    });

    // Inbound link strength, if it's been computed yet.
    let (link_pts, link_pass, link_detail) = match link_score {
        Some(s) => (
            ((s.min(100) as f64 / 100.0) * 10.0).round() as u32,
            s >= 40,
            format!("{}/100", s),
        ),
        None => (0, false, "not analysed yet".to_string()),
    };
    score += link_pts;
    factors.push(SeoScoreFactor {
        label: "Link Score".to_string(),
        passed: link_pass,
        detail: link_detail,
    });

    SeoScoreBreakdown {
        url: url.to_string(),
        score: score.min(100),
        factors,
    }
}

fn screenshots_dir() -> Result<std::path::PathBuf, String> {
    let dirs = directories::ProjectDirs::from("", "", "rustyseo")
        .ok_or("Could not determine project directories")?;
    let dir = dirs.data_dir().join("screenshots");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn sanitize_filename(url: &str) -> String {
    let sanitized: String = url
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    let trimmed: String = sanitized.chars().take(80).collect();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    format!("{}_{}.png", trimmed, timestamp)
}

/// Launch a fresh headless browser, navigate to `url`, and save a full-page PNG
/// screenshot. Runs synchronously - callers should invoke this via
/// `spawn_blocking` since it drives the CDP protocol with blocking calls.
pub fn capture_screenshot(url: &str) -> Result<String, String> {
    let dir = screenshots_dir()?;
    let filename = sanitize_filename(url);
    let path = dir.join(&filename);

    let options = LaunchOptions {
        headless: true,
        ..Default::default()
    };
    let browser = Browser::new(options).map_err(|e| format!("Failed to launch browser: {}", e))?;
    let tab = browser
        .new_tab()
        .map_err(|e| format!("Tab creation failed: {}", e))?;
    let png_data = tab
        .navigate_to(url)
        .map_err(|e| format!("Navigation failed: {}", e))?
        .wait_until_navigated()
        .map_err(|e| format!("Wait failed: {}", e))?
        .capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
        .map_err(|e| format!("Capture failed: {}", e))?;

    std::fs::write(&path, png_data).map_err(|e| format!("Failed to write screenshot: {}", e))?;
    Ok(path.display().to_string())
}

const OVERVIEW_CSV_HEADER: [&str; 45] = [
    "ID",
    "URL",
    "Title",
    "Title Length",
    "H1",
    "H1 Length",
    "Description",
    "Description Length",
    "H2",
    "H2 Length",
    "Status",
    "Mobile Friendly",
    "Language",
    "Indexability",
    "Anchor Links",
    "Content Type",
    "Canonicals",
    "Size (bytes)",
    "Word Count",
    "CSS Total Size",
    "External CSS Count",
    "Inline CSS Size",
    "First CSS URL",
    "CWV Desktop Performance",
    "CWV Desktop FCP",
    "CWV Desktop LCP",
    "CWV Desktop CLS",
    "CWV Desktop TBT",
    "CWV Desktop Speed Index",
    "CWV Mobile Performance",
    "CWV Mobile FCP",
    "CWV Mobile LCP",
    "CWV Mobile CLS",
    "CWV Mobile TBT",
    "CWV Mobile Speed Index",
    "Keyword 1",
    "Keyword 2",
    "Keyword 3",
    "Keyword 4",
    "Keyword 5",
    "Keyword 6",
    "Keyword 7",
    "Keyword 8",
    "Keyword 9",
    "Keyword 10",
];

fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn exports_dir() -> Result<std::path::PathBuf, String> {
    let dirs = directories::ProjectDirs::from("", "", "rustyseo")
        .ok_or("Could not determine project directories")?;
    let dir = dirs.data_dir().join("exports");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Write every crawled row to a timestamped CSV file and return its path.
pub fn write_overview_csv(table_data: &[Vec<String>]) -> Result<String, String> {
    if table_data.is_empty() {
        return Err("No data to export yet".to_string());
    }

    let dir = exports_dir()?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let path = dir.join(format!("overview_{}.csv", timestamp));

    let mut content = String::new();
    content.push_str(
        &OVERVIEW_CSV_HEADER
            .iter()
            .map(|h| csv_escape(h))
            .collect::<Vec<_>>()
            .join(","),
    );
    content.push('\n');

    for row in table_data {
        content.push_str(
            &row.iter()
                .map(|field| csv_escape(field))
                .collect::<Vec<_>>()
                .join(","),
        );
        content.push('\n');
    }

    std::fs::write(&path, content).map_err(|e| format!("Failed to write CSV: {}", e))?;
    Ok(path.display().to_string())
}
