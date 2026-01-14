// open the configuration files

use crate::{models::AppSettings, tui_println};

pub fn edit_file() {
    let path = AppSettings::path();
    let file = "cli-settings.toml";

    if path.exists() {
        std::process::Command::new("code")
            .arg(path().join(file))
            .spawn()
            .expect("Failed to open settings.toml");
    } else {
        tui_println!("No file found")
    }

    println!("Opening settings.toml...");
}
