use fret_ui::{ElementContext, UiHost};

use crate::primitives::menu::sub::MenuSubmenuModels;

pub(in crate::imui::menu_family_controls::submenu_state::clear) fn clear_submenu_runtime_models<
    H: UiHost,
>(
    cx: &mut ElementContext<'_, H>,
    models: &MenuSubmenuModels,
) {
    let _ = cx
        .app
        .models_mut()
        .update(&models.pointer_grace_intent, |value| *value = None);
    let _ = cx
        .app
        .models_mut()
        .update(&models.pointer_grace_timer, |value| *value = None);
    let _ = cx
        .app
        .models_mut()
        .update(&models.close_timer, |value| *value = None);
    let _ = cx
        .app
        .models_mut()
        .update(&models.focus_target, |value| *value = None);
    let _ = cx
        .app
        .models_mut()
        .update(&models.focus_timer, |value| *value = None);
    let _ = cx
        .app
        .models_mut()
        .update(&models.focus_retry_attempts, |value| *value = 0);
    let _ = cx
        .app
        .models_mut()
        .update(&models.open_timer, |value| *value = None);
}
