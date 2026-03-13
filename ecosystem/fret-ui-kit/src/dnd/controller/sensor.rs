use fret_core::{AppWindowId, Point, PointerId, Rect};
use fret_runtime::{DragKindId, FrameId, ModelStore, TickId};

use super::super::service::{DndServiceModel, update_dnd};
use super::super::{
    ActivationConstraint, AutoScrollConfig, CollisionStrategy, DndScopeId, DndUpdate,
};
use super::update_from_engine_output;

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
