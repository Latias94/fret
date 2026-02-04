//! Slider primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/slider/src/slider.tsx`
//!
//! This module currently focuses on the single-thumb slider outcome used by shadcn/ui. Radix
//! supports multi-thumb sliders; the headless helpers now cover Radix multi-thumb value updates,
//! but the higher-level widget recipes may still use the single-thumb surface.

use std::sync::Arc;

use fret_core::{Axis, KeyCode, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::SemanticsProps;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::direction::LayoutDirection;

pub use crate::declarative::slider::{
    start_slider_drag_from_pointer_axis, start_slider_drag_from_pointer_x,
    update_single_slider_model_from_pointer_axis, update_single_slider_model_from_pointer_x,
    update_slider_model_from_pointer_axis, update_slider_model_from_pointer_x,
};
pub use crate::headless::slider::{
    SliderValuesUpdate, closest_value_index, format_semantics_value, has_min_steps_between_values,
    next_sorted_values, normalize_value, snap_value, steps_between_values,
    update_multi_thumb_values,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SliderOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderSlideDirection {
    FromLeft,
    FromRight,
    FromBottom,
    FromTop,
}

pub fn slider_axis(orientation: SliderOrientation) -> Axis {
    match orientation {
        SliderOrientation::Horizontal => Axis::Horizontal,
        SliderOrientation::Vertical => Axis::Vertical,
    }
}

pub fn slider_slide_direction(
    orientation: SliderOrientation,
    dir: LayoutDirection,
    inverted: bool,
) -> SliderSlideDirection {
    match orientation {
        SliderOrientation::Horizontal => {
            let is_direction_ltr = dir == LayoutDirection::Ltr;
            let is_sliding_from_left =
                (is_direction_ltr && !inverted) || (!is_direction_ltr && inverted);
            if is_sliding_from_left {
                SliderSlideDirection::FromLeft
            } else {
                SliderSlideDirection::FromRight
            }
        }
        SliderOrientation::Vertical => {
            let is_sliding_from_bottom = !inverted;
            if is_sliding_from_bottom {
                SliderSlideDirection::FromBottom
            } else {
                SliderSlideDirection::FromTop
            }
        }
    }
}

pub fn slider_min_at_axis_start(
    orientation: SliderOrientation,
    dir: LayoutDirection,
    inverted: bool,
) -> bool {
    match slider_slide_direction(orientation, dir, inverted) {
        SliderSlideDirection::FromLeft | SliderSlideDirection::FromTop => true,
        SliderSlideDirection::FromRight | SliderSlideDirection::FromBottom => false,
    }
}

/// Convert a value-normalized percent `t` into a position percent measured from the axis start
/// (left/top), matching Radix `startEdge` outcomes.
pub fn slider_position_t(
    orientation: SliderOrientation,
    dir: LayoutDirection,
    inverted: bool,
    value_t: f32,
) -> f32 {
    let pos_t = if slider_min_at_axis_start(orientation, dir, inverted) {
        value_t
    } else {
        1.0 - value_t
    };
    pos_t.clamp(0.0, 1.0)
}

pub fn slider_is_back_key(slide_direction: SliderSlideDirection, key: KeyCode) -> bool {
    // Radix reference:
    // `repo-ref/primitives/packages/react/slider/src/slider.tsx` (`BACK_KEYS`).
    match slide_direction {
        SliderSlideDirection::FromLeft => matches!(
            key,
            KeyCode::Home | KeyCode::PageDown | KeyCode::ArrowDown | KeyCode::ArrowLeft
        ),
        SliderSlideDirection::FromRight => matches!(
            key,
            KeyCode::Home | KeyCode::PageDown | KeyCode::ArrowDown | KeyCode::ArrowRight
        ),
        SliderSlideDirection::FromBottom => matches!(
            key,
            KeyCode::Home | KeyCode::PageDown | KeyCode::ArrowDown | KeyCode::ArrowLeft
        ),
        SliderSlideDirection::FromTop => matches!(
            key,
            KeyCode::Home | KeyCode::PageDown | KeyCode::ArrowUp | KeyCode::ArrowLeft
        ),
    }
}

pub fn slider_step_direction_for_key(
    slide_direction: SliderSlideDirection,
    key: KeyCode,
) -> Option<f32> {
    match key {
        KeyCode::PageUp
        | KeyCode::PageDown
        | KeyCode::ArrowUp
        | KeyCode::ArrowDown
        | KeyCode::ArrowLeft
        | KeyCode::ArrowRight => Some(if slider_is_back_key(slide_direction, key) {
            -1.0
        } else {
            1.0
        }),
        _ => None,
    }
}

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

    #[test]
    fn slider_slide_direction_matches_radix_rules() {
        assert_eq!(
            slider_slide_direction(SliderOrientation::Horizontal, LayoutDirection::Ltr, false),
            SliderSlideDirection::FromLeft
        );
        assert_eq!(
            slider_slide_direction(SliderOrientation::Horizontal, LayoutDirection::Rtl, false),
            SliderSlideDirection::FromRight
        );
        assert_eq!(
            slider_slide_direction(SliderOrientation::Horizontal, LayoutDirection::Ltr, true),
            SliderSlideDirection::FromRight
        );
        assert_eq!(
            slider_slide_direction(SliderOrientation::Horizontal, LayoutDirection::Rtl, true),
            SliderSlideDirection::FromLeft
        );

        assert_eq!(
            slider_slide_direction(SliderOrientation::Vertical, LayoutDirection::Ltr, false),
            SliderSlideDirection::FromBottom
        );
        assert_eq!(
            slider_slide_direction(SliderOrientation::Vertical, LayoutDirection::Rtl, false),
            SliderSlideDirection::FromBottom
        );
        assert_eq!(
            slider_slide_direction(SliderOrientation::Vertical, LayoutDirection::Ltr, true),
            SliderSlideDirection::FromTop
        );
    }

    #[test]
    fn slider_step_direction_for_key_uses_back_key_table() {
        assert_eq!(
            slider_step_direction_for_key(SliderSlideDirection::FromLeft, KeyCode::ArrowLeft),
            Some(-1.0)
        );
        assert_eq!(
            slider_step_direction_for_key(SliderSlideDirection::FromLeft, KeyCode::ArrowRight),
            Some(1.0)
        );
        assert_eq!(
            slider_step_direction_for_key(SliderSlideDirection::FromRight, KeyCode::ArrowLeft),
            Some(1.0)
        );
        assert_eq!(
            slider_step_direction_for_key(SliderSlideDirection::FromRight, KeyCode::ArrowRight),
            Some(-1.0)
        );
        assert_eq!(
            slider_step_direction_for_key(SliderSlideDirection::FromTop, KeyCode::ArrowUp),
            Some(-1.0)
        );
        assert_eq!(
            slider_step_direction_for_key(SliderSlideDirection::FromBottom, KeyCode::ArrowDown),
            Some(-1.0)
        );
    }
}
