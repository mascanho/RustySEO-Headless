//! Background fetch/connect plumbing for the Data & Insights tab's seven
//! connectors. Follows the same "spawn + mpsc channel, poll on tick" pattern
//! already used for robots/sitemaps and screenshot fetches (see
//! `app::issues::spawn_robots_analysis` / `check_robots_results`) rather than
//! awaiting HTTP calls directly from the render loop.

use crate::connectors::google_oauth::GoogleService;
use crate::connectors::{ahrefs, bing, clarity, ga4, gbp, google_oauth, gsc, pagespeed};
use crate::models::App;
use tokio::sync::mpsc;

impl App {
    /// Dispatches 'c' (connect) for whichever connector sub-tab is active.
    /// Only GSC/GA4/GBP need this - the others authenticate with a static
    /// API key pasted into `cli-settings.toml`, so there's nothing to "connect".
    pub fn trigger_data_insights_connect(&mut self) {
        match self.data_insights_tab {
            0 => self.spawn_google_oauth(GoogleService::SearchConsole),
            1 => self.spawn_google_oauth(GoogleService::Ga4),
            5 => self.spawn_google_oauth(GoogleService::Gbp),
            _ => self.log(
                "This connector uses an API key from cli-settings.toml - nothing to connect."
                    .to_string(),
            ),
        }
    }

    /// Dispatches Enter (fetch/refresh) for whichever connector sub-tab is active.
    pub fn trigger_data_insights_fetch(&mut self) {
        match self.data_insights_tab {
            0 => self.spawn_gsc_fetch(),
            1 => self.spawn_ga4_fetch(),
            2 => self.spawn_clarity_fetch(),
            3 => self.spawn_bing_fetch(),
            4 => self.spawn_pagespeed_fetch(),
            5 => self.spawn_gbp_fetch(),
            _ => self.spawn_ahrefs_fetch(),
        }
    }

    // ---- Google OAuth (shared by GSC/GA4/GBP) ----

    pub fn spawn_google_oauth(&mut self, service: GoogleService) {
        if self.google_oauth_in_progress {
            self.log("An OAuth connection is already in progress.".to_string());
            return;
        }
        let Some(settings) = self.settings.clone() else {
            self.log("Settings not loaded yet.".to_string());
            return;
        };
        let (client_id, client_secret) = match service {
            GoogleService::SearchConsole => (
                settings.connectors.search_console.client_id.clone(),
                settings.connectors.search_console.client_secret.clone(),
            ),
            GoogleService::Ga4 => (
                settings.connectors.ga4.client_id.clone(),
                settings.connectors.ga4.client_secret.clone(),
            ),
            GoogleService::Gbp => (
                settings.connectors.gbp.client_id.clone(),
                settings.connectors.gbp.client_secret.clone(),
            ),
        };

        let (tx, rx) = mpsc::channel(1);
        self.google_oauth_receiver = Some(rx);
        self.google_oauth_in_progress = true;
        self.log(format!(
            "Opening browser to connect {}...",
            service.label()
        ));

        tokio::spawn(async move {
            let result = google_oauth::run_flow(client_id, client_secret, service).await;
            let _ = tx.send((service, result)).await;
        });
    }

    pub fn check_google_oauth_results(&mut self) {
        let Some(rx) = self.google_oauth_receiver.as_mut() else {
            return;
        };
        let Ok((service, result)) = rx.try_recv() else {
            return;
        };
        self.google_oauth_receiver = None;
        self.google_oauth_in_progress = false;

        let Some(settings) = self.settings.as_mut() else {
            return;
        };
        match result {
            Ok(new_tokens) => {
                let tokens = match service {
                    GoogleService::SearchConsole => &mut settings.connectors.search_console.tokens,
                    GoogleService::Ga4 => &mut settings.connectors.ga4.tokens,
                    GoogleService::Gbp => &mut settings.connectors.gbp.tokens,
                };
                tokens.access_token = new_tokens.access_token;
                tokens.expires_at = new_tokens.expires_at;
                // Google only returns a refresh_token on first consent; keep
                // the existing one on a reconnect if none came back.
                if !new_tokens.refresh_token.is_empty() {
                    tokens.refresh_token = new_tokens.refresh_token;
                }
                if let Err(e) = settings.save() {
                    self.log(format!("Connected, but failed to save settings: {}", e));
                } else {
                    self.log(format!("{} connected.", service.label()));
                }
            }
            Err(e) => {
                self.log(format!("{} connection failed: {}", service.label(), e));
            }
        }
    }

    // ---- Google Search Console ----

    pub fn spawn_gsc_fetch(&mut self) {
        let Some(settings) = self.settings.clone() else {
            return;
        };
        let cfg = settings.connectors.search_console;
        self.gsc_state.loading = true;
        self.gsc_state.error = None;

        let (tx, rx) = mpsc::channel(1);
        self.gsc_receiver = Some(rx);

        tokio::spawn(async move {
            let mut tokens = cfg.tokens;
            let result = gsc::fetch(&cfg.client_id, &cfg.client_secret, &cfg.site_url, &mut tokens)
                .await
                .map(|report| (report, tokens));
            let _ = tx.send(result).await;
        });
    }

    pub fn check_gsc_results(&mut self) {
        let Some(rx) = self.gsc_receiver.as_mut() else {
            return;
        };
        let Ok(result) = rx.try_recv() else {
            return;
        };
        self.gsc_receiver = None;
        self.gsc_state.loading = false;
        match result {
            Ok((report, tokens)) => {
                if let Some(settings) = self.settings.as_mut() {
                    settings.connectors.search_console.tokens = tokens;
                    let _ = settings.save();
                }
                self.gsc_state.data = Some(report);
            }
            Err(e) => self.gsc_state.error = Some(e),
        }
    }

    // ---- GA4 ----

    pub fn spawn_ga4_fetch(&mut self) {
        let Some(settings) = self.settings.clone() else {
            return;
        };
        let cfg = settings.connectors.ga4;
        self.ga4_state.loading = true;
        self.ga4_state.error = None;

        let (tx, rx) = mpsc::channel(1);
        self.ga4_receiver = Some(rx);

        tokio::spawn(async move {
            let mut tokens = cfg.tokens;
            let result = ga4::fetch(
                &cfg.client_id,
                &cfg.client_secret,
                &cfg.property_id,
                &mut tokens,
            )
            .await
            .map(|report| (report, tokens));
            let _ = tx.send(result).await;
        });
    }

    pub fn check_ga4_results(&mut self) {
        let Some(rx) = self.ga4_receiver.as_mut() else {
            return;
        };
        let Ok(result) = rx.try_recv() else {
            return;
        };
        self.ga4_receiver = None;
        self.ga4_state.loading = false;
        match result {
            Ok((report, tokens)) => {
                if let Some(settings) = self.settings.as_mut() {
                    settings.connectors.ga4.tokens = tokens;
                    let _ = settings.save();
                }
                self.ga4_state.data = Some(report);
            }
            Err(e) => self.ga4_state.error = Some(e),
        }
    }

    // ---- Google Business Profile ----

    pub fn spawn_gbp_fetch(&mut self) {
        let Some(settings) = self.settings.clone() else {
            return;
        };
        let cfg = settings.connectors.gbp;
        self.gbp_state.loading = true;
        self.gbp_state.error = None;

        let (tx, rx) = mpsc::channel(1);
        self.gbp_receiver = Some(rx);

        tokio::spawn(async move {
            let mut tokens = cfg.tokens;
            let result = gbp::fetch(
                &cfg.client_id,
                &cfg.client_secret,
                &cfg.account_id,
                &mut tokens,
            )
            .await
            .map(|locations| (locations, tokens));
            let _ = tx.send(result).await;
        });
    }

    pub fn check_gbp_results(&mut self) {
        let Some(rx) = self.gbp_receiver.as_mut() else {
            return;
        };
        let Ok(result) = rx.try_recv() else {
            return;
        };
        self.gbp_receiver = None;
        self.gbp_state.loading = false;
        match result {
            Ok((locations, tokens)) => {
                if let Some(settings) = self.settings.as_mut() {
                    settings.connectors.gbp.tokens = tokens;
                    let _ = settings.save();
                }
                self.gbp_state.data = Some(locations);
            }
            Err(e) => self.gbp_state.error = Some(e),
        }
    }

    // ---- Microsoft Clarity ----

    pub fn spawn_clarity_fetch(&mut self) {
        let Some(settings) = self.settings.clone() else {
            return;
        };
        let api_token = settings.connectors.clarity.api_token;
        self.clarity_state.loading = true;
        self.clarity_state.error = None;

        let (tx, rx) = mpsc::channel(1);
        self.clarity_receiver = Some(rx);

        tokio::spawn(async move {
            let result = clarity::fetch(&api_token).await;
            let _ = tx.send(result).await;
        });
    }

    pub fn check_clarity_results(&mut self) {
        let Some(rx) = self.clarity_receiver.as_mut() else {
            return;
        };
        let Ok(result) = rx.try_recv() else {
            return;
        };
        self.clarity_receiver = None;
        self.clarity_state.loading = false;
        match result {
            Ok(data) => self.clarity_state.data = Some(data),
            Err(e) => self.clarity_state.error = Some(e),
        }
    }

    // ---- Bing Webmaster Tools ----

    pub fn spawn_bing_fetch(&mut self) {
        let Some(settings) = self.settings.clone() else {
            return;
        };
        let cfg = settings.connectors.bing;
        self.bing_state.loading = true;
        self.bing_state.error = None;

        let (tx, rx) = mpsc::channel(1);
        self.bing_receiver = Some(rx);

        tokio::spawn(async move {
            let result = bing::fetch(&cfg.api_key, &cfg.site_url).await;
            let _ = tx.send(result).await;
        });
    }

    pub fn check_bing_results(&mut self) {
        let Some(rx) = self.bing_receiver.as_mut() else {
            return;
        };
        let Ok(result) = rx.try_recv() else {
            return;
        };
        self.bing_receiver = None;
        self.bing_state.loading = false;
        match result {
            Ok(data) => self.bing_state.data = Some(data),
            Err(e) => self.bing_state.error = Some(e),
        }
    }

    // ---- PageSpeed Insights ----

    pub fn spawn_pagespeed_fetch(&mut self) {
        let Some(settings) = self.settings.clone() else {
            return;
        };
        let api_key = settings.connectors.pagespeed.api_key;
        let url = self.input_url.clone();
        self.pagespeed_insights_state.loading = true;
        self.pagespeed_insights_state.error = None;

        let (tx, rx) = mpsc::channel(1);
        self.pagespeed_insights_receiver = Some(rx);

        tokio::spawn(async move {
            let result = pagespeed::fetch(&api_key, &url, "mobile").await;
            let _ = tx.send(result).await;
        });
    }

    pub fn check_pagespeed_insights_results(&mut self) {
        let Some(rx) = self.pagespeed_insights_receiver.as_mut() else {
            return;
        };
        let Ok(result) = rx.try_recv() else {
            return;
        };
        self.pagespeed_insights_receiver = None;
        self.pagespeed_insights_state.loading = false;
        match result {
            Ok(data) => self.pagespeed_insights_state.data = Some(data),
            Err(e) => self.pagespeed_insights_state.error = Some(e),
        }
    }

    // ---- Ahrefs ----

    pub fn spawn_ahrefs_fetch(&mut self) {
        let Some(settings) = self.settings.clone() else {
            return;
        };
        let cfg = settings.connectors.ahrefs;
        self.ahrefs_state.loading = true;
        self.ahrefs_state.error = None;

        let (tx, rx) = mpsc::channel(1);
        self.ahrefs_receiver = Some(rx);

        tokio::spawn(async move {
            let result = ahrefs::fetch(&cfg.api_token, &cfg.target).await;
            let _ = tx.send(result).await;
        });
    }

    pub fn check_ahrefs_results(&mut self) {
        let Some(rx) = self.ahrefs_receiver.as_mut() else {
            return;
        };
        let Ok(result) = rx.try_recv() else {
            return;
        };
        self.ahrefs_receiver = None;
        self.ahrefs_state.loading = false;
        match result {
            Ok(data) => self.ahrefs_state.data = Some(data),
            Err(e) => self.ahrefs_state.error = Some(e),
        }
    }

    /// Polls every Data & Insights receiver; called once per tick from
    /// `on_tick`, same as `check_robots_results`/`check_screenshot_results`.
    pub fn check_data_insights_results(&mut self) {
        self.check_google_oauth_results();
        self.check_gsc_results();
        self.check_ga4_results();
        self.check_gbp_results();
        self.check_clarity_results();
        self.check_bing_results();
        self.check_pagespeed_insights_results();
        self.check_ahrefs_results();
    }
}
