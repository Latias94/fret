use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::ActionId;
use fret_ui::element::{Length, PressableA11y, PressableProps};

use crate::imui::MenuItemOptions;
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;

pub(in crate::imui::menu_controls) struct MenuItemInteractionParts {
    pub(in crate::imui::menu_controls) props: PressableProps,
    pub(in crate::imui::menu_controls) runtime: MenuItemInteraction,
}

pub(in crate::imui::menu_controls) struct MenuItemInteraction {
    pub(in crate::imui::menu_controls) enabled: bool,
    pub(in crate::imui::menu_controls) close_popup: Option<fret_runtime::Model<bool>>,
    pub(in crate::imui::menu_controls) action: Option<ActionId>,
    pub(in crate::imui::menu_controls) activate_shortcut: Option<fret_runtime::KeyChord>,
    pub(in crate::imui::menu_controls) shortcut_repeat: bool,
    pub(in crate::imui::menu_controls) menubar_policy: Option<ImUiMenubarPolicyState>,
}

pub(super) struct MenuItemInteractionInput<'a> {
    pub(super) label: &'a Arc<str>,
    pub(super) options: &'a MenuItemOptions,
    pub(super) role: SemanticsRole,
    pub(super) checked: Option<bool>,
    pub(super) runtime: MenuItemInteraction,
}

pub(super) fn menu_item_interaction_parts(
    input: MenuItemInteractionInput<'_>,
) -> MenuItemInteractionParts {
    let MenuItemInteractionInput {
        label,
        options,
        role,
        checked,
        runtime,
    } = input;

    let mut props = PressableProps::default();
    props.enabled = runtime.enabled;
    props.focusable = runtime.enabled;
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

    MenuItemInteractionParts { props, runtime }
}
