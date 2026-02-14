//! Numeric text input control with editor-style commit/cancel outcomes.
//!
//! This control is intentionally lightweight:
//! - it owns a per-element draft `Model<String>` for text editing,
//! - commits parsed values on Enter,
//! - validates on commit (optional),
//! - cancels (reverts to formatted current value) on Escape,
//! - renders an inline error message when commit is rejected.

use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Edges, KeyCode, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiFocusActionHost};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, TextInputProps,
    TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::{EditorDensity, EditorTokenKeys};

#[derive(Debug, Clone)]
pub struct NumericInputOptions {
    pub layout: LayoutStyle,
    pub size: Size,
    pub placeholder: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub enabled: bool,
    pub focusable: bool,
}

impl Default for NumericInputOptions {
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
            size: Size::default(),
            placeholder: None,
            test_id: None,
            enabled: true,
            focusable: true,
        }
    }
}

pub type NumericFormatFn<T> = Arc<dyn Fn(T) -> Arc<str> + Send + Sync + 'static>;
pub type NumericParseFn<T> = Arc<dyn Fn(&str) -> Option<T> + Send + Sync + 'static>;
pub type NumericValidateFn<T> = Arc<dyn Fn(T) -> Option<Arc<str>> + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericInputOutcome {
    Committed,
    Canceled,
}

pub type OnNumericInputOutcome =
    Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, NumericInputOutcome) + 'static>;

#[derive(Clone)]
pub struct NumericInput<T> {
    model: Model<T>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    on_outcome: Option<OnNumericInputOutcome>,
    options: NumericInputOptions,
}

impl<T> NumericInput<T>
where
    T: Copy + Default + 'static,
{
    pub fn new(model: Model<T>, format: NumericFormatFn<T>, parse: NumericParseFn<T>) -> Self {
        Self {
            model,
            format,
            parse,
            validate: None,
            on_outcome: None,
            options: NumericInputOptions::default(),
        }
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn on_outcome(mut self, on_outcome: Option<OnNumericInputOutcome>) -> Self {
        self.on_outcome = on_outcome;
        self
    }

    pub fn options(mut self, options: NumericInputOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let draft = draft_model(cx);
        let error = error_model(cx);
        let current_value = cx
            .get_model_copied(&self.model, Invalidation::Paint)
            .unwrap_or_default();
        let current_text = (self.format)(current_value);

        let density = {
            let theme = Theme::global(&*cx.app);
            EditorDensity::resolve(theme)
        };

        let (chrome, text_style) = {
            let theme = Theme::global(&*cx.app);
            let resolved = resolve_input_chrome(
                theme,
                self.options.size,
                &ChromeRefinement::default(),
                InputTokenKeys {
                    bg: Some("component.input.bg"),
                    border: Some("component.input.border"),
                    border_focus: Some("component.input.border_focus"),
                    fg: Some("component.input.fg"),
                    selection: Some("component.input.selection"),
                    ..InputTokenKeys::none()
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

            (chrome, text_style)
        };

        let mut props = TextInputProps::new(draft.clone());
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
        props.test_id = self.options.test_id.clone();
        props.chrome = chrome;
        props.text_style = text_style.clone();

        let input = cx.text_input(props);
        let input_id = input.id;
        let is_focused = cx.is_focused_element(input_id);

        if !is_focused {
            let _ = cx
                .app
                .models_mut()
                .update(&draft, |s| *s = current_text.as_ref().to_string());
            let _ = cx.app.models_mut().update(&error, |e| *e = None);
            cx.with_state(
                || String::new(),
                |last| {
                    *last = current_text.as_ref().to_string();
                },
            );
        }

        let model_for_key = self.model.clone();
        let draft_for_key = draft.clone();
        let error_for_key = error.clone();
        let parse_for_key = self.parse.clone();
        let format_for_key = self.format.clone();
        let validate_for_key = self.validate.clone();
        let on_outcome_for_key = self.on_outcome.clone();
        cx.key_add_on_key_down_capture_for(
            input_id,
            Arc::new(move |host, action_cx: ActionCx, down| match down.key {
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    let text = host
                        .models_mut()
                        .read(&draft_for_key, |s| s.clone())
                        .unwrap_or_default();
                    if let Some(v) = (parse_for_key)(&text) {
                        if let Some(validate) = validate_for_key.as_ref() {
                            if let Some(msg) = validate(v) {
                                let _ =
                                    host.models_mut().update(&error_for_key, |e| *e = Some(msg));
                                host.request_redraw(action_cx.window);
                                return true;
                            }
                        }

                        let _ = host.models_mut().update(&model_for_key, |m| *m = v);
                        let formatted = (format_for_key)(v);
                        let _ = host
                            .models_mut()
                            .update(&draft_for_key, |s| *s = formatted.as_ref().to_string());
                        let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                        if let Some(cb) = on_outcome_for_key.as_ref() {
                            cb(host, action_cx, NumericInputOutcome::Committed);
                        }
                    } else {
                        let _ = host
                            .models_mut()
                            .update(&error_for_key, |e| *e = Some(Arc::from("Invalid number")));
                    }
                    host.request_redraw(action_cx.window);
                    true
                }
                KeyCode::Escape => {
                    let current = host
                        .models_mut()
                        .get_copied(&model_for_key)
                        .unwrap_or_default();
                    let formatted = (format_for_key)(current);
                    let _ = host
                        .models_mut()
                        .update(&draft_for_key, |s| *s = formatted.as_ref().to_string());
                    let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                    if let Some(cb) = on_outcome_for_key.as_ref() {
                        cb(host, action_cx, NumericInputOutcome::Canceled);
                    }
                    host.request_redraw(action_cx.window);
                    true
                }
                _ => false,
            }),
        );

        if is_focused {
            let draft_text = cx
                .get_model_cloned(&draft, Invalidation::Paint)
                .unwrap_or_default();
            let changed = cx.with_state(
                || String::new(),
                |last| {
                    if *last == draft_text {
                        false
                    } else {
                        *last = draft_text;
                        true
                    }
                },
            );
            if changed {
                let _ = cx.app.models_mut().update(&error, |e| *e = None);
            }
        }

        let error_msg = cx
            .get_model_cloned(&error, Invalidation::Paint)
            .unwrap_or(None);

        let error_color = {
            let theme = Theme::global(&*cx.app);
            theme
                .color_by_key(EditorTokenKeys::NUMERIC_ERROR_FG)
                .unwrap_or_else(|| theme.color_token("destructive"))
        };
        let error_el = error_msg.map(|msg| {
            cx.text_props(TextProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: msg,
                style: Some(TextStyle {
                    size: text_style.size,
                    line_height: text_style.line_height,
                    ..Default::default()
                }),
                color: Some(error_color),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                align: TextAlign::Start,
            })
        });

        let mut layout = self.options.layout;
        if layout.size.min_height.is_none() {
            layout.size.min_height = Some(density.row_height);
        }

        cx.flex(
            FlexProps {
                layout,
                direction: Axis::Vertical,
                gap: Px(4.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| {
                let mut out = vec![input];
                if let Some(error) = error_el {
                    out.push(error);
                }
                out
            },
        )
    }
}

fn draft_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let draft = cx.with_state(|| None::<Model<String>>, |st| st.clone());
    match draft {
        Some(draft) => draft,
        None => {
            let draft = cx.app.models_mut().insert(String::new());
            cx.with_state(
                || None::<Model<String>>,
                |st| {
                    if st.is_none() {
                        *st = Some(draft.clone());
                    }
                },
            );
            draft
        }
    }
}

fn error_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    let m = cx.with_state(|| None::<Model<Option<Arc<str>>>>, |st| st.clone());
    match m {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(
                || None::<Model<Option<Arc<str>>>>,
                |st| {
                    if st.is_none() {
                        *st = Some(m.clone());
                    }
                },
            );
            m
        }
    }
}
