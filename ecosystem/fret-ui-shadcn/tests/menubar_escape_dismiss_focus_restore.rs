use fret_app::App;
use fret_core::{AppWindowId, FrameId, KeyCode, Point, Px, Rect, Size as CoreSize};
use fret_runtime::CommandId;
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
        "menubar-escape-dismiss-focus-restore",
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

fn find_optional_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    id: &str,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes.iter().find(|n| n.test_id.as_deref() == Some(id))
}

#[test]
fn menubar_escape_closes_and_restores_focus_to_trigger() {
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
    let mut timers = TimerQueue::default();

    let build = move |cx: &mut ElementContext<'_, App>| {
        let file_menu = shadcn::MenubarMenu::new("File")
            .test_id("menubar-trigger")
            .entries(vec![
                shadcn::MenubarEntry::Item(
                    shadcn::MenubarItem::new("New")
                        .test_id("menubar-item-new")
                        .on_select(CommandId::from("edit.copy")),
                ),
                shadcn::MenubarEntry::Item(
                    shadcn::MenubarItem::new("Open")
                        .test_id("menubar-item-open")
                        .on_select(CommandId::from("edit.paste")),
                ),
            ]);
        vec![shadcn::Menubar::new([file_menu]).into_element(cx)]
    };

    // Frame 1: mount closed and click trigger.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        build,
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "menubar-trigger");
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
            build,
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "menubar-trigger");
    assert!(
        trigger.flags.expanded,
        "expected menubar trigger to be expanded after click"
    );
    assert!(
        has_test_id(&snap, "menubar-item-new"),
        "expected menu items to be present after open"
    );
    let new_item = find_optional_by_test_id(&snap, "menubar-item-new").expect("new item");
    assert!(
        !new_item.flags.hidden,
        "expected menu items to be visible after open"
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Escape);
    timers.ingest_effects(&mut app);
    timers.fire_all(&mut ui, &mut app, &mut services);

    // Frames after dismissal: allow close transition to settle before asserting semantics.
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
            build,
        );
    }

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "menubar-trigger");
    assert!(
        !trigger.flags.expanded,
        "expected trigger to collapse after Escape"
    );
    assert!(
        trigger.flags.focused,
        "expected focus to restore to trigger"
    );

    let new_item = find_optional_by_test_id(&snap, "menubar-item-new");
    assert!(
        new_item.is_none_or(|n| n.flags.hidden),
        "expected menu items to be absent or hidden after Escape"
    );
}
