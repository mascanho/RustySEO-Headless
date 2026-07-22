pub mod actions;
pub mod input;
pub mod issues;
pub mod link_score;
pub mod menu_actions;
pub mod modals;
pub mod navigation;
pub mod processing;
pub mod settings;
pub mod sidebar;
pub mod state;

pub use state::AppState;
pub use state::RustyColors;

use crate::models::App;

impl App {
    pub fn new() -> Self {
        Self::default()
    }
}
