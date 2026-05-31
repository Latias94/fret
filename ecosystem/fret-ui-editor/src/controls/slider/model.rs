use std::sync::Arc;

use crate::controls::numeric_input::NumericInputSelectionBehavior;
use fret_core::{PointerId, Px};
use fret_ui::GlobalElementId;
use fret_ui::element::{
    FlexItemStyle, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct SliderOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
    pub selection_behavior: NumericInputSelectionBehavior,
    pub clamp: bool,
    /// Quantize to a step size in value space (e.g. `0.01` for normalized floats).
    pub step: Option<f64>,
    pub show_value: bool,
    pub value_width: Px,
    pub allow_typing: bool,
    /// Explicit identity source for internal state (drag/typing focus restore).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple sliders from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub a11y_label: Option<Arc<str>>,
}

impl Default for SliderOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            prefix: None,
            suffix: None,
            selection_behavior: NumericInputSelectionBehavior::ReplaceAllOnFocus,
            clamp: true,
            step: None,
            show_value: true,
            value_width: Px(52.0),
            allow_typing: true,
            id_source: None,
            test_id: None,
            a11y_label: None,
        }
    }
}

pub(super) fn compose_affixed_value_text(
    value: &Arc<str>,
    prefix: Option<&Arc<str>>,
    suffix: Option<&Arc<str>>,
) -> Arc<str> {
    match (prefix, suffix) {
        (None, None) => value.clone(),
        _ => {
            let mut out = String::new();
            if let Some(prefix) = prefix {
                out.push_str(prefix);
            }
            out.push_str(value);
            if let Some(suffix) = suffix {
                out.push_str(suffix);
            }
            Arc::from(out)
        }
    }
}

pub(super) fn hidden_layout(mut layout: LayoutStyle) -> LayoutStyle {
    layout.size = SizeStyle {
        width: Length::Px(Px(0.0)),
        height: Length::Px(Px(0.0)),
        min_width: Some(Length::Px(Px(0.0))),
        min_height: Some(Length::Px(Px(0.0))),
        ..Default::default()
    };
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
        ..Default::default()
    };
    layout.overflow = Overflow::Clip;
    layout
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SliderMode {
    Slide,
    Typing,
}

#[derive(Debug)]
pub(super) struct SliderState {
    pub(super) mode: SliderMode,
    pub(super) slider_id: Option<GlobalElementId>,
    pub(super) dragging: bool,
    pub(super) pointer_id: Option<PointerId>,
}

impl Default for SliderState {
    fn default() -> Self {
        Self {
            mode: SliderMode::Slide,
            slider_id: None,
            dragging: false,
            pointer_id: None,
        }
    }
}
