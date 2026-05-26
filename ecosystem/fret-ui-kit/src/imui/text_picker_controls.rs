//! Immediate-mode input text picker recipes.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::UiHost;
use fret_ui::element::{ContainerProps, Length};

use super::{
    InputTextPickerOptions, InputTextPickerResponse, ResponseExt, SelectableOptions,
    UiWriterImUiFacadeExt,
};

mod keyboard;

use keyboard::{InputTextPickerKeyboardState, install_picker_keyboard_handler};

pub(super) fn input_text_completion_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    model: &fret_runtime::Model<String>,
    candidates: &[Arc<str>],
    options: InputTextPickerOptions,
) -> InputTextPickerResponse {
    input_text_picker_model_with_options(ui, id, model, candidates, options)
}

pub(super) fn input_text_history_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    model: &fret_runtime::Model<String>,
    history: &[Arc<str>],
    mut options: InputTextPickerOptions,
) -> InputTextPickerResponse {
    options.filter = super::InputTextPickerFilter::None;
    options.open_when_empty = true;
    options.hide_when_exact_match = false;
    input_text_picker_model_with_options(ui, id, model, history, options)
}

fn input_text_picker_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    model: &fret_runtime::Model<String>,
    candidates: &[Arc<str>],
    options: InputTextPickerOptions,
) -> InputTextPickerResponse {
    let current = ui.with_cx_mut(|cx| {
        cx.read_model(model, fret_ui::Invalidation::Paint, |_app, value| {
            value.clone()
        })
        .unwrap_or_default()
    });

    let test_id = options
        .test_id
        .clone()
        .or_else(|| options.input.test_id.clone());
    let mut input_options = options.input.clone();
    if input_options.test_id.is_none() {
        input_options.test_id = test_id
            .as_ref()
            .map(|base| Arc::from(format!("{base}.input")));
    }
    if matches!(input_options.a11y_role, Some(SemanticsRole::TextField)) {
        input_options.a11y_role = Some(SemanticsRole::ComboBox);
    }

    let visible_candidates = visible_candidates(&current, candidates, &options);
    let hide_for_exact_match = options.hide_when_exact_match
        && candidates
            .iter()
            .any(|candidate| candidate.as_ref() == current.as_str());
    let popup_open = ui.popup_open_model(id);
    let picker_candidate_visible =
        !visible_candidates.is_empty() && (options.open_when_empty || !current.is_empty());
    let input_enabled_by_scope =
        ui.with_cx_mut(|cx| options.input.enabled && !super::imui_is_disabled(cx));
    let keyboard_state = options.keyboard_navigation.then(|| {
        ui.with_cx_mut(|cx| {
            cx.local_model_keyed(
                format!("fret-ui-kit.imui.input-text-picker.keyboard.{id}"),
                InputTextPickerKeyboardState::default,
            )
        })
    });
    let (popup_is_open, popup_panel_id) = ui.with_cx_mut(|cx| {
        let open = cx
            .read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
                *value
            })
            .unwrap_or(false);
        let panel_id = super::with_popup_store_for_id(cx, id, |st, _app| st.panel_id);
        (open, panel_id)
    });
    let (active_source_index, pending_keyboard_pick, active_element) = keyboard_state
        .as_ref()
        .and_then(|state| {
            ui.with_cx_mut(|cx| {
                cx.app
                    .models_mut()
                    .update(state, |state| {
                        let picked = state.picked.take();
                        if !input_enabled_by_scope
                            || visible_candidates.is_empty()
                            || hide_for_exact_match
                        {
                            state.active_source_index = None;
                            state.active_element = None;
                        } else if let Some(active) = state.active_source_index
                            && !visible_candidates
                                .iter()
                                .any(|(source_index, _)| *source_index == active)
                        {
                            state.active_source_index = None;
                            state.active_element = None;
                        } else if state.active_source_index.is_none() {
                            state.active_element = None;
                        }
                        (state.active_source_index, picked, state.active_element)
                    })
                    .ok()
            })
        })
        .unwrap_or((None, None, None));
    let picker_expanded = popup_is_open
        && input_enabled_by_scope
        && picker_candidate_visible
        && !hide_for_exact_match;
    let assistive_semantics = super::text_controls::InputTextAssistiveSemantics {
        active_descendant: None,
        active_descendant_element: picker_expanded
            .then_some(active_element)
            .flatten()
            .map(|element| element.0),
        controls_element: picker_expanded
            .then_some(popup_panel_id)
            .flatten()
            .map(|element| element.0),
        expanded: Some(picker_expanded),
    };
    let mut input = ResponseExt::default();
    let root = ui.with_cx_mut(|cx| {
        let input_element =
            super::text_controls::input_text_model_element_with_options_and_semantics(
                cx,
                model.clone(),
                input_options,
                assistive_semantics,
                &mut input,
            );

        let mut props = ContainerProps::default();
        props.layout.size.width = Length::Fill;
        props.layout.size.height = Length::Auto;
        let root = cx.container(props, |_cx| vec![input_element]);
        if input.enabled()
            && options.keyboard_navigation
            && input.focused()
            && picker_candidate_visible
            && !hide_for_exact_match
            && let Some(state) = keyboard_state.clone()
        {
            install_picker_keyboard_handler(
                cx,
                root.id,
                model.clone(),
                popup_open.clone(),
                state,
                visible_candidates.clone(),
                options.keyboard_repeat,
            );
        }
        root
    });
    ui.add(root);
    let enabled = input.enabled();

    if enabled && (visible_candidates.is_empty() || hide_for_exact_match) {
        ui.close_popup(id);
    }
    if enabled
        && options.open_on_focus
        && input.focused()
        && picker_candidate_visible
        && !hide_for_exact_match
        && let Some(anchor) = input.rect()
    {
        ui.open_popup_at(id, anchor);
    }

    let mut picked_index = pending_keyboard_pick.as_ref().map(|pick| pick.source_index);
    let mut picked = pending_keyboard_pick
        .as_ref()
        .map(|pick| pick.value.clone());
    let item_test_id_base = test_id.clone();
    let selected_value = current.clone();
    let model_for_pick = model.clone();
    let popup_open_for_items = popup_open.clone();
    let keyboard_state_for_popup = keyboard_state.clone();
    let visible_candidates_for_popup_key = visible_candidates.clone();
    let popup_open_for_popup_key = popup_open.clone();
    let model_for_popup_key = model.clone();
    let install_popup_keyboard_handler = enabled
        && options.keyboard_navigation
        && input.focused()
        && picker_candidate_visible
        && !hide_for_exact_match;
    let keyboard_repeat = options.keyboard_repeat;
    let opened = ui.begin_popup_menu_with_options(id, input.id(), options.popup, |ui| {
        if install_popup_keyboard_handler
            && let Some(keyboard_state) = keyboard_state_for_popup.clone()
        {
            let cx = ui.cx_mut();
            let key_owner = cx.root_id();
            install_picker_keyboard_handler(
                cx,
                key_owner,
                model_for_popup_key.clone(),
                popup_open_for_popup_key.clone(),
                keyboard_state,
                visible_candidates_for_popup_key.clone(),
                keyboard_repeat,
            );
        }
        for (visible_index, (source_index, candidate)) in visible_candidates.iter().enumerate() {
            let checked = selected_value.as_str() == candidate.as_ref();
            let active = active_source_index == Some(*source_index);
            let item_test_id = item_test_id_base
                .as_ref()
                .map(|base| Arc::from(format!("{base}.option.{visible_index}")));
            let response = ui.selectable_with_options(
                candidate.clone(),
                SelectableOptions {
                    selected: checked,
                    highlighted: active,
                    test_id: item_test_id,
                    ..Default::default()
                },
            );
            if active
                && let (Some(state), Some(element)) =
                    (keyboard_state_for_popup.as_ref(), response.id())
            {
                let _ = ui
                    .cx_mut()
                    .app
                    .models_mut()
                    .update(state, |state| state.active_element = Some(element));
            }
            if response.clicked() {
                let next_value = candidate.to_string();
                let _ = ui
                    .cx_mut()
                    .app
                    .models_mut()
                    .update(&model_for_pick, |value| *value = next_value.clone());
                let _ = ui
                    .cx_mut()
                    .app
                    .models_mut()
                    .update(&popup_open_for_items, |open| *open = false);
                if let Some(state) = keyboard_state_for_popup.as_ref() {
                    let _ = ui.cx_mut().app.models_mut().update(state, |state| {
                        state.active_source_index = Some(*source_index);
                        state.active_element = response.id();
                    });
                }
                picked_index = Some(*source_index);
                picked = Some(candidate.clone());
            }
        }
    });

    if picked.is_some() {
        let selected_now = ui.with_cx_mut(|cx| {
            cx.read_model(model, fret_ui::Invalidation::Paint, |_app, value| {
                value.clone()
            })
            .unwrap_or_default()
        });
        let picked_changed = input.id().is_some_and(|element_id| {
            ui.with_cx_mut(|cx| super::model_value_changed_for(cx, element_id, selected_now))
        });
        input.merge_core_changed(picked_changed);
        input.merge_edited(picked_changed);
        input.merge_deactivated_after_edit(picked_changed);
    }

    InputTextPickerResponse {
        input,
        open: opened,
        picked_index,
        picked,
    }
}

fn visible_candidates(
    current: &str,
    candidates: &[Arc<str>],
    options: &InputTextPickerOptions,
) -> Vec<(usize, Arc<str>)> {
    candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| options.filter.matches(current, candidate.as_ref()))
        .take(options.max_items)
        .map(|(index, candidate)| (index, candidate.clone()))
        .collect()
}
