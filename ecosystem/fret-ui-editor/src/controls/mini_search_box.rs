//! Minimal search box control for editor filtering surfaces.
//!
//! v1 scope:
//! - single-line text input bound to a `Model<String>`
//! - optional clear button
//! - stable `test_id` anchors

use std::sync::Arc;

use fret_core::{Axis, Edges, KeyCode, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, PressableA11y,
    PressableProps, SizeStyle, TextInputProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::EditorDensity;
use crate::primitives::icons::editor_icon;

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

        let (density, chrome, text_style) = {
            let theme = Theme::global(&*cx.app);
            let density = EditorDensity::resolve(theme);
            let resolved = resolve_input_chrome(
                theme,
                self.options.size,
                &ChromeRefinement::default(),
                InputTokenKeys {
                    padding_x: Some("component.text_field.padding_x"),
                    padding_y: Some("component.text_field.padding_y"),
                    min_height: Some("component.text_field.min_height"),
                    radius: Some("component.text_field.radius"),
                    border_width: Some("component.text_field.border_width"),
                    bg: Some("component.text_field.bg"),
                    border: Some("component.text_field.border"),
                    border_focus: Some("component.text_field.border_focus"),
                    fg: Some("component.text_field.fg"),
                    text_px: Some("component.text_field.text_px"),
                    selection: Some("component.text_field.selection"),
                },
            );

            let mut chrome = fret_ui::TextInputStyle::from_theme(theme.snapshot());
            chrome.padding = resolved.padding;
            chrome.corner_radii = fret_core::Corners::all(resolved.radius);
            chrome.border = Edges::all(resolved.border_width);
            chrome.background = resolved.background;
            chrome.border_color = resolved.border_color;
            chrome.border_color_focused = resolved.border_color_focused;
            chrome.text_color = resolved.text_color;
            chrome.caret_color = resolved.text_color;
            chrome.selection_color = resolved.selection_color;

            let font_line_height = theme
                .metric_by_key("font.line_height")
                .unwrap_or_else(|| theme.metric_token("font.line_height"));
            let text_style = TextStyle {
                size: resolved.text_px,
                line_height: Some(font_line_height),
                ..Default::default()
            };
            (density, chrome, text_style)
        };

        let mut input_props = TextInputProps::new(self.model.clone());
        input_props.layout = LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                min_height: Some(density.row_height),
                ..Default::default()
            },
            ..Default::default()
        };
        input_props.enabled = self.options.enabled;
        input_props.focusable = self.options.focusable;
        input_props.placeholder = self.options.placeholder.clone();
        input_props.test_id = self.options.test_id.clone();
        input_props.chrome = chrome;
        input_props.text_style = text_style;

        let input = cx.text_input(input_props);

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
                    enabled: self.options.enabled,
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

        // Do not force focus ring policies here; callers can wrap if needed.
        root
    }
}
