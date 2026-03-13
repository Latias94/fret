use fret_core::{AppWindowId, Point, PointerId, Rect};
use fret_dnd::SensorEvent;
use fret_runtime::{DragKindId, FrameId, ModelStore, TickId};

use super::super::service::{DndServiceModel, update_dnd};
use super::super::{
    ActivationConstraint, AutoScrollConfig, CollisionStrategy, DND_SCOPE_DEFAULT, DndScopeId,
    DndUpdate,
};
use super::update_from_engine_output;

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
