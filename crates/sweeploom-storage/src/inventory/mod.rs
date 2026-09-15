//! Folder inspector inventory.

mod discover;
mod node;
mod scan;

pub use discover::{developer_roots, discover_projects, discover_projects_from, review_scan_roots};
pub use node::{DirectoryNode, InventoryLimits, InventoryReport};
pub use scan::{ScanTick, scan_inventory, scan_inventory_with};

#[cfg(test)]
mod tests;
