use crate::models::AppSettings;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Starts a file watcher for the settings file.
/// When the file is modified, sends a signal through the provided channel.
/// Uses debouncing to prevent multiple rapid reloads.
///
/// Returns the watcher handle - the caller must keep this alive for the watcher to work.
pub fn watch_settings(tx: Sender<()>) -> notify::Result<RecommendedWatcher> {
    // Debounce mechanism: only trigger after 300ms of no changes
    let last_event: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    let debounce_duration = Duration::from_millis(300);

    let tx_clone = tx.clone();
    let last_event_clone = last_event.clone();

    let watcher = RecommendedWatcher::new(
        move |result: Result<Event, notify::Error>| {
            if let Ok(event) = result {
                // Only trigger on modify events
                if event.kind.is_modify() || event.kind.is_create() {
                    let mut last = last_event_clone.lock().unwrap();
                    let now = Instant::now();

                    // Check if enough time has passed since the last event
                    let should_trigger = match *last {
                        Some(last_time) => now.duration_since(last_time) >= debounce_duration,
                        None => true,
                    };

                    if should_trigger {
                        *last = Some(now);
                        let _ = tx_clone.send(());
                    }
                }
            }
        },
        Config::default()
            .with_poll_interval(Duration::from_millis(500))
            .with_compare_contents(false), // Use mtime for efficiency
    )?;

    Ok(watcher)
}

/// Initializes the settings watcher and starts watching the settings file.
/// Returns the watcher handle which must be kept alive.
pub fn init_settings_watcher(tx: Sender<()>) -> Option<RecommendedWatcher> {
    match watch_settings(tx) {
        Ok(mut watcher) => {
            let path = AppSettings::path();
            if path.exists() {
                if let Err(e) = watcher.watch(path.as_ref(), RecursiveMode::NonRecursive) {
                    eprintln!("Failed to watch settings file: {}", e);
                    return None;
                }
            }
            Some(watcher)
        }
        Err(e) => {
            eprintln!("Failed to create settings watcher: {}", e);
            None
        }
    }
}
