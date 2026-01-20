use crate::models::AppSettings;

pub async fn ask(input: &str, settings: &AppSettings) -> Result<String, String> {
    Ok("Hello, world!".to_string())
}
