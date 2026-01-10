use std::fs;
use std::path::Path;

pub fn init_db() {
    // Ensure bookmarks.json exists
    if !Path::new("bookmarks.json").exists() {
        fs::write("bookmarks.json", "[]").expect("Failed to create bookmarks.json");
    }
}

pub fn load_bookmarks() -> Vec<String> {
    if let Ok(content) = fs::read_to_string("bookmarks.json") {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    }
}

pub fn add_bookmark(url: &str) {
    let mut bookmarks = load_bookmarks();
    if !bookmarks.contains(&url.to_string()) {
        bookmarks.push(url.to_string());
        if let Ok(json) = serde_json::to_string(&bookmarks) {
            let _ = fs::write("bookmarks.json", json);
        }
    }
}

pub fn remove_bookmark(url: &str) {
    let mut bookmarks = load_bookmarks();
    if let Some(pos) = bookmarks.iter().position(|r| r == url) {
        bookmarks.remove(pos);
        if let Ok(json) = serde_json::to_string(&bookmarks) {
            let _ = fs::write("bookmarks.json", json);
        }
    }
}

