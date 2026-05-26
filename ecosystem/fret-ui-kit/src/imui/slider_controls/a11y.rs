use fret_core::{SemanticsOrientation, SemanticsRole};
use fret_ui::element::SemanticsDecoration;
use fret_ui::{ElementContext, UiHost};

pub(super) struct SliderA11y {
    pub(super) current: f32,
    pub(super) min: f32,
    pub(super) max: f32,
    pub(super) decoration: SemanticsDecoration,
}

pub(super) fn resolve<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: &fret_runtime::Model<f32>,
    min: f32,
    max: f32,
    step: f32,
) -> SliderA11y {
    let current = cx
        .read_model(model, fret_ui::Invalidation::Paint, |_app, v| {
            super::super::slider_clamp_and_snap(*v, min, max, step)
        })
        .unwrap_or_else(|_| super::super::slider_clamp_and_snap(min, min, max, step));
    let (min, max) = super::super::slider_normalize_range(min, max);
    let step = super::super::slider_step_or_default(step);

    let mut decoration = SemanticsDecoration::default()
        .role(SemanticsRole::Slider)
        .orientation(SemanticsOrientation::Horizontal)
        .value(crate::headless::slider::format_semantics_value(current));

    if current.is_finite() {
        decoration = decoration.numeric_value(current as f64);
    }
    if min.is_finite() && max.is_finite() {
        decoration = decoration.numeric_range(min as f64, max as f64);
    }
    if step.is_finite() && step > 0.0 {
        decoration = decoration
            .numeric_step(step as f64)
            .numeric_jump((step * 10.0) as f64);
    }

    SliderA11y {
        current,
        min,
        max,
        decoration,
    }
}
