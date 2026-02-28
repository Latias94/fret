use super::*;

use fret_app::App;
use fret_core::{Point, PointerId, Px, Rect, Size};
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
