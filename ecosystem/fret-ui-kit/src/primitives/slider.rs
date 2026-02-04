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
use fret_runtime::Model;
use fret_ui::element::SemanticsProps;
use fret_ui::{ElementContext, UiHost};

pub use crate::declarative::slider::{
    start_slider_drag_from_pointer_x, update_single_slider_model_from_pointer_x,
    update_slider_model_from_pointer_x,
};
pub use crate::headless::slider::{
    SliderValuesUpdate, closest_value_index, format_semantics_value, has_min_steps_between_values,
    next_sorted_values, normalize_value, snap_value, steps_between_values,
    update_multi_thumb_values,
};

/// Returns a values model that behaves like Radix `useControllableState` (`value` / `defaultValue`).
pub fn slider_use_values_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Vec<f32>>>,
    default_value: impl FnOnce() -> Vec<f32>,
) -> crate::primitives::controllable_state::ControllableModel<Vec<f32>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

/// Semantics wrapper props for a slider root.
pub fn slider_root_semantics(
    label: Option<Arc<str>>,
    value: f32,
    disabled: bool,
) -> SemanticsProps {
    SemanticsProps {
        role: SemanticsRole::Generic,
        label,
        value: Some(format_semantics_value(value)),
        disabled,
        ..Default::default()
    }
}

/// Semantics wrapper props for an interactive slider thumb (Radix `role="slider"`).
pub fn slider_thumb_semantics(
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;
    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn slider_root_semantics_exposes_value_string() {
        let label = Arc::<str>::from("slider");
        let value = 12.0;

        let out = slider_root_semantics(Some(label.clone()), value, false);
        assert_eq!(out.label.as_deref(), Some(label.as_ref()));
        assert_eq!(out.value, Some(format_semantics_value(value)));
        assert_eq!(out.disabled, false);
    }

    #[test]
    fn slider_use_values_model_prefers_controlled_and_does_not_call_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(vec![0.25]);
        let called = Cell::new(0);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let out = slider_use_values_model(cx, Some(controlled.clone()), || {
                called.set(called.get() + 1);
                vec![0.0]
            });
            assert!(out.is_controlled());
            assert_eq!(out.model(), controlled);
        });

        assert_eq!(called.get(), 0);
    }
}
