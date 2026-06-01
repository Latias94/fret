//! Editor checkbox control (supports tri-state/mixed values).
//!
//! This intentionally keeps styling conservative and token-driven, so higher-level style adapters
//! can override tokens without pulling in a full design system dependency.

mod chrome;
mod indicator;
mod model;
mod options;

use fret_core::{Corners, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, Length, PressableProps, RingPlacement, RingStyle};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::checked_state::CheckedState;
use fret_ui_kit::primitives::checkbox::checkbox_a11y;

use crate::primitives::EditorTokenKeys;
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState, EditorWidgetVisuals};
use chrome::resolve_checkbox_chrome;
use indicator::checkbox_indicator_element;
use model::{CheckboxModel, checkbox_checked_state, checkbox_on_activate};

pub use options::CheckboxOptions;

#[derive(Clone)]
pub struct Checkbox {
    model: CheckboxModel,
    options: CheckboxOptions,
}

impl Checkbox {
    pub fn new(model: Model<bool>) -> Self {
        Self {
            model: CheckboxModel::Bool(model),
            options: CheckboxOptions::default(),
        }
    }

    /// Creates a checkbox bound to an optional boolean model.
    ///
    /// This maps `None` to the indeterminate/mixed outcome, aligned with Radix.
    pub fn new_optional(model: Model<Option<bool>>) -> Self {
        Self {
            model: CheckboxModel::OptionalBool(model),
            options: CheckboxOptions::default(),
        }
    }

    pub fn options(mut self, options: CheckboxOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let style = EditorStyle::resolve(theme);
        let density = style.density;
        let frame_chrome = style.frame_chrome_small();

        let checkbox_size = theme
            .metric_by_key(EditorTokenKeys::CHECKBOX_SIZE)
            .or_else(|| theme.metric_by_key("component.checkbox.size"))
            .unwrap_or(Px(16.0));
        let checkbox_radius = theme
            .metric_by_key(EditorTokenKeys::CHECKBOX_RADIUS)
            .or_else(|| theme.metric_by_key("component.checkbox.radius"))
            .unwrap_or(Px(4.0));

        let chrome = resolve_checkbox_chrome(theme, frame_chrome.bg);

        let checked_state = checkbox_checked_state(cx, &self.model);

        let mut layout = self.options.layout;
        if layout.size.width == Length::Auto {
            layout.size.width = Length::Px(density.hit_thickness);
        }
        if layout.size.height == Length::Auto {
            layout.size.height = Length::Px(density.hit_thickness);
        }
        if layout.size.min_height.is_none() {
            layout.size.min_height = Some(Length::Px(density.hit_thickness));
        }

        let focus_ring_bounds = match (layout.size.width, layout.size.height) {
            (Length::Px(w), Length::Px(h)) => {
                let box_size = Px(checkbox_size.0.min(w.0).min(h.0));
                let pad_x = ((w.0 - box_size.0) * 0.5).max(0.0);
                let pad_y = ((h.0 - box_size.0) * 0.5).max(0.0);
                Some(Rect::new(
                    Point::new(Px(pad_x), Px(pad_y)),
                    Size::new(box_size, box_size),
                ))
            }
            _ => None,
        };

        let a11y = checkbox_a11y(self.options.a11y_label.clone(), checked_state);

        let on_activate = checkbox_on_activate(self.model.clone(), self.options.enabled);

        let enabled_for_paint = self.options.enabled;
        let mut el = cx.pressable(
            PressableProps {
                layout,
                enabled: self.options.enabled,
                focusable: self.options.focusable,
                focus_ring: Some(RingStyle {
                    placement: RingPlacement::Outset,
                    width: Px(2.0),
                    offset: Px(2.0),
                    color: chrome.ring_color,
                    offset_color: None,
                    corner_radii: Corners::all(checkbox_radius),
                }),
                focus_ring_bounds,
                a11y,
                ..Default::default()
            },
            move |cx, st| {
                cx.pressable_add_on_activate(on_activate.clone());

                let theme = Theme::global(&*cx.app);
                let visuals = EditorWidgetVisuals::new(theme).selection_frame_visuals(
                    frame_chrome,
                    EditorFrameState {
                        enabled: enabled_for_paint,
                        hovered: st.hovered || st.hovered_raw,
                        pressed: st.pressed,
                        focused: st.focused,
                        open: false,
                        semantic: EditorFrameSemanticState::default(),
                    },
                    chrome.bg_unchecked,
                    chrome.bg_checked,
                    chrome.fg_checked,
                    checked_state != CheckedState::Unchecked,
                );

                vec![checkbox_indicator_element(
                    cx,
                    checked_state,
                    visuals,
                    checkbox_size,
                    checkbox_radius,
                    frame_chrome.border_width,
                )]
            },
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
    }
}
