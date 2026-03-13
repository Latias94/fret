use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use fret_core::{AppWindowId, PointerId};
use fret_dnd::{DndEngine, DndEngineUpdate};
use fret_runtime::{DragKindId, ModelStore};

use super::service::{DndServiceModel, read_dnd, update_dnd};
use super::{ActivationConstraint, DND_SCOPE_DEFAULT, DndScopeId, DndUpdate};

mod pointer;
mod sensor;

pub use pointer::{
    handle_pointer_cancel, handle_pointer_cancel_default_scope, handle_pointer_cancel_in_scope,
    handle_pointer_down, handle_pointer_down_default_scope, handle_pointer_down_in_scope,
    handle_pointer_move, handle_pointer_move_default_scope, handle_pointer_move_in_scope,
    handle_pointer_up, handle_pointer_up_default_scope, handle_pointer_up_in_scope,
};
pub use sensor::{handle_pointer_move_or_init_in_scope, handle_sensor_move_or_init_in_scope};

#[derive(Default)]
pub(crate) struct DndControllerService {
    windows: HashMap<AppWindowId, WindowController>,
}

#[derive(Default)]
struct WindowController {
    engines_by_key: HashMap<DndControllerKey, DndEngine>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct DndControllerKey {
    kind: DragKindId,
    scope: DndScopeId,
}

impl Hash for DndControllerKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.scope.hash(state);
    }
}

impl DndControllerService {
    pub(super) fn engine_mut(
        &mut self,
        window: AppWindowId,
        kind: DragKindId,
        scope: DndScopeId,
        constraint: ActivationConstraint,
    ) -> &mut DndEngine {
        let window = self.windows.entry(window).or_default();
        let key = DndControllerKey { kind, scope };
        let engine = window
            .engines_by_key
            .entry(key)
            .or_insert_with(|| DndEngine::new(constraint));
        engine.set_constraint(constraint);
        engine
    }
}

pub(super) fn update_from_engine_output(update: DndEngineUpdate) -> DndUpdate {
    DndUpdate {
        sensor: update.sensor,
        collisions: update.collisions,
        over: update.over,
        autoscroll: update.autoscroll,
    }
}

pub fn clear_pointer_in_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    kind: DragKindId,
    scope: DndScopeId,
    pointer_id: PointerId,
) {
    let _ = update_dnd(models, svc, |dnd| {
        let Some(window) = dnd.controller.windows.get_mut(&window) else {
            return;
        };
        let key = DndControllerKey { kind, scope };
        let Some(engine) = window.engines_by_key.get_mut(&key) else {
            return;
        };
        engine.clear_pointer(pointer_id);
    });
}

pub fn clear_pointer(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    kind: DragKindId,
    pointer_id: PointerId,
) {
    clear_pointer_in_scope(models, svc, window, kind, DND_SCOPE_DEFAULT, pointer_id);
}

pub fn clear_pointer_default_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    kind: DragKindId,
    pointer_id: PointerId,
) {
    clear_pointer(models, svc, window, kind, pointer_id);
}

pub fn pointer_is_tracking_any_sensor(
    models: &ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    pointer_id: PointerId,
) -> bool {
    read_dnd(models, svc, |dnd| {
        dnd.controller.windows.get(&window).is_some_and(|window| {
            window
                .engines_by_key
                .values()
                .any(|engine| engine.is_tracking(pointer_id))
        })
    })
    .unwrap_or(false)
}
