use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::ActivateReason;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::{AnyElement, PressableA11y, PressableProps};
use fret_ui::{ElementContext, UiHost};

use super::{BuiltTabItem, visual};
use crate::imui::ResponseExt;
use crate::primitives::tabs;

pub(super) struct BuiltTabTrigger {
    pub(super) element: AnyElement,
    pub(super) response: ResponseExt,
}

pub(super) fn render_tab_trigger<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected_model: &Model<Option<Arc<str>>>,
    item: &BuiltTabItem,
    selected: bool,
    pos_in_set: u32,
    set_size: u32,
) -> BuiltTabTrigger {
    let mut response = ResponseExt::default();
    let label = item.label.clone();
    let test_id = item.test_id.clone();
    let selected_model = selected_model.clone();
    let tab_id = item.id.clone();
    let enabled = item.enabled;
    let activate_shortcut = item.activate_shortcut;
    let shortcut_repeat = item.shortcut_repeat;

    let element = cx.keyed(("tab-trigger", item.id.clone()), |cx| {
        let response = &mut response;
        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled;
        props.a11y = PressableA11y {
            test_id: test_id.clone(),
            ..tabs::tab_a11y_with_collection(
                Some(label.clone()),
                selected,
                Some(pos_in_set),
                Some(set_size),
            )
        };

        cx.pressable_with_id(props, move |cx, state, element_id| {
            let behavior = super::super::active_trigger_behavior::install_active_trigger_behavior(
                cx,
                element_id,
                super::super::active_trigger_behavior::ActiveTriggerBehaviorOptions::default(),
            );
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

            if enabled {
                let selected_model_for_activate = selected_model.clone();
                let tab_id_for_activate = tab_id.clone();
                cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
                    if reason == ActivateReason::Keyboard {
                        super::super::mark_lifecycle_instant_if_inactive(
                            host,
                            acx,
                            &lifecycle_model_for_activate,
                            false,
                        );
                    }
                    let _ = host.update_model(&selected_model_for_activate, |value| {
                        *value = Some(tab_id_for_activate.clone())
                    });
                    host.record_transient_event(acx, super::super::KEY_CLICKED);
                    host.notify(acx);
                }));

                let selected_model_for_shortcut = selected_model.clone();
                let tab_id_for_shortcut = tab_id.clone();
                let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();
                cx.key_on_key_down_for(
                    element_id,
                    Arc::new(move |host, acx, down| {
                        if let Some(shortcut) = activate_shortcut {
                            let matches_shortcut =
                                down.key == shortcut.key && down.modifiers == shortcut.mods;
                            if matches_shortcut
                                && (!down.repeat || shortcut_repeat)
                                && !down.ime_composing
                            {
                                super::super::mark_lifecycle_instant_if_inactive(
                                    host,
                                    acx,
                                    &lifecycle_model_for_shortcut,
                                    false,
                                );
                                let _ = host.update_model(&selected_model_for_shortcut, |value| {
                                    *value = Some(tab_id_for_shortcut.clone())
                                });
                                host.record_transient_event(acx, super::super::KEY_CLICKED);
                                host.notify(acx);
                                return true;
                            }
                        }

                        false
                    }),
                );
            }

            let clicked = cx.take_transient_for(element_id, super::super::KEY_CLICKED);
            super::super::active_trigger_behavior::populate_active_trigger_response(
                cx,
                element_id,
                state,
                &behavior,
                super::super::active_trigger_behavior::ActiveTriggerResponseInput {
                    enabled,
                    clicked,
                    changed: false,
                    lifecycle_edited: false,
                },
                response,
            );

            vec![visual::tab_trigger_visual(
                cx,
                label.clone(),
                selected,
                enabled,
                state,
            )]
        })
    });

    BuiltTabTrigger { element, response }
}
