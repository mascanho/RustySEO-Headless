use std::sync::mpsc;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::{Registry, prelude::*};

pub struct TuiLayer {
    sender: mpsc::Sender<String>,
}

impl TuiLayer {
    pub fn new(sender: mpsc::Sender<String>) -> Self {
        Self { sender }
    }
}

impl<S: Subscriber> tracing_subscriber::Layer<S> for TuiLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = LogVisitor::new();
        event.record(&mut visitor);

        // Include level and timestamp
        let level = *event.metadata().level();
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let formatted = format!("[{}][{}] {}", timestamp, level, visitor.message);

        let _ = self.sender.send(formatted);
    }
}

struct LogVisitor {
    message: String,
}

impl LogVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
}

impl tracing::field::Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}

/// Trait to allow easy logging of any Display-able type directly to the TUI.
pub trait AppLoggable {
    /// Log the item as INFO level using Display
    fn log_info(&self);
    /// Log the item as ERROR level using Display
    fn log_error(&self);
    /// Log the item as DEBUG level using Display
    fn log_debug(&self);
}

/// Trait to allow easy logging of Debug types.
pub trait AppDebuggable {
    /// Log the item using Debug formatting {:?} at INFO level
    fn tui_dbg(&self);
}

// Support types with Display
impl<T: std::fmt::Display> AppLoggable for T {
    fn log_info(&self) {
        tracing::info!("{}", self);
    }
    fn log_error(&self) {
        tracing::error!("{}", self);
    }
    fn log_debug(&self) {
        tracing::debug!("{}", self);
    }
}

// Support types with Debug
impl<T: std::fmt::Debug> AppDebuggable for T {
    fn tui_dbg(&self) {
        tracing::info!("{:?}", self);
    }
}

/// A macro similar to println! but for the TUI logs.
/// Supports standard formatting like {:?} for objects.
#[macro_export]
macro_rules! tui_println {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

/// A macro similar to dbg! but for the TUI logs.
/// Prints the file, line, and the debug representation of the expression.
#[macro_export]
macro_rules! tui_dbg {
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                tracing::info!(
                    "[{}:{}] {} = {:?}",
                    file!(),
                    line!(),
                    stringify!($val),
                    &tmp
                );
                tmp
            }
        }
    };
}

pub fn init() -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();

    let tui_layer = TuiLayer::new(tx);

    let subscriber = Registry::default().with(tui_layer);

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    rx
}
