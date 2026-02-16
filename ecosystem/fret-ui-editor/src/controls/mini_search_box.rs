//! Minimal search box control for editor filtering surfaces.
//!
//! v1 scope:
//! - single-line text input bound to a `Model<String>`
//! - optional clear button
//! - stable `test_id` anchors

use std::sync::Arc;

use fret_core::{Axis, Edges, KeyCode, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, PressableA11y,
    PressableProps, SizeStyle, TextInputProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::chrome::resolve_editor_text_field_style;
use crate::primitives::icons::editor_icon;
use crate::primitives::input_group::{
    editor_input_group_divider, editor_input_group_frame, editor_input_group_inset,
    editor_input_group_row,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::EditorFrameState;
use crate::primitives::visuals::{editor_icon_button_bg, editor_icon_button_border};

#[derive(Debug, Clone)]
pub struct MiniSearchBoxOptions {
    pub layout: LayoutStyle,
    pub size: Size,
    pub placeholder: Option<Arc<str>>,
    pub enabled: bool,
    pub focusable: bool,
    pub test_id: Option<Arc<str>>,
    pub clear_test_id: Option<Arc<str>>,
}

impl Default for MiniSearchBoxOptions {
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
            placeholder: Some(Arc::from("Search…")),
            enabled: true,
            focusable: true,
            test_id: None,
            clear_test_id: None,
        }
    }
}

#[derive(Clone)]
pub struct MiniSearchBox {
    model: Model<String>,
    options: MiniSearchBoxOptions,
}

impl MiniSearchBox {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            options: MiniSearchBoxOptions::default(),
        }
    }

    pub fn options(mut self, options: MiniSearchBoxOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let has_value = cx
            .read_model_ref(&self.model, Invalidation::Layout, |s| !s.is_empty())
            .unwrap_or(false);
        let enabled_for_paint = self.options.enabled;

        let (density, frame_chrome, chrome, text_style) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let density = style.density;
            let frame_chrome = style.frame_chrome_small();
            let (chrome, text_style) = resolve_editor_text_field_style(
                theme,
                self.options.size,
                &ChromeRefinement::default(),
            );
            (density, frame_chrome, chrome, text_style)
        };

        let mut input_props = TextInputProps::new(self.model.clone());
        input_props.layout = LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                min_height: Some(density.row_height),
                ..Default::default()
            },
            ..Default::default()
        };
        input_props.enabled = self.options.enabled;
        input_props.focusable = self.options.focusable;
        input_props.placeholder = self.options.placeholder.clone();
        input_props.test_id = None;

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

        input_props.chrome = joined_chrome;
        input_props.text_style = text_style;

        let input = cx.text_input(input_props);
        let is_focused = cx.is_focused_element(input.id);

        // Basic affordance: Escape clears if there is text.
        let model_for_key = self.model.clone();
        cx.key_add_on_key_down_capture_for(
            input.id,
            Arc::new(move |host, action_cx: ActionCx, down| {
                if down.key != KeyCode::Escape {
                    return false;
                }
                let had_value = host
                    .models_mut()
                    .read(&model_for_key, |s| !s.is_empty())
                    .unwrap_or(false);
                if !had_value {
                    return false;
                }
                let _ = host.models_mut().update(&model_for_key, |s| s.clear());
                host.request_redraw(action_cx.window);
                true
            }),
        );

        let clear = has_value.then(|| {
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
                        label: Some(Arc::from("Clear search")),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, _st| {
                    let on_activate: OnActivate = Arc::new({
                        let model_for_clear = model_for_clear.clone();
                        move |host, action_cx, _reason: ActivateReason| {
                            let _ = host.models_mut().update(&model_for_clear, |s| s.clear());
                            host.request_redraw(action_cx.window);
                        }
                    });
                    cx.pressable_add_on_activate(on_activate);

                    let theme = Theme::global(&*cx.app);
                    let hovered = _st.hovered || _st.hovered_raw;
                    let pressed = _st.pressed;
                    let bg = editor_icon_button_bg(theme, enabled_for_paint, hovered, pressed);
                    let border =
                        editor_icon_button_border(theme, enabled_for_paint, hovered, pressed);
                    let border_width = if border.is_some() { Px(1.0) } else { Px(0.0) };

                    vec![cx.container(
                        fret_ui::element::ContainerProps {
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

        let divider = frame_chrome.border;
        let input = editor_input_group_inset(cx, frame_chrome.padding, input);

        let mut root = editor_input_group_frame(
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
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            root = root.test_id(test_id.clone());
        }

        // Do not force focus ring policies here; callers can wrap if needed.
        root
    }
}
