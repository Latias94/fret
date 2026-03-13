use std::collections::HashMap;

use fret_core::{Point, PointerId, Px, Rect};

use crate::{
    ActivationConstraint, AutoScrollConfig, AutoScrollRequest, CollisionStrategy, DndCollision,
    DndFrameOutput, DndItemId, PointerSensor, RegistrySnapshot, SensorEvent, SensorOutput,
    compute_dnd_frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DndOperationPhase {
    Pending,
    Dragging,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DndOperationState {
    pub phase: DndOperationPhase,
    pub start: Point,
    pub position: Point,
    pub translation: Point,
    pub collisions: Vec<DndCollision>,
    pub over: Option<DndItemId>,
    pub autoscroll: Option<AutoScrollRequest>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DndEngineUpdate {
    pub sensor: SensorOutput,
    pub collisions: Vec<DndCollision>,
    pub over: Option<DndItemId>,
    pub autoscroll: Option<AutoScrollRequest>,
}

impl Default for DndEngineUpdate {
    fn default() -> Self {
        Self {
            sensor: SensorOutput::Pending,
            collisions: Vec::new(),
            over: None,
            autoscroll: None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DndEngine {
    sensor: PointerSensor,
    operations: HashMap<PointerId, DndOperationState>,
}

impl DndEngine {
    pub fn new(constraint: ActivationConstraint) -> Self {
        Self {
            sensor: PointerSensor::new(constraint),
            operations: HashMap::new(),
        }
    }

    pub fn set_constraint(&mut self, constraint: ActivationConstraint) {
        self.sensor.set_constraint(constraint);
    }

    pub fn is_tracking(&self, pointer_id: PointerId) -> bool {
        self.sensor.is_tracking(pointer_id)
    }

    pub fn clear_pointer(&mut self, pointer_id: PointerId) {
        self.sensor.clear_pointer(pointer_id);
        self.operations.remove(&pointer_id);
    }

    pub fn operation(&self, pointer_id: PointerId) -> Option<&DndOperationState> {
        self.operations.get(&pointer_id)
    }

    pub fn handle(
        &mut self,
        snapshot: &RegistrySnapshot,
        sensor_event: SensorEvent,
        collision_strategy: CollisionStrategy,
        autoscroll: Option<(Rect, AutoScrollConfig)>,
    ) -> DndEngineUpdate {
        let pointer_id = pointer_id_from_event(sensor_event);
        let position = position_from_event(sensor_event);
        let sensor = self.sensor.handle(sensor_event);
        let frame = compute_dnd_frame(snapshot, position, collision_strategy, autoscroll);

        self.apply_sensor_output(pointer_id, position, sensor, Some(&frame));

        DndEngineUpdate {
            sensor,
            collisions: frame.collisions,
            over: frame.over,
            autoscroll: frame.autoscroll,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn handle_move_or_init(
        &mut self,
        snapshot: &RegistrySnapshot,
        pointer_id: PointerId,
        start_tick: u64,
        start_position: Point,
        position: Point,
        tick: u64,
        collision_strategy: CollisionStrategy,
        autoscroll: Option<(Rect, AutoScrollConfig)>,
    ) -> DndEngineUpdate {
        self.ensure_pointer(pointer_id, start_tick, start_position);
        self.handle(
            snapshot,
            SensorEvent::Move {
                pointer_id,
                position,
                tick,
            },
            collision_strategy,
            autoscroll,
        )
    }

    pub fn handle_sensor_move_or_init(
        &mut self,
        pointer_id: PointerId,
        start_tick: u64,
        start_position: Point,
        position: Point,
        tick: u64,
    ) -> SensorOutput {
        self.ensure_pointer(pointer_id, start_tick, start_position);
        let sensor = self.sensor.handle(SensorEvent::Move {
            pointer_id,
            position,
            tick,
        });
        self.apply_sensor_output(pointer_id, position, sensor, None);
        sensor
    }

    fn ensure_pointer(&mut self, pointer_id: PointerId, start_tick: u64, start_position: Point) {
        if self.sensor.is_tracking(pointer_id) {
            if !self.operations.contains_key(&pointer_id) {
                self.upsert_operation(
                    pointer_id,
                    DndOperationPhase::Pending,
                    start_position,
                    start_position,
                    None,
                );
            }
            return;
        }

        let _ = self.sensor.handle(SensorEvent::Down {
            pointer_id,
            position: start_position,
            tick: start_tick,
        });
        self.upsert_operation(
            pointer_id,
            DndOperationPhase::Pending,
            start_position,
            start_position,
            None,
        );
    }

    fn apply_sensor_output(
        &mut self,
        pointer_id: PointerId,
        position: Point,
        sensor: SensorOutput,
        frame: Option<&DndFrameOutput>,
    ) {
        match sensor {
            SensorOutput::Pending => {
                if self.sensor.is_tracking(pointer_id) {
                    let start = self
                        .operations
                        .get(&pointer_id)
                        .map(|operation| operation.start)
                        .unwrap_or(position);
                    self.upsert_operation(
                        pointer_id,
                        DndOperationPhase::Pending,
                        start,
                        position,
                        frame,
                    );
                } else {
                    self.operations.remove(&pointer_id);
                }
            }
            SensorOutput::DragStart {
                start, position, ..
            }
            | SensorOutput::DragMove {
                start, position, ..
            } => {
                self.upsert_operation(
                    pointer_id,
                    DndOperationPhase::Dragging,
                    start,
                    position,
                    frame,
                );
            }
            SensorOutput::DragEnd { .. } | SensorOutput::DragCancel { .. } => {
                self.operations.remove(&pointer_id);
            }
        }
    }

    fn upsert_operation(
        &mut self,
        pointer_id: PointerId,
        phase: DndOperationPhase,
        start: Point,
        position: Point,
        frame: Option<&DndFrameOutput>,
    ) {
        let (collisions, over, autoscroll) = frame
            .map(frame_fields)
            .or_else(|| self.operations.get(&pointer_id).map(operation_fields))
            .unwrap_or_else(|| (Vec::new(), None, None));

        self.operations.insert(
            pointer_id,
            DndOperationState {
                phase,
                start,
                position,
                translation: translation_between(start, position),
                collisions,
                over,
                autoscroll,
            },
        );
    }
}

fn frame_fields(
    frame: &DndFrameOutput,
) -> (
    Vec<DndCollision>,
    Option<DndItemId>,
    Option<AutoScrollRequest>,
) {
    (frame.collisions.clone(), frame.over, frame.autoscroll)
}

fn operation_fields(
    operation: &DndOperationState,
) -> (
    Vec<DndCollision>,
    Option<DndItemId>,
    Option<AutoScrollRequest>,
) {
    (
        operation.collisions.clone(),
        operation.over,
        operation.autoscroll,
    )
}

fn translation_between(start: Point, position: Point) -> Point {
    Point::new(Px(position.x.0 - start.x.0), Px(position.y.0 - start.y.0))
}

fn pointer_id_from_event(sensor_event: SensorEvent) -> PointerId {
    match sensor_event {
        SensorEvent::Down { pointer_id, .. }
        | SensorEvent::Move { pointer_id, .. }
        | SensorEvent::Up { pointer_id, .. }
        | SensorEvent::Cancel { pointer_id, .. } => pointer_id,
    }
}

fn position_from_event(sensor_event: SensorEvent) -> Point {
    match sensor_event {
        SensorEvent::Down { position, .. }
        | SensorEvent::Move { position, .. }
        | SensorEvent::Up { position, .. }
        | SensorEvent::Cancel { position, .. } => position,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_core::{Rect, Size};

    use crate::{DndItemId, Droppable};

    fn point(x: f32, y: f32) -> Point {
        Point::new(Px(x), Px(y))
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(point(x, y), Size::new(Px(w), Px(h)))
    }

    fn snapshot_with_overlap() -> RegistrySnapshot {
        RegistrySnapshot {
            draggables: vec![],
            droppables: vec![
                Droppable {
                    id: DndItemId(2),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 10,
                },
                Droppable {
                    id: DndItemId(1),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
            ],
        }
    }

    #[test]
    fn engine_output_matches_frame_and_keeps_collision_order() {
        let snapshot = snapshot_with_overlap();
        let pointer = point(5.0, 5.0);
        let mut engine = DndEngine::new(ActivationConstraint::None);

        let update = engine.handle(
            &snapshot,
            SensorEvent::Down {
                pointer_id: PointerId(1),
                position: pointer,
                tick: 0,
            },
            CollisionStrategy::PointerWithin,
            None,
        );

        assert_eq!(update.sensor, SensorOutput::Pending);
        assert_eq!(update.over, Some(DndItemId(2)));
        assert_eq!(update.collisions.len(), 2);
        assert_eq!(update.collisions[0].id, DndItemId(2));
        assert_eq!(update.collisions[1].id, DndItemId(1));

        let operation = engine
            .operation(PointerId(1))
            .expect("pointer should have an operation state");
        assert_eq!(operation.phase, DndOperationPhase::Pending);
        assert_eq!(operation.collisions, update.collisions);
        assert_eq!(operation.over, update.over);
    }

    #[test]
    fn handle_move_or_init_seeds_tracking_and_activates() {
        let snapshot = snapshot_with_overlap();
        let mut engine = DndEngine::new(ActivationConstraint::Distance { px: 4.0 });

        let update = engine.handle_move_or_init(
            &snapshot,
            PointerId(7),
            0,
            point(0.0, 0.0),
            point(5.0, 0.0),
            1,
            CollisionStrategy::ClosestCenter,
            None,
        );

        assert!(matches!(
            update.sensor,
            SensorOutput::DragStart {
                pointer_id: PointerId(7),
                ..
            }
        ));

        let operation = engine
            .operation(PointerId(7))
            .expect("pointer should stay tracked after activation");
        assert_eq!(operation.phase, DndOperationPhase::Dragging);
        assert_eq!(operation.start, point(0.0, 0.0));
        assert_eq!(operation.position, point(5.0, 0.0));
        assert_eq!(operation.translation, point(5.0, 0.0));
    }

    #[test]
    fn multi_pointer_operations_remain_isolated() {
        let snapshot = snapshot_with_overlap();
        let mut engine = DndEngine::new(ActivationConstraint::Distance { px: 2.0 });

        let _ = engine.handle(
            &snapshot,
            SensorEvent::Down {
                pointer_id: PointerId(1),
                position: point(0.0, 0.0),
                tick: 0,
            },
            CollisionStrategy::ClosestCenter,
            None,
        );
        let _ = engine.handle(
            &snapshot,
            SensorEvent::Down {
                pointer_id: PointerId(2),
                position: point(10.0, 0.0),
                tick: 0,
            },
            CollisionStrategy::ClosestCenter,
            None,
        );

        let first = engine.handle(
            &snapshot,
            SensorEvent::Move {
                pointer_id: PointerId(1),
                position: point(1.0, 0.0),
                tick: 1,
            },
            CollisionStrategy::ClosestCenter,
            None,
        );
        let second = engine.handle(
            &snapshot,
            SensorEvent::Move {
                pointer_id: PointerId(2),
                position: point(13.0, 0.0),
                tick: 1,
            },
            CollisionStrategy::ClosestCenter,
            None,
        );

        assert_eq!(first.sensor, SensorOutput::Pending);
        assert!(matches!(
            second.sensor,
            SensorOutput::DragStart {
                pointer_id: PointerId(2),
                ..
            }
        ));

        let first_operation = engine
            .operation(PointerId(1))
            .expect("first pointer should remain pending");
        assert_eq!(first_operation.phase, DndOperationPhase::Pending);
        assert_eq!(first_operation.translation, point(1.0, 0.0));

        let second_operation = engine
            .operation(PointerId(2))
            .expect("second pointer should activate independently");
        assert_eq!(second_operation.phase, DndOperationPhase::Dragging);
        assert_eq!(second_operation.translation, point(3.0, 0.0));
    }

    #[test]
    fn end_and_cancel_cleanup_remove_operation_state() {
        let snapshot = snapshot_with_overlap();
        let mut engine = DndEngine::new(ActivationConstraint::None);

        let _ = engine.handle_move_or_init(
            &snapshot,
            PointerId(11),
            0,
            point(0.0, 0.0),
            point(4.0, 0.0),
            1,
            CollisionStrategy::ClosestCenter,
            None,
        );
        let end = engine.handle(
            &snapshot,
            SensorEvent::Up {
                pointer_id: PointerId(11),
                position: point(5.0, 0.0),
                tick: 2,
            },
            CollisionStrategy::ClosestCenter,
            None,
        );
        assert!(matches!(
            end.sensor,
            SensorOutput::DragEnd {
                pointer_id: PointerId(11),
                ..
            }
        ));
        assert!(!engine.is_tracking(PointerId(11)));
        assert!(engine.operation(PointerId(11)).is_none());

        let _ = engine.handle_move_or_init(
            &snapshot,
            PointerId(12),
            0,
            point(0.0, 0.0),
            point(4.0, 0.0),
            1,
            CollisionStrategy::ClosestCenter,
            None,
        );
        let cancel = engine.handle(
            &snapshot,
            SensorEvent::Cancel {
                pointer_id: PointerId(12),
                position: point(6.0, 0.0),
                tick: 2,
            },
            CollisionStrategy::ClosestCenter,
            None,
        );
        assert!(matches!(
            cancel.sensor,
            SensorOutput::DragCancel {
                pointer_id: PointerId(12),
                ..
            }
        ));
        assert!(!engine.is_tracking(PointerId(12)));
        assert!(engine.operation(PointerId(12)).is_none());
    }

    #[test]
    fn sensor_move_or_init_updates_operation_without_frame_context() {
        let mut engine = DndEngine::new(ActivationConstraint::Distance { px: 4.0 });

        let sensor = engine.handle_sensor_move_or_init(
            PointerId(20),
            0,
            point(0.0, 0.0),
            point(5.0, 0.0),
            1,
        );

        assert!(matches!(
            sensor,
            SensorOutput::DragStart {
                pointer_id: PointerId(20),
                ..
            }
        ));

        let operation = engine
            .operation(PointerId(20))
            .expect("pointer should have operation state");
        assert_eq!(operation.phase, DndOperationPhase::Dragging);
        assert_eq!(operation.translation, point(5.0, 0.0));
        assert!(operation.collisions.is_empty());
        assert_eq!(operation.over, None);
        assert_eq!(operation.autoscroll, None);
    }
}
