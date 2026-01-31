use std::error::Error;

pub async fn extract_robots_blocked_urls(url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    let robots_txt = response.text().await?;
    let mut blocked_urls = Vec::new();

    for line in robots_txt.lines() {
        let trimmed_line = line.trim();
        
        // Handle different formats of Disallow directives
        if trimmed_line.to_lowercase().starts_with("disallow:") {
            // Extract the path after "Disallow:"
            let parts: Vec<&str> = trimmed_line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let path = parts[1].trim();
                
                // Only add meaningful disallow paths
                if !path.is_empty() && path != "/" {
                    // Prepend the domain if path is relative
                    let full_url = if path.starts_with('/') {
                        format!("{}{}", url.trim_end_matches("/robots.txt"), path)
                    } else {
                        path.to_string()
                    };
                    blocked_urls.push(full_url);
                } else if path == "/" {
                    // Root disallow - everything is blocked
                    blocked_urls.push("All pages blocked (Disallow: /)".to_string());
                }
            }
        }
    }

    Ok(blocked_urls)
}
