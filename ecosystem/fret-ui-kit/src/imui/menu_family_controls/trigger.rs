use std::sync::Arc;

use fret_core::{KeyCode, SemanticsRole};
use fret_runtime::Model;
use fret_ui::UiHost;
use fret_ui::action::ActivateReason;
use fret_ui::element::{PressableA11y, PressableProps};

use crate::imui::label_identity::parse_label_identity;
use crate::imui::{
    KEY_CLICKED, ResponseExt, UiWriterImUiFacadeExt, active_trigger_behavior,
    mark_lifecycle_instant_if_inactive,
};
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

use super::{ImUiMenubarPolicyState, visual};

pub(super) fn menu_trigger_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    logical_key: Arc<str>,
    label: Arc<str>,
    open: bool,
    open_model: Model<bool>,
    menubar_policy: Option<ImUiMenubarPolicyState>,
    enabled: bool,
    test_id: Option<Arc<str>>,
    activate_shortcut: Option<fret_runtime::KeyChord>,
    shortcut_repeat: bool,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let label = Arc::<str>::from(parts.visible);
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled;
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::MenuItem),
            label: Some(label.clone()),
            test_id,
            expanded: Some(open),
            ..Default::default()
        };

        cx.pressable_with_id(props, move |cx, state, id| {
            let open_model = open_model.clone();
            let menubar_policy = menubar_policy.clone();
            let logical_key = logical_key.clone();
            let behavior = active_trigger_behavior::install_active_trigger_behavior(
                cx,
                id,
                active_trigger_behavior::ActiveTriggerBehaviorOptions::default(),
            );
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

            cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
                if reason == ActivateReason::Keyboard {
                    mark_lifecycle_instant_if_inactive(
                        host,
                        acx,
                        &lifecycle_model_for_activate,
                        false,
                    );
                }
                host.record_transient_event(acx, KEY_CLICKED);
                host.notify(acx);
            }));

            if enabled {
                let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();
                cx.key_on_key_down_for(
                    id,
                    Arc::new(move |host, acx, down| {
                        if let Some(shortcut) = activate_shortcut {
                            let matches_shortcut =
                                down.key == shortcut.key && down.modifiers == shortcut.mods;
                            if matches_shortcut
                                && (!down.repeat || shortcut_repeat)
                                && !down.ime_composing
                            {
                                mark_lifecycle_instant_if_inactive(
                                    host,
                                    acx,
                                    &lifecycle_model_for_shortcut,
                                    false,
                                );
                                host.record_transient_event(acx, KEY_CLICKED);
                                host.notify(acx);
                                return true;
                            }
                        }

                        false
                    }),
                );
            }

            if let Some(menubar_policy) = menubar_policy.as_ref() {
                let (patient_click_sticky, patient_click_timer) =
                    menubar_trigger_row::ensure_trigger_patient_click_models(cx, id);
                menubar_trigger_row::register_trigger_in_registry(
                    cx,
                    menubar_policy.registry.clone(),
                    logical_key.clone(),
                    id,
                    open_model.clone(),
                    enabled,
                    None,
                );
                menubar_trigger_row::sync_trigger_row_state(
                    cx,
                    menubar_policy.group_active.clone(),
                    id,
                    open_model.clone(),
                    patient_click_sticky.clone(),
                    patient_click_timer.clone(),
                    enabled,
                    state.hovered || state.hovered_raw || state.hovered_raw_below_barrier,
                    state.pressed,
                    state.focused,
                );
                cx.pressable_add_on_activate(menubar_trigger_row::toggle_on_activate(
                    menubar_policy.group_active.clone(),
                    id,
                    open_model.clone(),
                    patient_click_sticky,
                    patient_click_timer,
                ));
                let open_model_for_arrows = open_model.clone();
                cx.key_add_on_key_down_for(
                    id,
                    Arc::new(move |host, _acx, down| {
                        if down.repeat {
                            return false;
                        }
                        match down.key {
                            KeyCode::ArrowDown | KeyCode::ArrowUp => {
                                let _ = host
                                    .models_mut()
                                    .update(&open_model_for_arrows, |value| *value = true);
                                true
                            }
                            _ => false,
                        }
                    }),
                );
            }

            let clicked = cx.take_transient_for(id, KEY_CLICKED);
            active_trigger_behavior::populate_active_trigger_response(
                cx,
                id,
                state,
                &behavior,
                active_trigger_behavior::ActiveTriggerResponseInput {
                    enabled,
                    clicked,
                    changed: false,
                    lifecycle_edited: false,
                },
                response,
            );

            vec![visual::menu_trigger_visual(
                cx,
                label.clone(),
                open,
                enabled,
                state,
            )]
        })
    });

    ui.add(element);
    response
}
