use crate::models::AppSettings;
use anyhow::Result;

pub fn read_settings() -> Result<AppSettings, Box<dyn std::error::Error>> {
    let app_settings = AppSettings::load();
    Ok(app_settings)
}

// GETS THE RECENT CRAWLS FROM THE FILE
pub fn recent_crawls() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let project_dirs = directories::ProjectDirs::from("", "", "rustyseo").unwrap();
    let recent_crawls_path = project_dirs.data_dir().join("recent-crawls.json");
    if recent_crawls_path.exists() {
        let content = std::fs::read_to_string(recent_crawls_path)?;
        let recent_crawls: Vec<String> = serde_json::from_str(&content)?;
        Ok(recent_crawls)
    } else {
        Ok(Vec::new())
    }
}
