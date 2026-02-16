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
    AnyElement, ContainerProps, HoverRegionProps, LayoutStyle, Length, PressableA11y,
    PressableProps, SizeStyle, TextAreaProps, TextInputProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::chrome::{
    joined_text_area_style, joined_text_input_style, resolve_editor_text_area_field_style,
    resolve_editor_text_field_style,
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
        let TextField { model, options } = self;

        let has_value = cx
            .read_model_ref(&model, Invalidation::Layout, |s| !s.is_empty())
            .unwrap_or(false);

        let enabled_for_paint = options.enabled;
        let clear_enabled = options.clear_button && has_value && enabled_for_paint;

        let (density, frame_chrome) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            (style.density, style.frame_chrome(options.size))
        };

        cx.hover_region(
            HoverRegionProps {
                layout: options.layout,
            },
            move |cx, hovered| {
                let input = if options.multiline {
                    let (chrome, text_style) = {
                        let theme = Theme::global(&*cx.app);
                        resolve_editor_text_area_field_style(
                            theme,
                            options.size,
                            &ChromeRefinement::default(),
                        )
                    };

                    let mut props = TextAreaProps::new(model.clone());
                    props.layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    props.enabled = options.enabled;
                    props.focusable = options.focusable;
                    props.a11y_label = options.a11y_label.clone();
                    props.test_id = options.test_id.clone();

                    props.chrome = joined_text_area_style(chrome);
                    props.text_style = text_style;
                    props.min_height = options.min_height.unwrap_or_else(|| {
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
                            options.size,
                            &ChromeRefinement::default(),
                        )
                    };

                    let mut props = TextInputProps::new(model.clone());
                    props.layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            min_height: Some(density.row_height),
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    props.enabled = options.enabled;
                    props.focusable = options.focusable;
                    props.placeholder = options.placeholder.clone();
                    props.a11y_label = options.a11y_label.clone();
                    props.test_id = options.test_id.clone();

                    props.chrome = joined_text_input_style(chrome);
                    props.text_style = TextStyle {
                        line_height: Some(density.row_height),
                        ..text_style
                    };

                    cx.text_input(props)
                };

                let is_focused = cx.is_focused_element(input.id);

                let clear = clear_enabled.then(|| {
                    let model_for_clear = model.clone();
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
                                    let _ =
                                        host.models_mut().update(&model_for_clear, |s| s.clear());
                                    host.request_redraw(action_cx.window);
                                }
                            });
                            cx.pressable_add_on_activate(on_activate);

                            let theme = Theme::global(&*cx.app);
                            let hovered = st.hovered || st.hovered_raw;
                            let pressed = st.pressed;
                            let bg =
                                editor_icon_button_bg(theme, enabled_for_paint, hovered, pressed);
                            let border = editor_icon_button_border(
                                theme,
                                enabled_for_paint,
                                hovered,
                                pressed,
                            );
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
                    if let Some(test_id) = options.clear_test_id.as_ref() {
                        el = el.test_id(test_id.clone());
                    }
                    el
                });

                let divider = frame_chrome.border;
                let input = editor_input_group_inset(cx, frame_chrome.padding, input);

                let frame = editor_input_group_frame(
                    cx,
                    LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    density,
                    frame_chrome,
                    EditorFrameState {
                        enabled: enabled_for_paint,
                        hovered,
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
                );

                vec![frame]
            },
        )
    }
}
