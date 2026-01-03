//! Collection semantics helpers (Radix-aligned outcomes).
//!
//! Radix uses a “Collection” pattern to attach per-item metadata (position in set / set size) so
//! assistive technologies can announce “item X of Y” in lists and menus.
//!
//! In Fret, this is stamped onto `PressableA11y` via a small extension trait.

pub use crate::declarative::collection_semantics::CollectionSemanticsExt;
