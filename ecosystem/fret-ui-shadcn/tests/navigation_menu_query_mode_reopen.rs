use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, Modifiers, MouseButtons, Point, PointerEvent, PointerId,
    PointerType, Px, Rect, SemanticsRole, Size as CoreSize,
};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::{AnyElement, LayoutQueryRegionProps, LayoutStyle, Length, SemanticsProps};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::{LayoutRefinement, Space, ui};
use fret_ui_shadcn::facade as shadcn;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;

#[path = "support/shadcn_motion.rs"]
mod shadcn_motion;

#[path = "support/timers.rs"]
mod timers;
use timers::TimerQueue;

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    root: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
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
        "navigation-menu-query-mode-reopen",
        root,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn settle_frames(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frames: u64,
    root: impl Fn() -> Box<dyn FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>>,
) {
    for tick in 0..frames {
        let request_semantics = tick + 1 == frames;
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            request_semantics,
            root(),
        );
    }
}

fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(id))
        .unwrap_or_else(|| panic!("missing semantics node with test_id={id:?}"))
}

fn has_test_id(snap: &fret_core::SemanticsSnapshot, id: &str) -> bool {
    snap.nodes.iter().any(|n| n.test_id.as_deref() == Some(id))
}

fn move_and_click_at(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    position: Point,
) {
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: PointerId(0),
            position,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: PointerId(0),
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: PointerId(0),
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            pointer_type: PointerType::Mouse,
            click_count: 1,
        }),
    );
}

fn build_root(
    model: Model<Option<Arc<str>>>,
    use_container_query: Rc<Cell<bool>>,
) -> impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement> {
    move |cx| {
        let alpha_content = cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Generic,
                test_id: Some(Arc::from("nav-content-alpha")),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );
        let beta_content = cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Generic,
                test_id: Some(Arc::from("nav-content-beta")),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        let theme = cx.theme_snapshot();
        let region_props = LayoutQueryRegionProps {
            layout: fret_ui_kit::declarative::style::layout_style(
                &theme,
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(Px(560.0))
                    .min_w_0(),
            ),
            name: None,
        };

        vec![cx.layout_query_region_with_id(region_props, move |cx, region_id| {
            let items = vec![
                shadcn::NavigationMenuItem::new("alpha", "Alpha", [alpha_content])
                    .trigger_test_id("nav-trigger-alpha"),
                shadcn::NavigationMenuItem::new("beta", "Beta", [beta_content])
                    .trigger_test_id("nav-trigger-beta"),
            ];

            let md_query = if use_container_query.get() {
                shadcn::NavigationMenuMdBreakpointQuery::Container
            } else {
                shadcn::NavigationMenuMdBreakpointQuery::Viewport
            };

            let menu = shadcn::NavigationMenu::new(model.clone())
                .items(items)
                .container_query_region(region_id)
                .md_breakpoint_query(md_query)
                .viewport_test_id("nav-viewport")
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_full()
                        .min_w_0(),
                )
                .into_element(cx);

            vec![cx.container(
                fret_ui::element::ContainerProps {
                    layout: LayoutStyle {
                        size: fret_ui::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |_cx| vec![menu],
            )]
        })]
    }
}

fn build_root_with_switch(
    model: Model<Option<Arc<str>>>,
    query_model: Model<bool>,
) -> impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement> {
    move |cx| {
        let alpha_content = cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Generic,
                test_id: Some(Arc::from("nav-content-alpha")),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );
        let beta_content = cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Generic,
                test_id: Some(Arc::from("nav-content-beta")),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        let use_container_query = cx.watch_model(&query_model).layout().copied().unwrap_or(false);
        let theme = cx.theme_snapshot();
        let region_props = LayoutQueryRegionProps {
            layout: fret_ui_kit::declarative::style::layout_style(
                &theme,
                LayoutRefinement::default().w_px(Px(560.0)).min_w_0(),
            ),
            name: None,
        };

        let switch = shadcn::Switch::new(query_model.clone())
            .test_id("query-switch")
            .a11y_label("Use container query breakpoints")
            .into_element(cx);

        let menu = cx.layout_query_region_with_id(region_props, move |cx, region_id| {
            let items = vec![
                shadcn::NavigationMenuItem::new("alpha", "Alpha", [alpha_content])
                    .trigger_test_id("nav-trigger-alpha"),
                shadcn::NavigationMenuItem::new("beta", "Beta", [beta_content])
                    .trigger_test_id("nav-trigger-beta"),
            ];

            let md_query = if use_container_query {
                shadcn::NavigationMenuMdBreakpointQuery::Container
            } else {
                shadcn::NavigationMenuMdBreakpointQuery::Viewport
            };

            let menu = shadcn::NavigationMenu::new(model.clone())
                .items(items)
                .container_query_region(region_id)
                .md_breakpoint_query(md_query)
                .viewport_test_id("nav-viewport")
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            vec![cx.container(
                fret_ui::element::ContainerProps {
                    layout: LayoutStyle {
                        size: fret_ui::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |_cx| vec![menu],
            )]
        });

        vec![
            ui::v_stack(move |_cx| vec![switch, menu])
                .gap(Space::N6)
                .items_start()
                .into_element(cx),
        ]
    }
}

#[test]
fn navigation_menu_reopens_after_switching_md_query_source() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(720.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
    );

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None::<Arc<str>>);
    let use_container_query = Rc::new(Cell::new(false));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    settle_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        3,
        {
            let model = model.clone();
            let use_container_query = use_container_query.clone();
            move || Box::new(build_root(model.clone(), use_container_query.clone()))
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "nav-trigger-beta");
    let trigger_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );

    move_and_click_at(&mut ui, &mut app, &mut services, trigger_point);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    settle_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        shadcn_motion::ticks_200() + 2,
        {
            let model = model.clone();
            let use_container_query = use_container_query.clone();
            move || Box::new(build_root(model.clone(), use_container_query.clone()))
        },
    );

    assert_eq!(
        app.models().get_cloned(&model).flatten().as_deref(),
        Some("beta"),
        "expected the beta item to open under viewport-driven md breakpoints"
    );
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        has_test_id(&snap, "nav-viewport"),
        "expected viewport panel while the menu is open"
    );
    assert!(
        has_test_id(&snap, "nav-content-beta"),
        "expected beta content to be mounted while open"
    );

    let trigger = find_by_test_id(&snap, "nav-trigger-beta");
    let trigger_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    move_and_click_at(&mut ui, &mut app, &mut services, trigger_point);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    settle_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        shadcn_motion::ticks_200() + 2,
        {
            let model = model.clone();
            let use_container_query = use_container_query.clone();
            move || Box::new(build_root(model.clone(), use_container_query.clone()))
        },
    );

    assert_eq!(
        app.models().get_cloned(&model).flatten().as_deref(),
        None,
        "expected the menu to close before switching the query source"
    );

    use_container_query.set(true);

    settle_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        4,
        {
            let model = model.clone();
            let use_container_query = use_container_query.clone();
            move || Box::new(build_root(model.clone(), use_container_query.clone()))
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "nav-trigger-beta");
    let trigger_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );

    move_and_click_at(&mut ui, &mut app, &mut services, trigger_point);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    settle_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        shadcn_motion::ticks_200() + 2,
        {
            let model = model.clone();
            let use_container_query = use_container_query.clone();
            move || Box::new(build_root(model.clone(), use_container_query.clone()))
        },
    );

    assert_eq!(
        app.models().get_cloned(&model).flatten().as_deref(),
        Some("beta"),
        "expected the beta item to reopen after switching from viewport to container md queries"
    );
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        has_test_id(&snap, "nav-viewport"),
        "expected viewport panel after reopening under container-driven md breakpoints"
    );
    assert!(
        has_test_id(&snap, "nav-content-beta"),
        "expected beta content after reopening under container-driven md breakpoints"
    );
}

#[test]
fn navigation_menu_reopens_after_clicking_query_switch() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(1280.0), Px(720.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
    );

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None::<Arc<str>>);
    let query_model: Model<bool> = app.models_mut().insert(false);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    settle_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        3,
        {
            let model = model.clone();
            let query_model = query_model.clone();
            move || Box::new(build_root_with_switch(model.clone(), query_model.clone()))
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "nav-trigger-beta");
    let trigger_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );

    move_and_click_at(&mut ui, &mut app, &mut services, trigger_point);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    settle_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        shadcn_motion::ticks_200() + 2,
        {
            let model = model.clone();
            let query_model = query_model.clone();
            move || Box::new(build_root_with_switch(model.clone(), query_model.clone()))
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "nav-trigger-beta");
    let trigger_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    move_and_click_at(&mut ui, &mut app, &mut services, trigger_point);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    settle_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        shadcn_motion::ticks_200() + 2,
        {
            let model = model.clone();
            let query_model = query_model.clone();
            move || Box::new(build_root_with_switch(model.clone(), query_model.clone()))
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let switch = find_by_test_id(&snap, "query-switch");
    let switch_point = Point::new(
        Px(switch.bounds.origin.x.0 + switch.bounds.size.width.0 * 0.5),
        Px(switch.bounds.origin.y.0 + switch.bounds.size.height.0 * 0.5),
    );
    move_and_click_at(&mut ui, &mut app, &mut services, switch_point);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    settle_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        4,
        {
            let model = model.clone();
            let query_model = query_model.clone();
            move || Box::new(build_root_with_switch(model.clone(), query_model.clone()))
        },
    );

    assert!(
        app.models().get_copied(&query_model).unwrap_or(false),
        "expected clicking the switch to enable container-driven md breakpoints"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "nav-trigger-beta");
    let trigger_point = Point::new(
        Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
        Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
    );
    move_and_click_at(&mut ui, &mut app, &mut services, trigger_point);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    settle_frames(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        shadcn_motion::ticks_200() + 2,
        {
            let model = model.clone();
            let query_model = query_model.clone();
            move || Box::new(build_root_with_switch(model.clone(), query_model.clone()))
        },
    );

    assert_eq!(
        app.models().get_cloned(&model).flatten().as_deref(),
        Some("beta"),
        "expected the beta item to reopen after toggling the query switch"
    );
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        has_test_id(&snap, "nav-viewport"),
        "expected viewport panel after reopening through the switch-driven query change"
    );
    assert!(
        has_test_id(&snap, "nav-content-beta"),
        "expected beta content after reopening through the switch-driven query change"
    );
}
