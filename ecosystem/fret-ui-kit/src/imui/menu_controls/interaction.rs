use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::ActionId;
use fret_ui::action::{ActivateReason, UiActionHostExt as _};
use fret_ui::element::{Length, PressableA11y, PressableProps, PressableState};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::command::ElementCommandGatingExt as _;
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::imui::{
    KEY_CLICKED, MenuItemOptions, ResponseExt, active_trigger_behavior, imui_is_disabled,
    mark_lifecycle_instant_if_inactive,
};

use super::keyboard;

pub(super) struct MenuItemInteractionParts {
    pub(super) props: PressableProps,
    pub(super) runtime: MenuItemInteraction,
}

pub(super) struct MenuItemInteraction {
    pub(super) enabled: bool,
    pub(super) close_popup: Option<fret_runtime::Model<bool>>,
    pub(super) action: Option<ActionId>,
    pub(super) activate_shortcut: Option<fret_runtime::KeyChord>,
    pub(super) shortcut_repeat: bool,
    pub(super) menubar_policy: Option<ImUiMenubarPolicyState>,
}

pub(super) fn resolve_menu_item_interaction<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &Arc<str>,
    options: &MenuItemOptions,
    role: SemanticsRole,
    checked: Option<bool>,
    action: Option<ActionId>,
) -> MenuItemInteractionParts {
    let mut enabled = options.enabled && !imui_is_disabled(cx);
    if let Some(action) = action.as_ref() {
        enabled = enabled && cx.action_is_enabled(action);
    }

    let mut props = PressableProps::default();
    props.enabled = enabled;
    props.focusable = enabled;
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Auto;
    props.a11y = PressableA11y {
        role: Some(role),
        label: Some(label.clone()),
        test_id: options.test_id.clone(),
        checked,
        expanded: options.expanded,
        ..Default::default()
    };

    MenuItemInteractionParts {
        props,
        runtime: MenuItemInteraction {
            enabled,
            close_popup: options.close_popup.clone(),
            action,
            activate_shortcut: options.activate_shortcut,
            shortcut_repeat: options.shortcut_repeat,
            menubar_policy: cx.provided::<ImUiMenubarPolicyState>().cloned(),
        },
    }
}

pub(super) fn install_menu_item_interaction<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    interaction: &MenuItemInteraction,
) -> active_trigger_behavior::ActiveTriggerBehavior {
    let behavior = active_trigger_behavior::install_active_trigger_behavior(
        cx,
        id,
        active_trigger_behavior::ActiveTriggerBehaviorOptions::default(),
    );

    if !interaction.enabled {
        return behavior;
    }

    install_activate_handler(cx, &behavior, interaction);
    keyboard::install_popup_menu_keyboard(cx, id, &behavior, interaction);
    keyboard::install_menubar_keyboard(cx, id, interaction.menubar_policy.as_ref());
    behavior
}

pub(super) fn populate_menu_item_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &active_trigger_behavior::ActiveTriggerBehavior,
    enabled: bool,
    response: &mut ResponseExt,
) {
    let clicked = cx.take_transient_for(id, KEY_CLICKED);
    active_trigger_behavior::populate_active_trigger_response(
        cx,
        id,
        state,
        behavior,
        active_trigger_behavior::ActiveTriggerResponseInput {
            enabled,
            clicked,
            changed: false,
            lifecycle_edited: false,
        },
        response,
    );
}

fn install_activate_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    behavior: &active_trigger_behavior::ActiveTriggerBehavior,
    interaction: &MenuItemInteraction,
) {
    let close_popup_for_activate = interaction.close_popup.clone();
    let action_for_activate = interaction.action.clone();
    let lifecycle_model_for_activate = behavior.lifecycle_model.clone();
    cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
        if reason == ActivateReason::Keyboard {
            mark_lifecycle_instant_if_inactive(host, acx, &lifecycle_model_for_activate, false);
        }
        if let Some(open) = close_popup_for_activate.as_ref() {
            let _ = host.update_model(open, |v| *v = false);
        }
        host.record_transient_event(acx, KEY_CLICKED);
        dispatch_menu_item_action(host, acx, reason, action_for_activate.clone());
        host.notify(acx);
    }));
}

pub(super) fn dispatch_menu_item_action(
    host: &mut dyn fret_ui::action::UiActionHost,
    acx: fret_ui::action::ActionCx,
    reason: ActivateReason,
    action: Option<ActionId>,
) {
    if let Some(action) = action {
        host.record_pending_command_dispatch_source(acx, &action, reason);
        host.dispatch_command(Some(acx.window), action);
    }
}
