use crate::connectors::google_oauth;
use crate::models::{GbpLocation, GbpLocations, GoogleOAuthTokens};
use serde::Deserialize;

#[derive(Deserialize, Default)]
struct LocationsResponse {
    #[serde(default)]
    locations: Vec<Location>,
}

#[derive(Deserialize, Default)]
struct Location {
    #[serde(default)]
    title: String,
    #[serde(default, rename = "storefrontAddress")]
    storefront_address: Option<Address>,
    #[serde(default, rename = "phoneNumbers")]
    phone_numbers: Option<PhoneNumbers>,
    #[serde(default, rename = "websiteUri")]
    website_uri: String,
}

#[derive(Deserialize, Default)]
struct Address {
    #[serde(default, rename = "addressLines")]
    address_lines: Vec<String>,
    #[serde(default)]
    locality: String,
}

#[derive(Deserialize, Default)]
struct PhoneNumbers {
    #[serde(default, rename = "primaryPhone")]
    primary_phone: String,
}

/// Lists the account's Business Profile locations (title/address/phone/site)
/// via the Business Information API. Reviews and performance metrics live in
/// separate, more involved APIs and aren't covered by this first pass.
pub async fn fetch(
    client_id: &str,
    client_secret: &str,
    account_id: &str,
    tokens: &mut GoogleOAuthTokens,
) -> Result<GbpLocations, String> {
    if account_id.is_empty() {
        return Err("Set connectors.gbp.account_id in cli-settings.toml".to_string());
    }
    google_oauth::refresh_if_needed(client_id, client_secret, tokens).await?;

    let api_url = format!(
        "https://mybusinessbusinessinformation.googleapis.com/v1/accounts/{}/locations",
        account_id
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&api_url)
        .bearer_auth(&tokens.access_token)
        .query(&[(
            "readMask",
            "title,storefrontAddress,phoneNumbers,websiteUri",
        )])
        .send()
        .await
        .map_err(|e| format!("Business Profile request failed: {}", e))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Business Profile API error: {}", text));
    }

    let parsed: LocationsResponse = resp
        .json()
        .await
        .map_err(|e| format!("Could not parse Business Profile response: {}", e))?;

    Ok(GbpLocations {
        locations: parsed
            .locations
            .into_iter()
            .map(|l| {
                let address = l
                    .storefront_address
                    .map(|a| {
                        let mut parts = a.address_lines;
                        if !a.locality.is_empty() {
                            parts.push(a.locality);
                        }
                        parts.join(", ")
                    })
                    .unwrap_or_default();
                GbpLocation {
                    title: l.title,
                    address,
                    phone: l
                        .phone_numbers
                        .map(|p| p.primary_phone)
                        .unwrap_or_default(),
                    website: l.website_uri,
                }
            })
            .collect(),
    })
}
