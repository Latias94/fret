//! Slider primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/slider/src/slider.tsx`
//!
//! This module currently focuses on the single-thumb slider outcome used by shadcn/ui. Radix
//! supports multi-thumb sliders; the headless helpers now cover Radix multi-thumb value updates,
//! but the higher-level widget recipes may still use the single-thumb surface.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::SemanticsProps;

pub use crate::declarative::slider::{
    start_slider_drag_from_pointer_x, update_single_slider_model_from_pointer_x,
    update_slider_model_from_pointer_x,
};
pub use crate::headless::slider::{
    SliderValuesUpdate, closest_value_index, format_semantics_value, has_min_steps_between_values,
    next_sorted_values, normalize_value, snap_value, steps_between_values,
    update_multi_thumb_values,
};

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
