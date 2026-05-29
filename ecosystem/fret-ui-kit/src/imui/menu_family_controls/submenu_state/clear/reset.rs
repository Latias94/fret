use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::primitives::menu::sub::MenuSubmenuModels;

pub(super) fn clear_active_submenu_models<H: UiHost>(
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

pub(super) fn clear_pending_submenu_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    models: &MenuSubmenuModels,
    submenu_value: &str,
    trigger_id: Option<GlobalElementId>,
) {
    let _ = cx
        .app
        .models_mut()
        .update(&models.pending_open_value, |value| {
            if value
                .as_ref()
                .is_some_and(|current| current.as_ref() == submenu_value)
            {
                *value = None;
            }
        });
    let _ = cx
        .app
        .models_mut()
        .update(&models.pending_open_trigger, |value| {
            if *value == trigger_id {
                *value = None;
            }
        });
}

pub(super) fn clear_submenu_runtime_models<H: UiHost>(
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
