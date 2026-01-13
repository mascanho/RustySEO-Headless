use crate::models::AppSettings;
use anyhow::Result;

pub fn read_settings() -> Result<AppSettings, Box<dyn std::error::Error>> {
    let app_settings = AppSettings::load();
    Ok(app_settings)
}
