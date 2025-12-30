#![deny(deprecated)]
//! General-purpose UI components built on top of `fret-ui`.
//!
//! This crate is intentionally domain-agnostic (no engine/editor-specific concepts).
//! Styling is token-driven and supports namespaced extension tokens (see ADR 0050).
//!
//! Note: This crate is declarative-only. Retained-widget authoring is intentionally not part of
//! the public component surface.

pub mod declarative;
pub mod headless;
pub mod overlay;
pub mod recipes;
pub mod tree;
pub mod window_overlays;

mod sizing;
mod style;
mod styled;

pub use sizing::{Sizable, Size};
pub use style::{
    ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, MetricRef, OverflowRefinement,
    Radius, Space,
};
pub use styled::{RefineStyle, Stylable, Styled, StyledExt};

pub use tree::{
    TreeEntry, TreeItem, TreeItemId, TreeRowRenderer, TreeRowState, TreeState, flatten_tree,
};
