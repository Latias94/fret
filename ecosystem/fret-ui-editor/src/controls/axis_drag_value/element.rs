use std::sync::{Arc, Mutex};

use fret_core::{KeyCode, Px, SemanticsInvalid};
use fret_ui::action::{
    ActionCx, PointerDownCx, PressablePointerDownResult, UiActionHost, UiFocusActionHost,
};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, TextInputProps};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::ChromeRefinement;

use crate::primitives::chrome::{joined_text_input_style, resolve_editor_text_field_style};
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::input_group::{
    editor_axis_segment, editor_icon_button_segment, editor_input_group_divider,
    editor_input_group_frame, editor_input_group_inset, editor_input_group_row,
    editor_input_value_text, editor_text_segment,
};
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;
use crate::primitives::numeric_text_entry::{
    NumericTextEntryFocusHandoffState, arm_numeric_text_entry_focus_handoff,
    clear_numeric_error_when_draft_changes, handle_numeric_text_entry_replace_key,
    numeric_text_entry_focus_state, sync_numeric_text_entry_focus,
    sync_numeric_text_entry_focus_handoff,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};
use crate::primitives::{DragValueCore, DragValueCoreOptions, constrain_numeric_value};

use super::AxisDragValue;
use super::ids::axis_drag_value_test_ids;
use super::model::{
    AxisDragValueMode, AxisDragValueOutcome, AxisDragValueState, axis_drag_value_input_text_style,
};
use super::session::{draft_model, emit_axis_drag_value_outcome, error_model, hidden_layout};

mod typing;

use typing::{AxisDragValueTypingFrameArgs, axis_drag_value_typing_field};

impl<T> AxisDragValue<T>
where
    T: DragValueScalar + Default,
{
    pub(super) fn into_element_keyed<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        let state: Arc<Mutex<AxisDragValueState>> = cx.slot_state(
            || Arc::new(Mutex::new(AxisDragValueState::default())),
            |s| s.clone(),
        );
        let focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>> = cx.slot_state(
            || Arc::new(Mutex::new(NumericTextEntryFocusHandoffState::default())),
            |s| s.clone(),
        );
        let on_outcome = self.on_outcome.clone();

        let draft = draft_model(cx);
        let error = error_model(cx);
        let focus_state = numeric_text_entry_focus_state(cx);
        let last_draft_text =
            cx.slot_state(|| Arc::new(Mutex::new(String::new())), |st| st.clone());

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
        let (prefix, suffix) = suppress_duplicate_chrome_affixes(
            value_text.as_ref(),
            self.options.prefix.clone(),
            self.options.suffix.clone(),
        );
        let reset_action = self.options.reset.clone();
        let test_ids = axis_drag_value_test_ids(
            self.options.test_id.clone(),
            reset_action
                .as_ref()
                .and_then(|reset| reset.test_id.clone()),
            typing,
        );
        let scrub_test_id = test_ids.scrub.clone();
        let active_typing_test_id = test_ids.active_typing.clone();
        let scrub_axis_test_id = test_ids.scrub_axis.clone();
        let scrub_value_test_id = test_ids.scrub_value.clone();
        let scrub_prefix_test_id = test_ids.scrub_prefix.clone();
        let scrub_suffix_test_id = test_ids.scrub_suffix.clone();
        let typing_axis_test_id = test_ids.typing_axis.clone();
        let typing_input_test_id = test_ids.typing_input.clone();
        let typing_prefix_test_id = test_ids.typing_prefix.clone();
        let typing_suffix_test_id = test_ids.typing_suffix.clone();
        let typing_error_icon_test_id = test_ids.typing_error_icon.clone();
        let scrub_reset_test_id = test_ids.scrub_reset.clone();
        let typing_reset_test_id = test_ids.typing_reset.clone();

        let (density, frame_chrome, (text_style, input_chrome)) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let frame_chrome = style.frame_chrome_small();
            let (input_chrome, text_style) = resolve_editor_text_field_style(
                theme,
                self.options.size,
                &ChromeRefinement::default(),
            );
            let text_style = axis_drag_value_input_text_style(text_style, style.density.row_height);

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
        scrub_opts.constraints = self.options.constraints;

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
                                hovered: resp.hovered(),
                                pressed: resp.dragging() || resp.pressed(),
                                focused: resp.focused() || cx.is_focused_element(scrub_id),
                                open: false,
                                semantic: EditorFrameSemanticState::default(),
                            },
                            move |cx, visuals| {
                                let affix_color = {
                                    let theme = Theme::global(&*cx.app);
                                    editor_muted_foreground(theme)
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
                                let value_text_el = editor_input_value_text(
                                    cx,
                                    density,
                                    frame_chrome.text_px,
                                    value_text_for_scrub.clone(),
                                    visuals.fg,
                                    Length::Fill,
                                );
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
        let constraints = self.options.constraints;
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

        props.chrome = joined_text_input_style(input_chrome);
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
                    ) && consumed
                    {
                        return true;
                    }

                    match down.key {
                        KeyCode::Enter | KeyCode::NumpadEnter => {
                            let text = host
                                .models_mut()
                                .read(&draft_for_keys, |s| s.clone())
                                .unwrap_or_default();
                            if let Some(v) = (parse)(&text) {
                                let v = constrain_numeric_value(constraints, v);
                                if let Some(validate) = validate.as_ref()
                                    && let Some(msg) = validate(v)
                                {
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

        let typing_field = axis_drag_value_typing_field(
            cx,
            AxisDragValueTypingFrameArgs {
                layout: input_group_layout,
                density,
                frame_chrome,
                is_focused,
                has_error,
                input,
                axis_label: self.axis_label.clone(),
                axis_tint: self.axis_tint,
                prefix: prefix.clone(),
                suffix: suffix.clone(),
                reset_action: self.options.reset.clone(),
                enabled: self.options.enabled,
                active_typing_test_id,
                typing_axis_test_id,
                typing_prefix_test_id,
                typing_suffix_test_id,
                typing_error_icon_test_id,
                typing_reset_test_id,
            },
        );

        // Render both: scrub stays mounted so focus can restore, typing stays mounted so focus
        // requests have a stable target.
        cx.container(Default::default(), move |_cx| vec![scrub, typing_field])
    }
}
