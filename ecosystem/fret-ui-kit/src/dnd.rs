use std::collections::HashMap;

use fret_core::{AppWindowId, Point, PointerId, Rect};
pub use fret_dnd::{
    ActivationConstraint, AutoScrollConfig, AutoScrollRequest, CollisionStrategy, DndCollision,
    DndItemId, SensorOutput,
};
use fret_dnd::{
    Draggable, Droppable, PointerSensor, RegistrySnapshot, SensorEvent, closest_center_collisions,
    compute_autoscroll, pointer_within_collisions,
};
use fret_runtime::{DragKindId, FrameId, TickId};
use fret_ui::UiHost;

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

#[derive(Default)]
struct DndRegistryService {
    windows: HashMap<AppWindowId, WindowRegistry>,
}

#[derive(Default)]
struct WindowRegistry {
    frame_id: FrameId,
    snapshot: RegistrySnapshot,
}

impl DndRegistryService {
    fn snapshot_mut_for_frame(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
    ) -> &mut RegistrySnapshot {
        let entry = self.windows.entry(window).or_default();
        if entry.frame_id != frame_id {
            entry.frame_id = frame_id;
            entry.snapshot.draggables.clear();
            entry.snapshot.droppables.clear();
        }
        &mut entry.snapshot
    }

    fn snapshot_for_frame(&mut self, window: AppWindowId, frame_id: FrameId) -> &RegistrySnapshot {
        let _ = self.snapshot_mut_for_frame(window, frame_id);
        &self.windows.get(&window).expect("window exists").snapshot
    }
}

#[derive(Default)]
struct DndControllerService {
    windows: HashMap<AppWindowId, WindowController>,
}

#[derive(Default)]
struct WindowController {
    sensors_by_kind: HashMap<DragKindId, PointerSensor>,
}

impl DndControllerService {
    fn sensor_mut(
        &mut self,
        window: AppWindowId,
        kind: DragKindId,
        constraint: ActivationConstraint,
    ) -> &mut PointerSensor {
        let window = self.windows.entry(window).or_default();
        let sensor = window
            .sensors_by_kind
            .entry(kind)
            .or_insert_with(|| PointerSensor::new(constraint));
        sensor.set_constraint(constraint);
        sensor
    }
}

fn with_dnd_mut<H: UiHost, R>(app: &mut H, f: impl FnOnce(&mut DndService, &mut H) -> R) -> R {
    app.with_global_mut(DndService::default, f)
}

pub fn register_droppable_rect<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    id: DndItemId,
    rect: Rect,
    z_index: i32,
    disabled: bool,
) {
    let frame_id = app.frame_id();
    with_dnd_mut(app, |svc, app| {
        let snapshot = svc.registry.snapshot_mut_for_frame(window, frame_id);
        snapshot.droppables.push(Droppable {
            id,
            rect,
            disabled,
            z_index,
        });
        let _ = app;
    });
}

pub fn register_draggable_rect<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    id: DndItemId,
    rect: Rect,
) {
    let frame_id = app.frame_id();
    with_dnd_mut(app, |svc, app| {
        let snapshot = svc.registry.snapshot_mut_for_frame(window, frame_id);
        snapshot.draggables.push(Draggable { id, rect });
        let _ = app;
    });
}

pub fn clear_pending_pointer<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    kind: DragKindId,
    pointer_id: PointerId,
) {
    with_dnd_mut(app, |svc, _app| {
        let Some(window) = svc.controller.windows.get_mut(&window) else {
            return;
        };
        let Some(sensor) = window.sensors_by_kind.get_mut(&kind) else {
            return;
        };
        sensor.clear_pointer(pointer_id);
    });
}

#[allow(clippy::too_many_arguments)]
pub fn update_pending_drag_move<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    kind: DragKindId,
    pointer_id: PointerId,
    start_tick: TickId,
    start_position: Point,
    position: Point,
    constraint: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    autoscroll: Option<(Rect, AutoScrollConfig)>,
) -> PendingDragUpdate {
    let frame_id = app.frame_id();
    let tick = app.tick_id();

    with_dnd_mut(app, |svc, _app| {
        let sensor = svc.controller.sensor_mut(window, kind, constraint);
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
            tick: tick.0,
        });

        let snapshot = svc.registry.snapshot_for_frame(window, frame_id);
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
}
