use super::*;

use fret_app::App;
use fret_core::{Point, PointerId, Px, Rect, Size};
use fret_dnd::{compute_dnd_frame, Droppable, RegistrySnapshot};
use fret_runtime::{DragKindId, FrameId, TickId};

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
}

fn mk_service(app: &mut App) -> DndServiceModel {
    DndServiceModel {
        model: app.models_mut().insert(service::DndService::default()),
    }
}

#[test]
fn activation_probe_starts_on_threshold_and_clears_tracking() {
    let mut app = App::new();
    let svc = mk_service(&mut app);
    let window = fret_core::AppWindowId::default();
    let pointer = PointerId(0);
    let probe = DndActivationProbe::new(
        svc.clone(),
        DndActivationProbeConfig::for_kind(DragKindId(77))
            .scope(DndScopeId(5))
            .activation_constraint(ActivationConstraint::Distance { px: 6.0 }),
    );

    let start = Point::new(Px(0.0), Px(0.0));
    let pending = probe.move_or_init(
        app.models_mut(),
        window,
        pointer,
        TickId(0),
        start,
        Point::new(Px(5.0), Px(0.0)),
        TickId(1),
    );
    assert!(matches!(pending, SensorOutput::Pending));
    assert!(pointer_is_tracking_any_sensor(
        app.models(),
        &svc,
        window,
        pointer
    ));

    let activated = probe.move_or_init(
        app.models_mut(),
        window,
        pointer,
        TickId(0),
        start,
        Point::new(Px(6.0), Px(0.0)),
        TickId(2),
    );
    assert!(matches!(
        activated,
        SensorOutput::DragStart { .. } | SensorOutput::DragMove { .. }
    ));

    probe.clear(app.models_mut(), window, pointer);
    assert!(!pointer_is_tracking_any_sensor(
        app.models(),
        &svc,
        window,
        pointer
    ));
}

#[test]
fn pointer_is_tracking_any_sensor_reflects_sensor_lifecycle() {
    let mut app = App::new();
    let svc = mk_service(&mut app);
    let window = fret_core::AppWindowId::default();
    let frame = FrameId(1);

    let kind = DragKindId(1);
    let scope = DndScopeId(1);
    let pointer = PointerId(0);

    assert!(!pointer_is_tracking_any_sensor(
        app.models(),
        &svc,
        window,
        pointer
    ));

    let _ = handle_pointer_down_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        kind,
        scope,
        pointer,
        Point::new(Px(0.0), Px(0.0)),
        TickId(0),
        ActivationConstraint::None,
        CollisionStrategy::ClosestCenter,
        None,
    );

    assert!(pointer_is_tracking_any_sensor(
        app.models(),
        &svc,
        window,
        pointer
    ));

    let _ = handle_pointer_up_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        kind,
        scope,
        pointer,
        Point::new(Px(1.0), Px(0.0)),
        TickId(1),
        ActivationConstraint::None,
        CollisionStrategy::ClosestCenter,
        None,
    );

    assert!(!pointer_is_tracking_any_sensor(
        app.models(),
        &svc,
        window,
        pointer
    ));
}

#[test]
fn registry_isolated_by_scope() {
    let mut app = App::new();
    let svc = mk_service(&mut app);
    let window = fret_core::AppWindowId::default();
    let frame = FrameId(1);

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

    let out = handle_pointer_down_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        DragKindId(1),
        DndScopeId(1),
        PointerId(0),
        Point::new(Px(0.0), Px(0.0)),
        TickId(0),
        ActivationConstraint::None,
        CollisionStrategy::ClosestCenter,
        None,
    );
    assert_eq!(out.over, Some(DndItemId(1)));

    let out = handle_pointer_move_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        DragKindId(1),
        DndScopeId(1),
        PointerId(0),
        Point::new(Px(105.0), Px(5.0)),
        TickId(1),
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
    let window = fret_core::AppWindowId::default();
    let frame = FrameId(1);

    let p0 = PointerId(0);
    let p1 = PointerId(1);

    let start = Point::new(Px(0.0), Px(0.0));
    let move_small = Point::new(Px(1.0), Px(0.0));

    let _ = handle_pointer_down_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        DragKindId(1),
        DndScopeId(1),
        p0,
        start,
        TickId(0),
        ActivationConstraint::DelayTicks { ticks: 10 },
        CollisionStrategy::ClosestCenter,
        None,
    );
    let _ = handle_pointer_move_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        DragKindId(1),
        DndScopeId(1),
        p0,
        move_small,
        TickId(1),
        ActivationConstraint::DelayTicks { ticks: 10 },
        CollisionStrategy::ClosestCenter,
        None,
    );

    let _ = handle_pointer_down_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        DragKindId(1),
        DndScopeId(2),
        p1,
        start,
        TickId(0),
        ActivationConstraint::None,
        CollisionStrategy::ClosestCenter,
        None,
    );
    let out_p1 = handle_pointer_move_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        DragKindId(1),
        DndScopeId(2),
        p1,
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

    let out_p0 = handle_pointer_move_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        DragKindId(1),
        DndScopeId(1),
        p0,
        move_small,
        TickId(2),
        ActivationConstraint::DelayTicks { ticks: 10 },
        CollisionStrategy::ClosestCenter,
        None,
    );

    assert!(matches!(out_p0.sensor, SensorOutput::Pending));
}

#[test]
fn controller_reports_collisions_in_deterministic_order() {
    let mut app = App::new();
    let svc = mk_service(&mut app);
    let window = fret_core::AppWindowId::default();
    let frame = FrameId(1);
    let scope = DndScopeId(7);

    register_droppable_rect_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        scope,
        DndItemId(2),
        rect(0.0, 0.0, 10.0, 10.0),
        10,
        false,
    );
    register_droppable_rect_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        scope,
        DndItemId(1),
        rect(0.0, 0.0, 10.0, 10.0),
        0,
        false,
    );

    let out = handle_pointer_down_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        DragKindId(3),
        scope,
        PointerId(0),
        Point::new(Px(5.0), Px(5.0)),
        TickId(0),
        ActivationConstraint::None,
        CollisionStrategy::PointerWithin,
        None,
    );

    assert_eq!(out.over, Some(DndItemId(2)));
    assert_eq!(out.collisions.len(), 2);
    assert_eq!(out.collisions[0].id, DndItemId(2));
    assert_eq!(out.collisions[1].id, DndItemId(1));
}

#[test]
fn controller_update_matches_headless_frame_output() {
    let mut app = App::new();
    let svc = mk_service(&mut app);
    let window = fret_core::AppWindowId::default();
    let frame = FrameId(1);
    let scope = DndScopeId(9);
    let kind = DragKindId(4);
    let pointer = PointerId(0);
    let pointer_position = Point::new(Px(95.0), Px(5.0));
    let autoscroll = Some((
        rect(0.0, 0.0, 100.0, 20.0),
        AutoScrollConfig {
            margin_px: 10.0,
            min_speed_px_per_tick: 0.0,
            max_speed_px_per_tick: 10.0,
        },
    ));

    register_droppable_rect_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        scope,
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
        scope,
        DndItemId(2),
        rect(90.0, 0.0, 10.0, 10.0),
        0,
        false,
    );

    let _ = handle_pointer_down_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        kind,
        scope,
        pointer,
        Point::new(Px(0.0), Px(0.0)),
        TickId(0),
        ActivationConstraint::None,
        CollisionStrategy::ClosestCenter,
        autoscroll,
    );

    let out = handle_pointer_move_in_scope(
        app.models_mut(),
        &svc,
        window,
        frame,
        kind,
        scope,
        pointer,
        pointer_position,
        TickId(1),
        ActivationConstraint::None,
        CollisionStrategy::ClosestCenter,
        autoscroll,
    );

    let expected = compute_dnd_frame(
        &RegistrySnapshot {
            draggables: vec![],
            droppables: vec![
                Droppable {
                    id: DndItemId(1),
                    rect: rect(0.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
                Droppable {
                    id: DndItemId(2),
                    rect: rect(90.0, 0.0, 10.0, 10.0),
                    disabled: false,
                    z_index: 0,
                },
            ],
        },
        pointer_position,
        CollisionStrategy::ClosestCenter,
        autoscroll,
    );

    assert_eq!(out.over, expected.over);
    assert_eq!(out.collisions, expected.collisions);
    assert_eq!(out.autoscroll, expected.autoscroll);
}
