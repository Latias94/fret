//! Immediate-mode input text picker recipes.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::UiHost;
use fret_ui::element::{ContainerProps, Length};

use super::{InputTextPickerOptions, InputTextPickerResponse, ResponseExt, UiWriterImUiFacadeExt};

mod candidates;
mod keyboard;
mod popup;

use candidates::resolve_text_picker_candidates;
use keyboard::{
    InputTextPickerKeyboardState, install_picker_keyboard_handler, reconcile_picker_keyboard_state,
};
use popup::{InputTextPickerPopupInput, render_text_picker_popup};

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

    let candidate_visibility = resolve_text_picker_candidates(&current, candidates, &options);
    let visible_candidates = candidate_visibility.visible_candidates;
    let hide_for_exact_match = candidate_visibility.hide_for_exact_match;
    let popup_open = ui.popup_open_model(id);
    let picker_candidate_visible = candidate_visibility.picker_candidate_visible;
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
                reconcile_picker_keyboard_state(
                    cx,
                    state,
                    input_enabled_by_scope,
                    &visible_candidates,
                    hide_for_exact_match,
                )
            })
        })
        .map(|snapshot| {
            (
                snapshot.active_source_index,
                snapshot.pending_pick,
                snapshot.active_element,
            )
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

    let popup = render_text_picker_popup(
        ui,
        InputTextPickerPopupInput {
            id,
            trigger: input.id(),
            popup: options.popup,
            model: model.clone(),
            popup_open: popup_open.clone(),
            keyboard_state: keyboard_state.clone(),
            visible_candidates: &visible_candidates,
            selected_value: current.clone(),
            active_source_index,
            pending_keyboard_pick,
            item_test_id_base: test_id.clone(),
            install_keyboard_handler: enabled
                && options.keyboard_navigation
                && input.focused()
                && picker_candidate_visible
                && !hide_for_exact_match,
            keyboard_repeat: options.keyboard_repeat,
        },
    );
    let opened = popup.opened;
    let picked_index = popup.picked_index;
    let picked = popup.picked;

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
