// open the configuration files

use crate::{models::AppSettings, tui_println};
use tracing::{error, info, warn};

pub fn edit_file() {
    let full_path = AppSettings::path();

    info!("Attempting to open settings file: {:?}", full_path);

    if !full_path.exists() {
        error!("Settings file not found at: {:?}", full_path);
        tui_println!("No file found");
        return;
    }

    let os = std::env::consts::OS;
    info!("Detected OS: {}", os);

    let (command, arg) = match os {
        "macos" => {
            info!("Using macOS 'open' command");
            ("open", None)
        }
        "windows" => {
            info!("Using Windows 'start' command");
            ("start", Some(""))
        }
        "linux" => match std::process::Command::new("which").arg("xdg-open").output() {
            Ok(output) if output.status.success() => {
                info!("Using Linux 'xdg-open' command");
                ("xdg-open", None)
            }
            Ok(output) => {
                warn!("xdg-open not found, exit status: {:?}", output.status);
                info!("Falling back to 'nano' editor");
                ("nano", None)
            }
            Err(e) => {
                error!("Failed to check for xdg-open: {}", e);
                info!("Falling back to 'nano' editor");
                ("nano", None)
            }
        },
        other => {
            warn!("Unsupported OS: {}, falling back to xdg-open", other);
            ("xdg-open", None)
        }
    };

    info!("Executing command: {} with arg: {:?}", command, full_path);

    let mut cmd = std::process::Command::new(command);
    cmd.arg(&full_path);
    if let Some(arg) = arg {
        cmd.arg(arg);
    }

    match cmd.spawn() {
        Ok(child) => {
            info!("Successfully spawned command with PID: {:?}", child.id());
            tui_println!("Opening settings.toml...");
        }
        Err(e) => {
            error!("Failed to execute command '{}': {}", command, e);
            error!("Command args: {:?}", cmd.get_args().collect::<Vec<_>>());
            tui_println!("Failed to open settings.toml: {}", e);
        }
    }
}
