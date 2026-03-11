//! Numeric text input control with editor-style commit/cancel outcomes.
//!
//! This control is intentionally lightweight:
//! - it owns a per-element draft `Model<String>` for text editing,
//! - commits parsed values on Enter,
//! - validates on commit (optional),
//! - cancels (reverts to formatted current value) on Escape,
//! - renders an inline error message when commit is rejected.

use std::panic::Location;
use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Color, Edges, KeyCode, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiFocusActionHost};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
    TextInputProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::EditorTokenKeys;
use crate::primitives::chrome::{joined_text_input_style, resolve_editor_text_field_style};
use crate::primitives::input_group::{
    EditorInputGroupFrameOverrides, editor_icon_segment,
    editor_joined_input_frame_segments_with_overrides, editor_text_segment,
};
use crate::primitives::style::EditorStyle;

#[derive(Debug, Clone)]
pub struct NumericInputOptions {
    pub layout: LayoutStyle,
    pub size: Size,
    pub placeholder: Option<Arc<str>>,
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
    /// Explicit identity source for internal state (draft/error models).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple numeric inputs from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub enabled: bool,
    pub focusable: bool,
    pub error_display: NumericInputErrorDisplay,
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
            prefix: None,
            suffix: None,
            id_source: None,
            test_id: None,
            enabled: true,
            focusable: true,
            error_display: NumericInputErrorDisplay::InlineTextAndIcon,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericInputErrorDisplay {
    None,
    InlineText,
    TrailingIcon,
    InlineTextAndIcon,
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

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
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
        let model_id = self.model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(
                ("fret-ui-editor.numeric_input", id_source, model_id),
                |cx| self.into_element_keyed(cx),
            )
        } else {
            cx.keyed(("fret-ui-editor.numeric_input", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model = self.model.clone();
        let parse = self.parse.clone();
        let format = self.format.clone();
        let validate = self.validate.clone();
        let on_outcome = self.on_outcome.clone();
        let options = self.options.clone();

        let draft = draft_model(cx);
        let error = error_model(cx);
        let current_value = cx
            .get_model_copied(&model, Invalidation::Paint)
            .unwrap_or_default();
        let current_text = (format)(current_value);

        let (density, frame_chrome, chrome, text_style) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let density = style.density;
            let frame_chrome = style.frame_chrome(options.size);
            let (chrome, text_style) =
                resolve_editor_text_field_style(theme, options.size, &ChromeRefinement::default());
            (density, frame_chrome, chrome, text_style)
        };

        let enabled_for_paint = options.enabled;
        let error_for_field = error.clone();
        let error_for_frame = error.clone();
        let error_for_trailing = error.clone();
        let text_style_for_field = text_style.clone();
        let placeholder = options.placeholder.clone();
        let focusable = options.focusable;
        let error_display = options.error_display;
        let prefix = options.prefix.clone();
        let suffix = options.suffix.clone();
        let error_icon_test_id = options
            .test_id
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{}.error", id.as_ref())));

        let frame_bg = frame_chrome.bg;
        let field = editor_joined_input_frame_segments_with_overrides(
            cx,
            LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            density,
            frame_chrome,
            enabled_for_paint,
            false,
            options.test_id.clone(),
            move |cx, _focused| {
                let has_error = cx
                    .get_model_cloned(&error_for_frame, Invalidation::Paint)
                    .unwrap_or(None)
                    .is_some();
                if has_error {
                    let theme = Theme::global(&*cx.app);
                    let error_border = theme
                        .color_by_key(EditorTokenKeys::NUMERIC_ERROR_BORDER)
                        .or_else(|| theme.color_by_key(EditorTokenKeys::NUMERIC_ERROR_FG))
                        .unwrap_or_else(|| theme.color_token("destructive"));
                    let error_bg = theme
                        .color_by_key(EditorTokenKeys::NUMERIC_ERROR_BG)
                        .unwrap_or_else(|| {
                            let mut out = mix(
                                frame_bg,
                                Color {
                                    a: 1.0,
                                    ..error_border
                                },
                                0.08,
                            );
                            out.a = 1.0;
                            out
                        });

                    EditorInputGroupFrameOverrides {
                        bg: Some(error_bg),
                        border: Some(error_border),
                    }
                } else {
                    EditorInputGroupFrameOverrides::none()
                }
            },
            move |cx| {
                let theme = Theme::global(&*cx.app);
                let affix_color = theme
                    .color_by_key("muted-foreground")
                    .or_else(|| theme.color_by_key("muted_foreground"))
                    .unwrap_or_else(|| theme.color_token("foreground"));
                let mut segments = Vec::new();

                if let Some(prefix) = prefix.clone() {
                    segments.push(editor_text_segment(
                        cx,
                        density,
                        frame_chrome.text_px,
                        prefix,
                        affix_color,
                        frame_chrome.padding,
                    ));
                }
                segments
            },
            move |cx| {
                let mut props = TextInputProps::new(draft.clone());
                props.layout = LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        min_height: Some(Length::Px(density.row_height)),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                props.enabled = enabled_for_paint;
                props.focusable = focusable;
                props.placeholder = placeholder.clone();
                props.test_id = None;
                props.chrome = joined_text_input_style(chrome);
                props.text_style = text_style_for_field.clone();

                let input = cx.text_input(props);
                let input_id = input.id;
                let is_focused = cx.is_focused_element(input_id);

                if !is_focused {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&draft, |s| *s = current_text.as_ref().to_string());
                    let _ = cx.app.models_mut().update(&error_for_field, |e| *e = None);
                    cx.with_state(
                        || String::new(),
                        |last| {
                            *last = current_text.as_ref().to_string();
                        },
                    );
                }

                let model_for_key = model.clone();
                let draft_for_key = draft.clone();
                let error_for_key = error_for_field.clone();
                let parse_for_key = parse.clone();
                let format_for_key = format.clone();
                let validate_for_key = validate.clone();
                let on_outcome_for_key = on_outcome.clone();
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
                                        let _ = host
                                            .models_mut()
                                            .update(&error_for_key, |e| *e = Some(msg));
                                        host.request_redraw(action_cx.window);
                                        return true;
                                    }
                                }

                                let _ = host.models_mut().update(&model_for_key, |m| *m = v);
                                let formatted = (format_for_key)(v);
                                let _ = host.models_mut().update(&draft_for_key, |s| {
                                    *s = formatted.as_ref().to_string()
                                });
                                let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                                if let Some(cb) = on_outcome_for_key.as_ref() {
                                    cb(host, action_cx, NumericInputOutcome::Committed);
                                }
                            } else {
                                let _ = host.models_mut().update(&error_for_key, |e| {
                                    *e = Some(Arc::from("Invalid number"))
                                });
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
                        let _ = cx.app.models_mut().update(&error_for_field, |e| *e = None);
                    }
                }

                input
            },
            move |cx| {
                let mut segments = Vec::new();
                let affix_color = {
                    let theme = Theme::global(&*cx.app);
                    theme
                        .color_by_key("muted-foreground")
                        .or_else(|| theme.color_by_key("muted_foreground"))
                        .unwrap_or_else(|| theme.color_token("foreground"))
                };

                if let Some(suffix) = suffix.clone() {
                    segments.push(editor_text_segment(
                        cx,
                        density,
                        frame_chrome.text_px,
                        suffix,
                        affix_color,
                        frame_chrome.padding,
                    ));
                }

                let show_icon = matches!(
                    error_display,
                    NumericInputErrorDisplay::TrailingIcon
                        | NumericInputErrorDisplay::InlineTextAndIcon
                );
                if !show_icon {
                    return segments;
                }

                let error_msg = cx
                    .get_model_cloned(&error_for_trailing, Invalidation::Paint)
                    .unwrap_or(None);
                if error_msg.is_none() {
                    return segments;
                }

                let error_border = {
                    let theme = Theme::global(&*cx.app);
                    theme
                        .color_by_key(EditorTokenKeys::NUMERIC_ERROR_BORDER)
                        .or_else(|| theme.color_by_key(EditorTokenKeys::NUMERIC_ERROR_FG))
                        .unwrap_or_else(|| theme.color_token("destructive"))
                };

                let mut icon = editor_icon_segment(
                    cx,
                    density,
                    fret_icons::ids::ui::STATUS_FAILED,
                    Some(Px(12.0)),
                    Some(fret_ui_kit::ColorRef::Color(error_border)),
                );
                if let Some(test_id) = error_icon_test_id.as_ref() {
                    icon = icon.test_id(test_id.clone());
                }
                segments.push(icon);
                segments
            },
        );

        let error_msg = cx
            .get_model_cloned(&error, Invalidation::Paint)
            .unwrap_or(None);

        let error_color = {
            let theme = Theme::global(&*cx.app);
            theme
                .color_by_key(EditorTokenKeys::NUMERIC_ERROR_FG)
                .unwrap_or_else(|| theme.color_token("destructive"))
        };
        let show_inline_error = matches!(
            error_display,
            NumericInputErrorDisplay::InlineText | NumericInputErrorDisplay::InlineTextAndIcon
        );

        let error_el = (show_inline_error).then_some(()).and_then(|_| {
            error_msg.map(|msg| {
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
                    style: Some(typography::as_content_text(TextStyle {
                        size: text_style.size,
                        line_height: text_style.line_height,
                        ..Default::default()
                    })),
                    color: Some(error_color),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                    align: TextAlign::Start,
                    ink_overflow: Default::default(),
                })
            })
        });

        let mut layout = options.layout;
        if layout.size.min_height.is_none() {
            layout.size.min_height = Some(Length::Px(density.row_height));
        }

        cx.flex(
            FlexProps {
                layout,
                direction: Axis::Vertical,
                gap: SpacingLength::Px(Px(4.0)),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Start,
                wrap: false,
            },
            move |_cx| {
                let mut out = vec![field];
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
