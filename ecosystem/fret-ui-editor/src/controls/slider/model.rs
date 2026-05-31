use std::sync::Arc;

use fret_core::{PointerId, Px};
use fret_ui::GlobalElementId;
use fret_ui::element::{InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle};

#[cfg(test)]
mod tests;

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
