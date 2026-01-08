//! Arrow primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/arrow/src/arrow.tsx`
//!
//! In Radix, the standalone `arrow` package provides a reusable arrow element that is composed
//! into other overlay-ish primitives. In Fret, arrow geometry is derived from `Popper` placement
//! (`AnchoredPanelLayout`) and rendered via a small renderer-agnostic "diamond" arrow helper.
//!
//! This module is intentionally a thin, Radix-named facade over `popper_arrow` for discoverability.

pub use crate::primitives::popper_arrow::{
    DiamondArrowStyle, diamond_arrow_element, diamond_arrow_element_refined, wrapper_insets,
};

