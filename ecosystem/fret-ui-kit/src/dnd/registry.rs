use std::collections::HashMap;

use fret_core::{AppWindowId, Rect};
use fret_dnd::{Draggable, Droppable, RegistrySnapshot};
use fret_runtime::{FrameId, ModelStore};

use super::service::{DndServiceModel, read_dnd, update_dnd};
use super::{DND_SCOPE_DEFAULT, DndItemId, DndScopeId};

#[derive(Default)]
pub(crate) struct DndRegistryService {
    windows: HashMap<AppWindowId, WindowRegistry>,
}

#[derive(Default)]
struct WindowRegistry {
    frame_id: FrameId,
    scopes: HashMap<DndScopeId, RegistrySnapshot>,
    droppable_rects: HashMap<DndScopeId, HashMap<DndItemId, Rect>>,
}

impl DndRegistryService {
    pub(crate) fn snapshot_mut_for_frame(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        scope: DndScopeId,
    ) -> &mut RegistrySnapshot {
        let entry = self.windows.entry(window).or_default();
        if entry.frame_id != frame_id {
            entry.frame_id = frame_id;
            for snapshot in entry.scopes.values_mut() {
                snapshot.draggables.clear();
                snapshot.droppables.clear();
            }
            entry.droppable_rects.clear();
        }
        entry.scopes.entry(scope).or_default()
    }

    pub(crate) fn droppable_rects_mut_for_frame(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        scope: DndScopeId,
    ) -> &mut HashMap<DndItemId, Rect> {
        let entry = self.windows.entry(window).or_default();
        if entry.frame_id != frame_id {
            entry.frame_id = frame_id;
            for snapshot in entry.scopes.values_mut() {
                snapshot.draggables.clear();
                snapshot.droppables.clear();
            }
            entry.droppable_rects.clear();
        }
        entry.droppable_rects.entry(scope).or_default()
    }

    pub(crate) fn snapshot_for_frame(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        scope: DndScopeId,
    ) -> &RegistrySnapshot {
        let snapshot = self.snapshot_mut_for_frame(window, frame_id, scope);
        &*snapshot
    }
}

pub fn droppable_rect_in_scope(
    models: &ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    scope: DndScopeId,
    id: DndItemId,
) -> Option<Rect> {
    read_dnd(models, svc, |dnd| {
        let window = dnd.registry.windows.get(&window)?;
        if window.frame_id != frame_id {
            return None;
        }
        window
            .droppable_rects
            .get(&scope)
            .and_then(|m| m.get(&id).copied())
    })
    .flatten()
}

pub fn register_droppable_rect_in_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    scope: DndScopeId,
    id: DndItemId,
    rect: Rect,
    z_index: i32,
    disabled: bool,
) {
    let _ = update_dnd(models, svc, |dnd| {
        let snapshot = dnd.registry.snapshot_mut_for_frame(window, frame_id, scope);
        snapshot.droppables.push(Droppable {
            id,
            rect,
            disabled,
            z_index,
        });

        let rects = dnd
            .registry
            .droppable_rects_mut_for_frame(window, frame_id, scope);
        if disabled {
            rects.remove(&id);
        } else {
            rects.insert(id, rect);
        }
    });
}

pub fn register_droppable_rect(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    id: DndItemId,
    rect: Rect,
    z_index: i32,
    disabled: bool,
) {
    register_droppable_rect_in_scope(
        models,
        svc,
        window,
        frame_id,
        DND_SCOPE_DEFAULT,
        id,
        rect,
        z_index,
        disabled,
    );
}

pub fn register_droppable_rect_default_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    id: DndItemId,
    rect: Rect,
    z_index: i32,
    disabled: bool,
) {
    register_droppable_rect(models, svc, window, frame_id, id, rect, z_index, disabled);
}

pub fn register_draggable_rect_in_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    scope: DndScopeId,
    id: DndItemId,
    rect: Rect,
) {
    let _ = update_dnd(models, svc, |dnd| {
        let snapshot = dnd.registry.snapshot_mut_for_frame(window, frame_id, scope);
        snapshot.draggables.push(Draggable { id, rect });
    });
}

pub fn register_draggable_rect(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    id: DndItemId,
    rect: Rect,
) {
    register_draggable_rect_in_scope(models, svc, window, frame_id, DND_SCOPE_DEFAULT, id, rect);
}

pub fn register_draggable_rect_default_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    id: DndItemId,
    rect: Rect,
) {
    register_draggable_rect(models, svc, window, frame_id, id, rect);
}
