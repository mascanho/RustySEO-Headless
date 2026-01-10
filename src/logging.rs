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

        // Include level
        let level = *event.metadata().level();
        let formatted = format!("[{}] {}", level, visitor.message);

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

pub fn init() -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();

    let tui_layer = TuiLayer::new(tx);

    let subscriber = Registry::default().with(tui_layer);

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    rx
}
