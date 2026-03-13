use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use fret_core::{AppWindowId, Point, PointerId, Rect};
use fret_dnd::{DndEngine, DndEngineUpdate, SensorEvent};
use fret_runtime::{DragKindId, FrameId, ModelStore, TickId};

use super::service::{DndServiceModel, read_dnd, update_dnd};
use super::{
    ActivationConstraint, AutoScrollConfig, CollisionStrategy, DND_SCOPE_DEFAULT, DndScopeId,
    DndUpdate,
};

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
    fn engine_mut(
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

fn update_from_engine_output(update: DndEngineUpdate) -> DndUpdate {
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

#[allow(clippy::too_many_arguments)]
fn update_from_sensor_event_in_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    scope: DndScopeId,
    sensor_event: SensorEvent,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    update_dnd(models, svc, |dnd| {
        let (controller, registry) = (&mut dnd.controller, &mut dnd.registry);
        let snapshot = registry.snapshot_for_frame(window, frame_id, scope);
        let update = controller
            .engine_mut(window, kind, scope, constraint)
            .handle(snapshot, sensor_event, collision_strategy, autoscroll);
        update_from_engine_output(update)
    })
    .unwrap_or_else(DndUpdate::pending)
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_down_in_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    scope: DndScopeId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    update_from_sensor_event_in_scope(
        models,
        svc,
        window,
        frame_id,
        kind,
        scope,
        SensorEvent::Down {
            pointer_id,
            position,
            tick: tick_id.0,
        },
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_move_in_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    scope: DndScopeId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    update_from_sensor_event_in_scope(
        models,
        svc,
        window,
        frame_id,
        kind,
        scope,
        SensorEvent::Move {
            pointer_id,
            position,
            tick: tick_id.0,
        },
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_up_in_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    scope: DndScopeId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    update_from_sensor_event_in_scope(
        models,
        svc,
        window,
        frame_id,
        kind,
        scope,
        SensorEvent::Up {
            pointer_id,
            position,
            tick: tick_id.0,
        },
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_cancel_in_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    scope: DndScopeId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    update_from_sensor_event_in_scope(
        models,
        svc,
        window,
        frame_id,
        kind,
        scope,
        SensorEvent::Cancel {
            pointer_id,
            position,
            tick: tick_id.0,
        },
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_move_or_init_in_scope(
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
) -> DndUpdate {
    update_dnd(models, svc, |dnd| {
        let (controller, registry) = (&mut dnd.controller, &mut dnd.registry);
        let snapshot = registry.snapshot_for_frame(window, frame_id, scope);
        let update = controller
            .engine_mut(window, kind, scope, constraint)
            .handle_move_or_init(
                snapshot,
                pointer_id,
                start_tick.0,
                start_position,
                position,
                tick_id.0,
                collision_strategy,
                autoscroll,
            );
        update_from_engine_output(update)
    })
    .unwrap_or_else(DndUpdate::pending)
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_down(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    handle_pointer_down_in_scope(
        models,
        svc,
        window,
        frame_id,
        kind,
        DND_SCOPE_DEFAULT,
        pointer_id,
        position,
        tick_id,
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_down_default_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    handle_pointer_down(
        models,
        svc,
        window,
        frame_id,
        kind,
        pointer_id,
        position,
        tick_id,
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_move(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    handle_pointer_move_in_scope(
        models,
        svc,
        window,
        frame_id,
        kind,
        DND_SCOPE_DEFAULT,
        pointer_id,
        position,
        tick_id,
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_move_default_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    handle_pointer_move(
        models,
        svc,
        window,
        frame_id,
        kind,
        pointer_id,
        position,
        tick_id,
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_up(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    handle_pointer_up_in_scope(
        models,
        svc,
        window,
        frame_id,
        kind,
        DND_SCOPE_DEFAULT,
        pointer_id,
        position,
        tick_id,
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_up_default_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    handle_pointer_up(
        models,
        svc,
        window,
        frame_id,
        kind,
        pointer_id,
        position,
        tick_id,
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_cancel(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    handle_pointer_cancel_in_scope(
        models,
        svc,
        window,
        frame_id,
        kind,
        DND_SCOPE_DEFAULT,
        pointer_id,
        position,
        tick_id,
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_cancel_default_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    frame_id: FrameId,
    kind: DragKindId,
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> DndUpdate {
    handle_pointer_cancel(
        models,
        svc,
        window,
        frame_id,
        kind,
        pointer_id,
        position,
        tick_id,
        constraint,
        collision_strategy,
        autoscroll,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn handle_sensor_move_or_init_in_scope(
    models: &mut ModelStore,
    svc: &DndServiceModel,
    window: AppWindowId,
    kind: DragKindId,
    scope: DndScopeId,
    pointer_id: PointerId,
    start_tick: TickId,
    start_position: Point,
    position: Point,
    tick_id: TickId,
    constraint: ActivationConstraint,
) -> fret_dnd::SensorOutput {
    update_dnd(models, svc, |dnd| {
        dnd.controller
            .engine_mut(window, kind, scope, constraint)
            .handle_sensor_move_or_init(
                pointer_id,
                start_tick.0,
                start_position,
                position,
                tick_id.0,
            )
    })
    .unwrap_or(fret_dnd::SensorOutput::Pending)
}
