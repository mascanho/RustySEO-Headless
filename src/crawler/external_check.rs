use reqwest::Client;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio::time::Duration;

const MAX_CONCURRENT_CHECKS: usize = 20;
const CHECK_TIMEOUT_SECS: u64 = 10;

/// Spawns background HEAD (falling back to GET) requests for each external URL
/// and streams back (url, status) pairs as they complete. Opt-in via
/// `settings.crawler.check_external_links`, since most crawls don't need every
/// off-site destination verified and it can add many extra requests.
pub fn spawn_external_link_check(urls: Vec<String>, user_agent: String) -> mpsc::Receiver<(String, String)> {
    let (tx, rx) = mpsc::channel(256);

    tokio::spawn(async move {
        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(CHECK_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| Client::new());

        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_CHECKS));
        let mut handles = Vec::with_capacity(urls.len());

        for url in urls {
            let client = client.clone();
            let tx = tx.clone();
            let semaphore = semaphore.clone();

            handles.push(tokio::spawn(async move {
                let _permit = semaphore.acquire().await;
                let status = check_single_url(&client, &url).await;
                let _ = tx.send((url, status)).await;
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }
    });

    rx
}

async fn check_single_url(client: &Client, url: &str) -> String {
    match client.head(url).send().await {
        Ok(res) => format_status(res.status()),
        Err(_) => match client.get(url).send().await {
            Ok(res) => format_status(res.status()),
            Err(e) => format!("Error: {}", describe_error(&e)),
        },
    }
}

fn format_status(status: reqwest::StatusCode) -> String {
    format!(
        "{} {}",
        status.as_u16(),
        status.canonical_reason().unwrap_or("")
    )
}

fn describe_error(e: &reqwest::Error) -> &'static str {
    if e.is_timeout() {
        "Timeout"
    } else if e.is_connect() {
        "Connection failed"
    } else {
        "Failed"
    }
}
