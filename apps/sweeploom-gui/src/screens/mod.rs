//! One module per primary surface.

mod ai;
mod browser;
mod history;
mod overview;
mod projects;
mod review;
mod rules;
mod session_actions;
mod session_observe;
mod session_plan;
mod sessions;
mod settings;
mod storage;

pub use ai::ui_ai;
pub use browser::ui_browser;
pub use history::ui_history;
pub use overview::ui_overview;
pub use projects::ui_projects;
pub use review::ui_review;
pub use rules::ui_rules;
pub use sessions::ui_sessions;
pub use settings::ui_settings;
pub use storage::ui_storage;
