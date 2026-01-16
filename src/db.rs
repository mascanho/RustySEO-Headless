use crate::crawler::PageData;
use directories::ProjectDirs;
use rusqlite::{Connection, Result as SqliteResult, params};
use std::fs;
use std::path::PathBuf;

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

    // Initialize SQLite database
    let db_path = get_db_path();
    let conn = Connection::open(&db_path).expect("Failed to open database");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pages (
            id INTEGER PRIMARY KEY,
            data TEXT NOT NULL
        )",
        [],
    )
    .expect("Failed to create pages table");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS crawl_sessions (
            id INTEGER PRIMARY KEY,
            start_time TEXT NOT NULL,
            end_time TEXT,
            total_pages INTEGER DEFAULT 0
        )",
        [],
    )
    .expect("Failed to create crawl_sessions table");
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

fn get_db_path() -> PathBuf {
    ProjectDirs::from("", "", "rustyseo")
        .expect("Could not determine project directories")
        .data_dir()
        .join("rustyseo.db")
}

pub fn save_page_data(page_data: &PageData) -> SqliteResult<()> {
    let conn = Connection::open(get_db_path())?;
    let data = serde_json::to_string(page_data).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    conn.execute(
        "INSERT OR REPLACE INTO pages (id, data) VALUES (?1, ?2)",
        params![page_data.id, data],
    )?;
    Ok(())
}

pub fn load_page_data(id: usize) -> Option<PageData> {
    let conn = Connection::open(get_db_path()).ok()?;
    let data: String = conn
        .query_row("SELECT data FROM pages WHERE id = ?1", params![id], |row| {
            row.get(0)
        })
        .ok()?;
    serde_json::from_str(&data).ok()
}

pub fn load_all_page_data() -> Vec<PageData> {
    let conn = Connection::open(get_db_path()).unwrap();
    let mut stmt = conn.prepare("SELECT data FROM pages ORDER BY id").unwrap();
    let page_iter = stmt
        .query_map([], |row| {
            let data: String = row.get(0)?;
            let page_data: PageData = serde_json::from_str(&data).unwrap();
            Ok(page_data)
        })
        .unwrap();
    page_iter.filter_map(|p| p.ok()).collect()
}

pub fn clear_page_data() -> SqliteResult<()> {
    let conn = Connection::open(get_db_path())?;
    conn.execute("DELETE FROM pages", [])?;
    Ok(())
}

pub fn get_page_count() -> usize {
    let conn = Connection::open(get_db_path()).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM pages", [], |row| row.get(0))
        .unwrap_or(0);
    count as usize
}
