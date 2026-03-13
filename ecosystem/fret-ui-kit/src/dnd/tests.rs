use super::*;

use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, MouseButtons, Point, PointerId, PointerType, Px, Rect, Size};
use fret_dnd::{Droppable, RegistrySnapshot, compute_dnd_frame};
use fret_runtime::{DragKindId, Effect, FrameId, ModelStore, TickId, TimerToken};
use fret_ui::action::{
    ActionCx, PointerDownCx, PointerMoveCx, UiActionHost, UiDragActionHost, UiFocusActionHost,
    UiPointerActionHost,
};

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
}

fn mk_service(app: &mut App) -> DndServiceModel {
    DndServiceModel {
        model: app.models_mut().insert(service::DndService::default()),
    }
}

struct PointerHost {
    app: App,
    capture_count: usize,
    release_count: usize,
}

impl Default for PointerHost {
    fn default() -> Self {
        Self {
            app: App::new(),
            capture_count: 0,
            release_count: 0,
        }
    }
}

impl UiActionHost for PointerHost {
    fn models_mut(&mut self) -> &mut ModelStore {
        self.app.models_mut()
    }

    fn push_effect(&mut self, effect: Effect) {
        self.app.push_effect(effect);
    }

    fn request_redraw(&mut self, window: AppWindowId) {
        self.app.request_redraw(window);
    }

    fn next_timer_token(&mut self) -> TimerToken {
        self.app.next_timer_token()
    }

    fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
        self.app.next_clipboard_token()
    }

    fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
        self.app.next_share_sheet_token()
    }
}

impl UiFocusActionHost for PointerHost {
    fn request_focus(&mut self, _target: fret_ui::elements::GlobalElementId) {}
}

impl UiDragActionHost for PointerHost {
    fn begin_drag_with_kind(
        &mut self,
        _pointer_id: PointerId,
        _kind: DragKindId,
        _source_window: AppWindowId,
        _start: Point,
    ) {
    }

    fn begin_cross_window_drag_with_kind(
        &mut self,
        _pointer_id: PointerId,
        _kind: DragKindId,
        _source_window: AppWindowId,
        _start: Point,
    ) {
    }

    fn drag(&self, _pointer_id: PointerId) -> Option<&fret_runtime::DragSession> {
        None
    }

    fn drag_mut(&mut self, _pointer_id: PointerId) -> Option<&mut fret_runtime::DragSession> {
        None
    }

    fn cancel_drag(&mut self, _pointer_id: PointerId) {}
}

impl UiPointerActionHost for PointerHost {
    fn bounds(&self) -> Rect {
        rect(0.0, 0.0, 800.0, 600.0)
    }

    fn capture_pointer(&mut self) {
        self.capture_count += 1;
    }

    fn release_pointer_capture(&mut self) {
        self.release_count += 1;
    }

    fn set_cursor_icon(&mut self, _icon: fret_core::CursorIcon) {}

    fn prevent_default(&mut self, _action: fret_runtime::DefaultAction) {}
}

fn action_cx(target: u64) -> ActionCx {
    ActionCx {
        window: AppWindowId::default(),
        target: fret_ui::elements::GlobalElementId(target),
    }
}

fn pointer_down(
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    pointer_type: PointerType,
) -> PointerDownCx {
    PointerDownCx {
        pointer_id,
        position,
        position_local: position,
        position_window: Some(position),
        tick_id,
        pixels_per_point: 1.0,
        button: fret_core::MouseButton::Left,
        modifiers: fret_core::Modifiers::default(),
        click_count: 1,
        pointer_type,
        hit_is_text_input: false,
        hit_is_pressable: false,
        hit_pressable_target: None,
    }
}

fn pointer_move(
    pointer_id: PointerId,
    position: Point,
    tick_id: TickId,
    pointer_type: PointerType,
) -> PointerMoveCx {
    PointerMoveCx {
        pointer_id,
        position,
        position_local: position,
        position_window: Some(position),
        tick_id,
        pixels_per_point: 1.0,
        velocity_window: None,
        buttons: MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
        modifiers: fret_core::Modifiers::default(),
        pointer_type,
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

#[test]
fn forwarders_can_prevent_activation_for_text_inputs_and_nested_pressables() {
    let mut host = PointerHost::default();
    let svc = mk_service(&mut host.app);
    let frame = FrameId(1);
    let pointer = PointerId(41);
    let start = Point::new(Px(0.0), Px(0.0));

    let forwarders = DndPointerForwarders::new(
        svc.clone(),
        frame,
        DndPointerForwardersConfig::for_kind(DragKindId(21))
            .prevent_activation_on_text_input(true)
            .prevent_activation_on_pressable_descendant(true),
    );
    let on_down = forwarders.on_pointer_down();

    let mut text_input_down = pointer_down(pointer, start, TickId(0), PointerType::Mouse);
    text_input_down.hit_is_text_input = true;
    assert!(!on_down(&mut host, action_cx(10), text_input_down));
    assert_eq!(host.capture_count, 0);
    assert!(!pointer_is_tracking_any_sensor(
        host.app.models(),
        &svc,
        AppWindowId::default(),
        pointer
    ));

    let mut nested_pressable_down = pointer_down(pointer, start, TickId(1), PointerType::Mouse);
    nested_pressable_down.hit_is_pressable = true;
    nested_pressable_down.hit_pressable_target = Some(fret_ui::elements::GlobalElementId(11));
    assert!(!on_down(&mut host, action_cx(10), nested_pressable_down));
    assert_eq!(host.capture_count, 0);
    assert!(!pointer_is_tracking_any_sensor(
        host.app.models(),
        &svc,
        AppWindowId::default(),
        pointer
    ));

    let mut handle_down = pointer_down(pointer, start, TickId(2), PointerType::Mouse);
    handle_down.hit_is_pressable = true;
    handle_down.hit_pressable_target = Some(fret_ui::elements::GlobalElementId(10));
    assert!(on_down(&mut host, action_cx(10), handle_down));
    assert_eq!(host.capture_count, 1);
    assert!(pointer_is_tracking_any_sensor(
        host.app.models(),
        &svc,
        AppWindowId::default(),
        pointer
    ));
}

#[test]
fn forwarders_support_custom_prevent_activation_hooks() {
    let mut host = PointerHost::default();
    let svc = mk_service(&mut host.app);
    let frame = FrameId(1);
    let pointer = PointerId(52);
    let start = Point::new(Px(0.0), Px(0.0));

    let forwarders = DndPointerForwarders::new(
        svc.clone(),
        frame,
        DndPointerForwardersConfig::for_kind(DragKindId(22)).prevent_activation(Arc::new(
            |_action_cx, down| down.pointer_type == PointerType::Pen,
        )),
    );
    let on_down = forwarders.on_pointer_down();

    assert!(!on_down(
        &mut host,
        action_cx(20),
        pointer_down(pointer, start, TickId(0), PointerType::Pen),
    ));
    assert_eq!(host.capture_count, 0);
    assert!(!pointer_is_tracking_any_sensor(
        host.app.models(),
        &svc,
        AppWindowId::default(),
        pointer
    ));
}

#[test]
fn forwarders_resolve_pointer_specific_activation_constraints() {
    let mut host = PointerHost::default();
    let svc = mk_service(&mut host.app);
    let frame = FrameId(1);
    let update_model = host.app.models_mut().insert(DndUpdate::pending());
    let forwarders = DndPointerForwarders::new(
        svc,
        frame,
        DndPointerForwardersConfig::for_kind(DragKindId(23))
            .activation_constraint(ActivationConstraint::None)
            .touch_activation_constraint(ActivationConstraint::DelayTicks { ticks: 3 })
            .update_model(update_model.clone()),
    );
    let on_down = forwarders.on_pointer_down();
    let on_move = forwarders.on_pointer_move();
    let cx = action_cx(30);
    let start = Point::new(Px(0.0), Px(0.0));

    let mouse_pointer = PointerId(61);
    assert!(on_down(
        &mut host,
        cx,
        pointer_down(mouse_pointer, start, TickId(0), PointerType::Mouse),
    ));
    assert!(on_move(
        &mut host,
        cx,
        pointer_move(mouse_pointer, start, TickId(1), PointerType::Mouse),
    ));
    let mouse_update = host
        .app
        .models()
        .read(&update_model, |value| value.clone())
        .expect("mouse update should be recorded");
    assert!(matches!(
        mouse_update.sensor,
        SensorOutput::DragStart { .. } | SensorOutput::DragMove { .. }
    ));

    let touch_pointer = PointerId(62);
    assert!(on_down(
        &mut host,
        cx,
        pointer_down(touch_pointer, start, TickId(0), PointerType::Touch),
    ));
    assert!(on_move(
        &mut host,
        cx,
        pointer_move(touch_pointer, start, TickId(1), PointerType::Touch),
    ));
    let pending_touch_update = host
        .app
        .models()
        .read(&update_model, |value| value.clone())
        .expect("touch update should be recorded");
    assert!(matches!(pending_touch_update.sensor, SensorOutput::Pending));

    assert!(on_move(
        &mut host,
        cx,
        pointer_move(touch_pointer, start, TickId(3), PointerType::Touch),
    ));
    let activated_touch_update = host
        .app
        .models()
        .read(&update_model, |value| value.clone())
        .expect("touch activation update should be recorded");
    assert!(matches!(
        activated_touch_update.sensor,
        SensorOutput::DragStart { .. } | SensorOutput::DragMove { .. }
    ));
}
