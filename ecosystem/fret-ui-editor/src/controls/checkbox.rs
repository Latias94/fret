//! Editor checkbox control (supports tri-state/mixed values).
//!
//! This intentionally keeps styling conservative and token-driven, so higher-level style adapters
//! can override tokens without pulling in a full design system dependency.

use std::sync::Arc;

use fret_core::{Axis, Corners, Edges, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableProps, RingPlacement, RingStyle, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::ColorRef;
use fret_ui_kit::primitives::checkbox::{
    CheckedState, checkbox_a11y, checked_state_from_optional_bool, toggle_optional_bool,
};

use crate::primitives::EditorTokenKeys;
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState, EditorWidgetVisuals};

#[derive(Debug, Clone)]
pub struct CheckboxOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for CheckboxOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Auto,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
enum CheckboxModel {
    Bool(Model<bool>),
    OptionalBool(Model<Option<bool>>),
}

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

        let bg_unchecked = theme
            .color_by_key("component.checkbox.bg")
            .or_else(|| theme.color_by_key("component.input.bg"))
            .unwrap_or(frame_chrome.bg);
        let bg_checked = theme.color_token("primary");
        let fg_checked = theme.color_token("primary-foreground");
        let ring_color = theme
            .color_by_key("ring")
            .unwrap_or_else(|| theme.color_token("primary"));

        let checked_state = match &self.model {
            CheckboxModel::Bool(model) => {
                let v = cx
                    .get_model_copied(model, Invalidation::Paint)
                    .unwrap_or(false);
                if v {
                    CheckedState::Checked
                } else {
                    CheckedState::Unchecked
                }
            }
            CheckboxModel::OptionalBool(model) => {
                let v = cx
                    .get_model_cloned(model, Invalidation::Paint)
                    .unwrap_or(None);
                checked_state_from_optional_bool(v)
            }
        };

        let icon_id = match checked_state {
            CheckedState::Checked => Some(fret_icons::ids::ui::CHECK),
            CheckedState::Indeterminate => Some(fret_icons::ids::ui::MINUS),
            CheckedState::Unchecked => None,
        };

        let icon_px = Px((checkbox_size.0 - 4.0).max(8.0));

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

        let (model_for_activate, enabled_for_activate) = (self.model.clone(), self.options.enabled);
        let on_activate: OnActivate = Arc::new(move |host, action_cx: ActionCx, _reason| {
            if !enabled_for_activate {
                return;
            }

            match &model_for_activate {
                CheckboxModel::Bool(model) => {
                    let _ = host.models_mut().update(model, |v| *v = !*v);
                }
                CheckboxModel::OptionalBool(model) => {
                    let _ = host
                        .models_mut()
                        .update(model, |v| *v = toggle_optional_bool(*v));
                }
            }
            host.request_redraw(action_cx.window);
        });

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
                    color: ring_color,
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
                    bg_unchecked,
                    bg_checked,
                    fg_checked,
                    checked_state != CheckedState::Unchecked,
                );

                let box_el = cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Px(checkbox_size),
                                height: Length::Px(checkbox_size),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        background: Some(visuals.bg),
                        border: Edges::all(frame_chrome.border_width),
                        border_color: Some(visuals.border),
                        corner_radii: Corners::all(checkbox_radius),
                        ..Default::default()
                    },
                    move |cx| {
                        let Some(icon) = icon_id else {
                            return vec![];
                        };

                        vec![cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: Axis::Horizontal,
                                gap: SpacingLength::Px(Px(0.0)),
                                padding: Edges::all(Px(0.0)).into(),
                                justify: MainAlign::Center,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            move |cx| {
                                vec![fret_ui_kit::declarative::icon::icon_with(
                                    cx,
                                    icon,
                                    Some(icon_px),
                                    Some(ColorRef::Color(visuals.icon)),
                                )]
                            },
                        )]
                    },
                );

                vec![cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: Axis::Horizontal,
                        gap: SpacingLength::Px(Px(0.0)),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| vec![box_el],
                )]
            },
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
    }
}
