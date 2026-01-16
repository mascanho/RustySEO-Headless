use directories::ProjectDirs;
use std::fs;
use std::io::Write;
use std::str::FromStr;

use crate::models::AppSettings;
use crate::{settings, tui_println};
use publicsuffix::{List, Psl};

pub async fn create_settings_file() -> ProjectDirs {
    let project_dirs = ProjectDirs::from("", "", "rustyseo").unwrap();

    // create if it doesn't exist
    if !project_dirs.data_dir().exists() {
        fs::create_dir_all(project_dirs.data_dir()).unwrap_or_else(|err| {
            eprintln!("Failed to create project directory: {}", err);
            std::process::exit(1);
        });
    }

    // create settings.toml in the data directory if it doesn't exist
    let settings_path = project_dirs.data_dir().join("cli-settings.toml");
    if !settings_path.exists() {
        let mut file = fs::File::create(&settings_path).unwrap_or_else(|err| {
            eprintln!("Failed to create settings.toml: {}", err);
            std::process::exit(1);
        });

        // Write default settings to TOML
        let default_settings = crate::models::AppSettings::default();
        let toml_string = toml::to_string_pretty(&default_settings).unwrap_or_else(|err| {
            eprintln!("Failed to serialize default settings: {}", err);
            std::process::exit(1);
        });

        file.write_all(toml_string.as_bytes())
            .unwrap_or_else(|err| {
                eprintln!("Failed to write to settings.toml: {}", err);
                std::process::exit(1);
            });
    }

    project_dirs
}

// THIS HANDLE THE COMPLETE RECENT CRAWLS STORING THEM TO BE SELECTED
pub async fn create_recent_crawls_file() -> ProjectDirs {
    let project_dirs = ProjectDirs::from("", "", "rustyseo").unwrap();

    // create if it doesn't exist
    if !project_dirs.data_dir().exists() {
        fs::create_dir_all(project_dirs.data_dir()).unwrap_or_else(|err| {
            eprintln!("Failed to create project directory: {}", err);
            std::process::exit(1);
        });
    }

    // create recent-crawls.json in the data directory if it doesn't exist
    let recent_crawls_path = project_dirs.data_dir().join("recent-crawls.json");
    if !recent_crawls_path.exists() {
        let mut file = fs::File::create(&recent_crawls_path).unwrap_or_else(|err| {
            eprintln!("Failed to create recent-crawls.json: {}", err);
            std::process::exit(1);
        });

        // Write empty array to JSON
        let empty_array = "[]";
        file.write_all(empty_array.as_bytes())
            .unwrap_or_else(|err| {
                eprintln!("Failed to write to recent-crawls.json: {}", err);
                std::process::exit(1);
            });
    }

    project_dirs
}

pub async fn add_recent_entry(url: String) {
    // GET THE MAIN DOMAIN OF THE CRAWLED URL
    let domain_str = match url::Url::parse(&url) {
        Ok(parsed_url) => parsed_url.domain().unwrap_or(&url).to_string(),
        Err(_) => url,
    };

    let list = List::new();
    let root_domain = list
        .domain(domain_str.as_bytes())
        .map(|d| String::from_utf8_lossy(d.as_bytes()).to_string())
        .unwrap_or_else(|| domain_str.to_string());

    tui_println!("Parsed domain_str: {}", domain_str);
    tui_println!("Root domain: {}", &root_domain);

    tokio::task::spawn_blocking(move || {
        let project_dirs =
            directories::ProjectDirs::from("", "", "rustyseo").expect("Failed to get project dirs");

        let recent_crawls_path = project_dirs.data_dir().join("recent-crawls.json");

        tui_println!("Recent crawls path: {}", recent_crawls_path.display());

        let mut recent_crawls: Vec<String> = if recent_crawls_path.exists() {
            let content = std::fs::read_to_string(&recent_crawls_path);
            match content {
                Ok(c) => serde_json::from_str(&c).unwrap_or_else(|e| {
                    tui_println!(
                        "Failed to parse recent-crawls.json: {}, resetting to empty",
                        e
                    );
                    Vec::new()
                }),
                Err(e) => {
                    tui_println!("Failed to read recent-crawls.json: {}, using empty", e);
                    Vec::new()
                }
            }
        } else {
            tui_println!("Recent crawls file does not exist, creating new");
            Vec::new()
        };

        if !recent_crawls.contains(&root_domain) {
            recent_crawls.insert(0, root_domain.clone());
            if recent_crawls.len() > 20 {
                recent_crawls.pop();
            }
            tui_println!("Added new domain, recent crawls: {:?}", recent_crawls);
        } else {
            tui_println!("Domain already in recent crawls");
        }

        if let Ok(json) = serde_json::to_string_pretty(&recent_crawls) {
            match std::fs::write(&recent_crawls_path, json) {
                Ok(_) => tui_println!(
                    "Successfully wrote to recent-crawls.json with {} entries",
                    recent_crawls.len()
                ),
                Err(e) => tui_println!("Failed to write to recent-crawls.json: {}", e),
            }
        } else {
            tui_println!("Failed to serialize recent crawls to JSON");
        }
    })
    .await
    .expect("spawn_blocking failed");
}
