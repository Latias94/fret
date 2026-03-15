//! Numeric text input control with editor-style commit/cancel outcomes.
//!
//! This control is intentionally lightweight:
//! - it owns a per-element draft `Model<String>` for text editing,
//! - commits parsed values on Enter,
//! - validates on commit (optional),
//! - cancels (reverts to formatted current value) on Escape,
//! - renders an inline error message when commit is rejected.

use std::panic::Location;
use std::sync::{Arc, Mutex};

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Edges, KeyCode, Px, SemanticsInvalid, TextAlign, TextStyle};
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
    EditorInputGroupFrameOverrides, derived_test_id, editor_icon_segment,
    editor_joined_input_frame_segments_with_overrides, editor_text_segment,
};
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;
use crate::primitives::numeric_text_entry::{
    clear_numeric_error_when_draft_changes, handle_numeric_text_entry_replace_key,
    numeric_text_entry_focus_state, sync_numeric_text_entry_focus,
};
use crate::primitives::{NumericPresentation, style::EditorStyle};

pub use crate::primitives::NumericInputSelectionBehavior;

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
    pub selection_behavior: NumericInputSelectionBehavior,
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
            selection_behavior: NumericInputSelectionBehavior::ReplaceAllOnFocus,
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

fn editor_numeric_input_text_style(
    base: TextStyle,
    density: crate::primitives::EditorDensity,
) -> TextStyle {
    typography::as_control_text(TextStyle {
        line_height: Some(density.row_height),
        ..base
    })
}

#[derive(Clone)]
pub struct NumericInput<T> {
    model: Model<T>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    on_outcome: Option<OnNumericInputOutcome>,
    options: NumericInputOptions,
    focus_target: Option<Arc<Mutex<Option<fret_ui::GlobalElementId>>>>,
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
            focus_target: None,
        }
    }

    /// Construct a numeric input from a shared editor authoring bundle.
    pub fn from_presentation(model: Model<T>, presentation: NumericPresentation<T>) -> Self {
        let mut input = Self::new(model, presentation.format(), presentation.parse());
        input.options.prefix = presentation.chrome_prefix().cloned();
        input.options.suffix = presentation.chrome_suffix().cloned();
        input
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

    pub(crate) fn focus_target(
        mut self,
        focus_target: Arc<Mutex<Option<fret_ui::GlobalElementId>>>,
    ) -> Self {
        self.focus_target = Some(focus_target);
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
        let focus_state = numeric_text_entry_focus_state(cx);
        let last_draft_text =
            cx.slot_state(|| Arc::new(Mutex::new(String::new())), |st| st.clone());
        let current_value = cx
            .get_model_copied(&model, Invalidation::Paint)
            .unwrap_or_default();
        let current_text = (format)(current_value);
        let has_error = cx
            .get_model_cloned(&error, Invalidation::Paint)
            .unwrap_or(None)
            .is_some();

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
        let text_style_for_field = editor_numeric_input_text_style(text_style.clone(), density);
        let placeholder = options.placeholder.clone();
        let focusable = options.focusable;
        let error_display = options.error_display;
        let selection_behavior = options.selection_behavior;
        let focus_target = self.focus_target.clone();
        let (prefix, suffix) = suppress_duplicate_chrome_affixes(
            current_text.as_ref(),
            options.prefix.clone(),
            options.suffix.clone(),
        );
        let input_test_id = derived_test_id(options.test_id.as_ref(), "input");
        let prefix_test_id = derived_test_id(options.test_id.as_ref(), "prefix");
        let suffix_test_id = derived_test_id(options.test_id.as_ref(), "suffix");
        let error_icon_test_id = derived_test_id(options.test_id.as_ref(), "error");
        let error_text_test_id = derived_test_id(options.test_id.as_ref(), "error-text");

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
            move |cx, focused| {
                let has_error = cx
                    .get_model_cloned(&error_for_frame, Invalidation::Paint)
                    .unwrap_or(None)
                    .is_some();
                EditorInputGroupFrameOverrides {
                    semantic: Some(crate::primitives::visuals::EditorFrameSemanticState {
                        typing: focused,
                        invalid: has_error,
                    }),
                    ..EditorInputGroupFrameOverrides::none()
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
                    let mut segment = editor_text_segment(
                        cx,
                        density,
                        frame_chrome.text_px,
                        prefix.clone(),
                        affix_color,
                        frame_chrome.padding,
                    );
                    if let Some(test_id) = prefix_test_id.as_ref() {
                        segment = segment.test_id(test_id.clone()).a11y_label(prefix);
                    }
                    segments.push(segment);
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
                props.test_id = input_test_id.clone();
                props.a11y_invalid = has_error.then_some(SemanticsInvalid::True);
                props.chrome = joined_text_input_style(chrome);
                props.text_style = text_style_for_field.clone();

                let input = cx.text_input(props);
                let input_id = input.id;
                if let Some(focus_target) = focus_target.as_ref() {
                    let mut slot = focus_target.lock().unwrap_or_else(|e| e.into_inner());
                    *slot = Some(input_id);
                }
                let is_focused = cx.is_focused_element(input_id);

                sync_numeric_text_entry_focus(
                    cx,
                    &focus_state,
                    is_focused,
                    &current_text,
                    &draft,
                    &error_for_field,
                    selection_behavior,
                );

                if !is_focused {
                    let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
                    *last = current_text.as_ref().to_string();
                }

                let model_for_key = model.clone();
                let draft_for_key = draft.clone();
                let error_for_key = error_for_field.clone();
                let focus_state_for_key = focus_state.clone();
                let last_draft_for_key = last_draft_text.clone();
                let parse_for_key = parse.clone();
                let format_for_key = format.clone();
                let validate_for_key = validate.clone();
                let on_outcome_for_key = on_outcome.clone();
                cx.key_add_on_key_down_capture_for(
                    input_id,
                    Arc::new(move |host, action_cx: ActionCx, down| match down.key {
                        _ => {
                            if let Some(consumed) = handle_numeric_text_entry_replace_key(
                                host,
                                action_cx,
                                down,
                                &focus_state_for_key,
                                &draft_for_key,
                                &error_for_key,
                            ) {
                                if consumed {
                                    return true;
                                }
                            }
                            match down.key {
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
                                                let mut last = last_draft_for_key
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                *last = text;
                                                host.request_redraw(action_cx.window);
                                                return true;
                                            }
                                        }

                                        let _ =
                                            host.models_mut().update(&model_for_key, |m| *m = v);
                                        let formatted = (format_for_key)(v);
                                        let _ = host.models_mut().update(&draft_for_key, |s| {
                                            *s = formatted.as_ref().to_string()
                                        });
                                        let _ =
                                            host.models_mut().update(&error_for_key, |e| *e = None);
                                        let mut last = last_draft_for_key
                                            .lock()
                                            .unwrap_or_else(|e| e.into_inner());
                                        *last = formatted.as_ref().to_string();
                                        if let Some(cb) = on_outcome_for_key.as_ref() {
                                            cb(host, action_cx, NumericInputOutcome::Committed);
                                        }
                                    } else {
                                        let _ = host.models_mut().update(&error_for_key, |e| {
                                            *e = Some(Arc::from("Invalid number"))
                                        });
                                        let mut last = last_draft_for_key
                                            .lock()
                                            .unwrap_or_else(|e| e.into_inner());
                                        *last = text;
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
                                    let _ = host.models_mut().update(&draft_for_key, |s| {
                                        *s = formatted.as_ref().to_string()
                                    });
                                    let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                                    let mut last = last_draft_for_key
                                        .lock()
                                        .unwrap_or_else(|e| e.into_inner());
                                    *last = formatted.as_ref().to_string();
                                    if let Some(cb) = on_outcome_for_key.as_ref() {
                                        cb(host, action_cx, NumericInputOutcome::Canceled);
                                    }
                                    host.request_redraw(action_cx.window);
                                    true
                                }
                                _ => false,
                            }
                        }
                    }),
                );

                clear_numeric_error_when_draft_changes(
                    cx,
                    is_focused,
                    &draft,
                    &error_for_field,
                    &last_draft_text,
                );

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
                    let mut segment = editor_text_segment(
                        cx,
                        density,
                        frame_chrome.text_px,
                        suffix.clone(),
                        affix_color,
                        frame_chrome.padding,
                    );
                    if let Some(test_id) = suffix_test_id.as_ref() {
                        segment = segment.test_id(test_id.clone()).a11y_label(suffix);
                    }
                    segments.push(segment);
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
                        .color_by_key(EditorTokenKeys::CONTROL_INVALID_BORDER)
                        .or_else(|| theme.color_by_key(EditorTokenKeys::NUMERIC_ERROR_BORDER))
                        .or_else(|| theme.color_by_key(EditorTokenKeys::CONTROL_INVALID_FG))
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
                .color_by_key(EditorTokenKeys::CONTROL_INVALID_FG)
                .or_else(|| theme.color_by_key(EditorTokenKeys::NUMERIC_ERROR_FG))
                .unwrap_or_else(|| theme.color_token("destructive"))
        };
        let show_inline_error = matches!(
            error_display,
            NumericInputErrorDisplay::InlineText | NumericInputErrorDisplay::InlineTextAndIcon
        );

        let error_el = (show_inline_error).then_some(()).and_then(|_| {
            error_msg.map(|msg| {
                let mut error = cx.text_props(TextProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: msg.clone(),
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
                });
                if let Some(test_id) = error_text_test_id.as_ref() {
                    error = error.test_id(test_id.clone()).a11y_label(msg.clone());
                }
                error
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

#[cfg(test)]
mod tests {
    use super::editor_numeric_input_text_style;
    use crate::controls::NumericInput;
    use crate::primitives::EditorDensity;
    use crate::primitives::NumericPresentation;
    use fret_app::App;
    use fret_core::Px;
    use fret_core::TextStyle;
    use std::sync::Arc;

    #[test]
    fn numeric_input_text_style_uses_density_row_height_for_edit_line_box() {
        let style = editor_numeric_input_text_style(
            TextStyle {
                size: Px(12.0),
                line_height: Some(Px(16.0)),
                ..Default::default()
            },
            EditorDensity {
                row_height: Px(24.0),
                ..Default::default()
            },
        );

        assert_eq!(style.line_height, Some(Px(24.0)));
    }

    #[test]
    fn numeric_input_from_presentation_adopts_format_parse_and_chrome_affixes() {
        let mut app = App::new();
        let model = app.models_mut().insert(1.25f64);
        let presentation = NumericPresentation::<f64>::fixed_decimals(2)
            .with_chrome_prefix("$")
            .with_chrome_suffix("ms");

        let input = NumericInput::from_presentation(model, presentation);

        assert_eq!((input.format)(1.25).as_ref(), "1.25");
        assert_eq!((input.parse)("1.25"), Some(1.25));
        assert_eq!(input.options.prefix, Some(Arc::from("$")));
        assert_eq!(input.options.suffix, Some(Arc::from("ms")));
    }
}

#[track_caller]
fn draft_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.local_model(String::new)
}

#[track_caller]
fn error_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    cx.local_model(|| None::<Arc<str>>)
}
