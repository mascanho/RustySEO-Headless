use directories::ProjectDirs;
use std::fs;
use std::io::Write;
use std::path::Path;

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
