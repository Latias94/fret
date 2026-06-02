use fret_core::Px;
use fret_ui::Theme;
use fret_ui::element::{LayoutStyle, Length};

use super::super::{EditorTokenKeys, NumericValueConstraints};

#[derive(Debug, Clone, Copy)]
pub struct DragValueCoreOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub scrub_on_double_click: bool,
    pub drag_threshold: Px,
    pub scrub_speed: f64,
    pub slow_multiplier: f64,
    pub fast_multiplier: f64,
    pub constraints: NumericValueConstraints,
}

impl Default for DragValueCoreOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: fret_ui::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            scrub_on_double_click: true,
            drag_threshold: Px(4.0),
            scrub_speed: 0.02,
            slow_multiplier: 0.1,
            fast_multiplier: 10.0,
            constraints: NumericValueConstraints::default(),
        }
    }
}

pub(super) fn resolve_options(
    theme: &Theme,
    mut opts: DragValueCoreOptions,
) -> DragValueCoreOptions {
    let scrub_speed = theme
        .metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_SPEED)
        .map(|m| m.0 as f64)
        .unwrap_or(opts.scrub_speed);
    let slow_multiplier = theme
        .metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_SLOW_MULTIPLIER)
        .map(|m| m.0 as f64)
        .unwrap_or(opts.slow_multiplier);
    let fast_multiplier = theme
        .metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_FAST_MULTIPLIER)
        .map(|m| m.0 as f64)
        .unwrap_or(opts.fast_multiplier);
    let drag_threshold = theme
        .metric_by_key(EditorTokenKeys::NUMERIC_SCRUB_DRAG_THRESHOLD)
        .unwrap_or(opts.drag_threshold);

    if !scrub_speed.is_finite() {
        opts.scrub_speed = 0.02;
    } else {
        opts.scrub_speed = scrub_speed.max(0.0);
    }
    opts.slow_multiplier = if slow_multiplier.is_finite() {
        slow_multiplier.max(0.0)
    } else {
        0.1
    };
    opts.fast_multiplier = if fast_multiplier.is_finite() {
        fast_multiplier.max(0.0)
    } else {
        10.0
    };
    opts.drag_threshold = if drag_threshold.0.is_finite() {
        Px(drag_threshold.0.max(0.0))
    } else {
        Px(4.0)
    };
    opts
}
