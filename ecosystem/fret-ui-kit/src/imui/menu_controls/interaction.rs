use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::ActionId;
use fret_ui::action::ActivateReason;
use fret_ui::element::{Length, PressableA11y, PressableProps, PressableState};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::command::ElementCommandGatingExt as _;
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::imui::{MenuItemOptions, ResponseExt, active_trigger_behavior, imui_is_disabled};

mod behavior;

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
    behavior::install_menu_item_behavior(cx, id, interaction)
}

pub(super) fn populate_menu_item_response<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    behavior: &active_trigger_behavior::ActiveTriggerBehavior,
    enabled: bool,
    response: &mut ResponseExt,
) {
    behavior::populate_menu_item_response(cx, id, state, behavior, enabled, response);
}

pub(super) fn dispatch_menu_item_action(
    host: &mut dyn fret_ui::action::UiActionHost,
    acx: fret_ui::action::ActionCx,
    reason: ActivateReason,
    action: Option<ActionId>,
) {
    behavior::dispatch_menu_item_action(host, acx, reason, action);
}
