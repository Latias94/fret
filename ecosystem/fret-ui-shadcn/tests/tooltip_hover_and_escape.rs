use fret_app::App;
use fret_core::{
    AppWindowId, Event, FrameId, KeyCode, Modifiers, MouseButtons, Point, PointerEvent, PointerId,
    PointerType, Px, Rect, Size as CoreSize,
};
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::dispatch_key_press;

#[path = "support/shadcn_motion.rs"]
mod shadcn_motion;

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
        "tooltip-hover-and-escape",
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

fn pointer_move_at(
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
}

fn build_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let trigger = fret_ui_shadcn::Button::new("Hover me")
        .test_id("tooltip-trigger")
        .into_element(cx);
    let content = fret_ui_shadcn::TooltipContent::new([cx.text("Tooltip")]).into_element(cx);
    let tooltip = fret_ui_shadcn::Tooltip::new(trigger, content)
        .open_delay_frames(0)
        .close_delay_frames(0)
        .panel_test_id("tooltip-panel")
        .into_element(cx);
    vec![tooltip]
}

#[test]
fn tooltip_hover_opens_and_leave_closes() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    // Frame 1: mount closed.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        build_tooltip,
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "tooltip-trigger");

    // Hover the trigger.
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(
            Px(trigger.bounds.origin.x.0 + 5.0),
            Px(trigger.bounds.origin.y.0 + 5.0),
        ),
    );

    // Frames: settle open presence.
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
            build_tooltip,
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        has_test_id(&snap, "tooltip-panel"),
        "expected tooltip panel to be present after hover"
    );

    // Move pointer away (leave).
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(
            Px(bounds.size.width.0 - 2.0),
            Px(bounds.size.height.0 - 2.0),
        ),
    );

    // Frames: settle close presence.
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
            build_tooltip,
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        !has_test_id(&snap, "tooltip-panel"),
        "expected tooltip panel to be removed after leave"
    );
}

#[test]
fn tooltip_escape_closes() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    // Frame 1: mount closed.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        build_tooltip,
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "tooltip-trigger");

    // Hover the trigger to open.
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(
            Px(trigger.bounds.origin.x.0 + 5.0),
            Px(trigger.bounds.origin.y.0 + 5.0),
        ),
    );

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
            build_tooltip,
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        has_test_id(&snap, "tooltip-panel"),
        "expected tooltip panel to be present before Escape"
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Escape);

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
            build_tooltip,
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        !has_test_id(&snap, "tooltip-panel"),
        "expected tooltip panel to be removed after Escape"
    );
}
