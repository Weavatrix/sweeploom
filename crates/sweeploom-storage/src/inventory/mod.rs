//! Folder inspector inventory.

mod node;
mod scan;

pub use node::{DirectoryNode, InventoryLimits, InventoryReport};
pub use scan::scan_inventory;

#[cfg(test)]
mod tests;
