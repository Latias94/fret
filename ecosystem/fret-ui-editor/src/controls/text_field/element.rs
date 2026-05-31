use std::panic::Location;
use std::sync::Arc;

use fret_core::{KeyCode, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, TextAreaProps, TextInputProps};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::ChromeRefinement;
use fret_ui_kit::typography;

use super::{OnTextFieldOutcome, TextField, TextFieldMode, TextFieldOptions, buffered};
use crate::primitives::chrome::{
    joined_text_area_style, joined_text_input_style, resolve_editor_text_area_field_style,
    resolve_editor_text_field_style,
};
use crate::primitives::input_group::{
    editor_clear_button_segment, editor_clear_button_segment_multiline, editor_joined_input_frame,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::text_entry::{
    EditorTextCancelBehavior, editor_text_entry_focus_state, sync_editor_text_entry_focus_selection,
};

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
        let draft_controller = options.draft_controller.clone();
        let submit_command = options.submit_command.clone();
        let assistive_semantics = options.assistive_semantics;
        let selection_behavior = options.selection_behavior;
        let cancel_behavior = options.cancel_behavior;
        let multiline = options.multiline;
        let stable_line_boxes = options.stable_line_boxes;
        let min_height = options.min_height;
        let focus_state = editor_text_entry_focus_state(cx);
        let draft = buffered.then(|| buffered::draft_model(cx));
        let buffered_state = buffered.then(|| buffered::buffered_state(cx));
        let current_text = cx
            .get_model_cloned(&model, Invalidation::Paint)
            .unwrap_or_default();

        if let (Some(draft), Some(buffered_state)) = (draft.as_ref(), buffered_state.as_ref()) {
            buffered::sync_draft_from_model_when_session_inactive(
                cx,
                draft,
                buffered_state,
                &current_text,
            );
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
        let draft_controller_for_input = draft_controller.clone();
        let current_text_for_input = current_text.clone();
        let on_outcome_for_input = on_outcome.clone();
        let submit_command_for_input = submit_command.clone();
        if !buffered && let Some(controller) = draft_controller.as_ref() {
            controller.unbind();
        }

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
                        buffered::sync_buffered_text_field_session(
                            cx,
                            area_id,
                            is_focused,
                            &current_text_for_input,
                            draft,
                            buffered_state,
                            blur_behavior,
                        );
                        if let Some(controller) = draft_controller_for_input.as_ref() {
                            controller.bind(
                                model_for_input.clone(),
                                draft.clone(),
                                buffered_state.clone(),
                                None,
                            );
                        }

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
                                    KeyCode::Escape => buffered::cancel_buffered_text_field(
                                        host,
                                        action_cx,
                                        &model_for_key,
                                        &draft_for_key,
                                        &buffered_state_for_key,
                                        on_outcome_for_key.as_ref(),
                                    ),
                                    KeyCode::Enter | KeyCode::NumpadEnter
                                        if buffered::is_multiline_buffered_commit_shortcut(
                                            down,
                                        ) =>
                                    {
                                        buffered::commit_buffered_text_field(
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
                        buffered::install_buffered_text_field_blur_handler(
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
                        buffered::sync_buffered_text_field_session(
                            cx,
                            input_id,
                            is_focused,
                            &current_text_for_input,
                            draft,
                            buffered_state,
                            blur_behavior,
                        );
                        if let Some(controller) = draft_controller_for_input.as_ref() {
                            controller.bind(
                                model_for_input.clone(),
                                draft.clone(),
                                buffered_state.clone(),
                                submit_command_for_input.clone(),
                            );
                        }

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
                                        buffered::commit_buffered_text_field(
                                            host,
                                            action_cx,
                                            &model_for_key,
                                            &draft_for_key,
                                            &buffered_state_for_key,
                                            on_outcome_for_key.as_ref(),
                                            submit_command_for_key.as_ref(),
                                        )
                                    }
                                    KeyCode::Escape => buffered::cancel_buffered_text_field(
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
                        buffered::install_buffered_text_field_blur_handler(
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
                        buffered::clear_buffered_text_field_state(&mut state);
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
