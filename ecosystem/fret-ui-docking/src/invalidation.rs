use std::collections::HashMap;

use fret_core::AppWindowId;
use fret_runtime::{Model, UiHost};

#[derive(Default)]
pub(crate) struct DockInvalidationService {
    by_window: HashMap<AppWindowId, Model<u64>>,
}

impl DockInvalidationService {
    pub(crate) fn model_for_window<H: UiHost>(app: &mut H, window: AppWindowId) -> Model<u64> {
        app.with_global_mut(DockInvalidationService::default, |svc, app| {
            if let Some(model) = svc.by_window.get(&window) {
                return model.clone();
            }

            let model = app.models_mut().insert(0u64);
            svc.by_window.insert(window, model.clone());
            model
        })
    }

    pub(crate) fn bump_windows<H: UiHost>(
        app: &mut H,
        windows: impl IntoIterator<Item = AppWindowId>,
    ) {
        app.with_global_mut(DockInvalidationService::default, |svc, app| {
            for window in windows {
                let model = if let Some(model) = svc.by_window.get(&window) {
                    model.clone()
                } else {
                    let model = app.models_mut().insert(0u64);
                    svc.by_window.insert(window, model.clone());
                    model
                };

                let _ = app
                    .models_mut()
                    .update(&model, |rev| *rev = rev.wrapping_add(1));
                app.request_redraw(window);
            }
        });
    }
}
