use std::sync::Arc;

use fret_runtime::KeyChord;
use fret_ui::UiHost;

use crate::imui::{
    MenuItemOptions, ResponseExt, UiWriterImUiFacadeExt, menu_controls, popup_overlay,
};
use crate::primitives::menu::sub_trigger;

pub(super) struct SubmenuTriggerInput {
    pub(super) enabled: bool,
    pub(super) open_before: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) popup_estimated_size: fret_core::Size,
    pub(super) popup_policy: Option<popup_overlay::ImUiPopupMenuPolicyState>,
    pub(super) submenu_value: Arc<str>,
}

pub(super) fn submenu_trigger<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    input: SubmenuTriggerInput,
) -> ResponseExt {
    menu_controls::menu_item_with_options_and_pressable_hook(
        ui,
        label,
        MenuItemOptions {
            enabled: input.enabled,
            test_id: input.test_id,
            submenu: true,
            expanded: Some(input.open_before),
            activate_shortcut: input.activate_shortcut,
            shortcut_repeat: input.shortcut_repeat,
            ..Default::default()
        },
        {
            let popup_estimated_size = input.popup_estimated_size;
            let popup_policy = input.popup_policy;
            let submenu_value = input.submenu_value;
            move |cx, state, item_id, item_enabled| {
                let Some(popup_policy) = popup_policy.as_ref() else {
                    return;
                };
                let geometry_hint = sub_trigger::MenuSubTriggerGeometryHint {
                    outer: cx.environment_viewport_bounds(fret_ui::Invalidation::Layout),
                    desired: popup_estimated_size,
                };
                let _ = sub_trigger::wire(
                    cx,
                    state,
                    item_id,
                    !item_enabled,
                    true,
                    submenu_value.clone(),
                    &popup_policy.submenu_models,
                    popup_policy.submenu_cfg,
                    Some(geometry_hint),
                );
            }
        },
    )
}
