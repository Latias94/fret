//! Axis-labeled drag value (joined input group).
//!
//! This is used by Vec/Transform-style inspectors where the axis marker ("X/Y/Z/W") should feel
//! like part of the numeric field instead of a separate, differently-styled widget.

use std::panic::Location;
use std::sync::{Arc, Mutex};

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, Edges, KeyCode, Px, SemanticsInvalid, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{
    ActionCx, OnActivate, PointerDownCx, PressablePointerDownResult, UiActionHost,
    UiFocusActionHost,
};
use fret_ui::element::{
    AnyElement, FlexItemStyle, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle,
    TextInputProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, Size};

use crate::controls::numeric_input::{
    NumericFormatFn, NumericInputSelectionBehavior, NumericParseFn, NumericValidateFn,
};
use crate::primitives::EditorTokenKeys;
use crate::primitives::chrome::resolve_editor_text_field_style;
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::input_group::{
    EditorInputGroupFrameOverrides, derived_test_id, editor_axis_segment,
    editor_icon_button_segment, editor_icon_segment, editor_input_group_divider,
    editor_input_group_frame, editor_input_group_frame_with_overrides, editor_input_group_inset,
    editor_input_group_row, editor_text_segment,
};
use crate::primitives::numeric_text_entry::{
    NumericTextEntryFocusHandoffState, arm_numeric_text_entry_focus_handoff,
    clear_numeric_error_when_draft_changes, handle_numeric_text_entry_replace_key,
    numeric_text_entry_focus_state, sync_numeric_text_entry_focus,
    sync_numeric_text_entry_focus_handoff,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};
use crate::primitives::{DragValueCore, DragValueCoreOptions, EditSessionOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AxisDragValueMode {
    Scrub,
    Typing,
}

#[derive(Clone)]
pub struct AxisDragValueResetAction {
    pub icon: fret_icons::IconId,
    pub a11y_label: Arc<str>,
    pub test_id: Option<Arc<str>>,
    pub on_activate: OnActivate,
}

#[derive(Debug)]
struct AxisDragValueState {
    mode: AxisDragValueMode,
    scrub_id: Option<fret_ui::GlobalElementId>,
    scrub_revision: u64,
    seen_input_focus: bool,
}

impl Default for AxisDragValueState {
    fn default() -> Self {
        Self {
            mode: AxisDragValueMode::Scrub,
            scrub_id: None,
            scrub_revision: 0,
            seen_input_focus: false,
        }
    }
}

#[derive(Clone)]
pub struct AxisDragValueOptions {
    pub layout: LayoutStyle,
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
    /// Explicit identity source for internal state (scrub/typing focus restore, draft string).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub reset: Option<AxisDragValueResetAction>,
    pub enabled: bool,
    pub focusable: bool,
    pub size: Size,
    pub selection_behavior: NumericInputSelectionBehavior,
}

impl Default for AxisDragValueOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            prefix: None,
            suffix: None,
            id_source: None,
            test_id: None,
            reset: None,
            enabled: true,
            focusable: true,
            size: Size::Small,
            selection_behavior: NumericInputSelectionBehavior::ReplaceAllOnFocus,
        }
    }
}

pub type AxisDragValueOutcome = EditSessionOutcome;
pub type OnAxisDragValueOutcome =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, AxisDragValueOutcome) + 'static>;

#[derive(Clone)]
pub struct AxisDragValue<T> {
    axis_label: Arc<str>,
    axis_tint: Color,
    model: Model<T>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    on_outcome: Option<OnAxisDragValueOutcome>,
    options: AxisDragValueOptions,
}

impl<T> AxisDragValue<T>
where
    T: DragValueScalar + Default,
{
    pub fn new(
        axis_label: Arc<str>,
        axis_tint: Color,
        model: Model<T>,
        format: NumericFormatFn<T>,
        parse: NumericParseFn<T>,
    ) -> Self {
        Self {
            axis_label,
            axis_tint,
            model,
            format,
            parse,
            validate: None,
            on_outcome: None,
            options: AxisDragValueOptions::default(),
        }
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn on_outcome(mut self, on_outcome: Option<OnAxisDragValueOutcome>) -> Self {
        self.on_outcome = on_outcome;
        self
    }

    pub fn options(mut self, options: AxisDragValueOptions) -> Self {
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
                ("fret-ui-editor.axis_drag_value", id_source, model_id),
                |cx| self.into_element_keyed(cx),
            )
        } else {
            cx.keyed(
                ("fret-ui-editor.axis_drag_value", callsite, model_id),
                |cx| self.into_element_keyed(cx),
            )
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let state: Arc<Mutex<AxisDragValueState>> = cx.with_state(
            || Arc::new(Mutex::new(AxisDragValueState::default())),
            |s| s.clone(),
        );
        let focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>> = cx.with_state(
            || Arc::new(Mutex::new(NumericTextEntryFocusHandoffState::default())),
            |s| s.clone(),
        );
        let on_outcome = self.on_outcome.clone();

        let draft = draft_model(cx);
        let error = error_model(cx);
        let focus_state = numeric_text_entry_focus_state(cx);
        let last_draft_text =
            cx.with_state(|| Arc::new(Mutex::new(String::new())), |st| st.clone());

        let value = cx
            .get_model_copied(&self.model, Invalidation::Paint)
            .unwrap_or_default();
        let value_text = (self.format)(value);
        let value_text_for_scrub = value_text.clone();

        let (mode, scrub_revision) = {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            (st.mode, st.scrub_revision)
        };
        let typing = mode == AxisDragValueMode::Typing;
        let prefix = self.options.prefix.clone();
        let suffix = self.options.suffix.clone();
        let reset_action = self.options.reset.clone();
        let scrub_test_id = self.options.test_id.clone();
        let typing_test_id = derived_test_id(self.options.test_id.as_ref(), "typing");
        let scrub_axis_test_id = derived_test_id(scrub_test_id.as_ref(), "axis");
        let scrub_value_test_id = derived_test_id(scrub_test_id.as_ref(), "value");
        let scrub_prefix_test_id = derived_test_id(scrub_test_id.as_ref(), "prefix");
        let scrub_suffix_test_id = derived_test_id(scrub_test_id.as_ref(), "suffix");
        let typing_axis_test_id = derived_test_id(typing_test_id.as_ref(), "axis");
        let typing_input_test_id = derived_test_id(typing_test_id.as_ref(), "input");
        let typing_prefix_test_id = derived_test_id(typing_test_id.as_ref(), "prefix");
        let typing_suffix_test_id = derived_test_id(typing_test_id.as_ref(), "suffix");
        let typing_error_icon_test_id = derived_test_id(typing_test_id.as_ref(), "error");
        let explicit_reset_test_id = reset_action
            .as_ref()
            .and_then(|reset| reset.test_id.clone());
        let scrub_reset_test_id = explicit_reset_test_id
            .clone()
            .or_else(|| derived_test_id(scrub_test_id.as_ref(), "reset"));
        let typing_reset_test_id = explicit_reset_test_id
            .as_ref()
            .map(|id| Arc::<str>::from(format!("{}.typing", id.as_ref())))
            .or_else(|| derived_test_id(typing_test_id.as_ref(), "reset"));

        let (density, frame_chrome, (text_style, input_chrome)) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let frame_chrome = style.frame_chrome_small();
            let (input_chrome, text_style) = resolve_editor_text_field_style(
                theme,
                self.options.size,
                &ChromeRefinement::default(),
            );

            (style.density, frame_chrome, (text_style, input_chrome))
        };

        let mut scrub_opts = DragValueCoreOptions::default();
        scrub_opts.layout = if typing {
            hidden_layout(self.options.layout)
        } else {
            self.options.layout
        };
        scrub_opts.enabled = self.options.enabled && mode == AxisDragValueMode::Scrub;
        scrub_opts.scrub_on_double_click = false;

        let model_for_change = self.model.clone();
        let on_change_live: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, T) + 'static> =
            Arc::new(move |host, action_cx, next| {
                let _ = host.models_mut().update(&model_for_change, |v| *v = next);
                host.request_redraw(action_cx.window);
            });
        let axis_label = self.axis_label.clone();
        let axis_tint = self.axis_tint;
        let enabled_for_paint = self.options.enabled;

        let state_for_scrub = state.clone();
        let focus_handoff_for_scrub = focus_handoff.clone();
        let on_outcome_for_scrub = on_outcome.clone();
        let value_text_for_scrub_root = value_text.clone();
        let prefix_for_scrub_root = prefix.clone();
        let suffix_for_scrub_root = suffix.clone();
        let scrub = cx.keyed(
            ("fret-ui-editor.axis_drag_value.scrub", scrub_revision),
            move |cx| {
                let state_for_scrub_record = state_for_scrub.clone();
                let focus_handoff_for_double_click = focus_handoff_for_scrub.clone();
                let prefix_for_scrub = prefix_for_scrub_root.clone();
                let suffix_for_scrub = suffix_for_scrub_root.clone();
                let on_outcome_for_scrub_commit = on_outcome_for_scrub.clone();
                let on_outcome_for_scrub_cancel = on_outcome_for_scrub.clone();
                DragValueCore::new(value, on_change_live)
                    .on_commit(Some(Arc::new(move |host, action_cx| {
                        emit_axis_drag_value_outcome(
                            host,
                            action_cx,
                            on_outcome_for_scrub_commit.as_ref(),
                            AxisDragValueOutcome::Committed,
                        );
                    })))
                    .on_cancel(Some(Arc::new(move |host, action_cx| {
                        emit_axis_drag_value_outcome(
                            host,
                            action_cx,
                            on_outcome_for_scrub_cancel.as_ref(),
                            AxisDragValueOutcome::Canceled,
                        );
                    })))
                    .a11y_label(value_text_for_scrub_root.clone())
                    .options(scrub_opts)
                    .into_element(cx, move |cx, resp| {
                        let scrub_id = cx.root_id();
                        let mut st = state_for_scrub_record
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        st.scrub_id = Some(scrub_id);

                        let state_for_double_click = state_for_scrub_record.clone();
                        let focus_handoff_for_double_click = focus_handoff_for_double_click.clone();
                        cx.pressable_add_on_pointer_down(Arc::new(
                            move |host, action_cx, down: PointerDownCx| {
                                if down.click_count < 2 {
                                    return PressablePointerDownResult::Continue;
                                }
                                let mut st = state_for_double_click
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                st.mode = AxisDragValueMode::Typing;
                                st.seen_input_focus = false;
                                {
                                    let mut handoff = focus_handoff_for_double_click
                                        .lock()
                                        .unwrap_or_else(|e| e.into_inner());
                                    arm_numeric_text_entry_focus_handoff(&mut handoff);
                                }
                                host.request_redraw(action_cx.window);
                                PressablePointerDownResult::SkipDefaultAndStopPropagation
                            },
                        ));

                        let divider = frame_chrome.border;

                        let mut scrub_frame = editor_input_group_frame(
                            cx,
                            LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    min_height: Some(Length::Px(density.row_height)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            density,
                            frame_chrome,
                            EditorFrameState {
                                enabled: true,
                                hovered: resp.hovered,
                                pressed: resp.dragging || resp.pressed,
                                focused: resp.focused || cx.is_focused_element(scrub_id),
                                open: false,
                                semantic: EditorFrameSemanticState::default(),
                            },
                            move |cx, visuals| {
                                let affix_color = {
                                    let theme = Theme::global(&*cx.app);
                                    theme
                                        .color_by_key("muted-foreground")
                                        .or_else(|| theme.color_by_key("muted_foreground"))
                                        .unwrap_or_else(|| theme.color_token("foreground"))
                                };
                                let mut axis = editor_axis_segment(
                                    cx,
                                    density,
                                    axis_label.clone(),
                                    axis_tint,
                                    visuals.bg,
                                );
                                if let Some(test_id) = scrub_axis_test_id.as_ref() {
                                    axis = axis
                                        .test_id(test_id.clone())
                                        .a11y_label(axis_label.clone());
                                }
                                let sep = editor_input_group_divider(cx, divider);
                                let value_text_el = cx.text_props(TextProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Fill,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    text: value_text_for_scrub.clone(),
                                    style: Some(typography::as_control_text(TextStyle {
                                        size: frame_chrome.text_px,
                                        line_height: Some(density.row_height),
                                        ..Default::default()
                                    })),
                                    color: Some(visuals.fg),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Ellipsis,
                                    align: TextAlign::Start,
                                    ink_overflow: Default::default(),
                                });
                                let mut value = editor_input_group_inset(
                                    cx,
                                    frame_chrome.padding,
                                    value_text_el,
                                );
                                if let Some(test_id) = scrub_value_test_id.as_ref() {
                                    value = value
                                        .test_id(test_id.clone())
                                        .a11y_label(value_text_for_scrub.clone());
                                }

                                let mut segments = vec![axis, sep];
                                if let Some(prefix) = prefix_for_scrub.clone() {
                                    let mut segment = editor_text_segment(
                                        cx,
                                        density,
                                        frame_chrome.text_px,
                                        prefix.clone(),
                                        affix_color,
                                        frame_chrome.padding,
                                    );
                                    if let Some(test_id) = scrub_prefix_test_id.as_ref() {
                                        segment =
                                            segment.test_id(test_id.clone()).a11y_label(prefix);
                                    }
                                    segments.push(segment);
                                    segments.push(editor_input_group_divider(cx, divider));
                                }
                                segments.push(value);
                                if let Some(suffix) = suffix_for_scrub.clone() {
                                    segments.push(editor_input_group_divider(cx, divider));
                                    let mut segment = editor_text_segment(
                                        cx,
                                        density,
                                        frame_chrome.text_px,
                                        suffix.clone(),
                                        affix_color,
                                        frame_chrome.padding,
                                    );
                                    if let Some(test_id) = scrub_suffix_test_id.as_ref() {
                                        segment =
                                            segment.test_id(test_id.clone()).a11y_label(suffix);
                                    }
                                    segments.push(segment);
                                }
                                if let Some(reset) = reset_action.clone() {
                                    segments.push(editor_input_group_divider(cx, divider));
                                    segments.push(editor_icon_button_segment(
                                        cx,
                                        density,
                                        enabled_for_paint,
                                        reset.a11y_label.clone(),
                                        reset.icon,
                                        Some(Px(12.0)),
                                        scrub_reset_test_id.clone(),
                                        reset.on_activate.clone(),
                                    ));
                                }

                                vec![editor_input_group_row(cx, Px(0.0), segments)]
                            },
                        );
                        if let Some(test_id) = scrub_test_id.as_ref() {
                            scrub_frame = scrub_frame.test_id(test_id.clone());
                        }
                        vec![scrub_frame]
                    })
            },
        );

        let input_group_layout = if typing {
            self.options.layout
        } else {
            hidden_layout(self.options.layout)
        };

        let parse = self.parse.clone();
        let format = self.format.clone();
        let validate = self.validate.clone();
        let model_for_commit = self.model.clone();
        let state_for_input = state.clone();
        let on_outcome_for_keys = on_outcome.clone();
        let focus_state_for_keys = focus_state.clone();
        let error_for_keys = error.clone();
        let draft_for_keys = draft.clone();
        let last_draft_text_for_keys = last_draft_text.clone();
        let has_error = cx
            .get_model_cloned(&error, Invalidation::Paint)
            .unwrap_or(None)
            .is_some();

        let mut props = TextInputProps::new(draft.clone());
        props.layout = LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                min_height: Some(Length::Px(density.row_height)),
                ..Default::default()
            },
            ..Default::default()
        };
        props.enabled = self.options.enabled && typing;
        props.focusable = self.options.focusable && typing;
        props.test_id = typing_input_test_id.clone();
        props.a11y_invalid = has_error.then_some(SemanticsInvalid::True);

        // Joined field: the frame is drawn by the input group. Keep the inner text input transparent
        // and borderless to avoid double chrome.
        let mut joined_chrome = input_chrome;
        joined_chrome.padding = Edges::all(Px(0.0));
        joined_chrome.border = Edges::all(Px(0.0));
        joined_chrome.corner_radii = fret_core::Corners::all(Px(0.0));
        joined_chrome.background = Color {
            a: 0.0,
            ..joined_chrome.background
        };
        joined_chrome.border_color = Color {
            a: 0.0,
            ..joined_chrome.border_color
        };
        joined_chrome.border_color_focused = joined_chrome.border_color;
        joined_chrome.focus_ring = None;

        props.chrome = joined_chrome;
        props.text_style = text_style.clone();

        let input = cx.text_input(props);
        let input_id = input.id;
        let is_focused = cx.is_focused_element(input_id);

        // Drive mode transitions from focus: if the user clicks away after the input actually
        // became focused, return to scrub mode.
        if typing {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            if is_focused {
                st.seen_input_focus = true;
            } else if st.seen_input_focus {
                st.mode = AxisDragValueMode::Scrub;
            }
        }

        sync_numeric_text_entry_focus(
            cx,
            &focus_state,
            is_focused,
            &value_text,
            &draft,
            &error,
            self.options.selection_behavior,
        );
        sync_numeric_text_entry_focus_handoff(
            cx,
            input_id,
            &focus_handoff,
            typing,
            input_id,
            is_focused,
        );

        if !is_focused {
            let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
            *last = value_text.as_ref().to_string();
        }

        cx.key_add_on_key_down_capture_for(
            input_id,
            Arc::new(
                move |host: &mut dyn UiFocusActionHost, action_cx: ActionCx, down| {
                    if let Some(consumed) = handle_numeric_text_entry_replace_key(
                        host,
                        action_cx,
                        down,
                        &focus_state_for_keys,
                        &draft_for_keys,
                        &error_for_keys,
                    ) {
                        if consumed {
                            return true;
                        }
                    }

                    match down.key {
                        KeyCode::Enter | KeyCode::NumpadEnter => {
                            let text = host
                                .models_mut()
                                .read(&draft_for_keys, |s| s.clone())
                                .unwrap_or_default();
                            if let Some(v) = (parse)(&text) {
                                if let Some(validate) = validate.as_ref() {
                                    if let Some(msg) = validate(v) {
                                        let _ = host
                                            .models_mut()
                                            .update(&error_for_keys, |e| *e = Some(msg));
                                        let mut last = last_draft_text_for_keys
                                            .lock()
                                            .unwrap_or_else(|e| e.into_inner());
                                        *last = text;
                                        host.request_redraw(action_cx.window);
                                        return true;
                                    }
                                }

                                let _ = host.models_mut().update(&model_for_commit, |m| *m = v);
                                let formatted = (format)(v);
                                let _ = host.models_mut().update(&draft_for_keys, |s| {
                                    *s = formatted.as_ref().to_string()
                                });
                                let _ = host.models_mut().update(&error_for_keys, |e| *e = None);
                                let mut last = last_draft_text_for_keys
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                *last = formatted.as_ref().to_string();

                                let mut st =
                                    state_for_input.lock().unwrap_or_else(|e| e.into_inner());
                                st.mode = AxisDragValueMode::Scrub;
                                st.scrub_revision = st.scrub_revision.wrapping_add(1);
                                if let Some(scrub_id) = st.scrub_id {
                                    host.request_focus(scrub_id);
                                }
                                emit_axis_drag_value_outcome(
                                    host,
                                    action_cx,
                                    on_outcome_for_keys.as_ref(),
                                    AxisDragValueOutcome::Committed,
                                );
                                host.request_redraw(action_cx.window);
                                true
                            } else {
                                let _ = host.models_mut().update(&error_for_keys, |e| {
                                    *e = Some(Arc::from("Invalid number"))
                                });
                                let mut last = last_draft_text_for_keys
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                *last = text;
                                host.request_redraw(action_cx.window);
                                true
                            }
                        }
                        KeyCode::Escape => {
                            let current = host
                                .models_mut()
                                .get_copied(&model_for_commit)
                                .unwrap_or_default();
                            let formatted = (format)(current);
                            let _ = host
                                .models_mut()
                                .update(&draft_for_keys, |s| *s = formatted.as_ref().to_string());
                            let _ = host.models_mut().update(&error_for_keys, |e| *e = None);
                            let mut last = last_draft_text_for_keys
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            *last = formatted.as_ref().to_string();

                            let mut st = state_for_input.lock().unwrap_or_else(|e| e.into_inner());
                            st.mode = AxisDragValueMode::Scrub;
                            st.scrub_revision = st.scrub_revision.wrapping_add(1);
                            if let Some(scrub_id) = st.scrub_id {
                                host.request_focus(scrub_id);
                            }
                            emit_axis_drag_value_outcome(
                                host,
                                action_cx,
                                on_outcome_for_keys.as_ref(),
                                AxisDragValueOutcome::Canceled,
                            );
                            host.request_redraw(action_cx.window);
                            true
                        }
                        _ => false,
                    }
                },
            ),
        );

        clear_numeric_error_when_draft_changes(cx, is_focused, &draft, &error, &last_draft_text);

        let typing_field = {
            let divider = frame_chrome.border;
            let reset_action = self.options.reset.clone();
            let enabled_for_paint = self.options.enabled;
            let prefix = prefix.clone();
            let suffix = suffix.clone();
            let axis_label = self.axis_label.clone();
            let axis_tint = self.axis_tint;
            let error_icon_test_id = typing_error_icon_test_id.clone();

            let mut typing_frame = editor_input_group_frame_with_overrides(
                cx,
                input_group_layout,
                density,
                frame_chrome,
                EditorFrameState {
                    enabled: true,
                    hovered: false,
                    pressed: false,
                    focused: is_focused,
                    open: false,
                    semantic: EditorFrameSemanticState {
                        typing: true,
                        invalid: has_error,
                    },
                },
                EditorInputGroupFrameOverrides::none(),
                move |cx, visuals| {
                    let affix_color = {
                        let theme = Theme::global(&*cx.app);
                        theme
                            .color_by_key("muted-foreground")
                            .or_else(|| theme.color_by_key("muted_foreground"))
                            .unwrap_or_else(|| theme.color_token("foreground"))
                    };
                    let mut axis =
                        editor_axis_segment(cx, density, axis_label.clone(), axis_tint, visuals.bg);
                    if let Some(test_id) = typing_axis_test_id.as_ref() {
                        axis = axis.test_id(test_id.clone()).a11y_label(axis_label.clone());
                    }
                    let sep = editor_input_group_divider(cx, divider);

                    // Wrap the text input so the group padding applies, without adding its own padding.
                    let input_wrap = editor_input_group_inset(cx, frame_chrome.padding, input);

                    let mut segments = vec![axis, sep];
                    if let Some(prefix) = prefix.clone() {
                        let mut segment = editor_text_segment(
                            cx,
                            density,
                            frame_chrome.text_px,
                            prefix.clone(),
                            affix_color,
                            frame_chrome.padding,
                        );
                        if let Some(test_id) = typing_prefix_test_id.as_ref() {
                            segment = segment.test_id(test_id.clone()).a11y_label(prefix);
                        }
                        segments.push(segment);
                        segments.push(editor_input_group_divider(cx, divider));
                    }
                    segments.push(input_wrap);
                    if let Some(suffix) = suffix.clone() {
                        segments.push(editor_input_group_divider(cx, divider));
                        let mut segment = editor_text_segment(
                            cx,
                            density,
                            frame_chrome.text_px,
                            suffix.clone(),
                            affix_color,
                            frame_chrome.padding,
                        );
                        if let Some(test_id) = typing_suffix_test_id.as_ref() {
                            segment = segment.test_id(test_id.clone()).a11y_label(suffix);
                        }
                        segments.push(segment);
                    }
                    if has_error {
                        let error_border = {
                            let theme = Theme::global(&*cx.app);
                            theme
                                .color_by_key(EditorTokenKeys::CONTROL_INVALID_BORDER)
                                .or_else(|| {
                                    theme.color_by_key(EditorTokenKeys::NUMERIC_ERROR_BORDER)
                                })
                                .or_else(|| theme.color_by_key(EditorTokenKeys::CONTROL_INVALID_FG))
                                .or_else(|| theme.color_by_key(EditorTokenKeys::NUMERIC_ERROR_FG))
                                .unwrap_or_else(|| theme.color_token("destructive"))
                        };
                        segments.push(editor_input_group_divider(cx, divider));
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
                    }
                    if let Some(reset) = reset_action {
                        segments.push(editor_input_group_divider(cx, divider));
                        segments.push(editor_icon_button_segment(
                            cx,
                            density,
                            enabled_for_paint,
                            reset.a11y_label,
                            reset.icon,
                            Some(Px(12.0)),
                            typing_reset_test_id.clone(),
                            reset.on_activate,
                        ));
                    }

                    vec![editor_input_group_row(cx, Px(0.0), segments)]
                },
            );
            if let Some(test_id) = typing_test_id.as_ref() {
                typing_frame = typing_frame.test_id(test_id.clone());
            }
            typing_frame
        };

        // Render both: scrub stays mounted so focus can restore, typing stays mounted so focus
        // requests have a stable target.
        cx.container(Default::default(), move |_cx| vec![scrub, typing_field])
    }
}

fn hidden_layout(mut layout: LayoutStyle) -> LayoutStyle {
    layout.size = SizeStyle {
        width: Length::Px(Px(0.0)),
        height: Length::Px(Px(0.0)),
        min_width: Some(Length::Px(Px(0.0))),
        min_height: Some(Length::Px(Px(0.0))),
        ..Default::default()
    };
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
        ..Default::default()
    };
    layout.overflow = Overflow::Clip;
    layout
}

fn emit_axis_drag_value_outcome(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    on_outcome: Option<&OnAxisDragValueOutcome>,
    outcome: AxisDragValueOutcome,
) {
    if let Some(cb) = on_outcome {
        cb(host, action_cx, outcome);
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
