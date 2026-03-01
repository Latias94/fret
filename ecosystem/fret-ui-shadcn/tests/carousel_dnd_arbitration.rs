use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, Point, PointerEvent, PointerId,
    PointerType, Px, Rect, SemanticsRole, Size as CoreSize,
};
use fret_runtime::{DragKindId, Model};
use fret_ui::element::{
    ContainerProps, InsetEdge, LayoutStyle, Length, PointerRegionProps, PositionStyle,
};
use fret_ui::tree::UiTree;
use fret_ui_kit::{LayoutRefinement, MetricRef, OverlayController, Space};
use std::sync::Arc;

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

const TEST_DND_KIND: DragKindId = DragKindId(9_001);

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    api_snapshot: Model<fret_ui_shadcn::CarouselApiSnapshot>,
    dnd_started: Model<bool>,
    handle_captures_pointer: bool,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "carousel-dnd-arbitration",
        move |cx| {
            let dnd_service = fret_ui_kit::dnd::dnd_service_model(cx);
            let frame_id = cx.frame_id;

            let on_down_started = dnd_started.clone();
            let on_down_service = dnd_service.clone();
            let on_down: fret_ui::action::OnPointerDown = Arc::new(move |host, action_cx, down| {
                if down.button != fret_core::MouseButton::Left {
                    return false;
                }

                let _ = host.models_mut().update(&on_down_started, |v| *v = false);

                if handle_captures_pointer {
                    host.capture_pointer();
                }

                let _ = fret_ui_kit::dnd::handle_pointer_down(
                    host.models_mut(),
                    &on_down_service,
                    action_cx.window,
                    frame_id,
                    TEST_DND_KIND,
                    down.pointer_id,
                    down.position,
                    down.tick_id,
                    fret_ui_kit::dnd::ActivationConstraint::Distance { px: 2.0 },
                    fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                    None,
                );

                // Important: do *not* consume the event. This keeps the underlying Carousel
                // armed on pointer down so it would steal capture on move if we didn't
                // explicitly arbitrate against DnD tracking.
                false
            });

            let on_move_started = dnd_started.clone();
            let on_move_service = dnd_service.clone();
            let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, action_cx, mv| {
                let update = fret_ui_kit::dnd::handle_pointer_move(
                    host.models_mut(),
                    &on_move_service,
                    action_cx.window,
                    frame_id,
                    TEST_DND_KIND,
                    mv.pointer_id,
                    mv.position,
                    mv.tick_id,
                    fret_ui_kit::dnd::ActivationConstraint::Distance { px: 2.0 },
                    fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                    None,
                );

                if matches!(
                    update.sensor,
                    fret_ui_kit::dnd::SensorOutput::DragStart { .. }
                        | fret_ui_kit::dnd::SensorOutput::DragMove { .. }
                ) {
                    let _ = host.models_mut().update(&on_move_started, |v| *v = true);
                }

                false
            });

            let on_up_service = dnd_service.clone();
            let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, action_cx, up| {
                let _ = fret_ui_kit::dnd::handle_pointer_up(
                    host.models_mut(),
                    &on_up_service,
                    action_cx.window,
                    frame_id,
                    TEST_DND_KIND,
                    up.pointer_id,
                    up.position,
                    up.tick_id,
                    fret_ui_kit::dnd::ActivationConstraint::Distance { px: 2.0 },
                    fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                    None,
                );

                host.release_pointer_capture();
                false
            });

            let on_cancel_service = dnd_service.clone();
            let on_cancel: fret_ui::action::OnPointerCancel =
                Arc::new(move |host, action_cx, cancel| {
                    let position = cancel.position.unwrap_or_else(|| host.bounds().origin);
                    let _ = fret_ui_kit::dnd::handle_pointer_cancel(
                        host.models_mut(),
                        &on_cancel_service,
                        action_cx.window,
                        frame_id,
                        TEST_DND_KIND,
                        cancel.pointer_id,
                        position,
                        cancel.tick_id,
                        fret_ui_kit::dnd::ActivationConstraint::Distance { px: 2.0 },
                        fret_ui_kit::dnd::CollisionStrategy::ClosestCenter,
                        None,
                    );

                    host.release_pointer_capture();
                    false
                });

            let mut handle_layout = LayoutStyle::default();
            handle_layout.position = PositionStyle::Absolute;
            handle_layout.size.width = Length::Px(Px(28.0));
            handle_layout.size.height = Length::Px(Px(28.0));
            handle_layout.inset.top = InsetEdge::Px(Px(8.0));
            handle_layout.inset.right = InsetEdge::Px(Px(8.0));

            let mut slide_layout = LayoutStyle::default();
            slide_layout.position = PositionStyle::Relative;
            slide_layout.size.width = Length::Fill;
            slide_layout.size.height = Length::Fill;

            let handle = cx
                .pointer_region(
                    PointerRegionProps {
                        layout: handle_layout,
                        enabled: true,
                        capture_phase_pointer_moves: true,
                    },
                    move |cx| {
                        cx.pointer_region_on_pointer_down(on_down);
                        cx.pointer_region_on_pointer_move(on_move);
                        cx.pointer_region_on_pointer_up(on_up);
                        cx.pointer_region_on_pointer_cancel(on_cancel);
                        Vec::new()
                    },
                )
                .attach_semantics(
                    fret_ui::element::SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id("carousel-dnd-handle"),
                );

            let slide_1 = cx.container(
                ContainerProps {
                    layout: slide_layout,
                    ..Default::default()
                },
                move |_cx| vec![handle],
            );
            let slide_2 = cx.container(Default::default(), move |_cx| vec![]);

            let carousel = fret_ui_shadcn::Carousel::new([slide_1, slide_2])
                .api_snapshot_model(api_snapshot)
                .track_start_neg_margin(Space::N0)
                .item_padding_start(Space::N0)
                .item_basis_main_px(Px(200.0))
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(200.0)))
                        .h_px(MetricRef::Px(Px(120.0))),
                )
                .refine_viewport_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(120.0))))
                .into_element(cx);

            vec![carousel]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn carousel_drag_changes_offset_without_dnd_arbitration() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api_snapshot = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());
    let dnd_started = app.models_mut().insert(false);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(200.0)),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        api_snapshot.clone(),
        dnd_started.clone(),
        false,
    );

    let pointer_id = PointerId(0);
    let down = Point::new(Px(100.0), Px(60.0));
    let moved = Point::new(Px(0.0), Px(60.0));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id,
            position: down,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id,
            position: moved,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        api_snapshot.clone(),
        dnd_started,
        false,
    );

    let offset = app
        .models()
        .read(&api_snapshot, |v| v.offset_px)
        .unwrap_or_default();
    assert!(offset > 0.0);
}

#[test]
fn carousel_dnd_tracking_blocks_carousel_drag_when_handle_does_not_capture() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api_snapshot = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());
    let dnd_started = app.models_mut().insert(false);
    let dnd_service = fret_ui_kit::dnd::dnd_service_model_global(&mut app);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(200.0)),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        api_snapshot.clone(),
        dnd_started.clone(),
        false,
    );

    let pointer_id = PointerId(0);
    let down = Point::new(Px(180.0), Px(20.0));
    let moved = Point::new(Px(80.0), Px(20.0));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id,
            position: down,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(fret_ui_kit::dnd::pointer_is_tracking_any_sensor(
        app.models(),
        &dnd_service,
        window,
        pointer_id
    ));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id,
            position: moved,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    // Re-render so the API snapshot reflects any offset changes.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        api_snapshot.clone(),
        dnd_started.clone(),
        false,
    );

    let offset = app
        .models()
        .read(&api_snapshot, |v| v.offset_px)
        .unwrap_or_default();
    assert_eq!(offset, 0.0);
    assert_eq!(app.models().get_copied(&dnd_started), Some(false));

    // Cleanup: the DnD sensor will stay "tracking" until it sees an up/cancel.
    fret_ui_kit::dnd::clear_pointer_default_scope(
        app.models_mut(),
        &dnd_service,
        window,
        TEST_DND_KIND,
        pointer_id,
    );
}

#[test]
fn carousel_dnd_handle_capture_enables_dnd_activation_without_carousel_scroll() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let api_snapshot = app
        .models_mut()
        .insert(fret_ui_shadcn::CarouselApiSnapshot::default());
    let dnd_started = app.models_mut().insert(false);
    let dnd_service = fret_ui_kit::dnd::dnd_service_model_global(&mut app);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(320.0), Px(200.0)),
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        api_snapshot.clone(),
        dnd_started.clone(),
        true,
    );

    let pointer_id = PointerId(0);
    let down = Point::new(Px(180.0), Px(20.0));
    let moved = Point::new(Px(80.0), Px(20.0));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id,
            position: down,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id,
            position: moved,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id,
            position: moved,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 0,
            pointer_type: PointerType::Mouse,
        }),
    );

    // Re-render so the API snapshot reflects any offset changes.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        api_snapshot.clone(),
        dnd_started.clone(),
        true,
    );

    let offset = app
        .models()
        .read(&api_snapshot, |v| v.offset_px)
        .unwrap_or_default();
    assert_eq!(offset, 0.0);
    assert_eq!(app.models().get_copied(&dnd_started), Some(true));
    assert!(!fret_ui_kit::dnd::pointer_is_tracking_any_sensor(
        app.models(),
        &dnd_service,
        window,
        pointer_id
    ));
}
