//! One module per primary surface.

mod ai;
mod ai_rows;
mod browser;
mod browser_later;
mod browser_state;
mod browser_tabs;
mod browser_tabs_setup;
mod browser_tree_detail;
mod browser_trees;
mod explorer_rows;
mod history;
mod history_rows;
mod overview;
mod project_facts;
mod project_rows;
mod projects;
mod review;
mod rules;
mod session_actions;
mod session_detail;
mod session_fills;
mod session_label;
mod session_members;
mod session_observe;
mod session_plan;
mod session_pressure;
mod session_raw;
mod session_rows;
mod sessions;
mod settings;
mod storage;

pub use ai::ui_ai;
pub use ai_rows::AiGroup;
pub use browser::ui_browser;
pub use browser_state::BrowserUi;
pub use history::ui_history;
pub use overview::ui_overview;
pub(crate) use project_rows::ProjectCard;
pub use project_rows::ProjectGroup;
pub use projects::ui_projects;
pub use review::ui_review;
pub use rules::ui_rules;
pub use sessions::ui_sessions;
pub use settings::ui_settings;
pub use storage::ui_storage;

#[cfg(test)]
mod history_rows_tests;
#[cfg(test)]
mod project_rows_tests;
#[cfg(test)]
mod session_rows_tests;
