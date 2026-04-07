//! Reusable editor text field control.
//!
//! v1 scope:
//! - single-line input (`TextInput`)
//! - optional multiline mode (`TextArea`) with a minimum height
//! - optional clear affordance

use std::cell::Cell;
use std::panic::Location;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{KeyCode, NodeId, Px, TextStyle};
use fret_runtime::{CommandId, Model, TimerToken};
use fret_ui::action::{ActionCx, ActivateReason, KeyDownCx, OnActivate, UiFocusActionHost};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, TextAreaProps, TextInputProps};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::chrome::{
    joined_text_area_style, joined_text_input_style, resolve_editor_text_area_field_style,
    resolve_editor_text_field_style,
};
use crate::primitives::input_group::{
    editor_clear_button_segment, editor_clear_button_segment_multiline, editor_joined_input_frame,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::text_entry::{
    EditorTextCancelBehavior, EditorTextSelectionBehavior, editor_text_entry_focus_state,
    sync_editor_text_entry_focus_selection,
};
use crate::primitives::{EditSession, EditSessionOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFieldMode {
    #[default]
    PlainText,
    Password,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFieldBlurBehavior {
    /// Accept the current draft when focus leaves a buffered field.
    #[default]
    Commit,
    /// Restore the pre-edit value when focus leaves a buffered field.
    Cancel,
    /// Leave the draft session open even after blur.
    ///
    /// This preserves the old deferred policy for specialized surfaces that want an external
    /// owner to decide how blur should finish the session.
    PreserveDraft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextFieldAssistiveSemantics {
    /// Active-descendant semantics for an assistive surface such as completion or history.
    pub active_descendant: Option<NodeId>,
    /// Declarative element id for the current active assistive option.
    pub active_descendant_element: Option<u64>,
    /// Declarative element id for an assistive surface controlled by this field.
    pub controls_element: Option<u64>,
    /// Whether the assistive surface is currently expanded.
    pub expanded: Option<bool>,
}

pub type TextFieldOutcome = EditSessionOutcome;
pub type OnTextFieldOutcome =
    Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, TextFieldOutcome) + 'static>;

#[derive(Debug, Clone)]
pub struct TextFieldOptions {
    pub layout: LayoutStyle,
    pub size: Size,
    pub placeholder: Option<Arc<str>>,
    /// Explicit identity source for internal buffered state.
    ///
    /// Use this when helper code renders multiple text fields from the same callsite and model
    /// identity alone is not enough to distinguish their edit sessions.
    pub id_source: Option<Arc<str>>,
    pub enabled: bool,
    pub focusable: bool,
    pub clear_button: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub clear_test_id: Option<Arc<str>>,
    /// Optional sink for the outer joined-field root id.
    ///
    /// Recipes such as anchored completion/history popups use this to keep the whole field chrome
    /// in the same dismissable branch even when the assistive surface is anchored to the input.
    pub field_id_out: Option<Rc<Cell<Option<GlobalElementId>>>>,
    /// Optional sink for the inner text-entry element id.
    ///
    /// Recipes use this to anchor input-owned assistive surfaces and restore focus to the actual
    /// text entry node rather than the surrounding field container.
    pub input_id_out: Option<Rc<Cell<Option<GlobalElementId>>>>,
    /// Visual text mode for the single-line input surface.
    ///
    /// Password mode currently maps to `TextInputProps::obscure_text`; multiline text areas remain
    /// plain text until editor-owned multiline/password policy is defined.
    pub mode: TextFieldMode,
    /// When true, single-line fields edit a local draft and only commit on explicit accept.
    ///
    /// Both single-line and multiline editor fields can use a local draft session. Multiline
    /// commit remains editor-owned: blur commits by default, while Ctrl/Cmd+Enter acts as an
    /// explicit commit shortcut.
    pub buffered: bool,
    /// How a buffered field should finish its local draft when focus leaves the editing surface.
    pub blur_behavior: TextFieldBlurBehavior,
    /// Optional submit command for Enter on single-line text inputs.
    ///
    /// For buffered fields this runs after the local draft has been committed into the bound
    /// model. Multiline text areas intentionally do not route Enter through this field today.
    pub submit_command: Option<CommandId>,
    /// Placeholder semantics for completion/history popups owned outside the field.
    ///
    /// This does not implement those surfaces; it only exposes the relationship hooks needed to
    /// wire them later without changing the public editor control surface again.
    pub assistive_semantics: TextFieldAssistiveSemantics,
    pub selection_behavior: EditorTextSelectionBehavior,
    pub cancel_behavior: EditorTextCancelBehavior,

    /// When true, uses `TextArea` (multiline) instead of `TextInput`.
    pub multiline: bool,
    /// If true, opt into stable multiline line boxes (fixed line height + forced strut).
    ///
    /// This is intended for UI/form surfaces where baseline stability matters more than avoiding
    /// ink clipping for tall fallback glyphs.
    pub stable_line_boxes: bool,
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
            id_source: None,
            enabled: true,
            focusable: true,
            clear_button: false,
            a11y_label: None,
            test_id: None,
            clear_test_id: None,
            field_id_out: None,
            input_id_out: None,
            mode: TextFieldMode::PlainText,
            buffered: true,
            blur_behavior: TextFieldBlurBehavior::Commit,
            submit_command: None,
            assistive_semantics: TextFieldAssistiveSemantics::default(),
            selection_behavior: EditorTextSelectionBehavior::PreserveSelection,
            cancel_behavior: EditorTextCancelBehavior::None,
            multiline: false,
            stable_line_boxes: true,
            min_height: None,
        }
    }
}

#[derive(Clone)]
pub struct TextField {
    model: Model<String>,
    on_outcome: Option<OnTextFieldOutcome>,
    options: TextFieldOptions,
}

#[derive(Debug, Default)]
struct BufferedTextFieldState {
    was_focused: bool,
    session: EditSession<String>,
    blur_timer: Option<TimerToken>,
    pending_blur: Option<TextFieldBlurBehavior>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BufferedTextFieldPendingBlurPlan {
    Keep,
    Clear,
    Arm(TextFieldBlurBehavior),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BufferedTextFieldFocusPlan {
    begin_session: bool,
    cancel_pending_blur: bool,
    pending_blur: BufferedTextFieldPendingBlurPlan,
}

fn plan_buffered_text_field_focus_transition(
    was_focused: bool,
    session_active: bool,
    is_focused: bool,
    blur_behavior: TextFieldBlurBehavior,
    has_pending_blur: bool,
) -> BufferedTextFieldFocusPlan {
    if is_focused {
        return BufferedTextFieldFocusPlan {
            begin_session: !session_active,
            cancel_pending_blur: has_pending_blur,
            pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
        };
    }

    if was_focused && session_active {
        return BufferedTextFieldFocusPlan {
            begin_session: false,
            cancel_pending_blur: has_pending_blur,
            pending_blur: match blur_behavior {
                TextFieldBlurBehavior::Commit | TextFieldBlurBehavior::Cancel => {
                    BufferedTextFieldPendingBlurPlan::Arm(blur_behavior)
                }
                TextFieldBlurBehavior::PreserveDraft => BufferedTextFieldPendingBlurPlan::Clear,
            },
        };
    }

    if session_active {
        return BufferedTextFieldFocusPlan {
            begin_session: false,
            cancel_pending_blur: false,
            pending_blur: BufferedTextFieldPendingBlurPlan::Keep,
        };
    }

    BufferedTextFieldFocusPlan {
        begin_session: false,
        cancel_pending_blur: has_pending_blur,
        pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
    }
}

impl TextField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            on_outcome: None,
            options: TextFieldOptions::default(),
        }
    }

    pub fn on_outcome(mut self, on_outcome: Option<OnTextFieldOutcome>) -> Self {
        self.on_outcome = on_outcome;
        self
    }

    pub fn options(mut self, options: TextFieldOptions) -> Self {
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
            cx.keyed(("fret-ui-editor.text_field", id_source, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.text_field", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let TextField {
            model,
            on_outcome,
            options,
        } = self;

        let layout = options.layout;
        let size = options.size;
        let placeholder = options.placeholder.clone();
        let enabled_for_paint = options.enabled;
        let focusable = options.focusable;
        let clear_button = options.clear_button;
        let a11y_label = options.a11y_label.clone();
        let test_id = options.test_id.clone();
        let clear_test_id = options.clear_test_id.clone();
        let field_id_out = options.field_id_out.clone();
        let input_id_out = options.input_id_out.clone();
        let mode = options.mode;
        let buffered = options.buffered;
        let blur_behavior = options.blur_behavior;
        let submit_command = options.submit_command.clone();
        let assistive_semantics = options.assistive_semantics;
        let selection_behavior = options.selection_behavior;
        let cancel_behavior = options.cancel_behavior;
        let multiline = options.multiline;
        let stable_line_boxes = options.stable_line_boxes;
        let min_height = options.min_height;
        let focus_state = editor_text_entry_focus_state(cx);
        let draft = buffered.then(|| draft_model(cx));
        let buffered_state = buffered.then(|| buffered_state(cx));
        let current_text = cx
            .get_model_cloned(&model, Invalidation::Paint)
            .unwrap_or_default();

        if let (Some(draft), Some(buffered_state)) = (draft.as_ref(), buffered_state.as_ref()) {
            sync_draft_from_model_when_session_inactive(cx, draft, buffered_state, &current_text);
        }

        let (density, frame_chrome) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            (style.density, style.frame_chrome(size))
        };

        let model_for_input = model.clone();
        let model_for_trailing = model.clone();
        let draft_for_input = draft.clone();
        let draft_for_trailing = draft.clone();
        let buffered_state_for_input = buffered_state.clone();
        let buffered_state_for_trailing = buffered_state.clone();
        let current_text_for_input = current_text.clone();
        let on_outcome_for_input = on_outcome.clone();
        let submit_command_for_input = submit_command.clone();

        let field = editor_joined_input_frame(
            cx,
            layout,
            density,
            frame_chrome,
            enabled_for_paint,
            false,
            None,
            move |cx| {
                if multiline {
                    let (chrome, text_style) = {
                        let theme = Theme::global(&*cx.app);
                        resolve_editor_text_area_field_style(
                            theme,
                            size,
                            &ChromeRefinement::default(),
                        )
                    };
                    let text_style = if stable_line_boxes {
                        let theme = Theme::global(&*cx.app);
                        typography::text_area_control_text_style_scaled(
                            theme,
                            fret_core::FontId::ui(),
                            text_style.size,
                        )
                    } else {
                        text_style
                    };

                    let input_model = draft_for_input
                        .clone()
                        .unwrap_or_else(|| model_for_input.clone());
                    let mut props = TextAreaProps::new(input_model);
                    props.layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    props.enabled = enabled_for_paint;
                    props.focusable = focusable;
                    props.a11y_label = a11y_label.clone();
                    props.test_id = test_id.clone();

                    props.chrome = joined_text_area_style(chrome);
                    props.text_style = text_style;
                    props.min_height = min_height.unwrap_or_else(|| {
                        let baseline = Px(80.0);
                        let dense = Px(density.row_height.0 * 3.0);
                        Px(baseline.0.max(dense.0))
                    });

                    let area = cx.text_area(props);
                    if let Some(out) = input_id_out.as_ref() {
                        out.set(Some(area.id));
                    }
                    let area_id = area.id;
                    let is_focused = cx.is_focused_element(area_id);
                    if let (Some(draft), Some(buffered_state)) =
                        (draft_for_input.as_ref(), buffered_state_for_input.as_ref())
                    {
                        sync_buffered_text_field_session(
                            cx,
                            area_id,
                            is_focused,
                            &current_text_for_input,
                            draft,
                            buffered_state,
                            blur_behavior,
                        );

                        let model_for_key = model_for_input.clone();
                        let draft_for_key = draft.clone();
                        let buffered_state_for_key = buffered_state.clone();
                        let on_outcome_for_key = on_outcome_for_input.clone();
                        cx.key_add_on_key_down_capture_for(
                            area_id,
                            Arc::new(move |host, action_cx: ActionCx, down| {
                                if down.ime_composing || down.repeat {
                                    return false;
                                }
                                match down.key {
                                    KeyCode::Escape => cancel_buffered_text_field(
                                        host,
                                        action_cx,
                                        &model_for_key,
                                        &draft_for_key,
                                        &buffered_state_for_key,
                                        on_outcome_for_key.as_ref(),
                                    ),
                                    KeyCode::Enter | KeyCode::NumpadEnter
                                        if is_multiline_buffered_commit_shortcut(down) =>
                                    {
                                        commit_buffered_text_field(
                                            host,
                                            action_cx,
                                            &model_for_key,
                                            &draft_for_key,
                                            &buffered_state_for_key,
                                            on_outcome_for_key.as_ref(),
                                            None,
                                        )
                                    }
                                    _ => false,
                                }
                            }),
                        );
                    }

                    let has_value = if let Some(draft) = draft_for_input.as_ref() {
                        cx.read_model_ref(draft, Invalidation::Paint, |s| !s.is_empty())
                            .unwrap_or(false)
                    } else {
                        cx.read_model_ref(&model_for_input, Invalidation::Paint, |s| !s.is_empty())
                            .unwrap_or(false)
                    };
                    sync_editor_text_entry_focus_selection(
                        cx,
                        &focus_state,
                        area_id,
                        is_focused,
                        has_value,
                        selection_behavior,
                    );
                    if let (Some(draft), Some(buffered_state)) =
                        (draft_for_input.as_ref(), buffered_state_for_input.as_ref())
                    {
                        install_buffered_text_field_blur_handler(
                            cx,
                            area_id,
                            model_for_input.clone(),
                            draft.clone(),
                            buffered_state.clone(),
                            on_outcome_for_input.clone(),
                        );
                    }
                    if !buffered && matches!(cancel_behavior, EditorTextCancelBehavior::Clear) {
                        let model_for_escape = model_for_input.clone();
                        cx.key_add_on_key_down_capture_for(
                            area_id,
                            Arc::new(move |host, action_cx, down| {
                                if down.key != KeyCode::Escape {
                                    return false;
                                }
                                let _ = host.models_mut().update(&model_for_escape, |s| s.clear());
                                host.request_redraw(action_cx.window);
                                true
                            }),
                        );
                    }

                    area
                } else {
                    let (chrome, text_style) = {
                        let theme = Theme::global(&*cx.app);
                        resolve_editor_text_field_style(theme, size, &ChromeRefinement::default())
                    };

                    let input_model = draft_for_input
                        .clone()
                        .unwrap_or_else(|| model_for_input.clone());
                    let mut props = TextInputProps::new(input_model);
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
                    props.a11y_label = a11y_label.clone();
                    props.test_id = test_id.clone();
                    props.obscure_text = matches!(mode, TextFieldMode::Password);
                    props.active_descendant = assistive_semantics.active_descendant;
                    props.active_descendant_element = assistive_semantics.active_descendant_element;
                    props.controls_element = assistive_semantics.controls_element;
                    props.expanded = assistive_semantics.expanded;
                    if !buffered {
                        props.submit_command = submit_command_for_input.clone();
                    }
                    if !buffered && matches!(cancel_behavior, EditorTextCancelBehavior::Clear) {
                        props.cancel_command = Some("text.clear".into());
                    }

                    props.chrome = joined_text_input_style(chrome);
                    props.text_style = typography::as_control_text(TextStyle {
                        line_height: Some(density.row_height),
                        ..text_style
                    });

                    let input = cx.text_input(props);
                    if let Some(out) = input_id_out.as_ref() {
                        out.set(Some(input.id));
                    }
                    let input_id = input.id;
                    let is_focused = cx.is_focused_element(input_id);

                    if let (Some(draft), Some(buffered_state)) =
                        (draft_for_input.as_ref(), buffered_state_for_input.as_ref())
                    {
                        sync_buffered_text_field_session(
                            cx,
                            input_id,
                            is_focused,
                            &current_text_for_input,
                            draft,
                            buffered_state,
                            blur_behavior,
                        );

                        let model_for_key = model_for_input.clone();
                        let draft_for_key = draft.clone();
                        let buffered_state_for_key = buffered_state.clone();
                        let on_outcome_for_key = on_outcome_for_input.clone();
                        let submit_command_for_key = submit_command_for_input.clone();
                        cx.key_add_on_key_down_capture_for(
                            input_id,
                            Arc::new(move |host, action_cx: ActionCx, down| {
                                if down.ime_composing || down.repeat {
                                    return false;
                                }
                                match down.key {
                                    KeyCode::Enter | KeyCode::NumpadEnter => {
                                        commit_buffered_text_field(
                                            host,
                                            action_cx,
                                            &model_for_key,
                                            &draft_for_key,
                                            &buffered_state_for_key,
                                            on_outcome_for_key.as_ref(),
                                            submit_command_for_key.as_ref(),
                                        )
                                    }
                                    KeyCode::Escape => cancel_buffered_text_field(
                                        host,
                                        action_cx,
                                        &model_for_key,
                                        &draft_for_key,
                                        &buffered_state_for_key,
                                        on_outcome_for_key.as_ref(),
                                    ),
                                    _ => false,
                                }
                            }),
                        );
                    }

                    let has_value = if let Some(draft) = draft_for_input.as_ref() {
                        cx.read_model_ref(draft, Invalidation::Paint, |s| !s.is_empty())
                            .unwrap_or(false)
                    } else {
                        cx.read_model_ref(&model_for_input, Invalidation::Paint, |s| !s.is_empty())
                            .unwrap_or(false)
                    };
                    sync_editor_text_entry_focus_selection(
                        cx,
                        &focus_state,
                        input_id,
                        is_focused,
                        has_value,
                        selection_behavior,
                    );
                    if let (Some(draft), Some(buffered_state)) =
                        (draft_for_input.as_ref(), buffered_state_for_input.as_ref())
                    {
                        install_buffered_text_field_blur_handler(
                            cx,
                            input_id,
                            model_for_input.clone(),
                            draft.clone(),
                            buffered_state.clone(),
                            on_outcome_for_input.clone(),
                        );
                    }
                    input
                }
            },
            move |cx| {
                let has_value = if let Some(draft) = draft_for_trailing.as_ref() {
                    cx.read_model_ref(draft, Invalidation::Layout, |s| !s.is_empty())
                        .unwrap_or(false)
                } else {
                    cx.read_model_ref(&model_for_trailing, Invalidation::Layout, |s| !s.is_empty())
                        .unwrap_or(false)
                };
                if !(clear_button && has_value && enabled_for_paint) {
                    return Vec::new();
                }

                let model_for_clear = model_for_trailing.clone();
                let on_activate: OnActivate = if let (Some(draft), Some(buffered_state)) = (
                    draft_for_trailing.clone(),
                    buffered_state_for_trailing.clone(),
                ) {
                    Arc::new(move |host, action_cx, _reason: ActivateReason| {
                        let _ = host.models_mut().update(&draft, |s| s.clear());
                        let _ = host.models_mut().update(&model_for_clear, |s| s.clear());
                        let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
                        state.was_focused = false;
                        clear_buffered_text_field_pending_blur(&mut state);
                        let _ = state.session.commit();
                        host.request_redraw(action_cx.window);
                    })
                } else {
                    Arc::new(move |host, action_cx, _reason: ActivateReason| {
                        let _ = host.models_mut().update(&model_for_clear, |s| s.clear());
                        host.request_redraw(action_cx.window);
                    })
                };

                if multiline {
                    vec![editor_clear_button_segment_multiline(
                        cx,
                        density,
                        frame_chrome,
                        enabled_for_paint,
                        Arc::from("Clear text"),
                        clear_test_id.clone(),
                        on_activate,
                    )]
                } else {
                    vec![editor_clear_button_segment(
                        cx,
                        density,
                        enabled_for_paint,
                        Arc::from("Clear text"),
                        clear_test_id.clone(),
                        on_activate,
                    )]
                }
            },
        );

        if let Some(out) = field_id_out.as_ref() {
            out.set(Some(field.id));
        }

        field
    }
}

#[track_caller]
fn draft_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.local_model(String::new)
}

#[track_caller]
fn buffered_state<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Arc<Mutex<BufferedTextFieldState>> {
    cx.slot_state(
        || Arc::new(Mutex::new(BufferedTextFieldState::default())),
        |st| st.clone(),
    )
}

fn sync_draft_from_model_when_session_inactive<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    draft: &Model<String>,
    buffered_state: &Arc<Mutex<BufferedTextFieldState>>,
    current_text: &str,
) {
    let session_active = buffered_state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .session
        .is_active();
    if session_active {
        return;
    }

    let next = current_text.to_owned();
    let _ = cx.app.models_mut().update(draft, |text| {
        if text.as_str() != next.as_str() {
            *text = next.clone();
        }
    });
}

fn sync_buffered_text_field_session<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input_id: GlobalElementId,
    is_focused: bool,
    current_text: &str,
    draft: &Model<String>,
    buffered_state: &Arc<Mutex<BufferedTextFieldState>>,
    blur_behavior: TextFieldBlurBehavior,
) {
    let (begin_session, cancel_blur_token, arm_blur_token) = {
        let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
        let plan = plan_buffered_text_field_focus_transition(
            state.was_focused,
            state.session.is_active(),
            is_focused,
            blur_behavior,
            state.blur_timer.is_some() || state.pending_blur.is_some(),
        );

        let cancel_blur_token = if plan.cancel_pending_blur {
            state.blur_timer.take()
        } else {
            None
        };
        let arm_blur_token = match plan.pending_blur {
            BufferedTextFieldPendingBlurPlan::Keep => None,
            BufferedTextFieldPendingBlurPlan::Clear => {
                state.blur_timer = None;
                state.pending_blur = None;
                None
            }
            BufferedTextFieldPendingBlurPlan::Arm(next_blur_behavior) => {
                let token = cx.app.next_timer_token();
                state.blur_timer = Some(token);
                state.pending_blur = Some(next_blur_behavior);
                Some(token)
            }
        };
        if plan.begin_session {
            state.session.begin(current_text.to_owned());
        }

        state.was_focused = is_focused;
        (plan.begin_session, cancel_blur_token, arm_blur_token)
    };

    if let Some(token) = cancel_blur_token {
        cx.cancel_timer(token);
    }
    if let Some(token) = arm_blur_token {
        cx.set_timer_for(input_id, token, Duration::ZERO);
    }

    if begin_session {
        let next = current_text.to_owned();
        let _ = cx.app.models_mut().update(draft, |text| {
            if text.as_str() != next.as_str() {
                *text = next.clone();
            }
        });
    }
}

fn install_buffered_text_field_blur_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input_id: GlobalElementId,
    model: Model<String>,
    draft: Model<String>,
    buffered_state: Arc<Mutex<BufferedTextFieldState>>,
    on_outcome: Option<OnTextFieldOutcome>,
) {
    cx.timer_add_on_timer_for(
        input_id,
        Arc::new(move |host, action_cx, token| {
            let blur_behavior = {
                let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
                if state.blur_timer != Some(token) {
                    return false;
                }
                state.blur_timer = None;
                state.pending_blur.take()
            };

            match blur_behavior {
                Some(TextFieldBlurBehavior::Commit) => commit_buffered_text_field(
                    host,
                    action_cx,
                    &model,
                    &draft,
                    &buffered_state,
                    on_outcome.as_ref(),
                    None,
                ),
                Some(TextFieldBlurBehavior::Cancel) => cancel_buffered_text_field(
                    host,
                    action_cx,
                    &model,
                    &draft,
                    &buffered_state,
                    on_outcome.as_ref(),
                ),
                Some(TextFieldBlurBehavior::PreserveDraft) | None => false,
            }
        }),
    );
}

fn clear_buffered_text_field_pending_blur(state: &mut BufferedTextFieldState) {
    state.blur_timer = None;
    state.pending_blur = None;
}

fn commit_buffered_text_field(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    model: &Model<String>,
    draft: &Model<String>,
    buffered_state: &Arc<Mutex<BufferedTextFieldState>>,
    on_outcome: Option<&OnTextFieldOutcome>,
    submit_command: Option<&CommandId>,
) -> bool {
    let next = host.models_mut().get_cloned(draft).unwrap_or_default();
    let should_emit_outcome = {
        let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
        let changed = state.session.changed_from(&next);
        clear_buffered_text_field_pending_blur(&mut state);
        let _ = state.session.commit();
        changed
    };

    {
        let next_for_update = next.clone();
        let _ = host.models_mut().update(model, |text| {
            if text.as_str() != next_for_update.as_str() {
                *text = next_for_update.clone();
            }
        });
    }
    if should_emit_outcome && let Some(cb) = on_outcome {
        cb(host, action_cx, TextFieldOutcome::Committed);
    }
    if let Some(command) = submit_command {
        host.dispatch_command(Some(action_cx.window), command.clone());
    }
    host.request_redraw(action_cx.window);
    true
}

fn cancel_buffered_text_field(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    model: &Model<String>,
    draft: &Model<String>,
    buffered_state: &Arc<Mutex<BufferedTextFieldState>>,
    on_outcome: Option<&OnTextFieldOutcome>,
) -> bool {
    let current_draft = host.models_mut().get_cloned(draft).unwrap_or_default();
    let current_model = host.models_mut().get_cloned(model).unwrap_or_default();
    let (revert, should_emit_outcome) = {
        let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
        let changed = state.session.changed_from(&current_draft);
        clear_buffered_text_field_pending_blur(&mut state);
        let revert = state
            .session
            .cancel()
            .unwrap_or_else(|| current_model.clone());
        (revert, changed)
    };

    {
        let revert_for_draft = revert.clone();
        let _ = host.models_mut().update(draft, |text| {
            if text.as_str() != revert_for_draft.as_str() {
                *text = revert_for_draft.clone();
            }
        });
    }
    let _ = host.models_mut().update(model, |text| {
        if text.as_str() != revert.as_str() {
            *text = revert.clone();
        }
    });
    if should_emit_outcome && let Some(cb) = on_outcome {
        cb(host, action_cx, TextFieldOutcome::Canceled);
    }
    host.request_redraw(action_cx.window);
    true
}

fn is_multiline_buffered_commit_shortcut(down: KeyDownCx) -> bool {
    (down.modifiers.ctrl || down.modifiers.meta) && !down.modifiers.alt && !down.modifiers.alt_gr
}

#[cfg(test)]
mod tests {
    use super::{
        BufferedTextFieldFocusPlan, BufferedTextFieldPendingBlurPlan, TextFieldBlurBehavior,
        TextFieldOptions, plan_buffered_text_field_focus_transition,
    };

    #[test]
    fn focus_begin_starts_session_and_clears_pending_blur() {
        assert_eq!(
            plan_buffered_text_field_focus_transition(
                false,
                false,
                true,
                TextFieldBlurBehavior::Commit,
                true,
            ),
            BufferedTextFieldFocusPlan {
                begin_session: true,
                cancel_pending_blur: true,
                pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
            }
        );
    }

    #[test]
    fn refocus_cancels_pending_blur_without_restarting_active_session() {
        assert_eq!(
            plan_buffered_text_field_focus_transition(
                false,
                true,
                true,
                TextFieldBlurBehavior::Commit,
                true,
            ),
            BufferedTextFieldFocusPlan {
                begin_session: false,
                cancel_pending_blur: true,
                pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
            }
        );
    }

    #[test]
    fn blur_commit_arms_pending_commit() {
        assert_eq!(
            plan_buffered_text_field_focus_transition(
                true,
                true,
                false,
                TextFieldBlurBehavior::Commit,
                false,
            ),
            BufferedTextFieldFocusPlan {
                begin_session: false,
                cancel_pending_blur: false,
                pending_blur: BufferedTextFieldPendingBlurPlan::Arm(TextFieldBlurBehavior::Commit),
            }
        );
    }

    #[test]
    fn blur_cancel_arms_pending_cancel() {
        assert_eq!(
            plan_buffered_text_field_focus_transition(
                true,
                true,
                false,
                TextFieldBlurBehavior::Cancel,
                false,
            ),
            BufferedTextFieldFocusPlan {
                begin_session: false,
                cancel_pending_blur: false,
                pending_blur: BufferedTextFieldPendingBlurPlan::Arm(TextFieldBlurBehavior::Cancel),
            }
        );
    }

    #[test]
    fn blur_preserve_draft_clears_pending_blur_without_arming_timer() {
        assert_eq!(
            plan_buffered_text_field_focus_transition(
                true,
                true,
                false,
                TextFieldBlurBehavior::PreserveDraft,
                true,
            ),
            BufferedTextFieldFocusPlan {
                begin_session: false,
                cancel_pending_blur: true,
                pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
            }
        );
    }

    #[test]
    fn active_unfocused_session_keeps_existing_pending_blur_state() {
        assert_eq!(
            plan_buffered_text_field_focus_transition(
                false,
                true,
                false,
                TextFieldBlurBehavior::Commit,
                true,
            ),
            BufferedTextFieldFocusPlan {
                begin_session: false,
                cancel_pending_blur: false,
                pending_blur: BufferedTextFieldPendingBlurPlan::Keep,
            }
        );
    }

    #[test]
    fn inactive_unfocused_state_clears_stale_pending_blur() {
        assert_eq!(
            plan_buffered_text_field_focus_transition(
                false,
                false,
                false,
                TextFieldBlurBehavior::Commit,
                true,
            ),
            BufferedTextFieldFocusPlan {
                begin_session: false,
                cancel_pending_blur: true,
                pending_blur: BufferedTextFieldPendingBlurPlan::Clear,
            }
        );
    }

    #[test]
    fn text_field_defaults_to_stable_line_boxes() {
        assert!(TextFieldOptions::default().stable_line_boxes);
    }
}
