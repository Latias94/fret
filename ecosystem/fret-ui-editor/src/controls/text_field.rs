//! Reusable editor text field control.
//!
//! v1 scope:
//! - single-line input (`TextInput`)
//! - optional multiline mode (`TextArea`) with a minimum height
//! - optional clear affordance

use std::sync::Arc;

use fret_core::{Axis, Edges, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, SizeStyle, TextAreaProps, TextInputProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::{
    resolve_editor_text_area_field_style, resolve_editor_text_field_style,
};
use crate::primitives::icons::editor_icon;
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

        let density = {
            let theme = Theme::global(&*cx.app);
            EditorDensity::resolve(theme)
        };

        let enabled_for_paint = self.options.enabled;
        let clear_enabled = self.options.clear_button && has_value && enabled_for_paint;

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
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            };
            props.enabled = self.options.enabled;
            props.focusable = self.options.focusable;
            props.a11y_label = self.options.a11y_label.clone();
            props.test_id = self.options.test_id.clone();
            props.chrome = chrome;
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
                    height: Length::Auto,
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
            props.chrome = chrome;
            props.text_style = TextStyle {
                line_height: Some(density.row_height),
                ..text_style
            };

            let el = cx.text_input(props);
            el
        };

        let clear = clear_enabled.then(|| {
            let model_for_clear = self.model.clone();
            let mut el = cx.pressable(
                PressableProps {
                    enabled: enabled_for_paint,
                    focusable: false,
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
                            corner_radii: fret_core::Corners::all(Px(6.0)),
                            ..Default::default()
                        },
                        move |cx| {
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
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
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
                    )]
                },
            );

            if let Some(test_id) = self.options.clear_test_id.as_ref() {
                el = el.test_id(test_id.clone());
            }
            el
        });

        let root = cx.flex(
            FlexProps {
                layout: self.options.layout,
                direction: Axis::Horizontal,
                gap: Px(4.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| {
                let mut out = vec![input];
                if let Some(clear) = clear {
                    out.push(clear);
                }
                out
            },
        );

        root
    }
}
