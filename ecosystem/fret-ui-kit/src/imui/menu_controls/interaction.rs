use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{KeyCode, Modifiers, SemanticsRole};
use fret_runtime::ActionId;
use fret_ui::action::{ActivateReason, KeyDownCx, UiActionHostExt as _};
use fret_ui::element::{Length, PressableA11y, PressableProps, PressableState};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::command::ElementCommandGatingExt as _;
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::imui::{
    KEY_CLICKED, MenuItemOptions, ResponseExt, active_trigger_behavior, imui_is_disabled,
    mark_lifecycle_instant_if_inactive,
};
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

pub(super) struct MenuItemInteractionParts {
    pub(super) props: PressableProps,
    pub(super) runtime: MenuItemInteraction,
}

pub(super) struct MenuItemInteraction {
    pub(super) enabled: bool,
    close_popup: Option<fret_runtime::Model<bool>>,
    action: Option<ActionId>,
    activate_shortcut: Option<fret_runtime::KeyChord>,
    shortcut_repeat: bool,
    menubar_policy: Option<ImUiMenubarPolicyState>,
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
    install_popup_menu_keyboard(cx, id, &behavior, interaction);
    install_menubar_keyboard(cx, id, interaction.menubar_policy.as_ref());
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

fn install_popup_menu_keyboard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    behavior: &active_trigger_behavior::ActiveTriggerBehavior,
    interaction: &MenuItemInteraction,
) {
    let nav_items = cx
        .inherited_state::<crate::imui::popup_overlay::ImUiMenuNavState>()
        .map(|st| st.items.clone());
    if let Some(nav_items) = nav_items.as_ref() {
        nav_items.borrow_mut().push(id);
    }
    let Some(nav_items) = nav_items else {
        return;
    };

    let close_popup_for_key = interaction.close_popup.clone();
    let action_for_shortcut = interaction.action.clone();
    let activate_shortcut = interaction.activate_shortcut;
    let shortcut_repeat = interaction.shortcut_repeat;
    let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();
    cx.key_on_key_down_for(
        id,
        Arc::new(move |host, acx, down| {
            if let Some(shortcut) = activate_shortcut {
                let matches_shortcut = down.key == shortcut.key && down.modifiers == shortcut.mods;
                if matches_shortcut && (!down.repeat || shortcut_repeat) && !down.ime_composing {
                    mark_lifecycle_instant_if_inactive(
                        host,
                        acx,
                        &lifecycle_model_for_shortcut,
                        false,
                    );
                    if let Some(open) = close_popup_for_key.as_ref() {
                        let _ = host.update_model(open, |v| *v = false);
                    }
                    host.record_transient_event(acx, KEY_CLICKED);
                    dispatch_menu_item_action(
                        host,
                        acx,
                        ActivateReason::Keyboard,
                        action_for_shortcut.clone(),
                    );
                    host.notify(acx);
                    return true;
                }
            }

            move_popup_menu_focus(host, acx, &down, &nav_items, id)
        }),
    );
}

fn install_menubar_keyboard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    menubar_policy: Option<&ImUiMenubarPolicyState>,
) {
    let Some(menubar_policy) = menubar_policy else {
        return;
    };

    let suppress_close_auto_focus = menubar_policy.suppress_close_auto_focus_once.clone();
    cx.key_prepend_on_key_down_for(
        id,
        Arc::new(move |host, _acx, down| {
            if down.repeat || down.modifiers != Modifiers::default() {
                return false;
            }
            if matches!(down.key, KeyCode::ArrowLeft | KeyCode::ArrowRight) {
                let _ = host
                    .models_mut()
                    .update(&suppress_close_auto_focus, |value| *value = true);
            }
            false
        }),
    );
    menubar_trigger_row::wire_switch_open_menu_on_horizontal_arrows(
        cx,
        id,
        menubar_policy.group_active.clone(),
        menubar_policy.registry.clone(),
    );
}

fn move_popup_menu_focus(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    acx: fret_ui::action::ActionCx,
    down: &KeyDownCx,
    nav_items: &Rc<RefCell<Vec<GlobalElementId>>>,
    item_id: GlobalElementId,
) -> bool {
    if down.repeat || down.modifiers != Modifiers::default() {
        return false;
    }

    let (dir, jump_to) = match down.key {
        KeyCode::ArrowDown => (1isize, None),
        KeyCode::ArrowUp => (-1isize, None),
        KeyCode::Home => (0isize, Some(0usize)),
        KeyCode::End => (0isize, Some(usize::MAX)),
        _ => return false,
    };

    let items = nav_items.borrow();
    if items.is_empty() {
        return false;
    }
    let len = items.len();
    let idx = items.iter().position(|id| *id == item_id);
    let next_idx = if let Some(jump) = jump_to {
        if jump == usize::MAX {
            len - 1
        } else {
            jump.min(len - 1)
        }
    } else {
        let current = idx.unwrap_or_else(|| if dir < 0 { len - 1 } else { 0 });
        ((current as isize + dir + len as isize) % len as isize) as usize
    };

    host.request_focus(items[next_idx]);
    host.notify(acx);
    true
}

fn dispatch_menu_item_action(
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
