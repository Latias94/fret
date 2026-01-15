use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use fret_core::{AppWindowId, Point, PointerId, Rect};
pub use fret_dnd::{
    ActivationConstraint, AutoScrollConfig, AutoScrollRequest, CollisionStrategy, DndCollision,
    DndItemId, InsertionSide, SensorOutput, insertion_side_for_pointer,
};
use fret_dnd::{
    Draggable, Droppable, PointerSensor, RegistrySnapshot, SensorEvent, closest_center_collisions,
    compute_autoscroll, pointer_within_collisions,
};
use fret_runtime::{DragKindId, FrameId, Model, ModelStore, TickId};
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DndScopeId(pub u64);

pub const DND_SCOPE_DEFAULT: DndScopeId = DndScopeId(0);

#[derive(Debug, Clone)]
pub struct PendingDragUpdate {
    pub sensor: SensorOutput,
    pub collisions: Vec<DndCollision>,
    pub over: Option<DndItemId>,
    pub autoscroll: Option<AutoScrollRequest>,
}

impl Default for PendingDragUpdate {
    fn default() -> Self {
        Self {
            sensor: SensorOutput::Pending,
            collisions: Vec::new(),
            over: None,
            autoscroll: None,
        }
    }
}

#[derive(Default)]
struct DndService {
    registry: DndRegistryService,
    controller: DndControllerService,
}

#[derive(Clone)]
pub struct DndServiceModel {
    model: Model<DndService>,
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

#[derive(Default)]
struct DndRegistryService {
    windows: HashMap<AppWindowId, WindowRegistry>,
}

#[derive(Default)]
struct WindowRegistry {
    frame_id: FrameId,
    scopes: HashMap<DndScopeId, RegistrySnapshot>,
}

impl DndRegistryService {
    fn snapshot_mut_for_frame(
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
        }
        entry.scopes.entry(scope).or_default()
    }

    fn snapshot_for_frame(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        scope: DndScopeId,
    ) -> &RegistrySnapshot {
        let snapshot = self.snapshot_mut_for_frame(window, frame_id, scope);
        &*snapshot
    }
}

#[derive(Default)]
struct DndControllerService {
    windows: HashMap<AppWindowId, WindowController>,
}

#[derive(Default)]
struct WindowController {
    sensors_by_key: HashMap<DndControllerKey, PointerSensor>,
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
    fn sensor_mut(
        &mut self,
        window: AppWindowId,
        kind: DragKindId,
        scope: DndScopeId,
        constraint: ActivationConstraint,
    ) -> &mut PointerSensor {
        let window = self.windows.entry(window).or_default();
        let key = DndControllerKey { kind, scope };
        let sensor = window
            .sensors_by_key
            .entry(key)
            .or_insert_with(|| PointerSensor::new(constraint));
        sensor.set_constraint(constraint);
        sensor
    }
}

fn update_dnd<R>(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    f: impl FnOnce(&mut DndService) -> R,
) -> Option<R> {
    models.update(&svc.model, f).ok()
}

fn read_dnd<R>(
    models: &ModelStore,
    svc: &DndServiceModel,
    f: impl FnOnce(&DndService) -> R,
) -> Option<R> {
    models.read(&svc.model, f).ok()
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
        let snapshot = window.scopes.get(&scope)?;
        snapshot
            .droppables
            .iter()
            .find(|d| d.id == id && !d.disabled)
            .map(|d| d.rect)
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

pub fn clear_pending_pointer_in_scope(
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
        let Some(sensor) = window.sensors_by_key.get_mut(&key) else {
            return;
        };
        sensor.clear_pointer(pointer_id);
    });
}

pub fn clear_pending_pointer(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    kind: DragKindId,
    pointer_id: PointerId,
) {
    clear_pending_pointer_in_scope(models, svc, window, kind, DND_SCOPE_DEFAULT, pointer_id);
}

pub fn clear_pending_pointer_default_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    kind: DragKindId,
    pointer_id: PointerId,
) {
    clear_pending_pointer(models, svc, window, kind, pointer_id);
}

#[allow(clippy::too_many_arguments)]
pub fn update_pending_drag_move_in_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    scope: DndScopeId,
    pointer_id: PointerId,
    start_tick: TickId,
    start_position: Point,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> PendingDragUpdate {
    update_dnd(models, svc, |dnd| {
        let sensor = dnd.controller.sensor_mut(window, kind, scope, constraint);
        if !sensor.is_tracking(pointer_id) {
            let _ = sensor.handle(SensorEvent::Down {
                pointer_id,
                position: start_position,
                tick: start_tick.0,
            });
        }

        let sensor = sensor.handle(SensorEvent::Move {
            pointer_id,
            position,
            tick: tick_id.0,
        });

        let snapshot = dnd.registry.snapshot_for_frame(window, frame_id, scope);
        let collisions = match collision_strategy {
            CollisionStrategy::PointerWithin => pointer_within_collisions(snapshot, position),
            CollisionStrategy::ClosestCenter => closest_center_collisions(snapshot, position),
        };
        let over = collisions.first().map(|c| c.id);

        let autoscroll =
            autoscroll.and_then(|(container, cfg)| compute_autoscroll(cfg, container, position));

        PendingDragUpdate {
            sensor,
            collisions,
            over,
            autoscroll,
        }
    })
    .unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
pub fn update_pending_drag_move(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    pointer_id: PointerId,
    start_tick: TickId,
    start_position: Point,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> PendingDragUpdate {
    update_pending_drag_move_in_scope(
        models,
        svc,
        window,
        frame_id,
        kind,
        DND_SCOPE_DEFAULT,
        pointer_id,
        start_tick,
        start_position,
        position,
        tick_id,
        constraint,
        collision_strategy,
        autoscroll,
    )
}

pub fn update_pending_drag_move_default_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    pointer_id: PointerId,
    start_tick: TickId,
    start_position: Point,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> PendingDragUpdate {
    update_pending_drag_move(
        models,
        svc,
        window,
        frame_id,
        kind,
        pointer_id,
        start_tick,
        start_position,
        position,
        tick_id,
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{Px, Rect, Size};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    fn mk_service(app: &mut App) -> DndServiceModel {
        DndServiceModel {
            model: app.models_mut().insert(DndService::default()),
        }
    }

    #[test]
    fn registry_isolated_by_scope() {
        let mut app = App::new();
        let svc = mk_service(&mut app);
        let window = AppWindowId::default();
        let frame = FrameId(1);
        let tick = TickId(1);

        register_droppable_rect_in_scope(
            app.models_mut(),
            &svc,
            window,
            frame,
            DndScopeId(1),
            DndItemId(1),
            rect(0.0, 0.0, 10.0, 10.0),
            0,
            false,
        );
        register_droppable_rect_in_scope(
            app.models_mut(),
            &svc,
            window,
            frame,
            DndScopeId(2),
            DndItemId(2),
            rect(100.0, 0.0, 10.0, 10.0),
            0,
            false,
        );

        let out = update_pending_drag_move_in_scope(
            app.models_mut(),
            &svc,
            window,
            frame,
            DragKindId(1),
            DndScopeId(1),
            PointerId(0),
            TickId(0),
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(105.0), Px(5.0)),
            tick,
            ActivationConstraint::None,
            CollisionStrategy::ClosestCenter,
            None,
        );

        assert_eq!(out.over, Some(DndItemId(1)));
    }

    #[test]
    fn sensors_are_scoped_to_avoid_constraint_cross_talk() {
        let mut app = App::new();
        let svc = mk_service(&mut app);
        let window = AppWindowId::default();
        let frame = FrameId(1);

        let p0 = PointerId(0);
        let p1 = PointerId(1);

        let start = Point::new(Px(0.0), Px(0.0));
        let move_small = Point::new(Px(1.0), Px(0.0));

        let _ = update_pending_drag_move_in_scope(
            app.models_mut(),
            &svc,
            window,
            frame,
            DragKindId(1),
            DndScopeId(1),
            p0,
            TickId(0),
            start,
            move_small,
            TickId(1),
            ActivationConstraint::DelayTicks { ticks: 10 },
            CollisionStrategy::ClosestCenter,
            None,
        );

        let out_p1 = update_pending_drag_move_in_scope(
            app.models_mut(),
            &svc,
            window,
            frame,
            DragKindId(1),
            DndScopeId(2),
            p1,
            TickId(0),
            start,
            move_small,
            TickId(1),
            ActivationConstraint::None,
            CollisionStrategy::ClosestCenter,
            None,
        );
        assert!(matches!(
            out_p1.sensor,
            SensorOutput::DragStart { .. } | SensorOutput::DragMove { .. }
        ));

        let out_p0 = update_pending_drag_move_in_scope(
            app.models_mut(),
            &svc,
            window,
            frame,
            DragKindId(1),
            DndScopeId(1),
            p0,
            TickId(0),
            start,
            move_small,
            TickId(2),
            ActivationConstraint::DelayTicks { ticks: 10 },
            CollisionStrategy::ClosestCenter,
            None,
        );

        assert!(matches!(out_p0.sensor, SensorOutput::Pending));
    }
}
