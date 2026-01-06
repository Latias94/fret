//! Slider primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/slider/src/slider.tsx`
//!
//! This module currently focuses on the single-thumb slider outcome used by shadcn/ui. Radix
//! supports multi-thumb sliders; we can extend this facade once multi-thumb modeling is settled.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::SemanticsProps;

pub use crate::declarative::slider::update_single_slider_model_from_pointer_x;
pub use crate::headless::slider::{format_semantics_value, normalize_value, snap_value};

/// Semantics wrapper props for a slider root.
pub fn slider_root_semantics(
    label: Option<Arc<str>>,
    value: f32,
    disabled: bool,
) -> SemanticsProps {
    SemanticsProps {
        role: SemanticsRole::Slider,
        label,
        value: Some(format_semantics_value(value)),
        disabled,
        ..Default::default()
    }
}
