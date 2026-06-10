//! Sortable/reorder recipe built on the headless `fret-dnd` toolbox.
//!
//! This is intentionally not a "full component": it focuses on the DnD policy wiring and keeps
//! visuals/content fully caller-owned.

mod reorder_list;

pub use reorder_list::{SortableReorderListProps, sortable_reorder_list};

#[cfg(test)]
mod tests;
