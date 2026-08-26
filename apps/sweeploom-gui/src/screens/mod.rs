//! One module per primary surface.

mod ai;
mod browser;
mod browser_later;
mod browser_state;
mod browser_tabs;
mod browser_trees;
mod history;
mod overview;
mod project_facts;
mod project_rows;
mod projects;
mod review;
mod rules;
mod session_actions;
mod session_label;
mod session_members;
mod session_observe;
mod session_plan;
mod session_raw;
mod sessions;
mod settings;
mod storage;

pub use ai::ui_ai;
pub use browser::ui_browser;
pub use browser_state::BrowserUi;
pub use history::ui_history;
pub use overview::ui_overview;
pub use project_rows::ProjectGroup;
pub use projects::ui_projects;
pub use review::ui_review;
pub use rules::ui_rules;
pub use sessions::ui_sessions;
pub use settings::ui_settings;
pub use storage::ui_storage;
