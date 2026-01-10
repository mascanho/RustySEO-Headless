use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

fn get_bookmarks_path() -> PathBuf {
    ProjectDirs::from("", "", "rustyseo")
        .expect("Could not determine project directories")
        .data_dir()
        .join("bookmarks.json")
}

pub fn init_db() {
    // Ensure bookmarks.json exists
    let bookmarks_path = get_bookmarks_path();
    if !bookmarks_path.exists() {
        fs::write(&bookmarks_path, "[]").expect("Failed to create bookmarks.json");
    }
}

pub fn load_bookmarks() -> Vec<String> {
    let bookmarks_path = get_bookmarks_path();
    if let Ok(content) = fs::read_to_string(bookmarks_path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    }
}

pub fn add_bookmark(url: &str) {
    let bookmarks_path = get_bookmarks_path();
    let mut bookmarks = load_bookmarks();
    if !bookmarks.contains(&url.to_string()) {
        bookmarks.push(url.to_string());
        if let Ok(json) = serde_json::to_string(&bookmarks) {
            let _ = fs::write(&bookmarks_path, json);
        }
    }
}

pub fn remove_bookmark(url: &str) {
    let bookmarks_path = get_bookmarks_path();
    let mut bookmarks = load_bookmarks();
    if let Some(pos) = bookmarks.iter().position(|r| r == url) {
        bookmarks.remove(pos);
        if let Ok(json) = serde_json::to_string(&bookmarks) {
            let _ = fs::write(&bookmarks_path, json);
        }
    }
}
