use fret_app::App;
use fret_core::{AppWindowId, FrameId, KeyCode, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::tree::UiTree;
use fret_ui::ElementContext;
use fret_ui_kit::OverlayController;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::{click_at, dispatch_key_press};

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
        "navigation-menu-escape-dismiss-focus-restore",
        root,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
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

fn build_menu(cx: &mut ElementContext<'_, App>, model: Model<Option<Arc<str>>>) -> Vec<AnyElement> {
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

    let items = vec![
        fret_ui_shadcn::NavigationMenuItem::new("alpha", "Alpha", [alpha_content])
            .trigger_test_id("nav-trigger-alpha"),
        fret_ui_shadcn::NavigationMenuItem::new("beta", "Beta", [beta_content])
            .trigger_test_id("nav-trigger-beta"),
    ];

    vec![fret_ui_shadcn::NavigationMenu::new(model)
        .items(items)
        .viewport_test_id("nav-viewport")
        .into_element(cx)]
}

fn build_root(
    model: Model<Option<Arc<str>>>,
) -> impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement> {
    move |cx| build_menu(cx, model)
}

#[test]
fn navigation_menu_escape_closes_and_restores_focus_to_trigger() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let model: Model<Option<Arc<str>>> = app.models_mut().insert(None::<Arc<str>>);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    // Frame 1: mount closed and click trigger.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        build_root(model.clone()),
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "nav-trigger-alpha");
    click_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(
            Px(trigger.bounds.origin.x.0 + 5.0),
            Px(trigger.bounds.origin.y.0 + 5.0),
        ),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    // Frame 2+: settle open overlays before dismiss.
    let open_settle_frames = shadcn_motion::ticks_100() + 2;
    for tick in 0..open_settle_frames {
        let request_semantics = tick + 1 == open_settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            request_semantics,
            build_root(model.clone()),
        );
    }

    assert_eq!(
        app.models().get_cloned(&model).flatten().as_deref(),
        Some("alpha"),
        "expected open value after click"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        has_test_id(&snap, "nav-viewport"),
        "expected viewport to be present when open"
    );
    assert!(
        has_test_id(&snap, "nav-content-alpha"),
        "expected alpha content marker to be present when open"
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    // Frames after dismissal: allow close transition to settle.
    let close_settle_frames = shadcn_motion::ticks_100() + 2;
    for tick in 0..close_settle_frames {
        let request_semantics = tick + 1 == close_settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            request_semantics,
            build_root(model.clone()),
        );
    }

    assert_eq!(
        app.models().get_cloned(&model).flatten().as_deref(),
        None,
        "expected cleared value after Escape"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "nav-trigger-alpha");
    assert!(
        trigger.flags.focused,
        "expected focus on trigger after Escape"
    );
    assert!(
        !has_test_id(&snap, "nav-viewport"),
        "expected viewport to be removed after Escape"
    );
}
