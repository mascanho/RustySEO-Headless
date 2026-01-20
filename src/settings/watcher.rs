use crate::models::AppSettings;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::Sender;
use std::time::Duration;

pub fn watch_settings(tx: Sender<()>) -> notify::Result<()> {
    let mut watcher: RecommendedWatcher = Watcher::new(
        move |_| {
            tx.send(()).unwrap();
        },
        notify::Config::default().with_poll_interval(Duration::from_millis(500)),
    )?;

    let path = AppSettings::path();
    if path.exists() {
        watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
    }

    // The watcher needs to be kept alive.
    // We can do this by returning it, but in this case we'll just leak it.
    // This is not ideal, but it's a simple way to keep the watcher running for the lifetime of the application.
    std::mem::forget(watcher);

    Ok(())
}
