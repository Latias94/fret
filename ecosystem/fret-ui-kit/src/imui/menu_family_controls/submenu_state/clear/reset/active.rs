use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::primitives::menu::sub::MenuSubmenuModels;

pub(in crate::imui::menu_family_controls::submenu_state::clear) fn clear_active_submenu_models<
    H: UiHost,
>(
    cx: &mut ElementContext<'_, H>,
    models: &MenuSubmenuModels,
    submenu_value: &str,
    trigger_id: Option<GlobalElementId>,
    clear_geometry: bool,
) {
    let _ = cx.app.models_mut().update(&models.open_value, |value| {
        if value
            .as_ref()
            .is_some_and(|current| current.as_ref() == submenu_value)
        {
            *value = None;
        }
    });
    let _ = cx.app.models_mut().update(&models.trigger, |value| {
        if *value == trigger_id {
            *value = None;
        }
    });
    if clear_geometry {
        let _ = cx
            .app
            .models_mut()
            .update(&models.geometry, |value| *value = None);
    }
}
