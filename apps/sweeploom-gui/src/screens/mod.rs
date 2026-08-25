//! One module per primary surface.

mod overview;
mod projects;
mod sessions;
mod settings;
mod storage;

pub use overview::ui_overview;
pub use projects::ui_projects;
pub use sessions::{ui_process_table, ui_sessions};
pub use settings::ui_settings;
pub use storage::ui_storage;
