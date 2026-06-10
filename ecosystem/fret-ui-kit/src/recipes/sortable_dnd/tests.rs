use super::*;

use std::cell::RefCell;
use std::rc::Rc;

use fret_app::App;
use fret_core::{
    AppWindowId, Modifiers, MouseButtons, PathCommand, PathConstraints, PathId, PathMetrics,
    PathService, PathStyle, Point, PointerType, Rect, Size, SvgId, SvgService, TextBlobId,
    TextConstraints, TextInput, TextMetrics, TextService,
};
use fret_runtime::{FrameId, TickId};
use fret_ui::ThemeConfig;
use fret_ui::{Theme, UiTree};

#[derive(Default)]
struct FakeServices;

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

fn bump_tick(app: &mut App) {
    app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
}

fn bump_frame(app: &mut App) {
    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[allow(clippy::too_many_arguments)]
fn render(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut FakeServices,
    window: AppWindowId,
    bounds: Rect,
    items: Model<Vec<DndItemId>>,
    row_ids: &Rc<RefCell<Vec<fret_ui::GlobalElementId>>>,
    props: SortableReorderListProps,
) -> fret_core::NodeId {
    let row_ids = row_ids.clone();
    fret_ui::declarative::render_root(ui, app, services, window, bounds, "sortable", |cx| {
        row_ids.borrow_mut().clear();
        let el = sortable_reorder_list(cx, items, props, |cx, id| {
            row_ids.borrow_mut().push(cx.root_id());
            vec![cx.text(format!("Item {}", id.0))]
        });
        vec![el]
    })
}

#[test]
fn sortable_reorder_moves_item_to_over_index() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    Theme::with_global_mut(&mut app, |theme| {
        theme.apply_config(&ThemeConfig {
            name: "Test".to_string(),
            ..ThemeConfig::default()
        });
    });

    let items = app
        .models_mut()
        .insert(vec![DndItemId(1), DndItemId(2), DndItemId(3)]);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
    );
    let mut services = FakeServices;

    let row_ids: Rc<RefCell<Vec<fret_ui::GlobalElementId>>> = Rc::new(RefCell::new(Vec::new()));

    // Needs two frames: geometry comes from `last_bounds_for_element` (prev-bounds snapshot).
    for _ in 0..2 {
        bump_tick(&mut app);
        bump_frame(&mut app);
        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            items.clone(),
            &row_ids,
            SortableReorderListProps {
                row_height: Px(32.0),
                activation: ActivationConstraint::Distance { px: 6.0 },
                collision_strategy: CollisionStrategy::ClosestCenter,
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    }

    let elements = row_ids.borrow().clone();
    assert_eq!(elements.len(), 3);

    let nodes = elements
        .iter()
        .map(|&el| fret_ui::elements::node_for_element(&mut app, window, el).expect("node"))
        .collect::<Vec<_>>();
    let rects = nodes
        .iter()
        .map(|&n| ui.debug_node_bounds(n).expect("bounds"))
        .collect::<Vec<_>>();

    assert!(
        rects[0].size.width.0 > 0.0 && rects[0].size.height.0 > 0.0,
        "expected non-empty row bounds"
    );
    assert!(
        rects[0].origin.y.0 < rects[1].origin.y.0 && rects[1].origin.y.0 < rects[2].origin.y.0,
        "expected stacked rows to have increasing y origins"
    );

    let center = |r: Rect| {
        Point::new(
            Px(r.origin.x.0 + r.size.width.0 * 0.5),
            Px(r.origin.y.0 + r.size.height.0 * 0.5),
        )
    };

    let start = center(rects[0]);
    // Drop on the lower half of the target row so we insert "after" the `over` item.
    let target = Point::new(
        Px(rects[2].origin.x.0 + rects[2].size.width.0 * 0.5),
        Px(rects[2].origin.y.0 + rects[2].size.height.0 * 0.75),
    );

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured_for(PointerId(0)).is_some(),
        "expected pointer to be captured after down"
    );

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: target,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured_for(PointerId(0)).is_some(),
        "expected pointer to remain captured during move"
    );

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: target,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured_for(PointerId(0)).is_none(),
        "expected pointer capture to be released after up"
    );

    bump_tick(&mut app);
    bump_frame(&mut app);
    let root = render(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        items.clone(),
        &row_ids,
        SortableReorderListProps {
            row_height: Px(32.0),
            activation: ActivationConstraint::Distance { px: 6.0 },
            collision_strategy: CollisionStrategy::ClosestCenter,
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let after = app.models().get_cloned(&items).unwrap_or_default();
    assert_eq!(after, vec![DndItemId(2), DndItemId(3), DndItemId(1)]);
}

#[test]
fn sortable_reorder_inserts_before_over_when_dropping_on_upper_half() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    Theme::with_global_mut(&mut app, |theme| {
        theme.apply_config(&ThemeConfig {
            name: "Test".to_string(),
            ..ThemeConfig::default()
        });
    });

    let items = app
        .models_mut()
        .insert(vec![DndItemId(1), DndItemId(2), DndItemId(3)]);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
    );
    let mut services = FakeServices;

    let row_ids: Rc<RefCell<Vec<fret_ui::GlobalElementId>>> = Rc::new(RefCell::new(Vec::new()));

    for _ in 0..2 {
        bump_tick(&mut app);
        bump_frame(&mut app);
        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            items.clone(),
            &row_ids,
            SortableReorderListProps {
                row_height: Px(32.0),
                activation: ActivationConstraint::Distance { px: 6.0 },
                collision_strategy: CollisionStrategy::ClosestCenter,
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    }

    let elements = row_ids.borrow().clone();
    let nodes = elements
        .iter()
        .map(|&el| fret_ui::elements::node_for_element(&mut app, window, el).expect("node"))
        .collect::<Vec<_>>();
    let rects = nodes
        .iter()
        .map(|&n| ui.debug_node_bounds(n).expect("bounds"))
        .collect::<Vec<_>>();

    let start = Point::new(
        Px(rects[0].origin.x.0 + rects[0].size.width.0 * 0.5),
        Px(rects[0].origin.y.0 + rects[0].size.height.0 * 0.5),
    );
    // Drop on the upper half of the target row so we insert "before" the `over` item.
    let target = Point::new(
        Px(rects[2].origin.x.0 + rects[2].size.width.0 * 0.5),
        Px(rects[2].origin.y.0 + rects[2].size.height.0 * 0.25),
    );

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: target,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: target,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    bump_tick(&mut app);
    bump_frame(&mut app);
    let root = render(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        items.clone(),
        &row_ids,
        SortableReorderListProps {
            row_height: Px(32.0),
            activation: ActivationConstraint::Distance { px: 6.0 },
            collision_strategy: CollisionStrategy::ClosestCenter,
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let after = app.models().get_cloned(&items).unwrap_or_default();
    assert_eq!(after, vec![DndItemId(2), DndItemId(1), DndItemId(3)]);
}

#[test]
fn sortable_reorder_does_not_move_without_activation() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    Theme::with_global_mut(&mut app, |theme| {
        theme.apply_config(&ThemeConfig {
            name: "Test".to_string(),
            ..ThemeConfig::default()
        });
    });

    let items = app
        .models_mut()
        .insert(vec![DndItemId(1), DndItemId(2), DndItemId(3)]);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
    );
    let mut services = FakeServices;

    let row_ids: Rc<RefCell<Vec<fret_ui::GlobalElementId>>> = Rc::new(RefCell::new(Vec::new()));

    for _ in 0..2 {
        bump_tick(&mut app);
        bump_frame(&mut app);
        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            items.clone(),
            &row_ids,
            SortableReorderListProps {
                row_height: Px(32.0),
                activation: ActivationConstraint::Distance { px: 9999.0 },
                collision_strategy: CollisionStrategy::ClosestCenter,
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    }

    let elements = row_ids.borrow().clone();
    let nodes = elements
        .iter()
        .map(|&el| fret_ui::elements::node_for_element(&mut app, window, el).expect("node"))
        .collect::<Vec<_>>();
    let rects = nodes
        .iter()
        .map(|&n| ui.debug_node_bounds(n).expect("bounds"))
        .collect::<Vec<_>>();

    let center = |r: Rect| {
        Point::new(
            Px(r.origin.x.0 + r.size.width.0 * 0.5),
            Px(r.origin.y.0 + r.size.height.0 * 0.5),
        )
    };
    let start = center(rects[0]);
    let small_move = Point::new(Px(start.x.0 + 2.0), start.y);

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(ui.captured_for(PointerId(0)).is_some());

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: small_move,
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(ui.captured_for(PointerId(0)).is_some());

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: small_move,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(ui.captured_for(PointerId(0)).is_none());

    let after = app.models().get_cloned(&items).unwrap_or_default();
    assert_eq!(after, vec![DndItemId(1), DndItemId(2), DndItemId(3)]);
}

#[test]
fn sortable_reorder_clears_state_on_buttons_release() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    Theme::with_global_mut(&mut app, |theme| {
        theme.apply_config(&ThemeConfig {
            name: "Test".to_string(),
            ..ThemeConfig::default()
        });
    });

    let items = app
        .models_mut()
        .insert(vec![DndItemId(1), DndItemId(2), DndItemId(3)]);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
    );
    let mut services = FakeServices;

    let row_ids: Rc<RefCell<Vec<fret_ui::GlobalElementId>>> = Rc::new(RefCell::new(Vec::new()));

    for _ in 0..2 {
        bump_tick(&mut app);
        bump_frame(&mut app);
        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            items.clone(),
            &row_ids,
            SortableReorderListProps {
                row_height: Px(32.0),
                activation: ActivationConstraint::Distance { px: 6.0 },
                collision_strategy: CollisionStrategy::ClosestCenter,
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    }

    let elements = row_ids.borrow().clone();
    let nodes = elements
        .iter()
        .map(|&el| fret_ui::elements::node_for_element(&mut app, window, el).expect("node"))
        .collect::<Vec<_>>();
    let rects = nodes
        .iter()
        .map(|&n| ui.debug_node_bounds(n).expect("bounds"))
        .collect::<Vec<_>>();

    let center = |r: Rect| {
        Point::new(
            Px(r.origin.x.0 + r.size.width.0 * 0.5),
            Px(r.origin.y.0 + r.size.height.0 * 0.5),
        )
    };
    let start = center(rects[0]);

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(ui.captured_for(PointerId(0)).is_some());

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: start,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured_for(PointerId(0)).is_none(),
        "expected capture release when buttons are no longer pressed"
    );

    let after = app.models().get_cloned(&items).unwrap_or_default();
    assert_eq!(after, vec![DndItemId(1), DndItemId(2), DndItemId(3)]);
}

#[test]
fn sortable_reorder_clears_tracking_on_pointer_cancel() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    Theme::with_global_mut(&mut app, |theme| {
        theme.apply_config(&ThemeConfig {
            name: "Test".to_string(),
            ..ThemeConfig::default()
        });
    });

    let items = app
        .models_mut()
        .insert(vec![DndItemId(1), DndItemId(2), DndItemId(3)]);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(160.0)),
    );
    let mut services = FakeServices;

    let row_ids: Rc<RefCell<Vec<fret_ui::GlobalElementId>>> = Rc::new(RefCell::new(Vec::new()));

    for _ in 0..2 {
        bump_tick(&mut app);
        bump_frame(&mut app);
        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            items.clone(),
            &row_ids,
            SortableReorderListProps {
                row_height: Px(32.0),
                activation: ActivationConstraint::Distance { px: 6.0 },
                collision_strategy: CollisionStrategy::ClosestCenter,
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    }

    let elements = row_ids.borrow().clone();
    let nodes = elements
        .iter()
        .map(|&el| fret_ui::elements::node_for_element(&mut app, window, el).expect("node"))
        .collect::<Vec<_>>();
    let rects = nodes
        .iter()
        .map(|&n| ui.debug_node_bounds(n).expect("bounds"))
        .collect::<Vec<_>>();

    let start = Point::new(
        Px(rects[0].origin.x.0 + rects[0].size.width.0 * 0.5),
        Px(rects[0].origin.y.0 + rects[0].size.height.0 * 0.5),
    );

    let dnd = dnd::dnd_service_model_global(&mut app);

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    assert!(ui.captured_for(PointerId(0)).is_some());
    assert!(dnd::pointer_is_tracking_any_sensor(
        app.models(),
        &dnd,
        window,
        PointerId(0)
    ));

    bump_tick(&mut app);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: PointerId(0),
            position: Some(start),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );

    assert!(
        ui.captured_for(PointerId(0)).is_none(),
        "expected pointer capture to be released after pointer cancel"
    );
    assert!(!dnd::pointer_is_tracking_any_sensor(
        app.models(),
        &dnd,
        window,
        PointerId(0)
    ));

    let after = app.models().get_cloned(&items).unwrap_or_default();
    assert_eq!(after, vec![DndItemId(1), DndItemId(2), DndItemId(3)]);
}
