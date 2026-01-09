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
        // Optionally, write default contents to the file
        file.write_all(b"# RustySEO settings\n")
            .unwrap_or_else(|err| {
                eprintln!("Failed to write to settings.toml: {}", err);
                std::process::exit(1);
            });
    }

    project_dirs
}
