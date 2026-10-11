//! Folder inspector inventory.

mod discover;
mod node;
mod roots;
mod scan;
mod scan_fold;
mod scan_meta;
mod scan_queue;

pub use discover::{discover_projects, discover_projects_from};
pub use node::{DirectoryNode, InventoryLimits, InventoryReport};
pub use roots::{developer_roots, is_discoverable_below, review_scan_roots};
pub use scan::{ScanTick, scan_inventory, scan_inventory_with};

#[cfg(test)]
mod parallel_tests;
#[cfg(test)]
mod tests;
