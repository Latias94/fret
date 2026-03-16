use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButton, MouseButtons, Point, PointerCancelEvent,
    PointerCancelReason, PointerEvent, PointerId, PointerType, Px, Rect, Size as CoreSize,
};
use fret_runtime::Model;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

fn window_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(960.0), Px(480.0)),
    )
}

fn seed_items(app: &mut App) -> Model<Vec<fret_ui_shadcn::raw::extras::KanbanItem>> {
    app.models_mut().insert(vec![
        fret_ui_shadcn::raw::extras::KanbanItem::new("card-1", "Write docs", "backlog"),
        fret_ui_shadcn::raw::extras::KanbanItem::new("card-2", "Port block", "backlog"),
        fret_ui_shadcn::raw::extras::KanbanItem::new("card-3", "Add gates", "in_progress"),
        fret_ui_shadcn::raw::extras::KanbanItem::new("card-4", "Fix regressions", "in_progress"),
        fret_ui_shadcn::raw::extras::KanbanItem::new("card-5", "Ship", "done"),
    ])
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    items: Model<Vec<fret_ui_shadcn::raw::extras::KanbanItem>>,
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
        "kanban-dnd-forwarders",
        move |cx| {
            let columns = vec![
                fret_ui_shadcn::raw::extras::KanbanColumn::new("backlog", "Backlog"),
                fret_ui_shadcn::raw::extras::KanbanColumn::new("in_progress", "In Progress"),
                fret_ui_shadcn::raw::extras::KanbanColumn::new("done", "Done"),
            ];

            vec![
                fret_ui_shadcn::raw::extras::Kanban::new(columns, items.clone())
                    .test_id("kanban-dnd-forwarders")
                    .into_element(cx),
            ]
        },
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn node_id_by_test_id(ui: &UiTree<App>, test_id: &str) -> fret_core::NodeId {
    ui.semantics_snapshot()
        .expect("semantics snapshot")
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .unwrap_or_else(|| panic!("missing semantics node test_id={test_id}"))
        .id
}

fn point_in_rect(rect: Rect, x_factor: f32, y_factor: f32) -> Point {
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * x_factor),
        Px(rect.origin.y.0 + rect.size.height.0 * y_factor),
    )
}

fn item_ids(app: &App, items: &Model<Vec<fret_ui_shadcn::raw::extras::KanbanItem>>) -> Vec<String> {
    app.models()
        .read(items, |items| {
            items
                .iter()
                .map(|item| item.id.as_ref().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[test]
fn kanban_drag_reorders_cards_with_forwarders() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let items = seed_items(&mut app);
    let dnd_service = fret_ui_kit::dnd::dnd_service_model_global(&mut app);

    for _ in 0..2 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            items.clone(),
        );
    }

    let card_1 = ui
        .debug_node_bounds(node_id_by_test_id(&ui, "shadcn-extras.kanban.card-card-1"))
        .expect("card-1 bounds");
    let card_2 = ui
        .debug_node_bounds(node_id_by_test_id(&ui, "shadcn-extras.kanban.card-card-2"))
        .expect("card-2 bounds");

    let pointer_id = PointerId(0);
    let start = point_in_rect(card_1, 0.5, 0.5);
    let target = point_in_rect(card_2, 0.5, 0.8);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id,
            position: start,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );

    assert!(
        ui.captured_for(pointer_id).is_none(),
        "expected no capture before drag activation"
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
            position: target,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    assert!(
        ui.captured_for(pointer_id).is_some(),
        "expected capture after drag activation"
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
        &Event::Pointer(PointerEvent::Up {
            pointer_id,
            position: target,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 0,
            pointer_type: PointerType::Mouse,
        }),
    );

    assert!(
        ui.captured_for(pointer_id).is_none(),
        "expected capture release after drop"
    );
    assert!(!fret_ui_kit::dnd::pointer_is_tracking_any_sensor(
        app.models(),
        &dnd_service,
        window,
        pointer_id
    ));
    assert_eq!(
        item_ids(&app, &items),
        vec!["card-2", "card-1", "card-3", "card-4", "card-5"]
    );
}

#[test]
fn kanban_pointer_cancel_clears_tracking_after_activation() {
    let window = AppWindowId::default();
    let bounds = window_bounds();

    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let items = seed_items(&mut app);
    let dnd_service = fret_ui_kit::dnd::dnd_service_model_global(&mut app);

    for _ in 0..2 {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            items.clone(),
        );
    }

    let card_1 = ui
        .debug_node_bounds(node_id_by_test_id(&ui, "shadcn-extras.kanban.card-card-1"))
        .expect("card-1 bounds");
    let card_2 = ui
        .debug_node_bounds(node_id_by_test_id(&ui, "shadcn-extras.kanban.card-card-2"))
        .expect("card-2 bounds");

    let pointer_id = PointerId(0);
    let start = point_in_rect(card_1, 0.5, 0.5);
    let target = point_in_rect(card_2, 0.5, 0.8);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id,
            position: start,
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
            position: target,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    assert!(
        ui.captured_for(pointer_id).is_some(),
        "expected capture after drag activation"
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
        &Event::PointerCancel(PointerCancelEvent {
            pointer_id,
            position: Some(target),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            reason: PointerCancelReason::LeftWindow,
        }),
    );

    assert!(
        ui.captured_for(pointer_id).is_none(),
        "expected capture release after pointer cancel"
    );
    assert!(!fret_ui_kit::dnd::pointer_is_tracking_any_sensor(
        app.models(),
        &dnd_service,
        window,
        pointer_id
    ));
    assert_eq!(
        item_ids(&app, &items),
        vec!["card-1", "card-2", "card-3", "card-4", "card-5"]
    );
}
