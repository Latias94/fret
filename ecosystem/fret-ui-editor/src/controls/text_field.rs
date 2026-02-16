//! Reusable editor text field control.
//!
//! v1 scope:
//! - single-line input (`TextInput`)
//! - optional multiline mode (`TextArea`) with a minimum height
//! - optional clear affordance

use std::sync::Arc;

use fret_core::{Edges, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle,
    TextAreaProps, TextInputProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::chrome::{
    resolve_editor_text_area_field_style, resolve_editor_text_field_style,
};
use crate::primitives::icons::editor_icon;
use crate::primitives::input_group::{
    editor_input_group_divider, editor_input_group_frame, editor_input_group_inset,
    editor_input_group_row,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::EditorFrameState;
use crate::primitives::visuals::{editor_icon_button_bg, editor_icon_button_border};

#[derive(Debug, Clone)]
pub struct TextFieldOptions {
    pub layout: LayoutStyle,
    pub size: Size,
    pub placeholder: Option<Arc<str>>,
    pub enabled: bool,
    pub focusable: bool,
    pub clear_button: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub clear_test_id: Option<Arc<str>>,

    /// When true, uses `TextArea` (multiline) instead of `TextInput`.
    pub multiline: bool,
    /// Minimum height for multiline text areas.
    pub min_height: Option<Px>,
}

impl Default for TextFieldOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            size: Size::Small,
            placeholder: None,
            enabled: true,
            focusable: true,
            clear_button: false,
            a11y_label: None,
            test_id: None,
            clear_test_id: None,
            multiline: false,
            min_height: None,
        }
    }
}

#[derive(Clone)]
pub struct TextField {
    model: Model<String>,
    options: TextFieldOptions,
}

impl TextField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            options: TextFieldOptions::default(),
        }
    }

    pub fn options(mut self, options: TextFieldOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let has_value = cx
            .read_model_ref(&self.model, Invalidation::Layout, |s| !s.is_empty())
            .unwrap_or(false);

        let enabled_for_paint = self.options.enabled;
        let clear_enabled = self.options.clear_button && has_value && enabled_for_paint;

        let (density, frame_chrome) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            (style.density, style.frame_chrome(self.options.size))
        };

        let input = if self.options.multiline {
            let (chrome, text_style) = {
                let theme = Theme::global(&*cx.app);
                resolve_editor_text_area_field_style(
                    theme,
                    self.options.size,
                    &ChromeRefinement::default(),
                )
            };

            let mut props = TextAreaProps::new(self.model.clone());
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };
            props.enabled = self.options.enabled;
            props.focusable = self.options.focusable;
            props.a11y_label = self.options.a11y_label.clone();
            props.test_id = self.options.test_id.clone();

            // Joined field: the frame is drawn by the input group. Keep the inner text area
            // transparent and borderless to avoid double chrome.
            let mut joined_chrome = chrome;
            joined_chrome.padding_x = Px(0.0);
            joined_chrome.padding_y = Px(0.0);
            joined_chrome.border = Edges::all(Px(0.0));
            joined_chrome.corner_radii = fret_core::Corners::all(Px(0.0));
            joined_chrome.background = fret_core::Color {
                a: 0.0,
                ..joined_chrome.background
            };
            joined_chrome.border_color = fret_core::Color {
                a: 0.0,
                ..joined_chrome.border_color
            };
            joined_chrome.focus_ring = None;

            props.chrome = joined_chrome;
            props.text_style = text_style;
            props.min_height = self.options.min_height.unwrap_or_else(|| {
                let baseline = Px(80.0);
                let dense = Px(density.row_height.0 * 3.0);
                Px(baseline.0.max(dense.0))
            });

            cx.text_area(props)
        } else {
            let (chrome, text_style) = {
                let theme = Theme::global(&*cx.app);
                resolve_editor_text_field_style(
                    theme,
                    self.options.size,
                    &ChromeRefinement::default(),
                )
            };

            let mut props = TextInputProps::new(self.model.clone());
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    min_height: Some(density.row_height),
                    ..Default::default()
                },
                ..Default::default()
            };
            props.enabled = self.options.enabled;
            props.focusable = self.options.focusable;
            props.placeholder = self.options.placeholder.clone();
            props.a11y_label = self.options.a11y_label.clone();
            props.test_id = self.options.test_id.clone();

            // Joined field: the frame is drawn by the input group. Keep the inner text input
            // transparent and borderless to avoid double chrome.
            let mut joined_chrome = chrome;
            joined_chrome.padding = Edges::all(Px(0.0));
            joined_chrome.border = Edges::all(Px(0.0));
            joined_chrome.corner_radii = fret_core::Corners::all(Px(0.0));
            joined_chrome.background = fret_core::Color {
                a: 0.0,
                ..joined_chrome.background
            };
            joined_chrome.border_color = fret_core::Color {
                a: 0.0,
                ..joined_chrome.border_color
            };
            joined_chrome.border_color_focused = joined_chrome.border_color;
            joined_chrome.focus_ring = None;

            props.chrome = joined_chrome;
            props.text_style = TextStyle {
                line_height: Some(density.row_height),
                ..text_style
            };

            cx.text_input(props)
        };

        let is_focused = cx.is_focused_element(input.id);

        let clear = clear_enabled.then(|| {
            let model_for_clear = self.model.clone();
            let mut el = cx.pressable(
                PressableProps {
                    enabled: enabled_for_paint,
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(density.hit_thickness),
                            height: Length::Px(density.row_height),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    a11y: PressableA11y {
                        label: Some(Arc::from("Clear text")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, st| {
                    let on_activate: OnActivate = Arc::new({
                        let model_for_clear = model_for_clear.clone();
                        move |host, action_cx, _reason: ActivateReason| {
                            let _ = host.models_mut().update(&model_for_clear, |s| s.clear());
                            host.request_redraw(action_cx.window);
                        }
                    });
                    cx.pressable_add_on_activate(on_activate);

                    let theme = Theme::global(&*cx.app);
                    let hovered = st.hovered || st.hovered_raw;
                    let pressed = st.pressed;
                    let bg = editor_icon_button_bg(theme, enabled_for_paint, hovered, pressed);
                    let border =
                        editor_icon_button_border(theme, enabled_for_paint, hovered, pressed);
                    let border_width = if border.is_some() { Px(1.0) } else { Px(0.0) };

                    vec![cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            background: bg,
                            border: Edges::all(border_width),
                            border_color: border,
                            corner_radii: fret_core::Corners::all(Px(0.0)),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![editor_icon(
                                cx,
                                density,
                                fret_icons::ids::ui::CLOSE,
                                Some(Px(12.0)),
                            )]
                        },
                    )]
                },
            );
            if let Some(test_id) = self.options.clear_test_id.as_ref() {
                el = el.test_id(test_id.clone());
            }
            el
        });

        let divider = frame_chrome.border;
        let input = editor_input_group_inset(cx, frame_chrome.padding, input);

        editor_input_group_frame(
            cx,
            self.options.layout,
            density,
            frame_chrome,
            EditorFrameState {
                enabled: enabled_for_paint,
                hovered: false,
                pressed: false,
                focused: is_focused,
                open: false,
            },
            move |cx, _visuals| {
                let mut out = vec![input];
                if let Some(clear) = clear {
                    out.push(editor_input_group_divider(cx, divider));
                    out.push(clear);
                }
                vec![editor_input_group_row(cx, Px(0.0), out)]
            },
        )
    }
}
