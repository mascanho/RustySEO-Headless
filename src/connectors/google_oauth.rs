use crate::models::GoogleOAuthTokens;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// All three Google-backed connectors (Search Console, GA4, Business
/// Profile) share one loopback port since only one OAuth flow runs at a
/// time - the user must register this exact redirect URI on their OAuth
/// "Desktop app" client in Google Cloud Console.
pub const REDIRECT_PORT: u16 = 8721;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoogleService {
    SearchConsole,
    Ga4,
    Gbp,
}

impl GoogleService {
    pub fn scope(self) -> &'static str {
        match self {
            GoogleService::SearchConsole => "https://www.googleapis.com/auth/webmasters.readonly",
            GoogleService::Ga4 => "https://www.googleapis.com/auth/analytics.readonly",
            GoogleService::Gbp => "https://www.googleapis.com/auth/business.manage",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            GoogleService::SearchConsole => "Search Console",
            GoogleService::Ga4 => "GA4",
            GoogleService::Gbp => "Business Profile",
        }
    }

    pub fn settings_key(self) -> &'static str {
        match self {
            GoogleService::SearchConsole => "search_console",
            GoogleService::Ga4 => "ga4",
            GoogleService::Gbp => "gbp",
        }
    }
}

fn redirect_uri() -> String {
    format!("http://127.0.0.1:{}/callback", REDIRECT_PORT)
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Runs one full "installed app" OAuth2 authorization-code flow: opens the
/// user's browser to Google's consent screen, listens on a loopback port for
/// the redirect, and exchanges the returned code for tokens.
///
/// This does its own waiting (binding the port, then blocking on `accept`),
/// so callers must run it inside `tokio::spawn` rather than awaiting it
/// directly from the render loop - the same "spawn + mpsc channel, poll on
/// tick" pattern already used for robots/sitemaps and screenshot fetches.
pub async fn run_flow(
    client_id: String,
    client_secret: String,
    service: GoogleService,
) -> Result<GoogleOAuthTokens, String> {
    if client_id.is_empty() || client_secret.is_empty() {
        return Err(format!(
            "Set connectors.{}.client_id/client_secret in cli-settings.toml first",
            service.settings_key()
        ));
    }

    let redirect_uri = redirect_uri();
    let mut auth_url = url::Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .map_err(|e| e.to_string())?;
    auth_url
        .query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", service.scope())
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");

    let listener = TcpListener::bind(("127.0.0.1", REDIRECT_PORT))
        .await
        .map_err(|e| format!("Could not open loopback port {}: {}", REDIRECT_PORT, e))?;

    crate::ui::modals::dashboard_menu::open_in_browser(auth_url.as_str());

    let code = accept_one_code(listener).await?;

    let client = reqwest::Client::new();
    exchange_code(&client, &client_id, &client_secret, &code, &redirect_uri).await
}

/// Accepts exactly one loopback connection, extracts `code`/`error` from the
/// redirected request's query string, and replies with a small confirmation
/// page so the browser tab doesn't hang.
async fn accept_one_code(listener: TcpListener) -> Result<String, String> {
    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|e| format!("Loopback accept failed: {}", e))?;

    let mut buf = [0u8; 8192];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| format!("Failed reading redirect request: {}", e))?;
    let request = String::from_utf8_lossy(&buf[..n]);

    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or("Malformed redirect request")?;

    let dummy_base = url::Url::parse("http://localhost").unwrap();
    let parsed = dummy_base
        .join(path)
        .map_err(|e| format!("Bad redirect path: {}", e))?;

    let code = parsed
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.into_owned());
    let error = parsed
        .query_pairs()
        .find(|(k, _)| k == "error")
        .map(|(_, v)| v.into_owned());

    let body = if code.is_some() {
        "<html><body><h2>RustySEO connected. You can close this tab.</h2></body></html>"
    } else {
        "<html><body><h2>RustySEO: connection failed or was cancelled.</h2></body></html>"
    };
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes()).await;

    if let Some(err) = error {
        return Err(format!("Google denied access: {}", err));
    }
    code.ok_or_else(|| "No authorization code in redirect".to_string())
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    expires_in: i64,
}

async fn exchange_code(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> Result<GoogleOAuthTokens, String> {
    let params = [
        ("code", code),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];
    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed: {}", text));
    }

    let parsed: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Could not parse token response: {}", e))?;

    Ok(GoogleOAuthTokens {
        access_token: parsed.access_token,
        // Google only returns a refresh_token on first consent for a given
        // client+account; the caller preserves the previous one if this is
        // empty on a reconnect.
        refresh_token: parsed.refresh_token.unwrap_or_default(),
        expires_at: now_unix() + parsed.expires_in,
    })
}

#[derive(Deserialize)]
struct RefreshResponse {
    access_token: String,
    expires_in: i64,
}

/// Ensures `tokens.access_token` is valid, refreshing it via the stored
/// refresh_token if it's missing or about to expire. Mutates `tokens` in
/// place; the caller is responsible for persisting the updated tokens.
pub async fn refresh_if_needed(
    client_id: &str,
    client_secret: &str,
    tokens: &mut GoogleOAuthTokens,
) -> Result<(), String> {
    if !tokens.access_token.is_empty() && tokens.expires_at > now_unix() + 30 {
        return Ok(());
    }
    if tokens.refresh_token.is_empty() {
        return Err("Not connected - press 'c' to connect".to_string());
    }

    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", tokens.refresh_token.as_str()),
        ("grant_type", "refresh_token"),
    ];
    let resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token refresh failed: {}", e))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed: {}", text));
    }

    let parsed: RefreshResponse = resp
        .json()
        .await
        .map_err(|e| format!("Could not parse refresh response: {}", e))?;

    tokens.access_token = parsed.access_token;
    tokens.expires_at = now_unix() + parsed.expires_in;
    Ok(())
}
