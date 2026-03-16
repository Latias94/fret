use fret_app::App;
use fret_core::{AppWindowId, FrameId, KeyCode, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;

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
        "dialog-escape-dismiss-focus-restore",
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

#[test]
fn dialog_escape_closes_and_restores_focus_to_trigger() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::facade::themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::facade::themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::facade::themes::ShadcnColorScheme::Light,
    );

    let open: Model<bool> = app.models_mut().insert(false);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let build = move |open: Model<bool>| {
        move |cx: &mut ElementContext<'_, App>| {
            vec![shadcn::Dialog::new(open).into_element_parts(
                cx,
                |cx| {
                    shadcn::DialogTrigger::new(
                        shadcn::Button::new("Open")
                            .test_id("dialog-trigger")
                            .into_element(cx),
                    )
                },
                shadcn::DialogPortal::new(),
                shadcn::DialogOverlay::new(),
                |cx| shadcn::DialogContent::new([cx.text("Content")]).into_element(cx),
            )]
        }
    };

    // Frame 1: mount closed and click trigger.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        build(open.clone()),
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "dialog-trigger");
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
    assert_eq!(
        app.models().get_copied(&open),
        Some(true),
        "expected open after click"
    );

    // Frame 2+: settle open overlays before dismiss.
    let settle_frames = shadcn_motion::ticks_100() + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            request_semantics,
            build(open.clone()),
        );
    }

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    // Frame after dismissal.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        build(open.clone()),
    );

    assert_eq!(
        app.models().get_copied(&open),
        Some(false),
        "expected dialog to close after Escape"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "dialog-trigger");
    assert!(
        trigger.flags.focused,
        "expected focus to restore to the trigger after Escape"
    );
}
