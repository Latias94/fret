use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::click_at;

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
        "popover-outside-click-dismiss-focus-restore",
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
fn popover_outside_click_closes_and_activates_underlay() {
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
    let underlay_activated: Model<bool> = app.models_mut().insert(false);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    let build = move |open: Model<bool>, underlay_activated: Model<bool>| {
        move |cx: &mut ElementContext<'_, App>| {
            let on_underlay_activate: OnActivate = Arc::new(move |host, acx, _reason| {
                let _ = host.models_mut().update(&underlay_activated, |v| *v = true);
                host.request_redraw(acx.window);
            });

            let underlay = shadcn::Button::new("Underlay")
                .test_id("underlay")
                .on_activate(on_underlay_activate)
                .refine_layout(
                    LayoutRefinement::default()
                        .absolute()
                        .right(Space::N4)
                        .bottom(Space::N4),
                )
                .into_element(cx);

            let popover = shadcn::Popover::from_open(open).into_element_with(
                cx,
                |cx| {
                    shadcn::PopoverTrigger::new(
                        shadcn::Button::new("Open")
                            .test_id("popover-trigger")
                            .into_element(cx),
                    )
                    .into_element(cx)
                },
                |cx| shadcn::PopoverContent::new([cx.text("Content")]).into_element(cx),
            );

            vec![underlay, popover]
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
        build(open.clone(), underlay_activated.clone()),
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "popover-trigger");
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

    // Frame 2+: settle open overlays before clicking outside.
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
            build(open.clone(), underlay_activated.clone()),
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let underlay = find_by_test_id(&snap, "underlay");

    // Click outside the popover content (on a focusable underlay) so the outside-press click
    // still activates the underlying control.
    click_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(
            Px(underlay.bounds.origin.x.0 + 5.0),
            Px(underlay.bounds.origin.y.0 + 5.0),
        ),
    );
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    // Frames after dismissal to ensure focus restore settles.
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
            build(open.clone(), underlay_activated.clone()),
        );
    }

    assert_eq!(
        app.models().get_copied(&open),
        Some(false),
        "expected popover to close after outside click"
    );
    assert_eq!(
        app.models().get_copied(&underlay_activated),
        Some(true),
        "expected outside click to still activate the underlay"
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "underlay");
    assert!(trigger.flags.focused, "expected underlay to be focused");
}
