use fret_runtime::{Model, ModelStore};
use fret_ui::{ElementContext, UiHost};

use super::{controller::DndControllerService, registry::DndRegistryService};

#[derive(Default)]
pub(crate) struct DndService {
    pub(crate) registry: DndRegistryService,
    pub(crate) controller: DndControllerService,
}

#[derive(Clone)]
pub struct DndServiceModel {
    pub(crate) model: Model<DndService>,
}

#[derive(Default)]
struct DndServiceModelGlobal {
    model: Option<DndServiceModel>,
}

pub fn dnd_service_model_global<H: UiHost>(app: &mut H) -> DndServiceModel {
    app.with_global_mut(DndServiceModelGlobal::default, |st, app| {
        if let Some(model) = st.model.clone() {
            return model;
        }

        let model = DndServiceModel {
            model: app.models_mut().insert(DndService::default()),
        };
        st.model = Some(model.clone());
        model
    })
}

pub fn dnd_service_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> DndServiceModel {
    dnd_service_model_global(cx.app)
}

pub(crate) fn update_dnd<R>(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    f: impl FnOnce(&mut DndService) -> R,
) -> Option<R> {
    models.update(&svc.model, f).ok()
}

pub(crate) fn read_dnd<R>(
    models: &ModelStore,
    svc: &DndServiceModel,
    f: impl FnOnce(&DndService) -> R,
) -> Option<R> {
    models.read(&svc.model, f).ok()
}
