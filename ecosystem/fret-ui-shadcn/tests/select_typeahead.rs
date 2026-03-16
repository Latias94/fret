use fret_app::App;
use fret_core::{AppWindowId, FrameId, KeyCode, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;
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
        "select-typeahead",
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
fn select_typeahead_key_selects_matching_item() {
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

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let items: Vec<shadcn::SelectItem> = ["apple", "banana", "blueberry"]
        .into_iter()
        .map(|v| shadcn::SelectItem::new(v, v))
        .collect();

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;
    let mut timers = TimerQueue::default();

    // Frame 1: mount closed and click trigger.
    let value_frame_1 = value.clone();
    let open_frame_1 = open.clone();
    let items_frame_1 = items.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| {
            vec![
                shadcn::Select::new(value_frame_1, open_frame_1)
                    .value(shadcn::SelectValue::new().placeholder("Select"))
                    .a11y_label("Select")
                    .trigger_test_id("select-trigger")
                    .items(items_frame_1)
                    .into_element(cx),
            ]
        },
    );

    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let trigger = find_by_test_id(&snap, "select-trigger");
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

    // Frame 2+: settle open overlays before typeahead.
    let settle_frames = shadcn_motion::ticks_100() + 2;
    for tick in 0..settle_frames {
        let request_semantics = tick + 1 == settle_frames;
        let value_frame = value.clone();
        let open_frame = open.clone();
        let items_frame = items.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            request_semantics,
            move |cx| {
                vec![
                    shadcn::Select::new(value_frame, open_frame)
                        .value(shadcn::SelectValue::new().placeholder("Select"))
                        .a11y_label("Select")
                        .trigger_test_id("select-trigger")
                        .items(items_frame)
                        .into_element(cx),
                ]
            },
        );
    }

    // Typeahead: press 'b' then accept.
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::KeyB);
    // Note: select typeahead can schedule timers (buffer timeout). We intentionally do not
    // deliver those timers in this test; this gate only cares about the selection outcome.
    let value_after_typeahead = value.clone();
    let open_after_typeahead = open.clone();
    let items_after_typeahead = items.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        move |cx| {
            vec![
                shadcn::Select::new(value_after_typeahead, open_after_typeahead)
                    .value(shadcn::SelectValue::new().placeholder("Select"))
                    .a11y_label("Select")
                    .trigger_test_id("select-trigger")
                    .items(items_after_typeahead)
                    .into_element(cx),
            ]
        },
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Enter);
    let value_after_enter = value.clone();
    let open_after_enter = open.clone();
    let items_after_enter = items.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        move |cx| {
            vec![
                shadcn::Select::new(value_after_enter, open_after_enter)
                    .value(shadcn::SelectValue::new().placeholder("Select"))
                    .a11y_label("Select")
                    .trigger_test_id("select-trigger")
                    .items(items_after_enter)
                    .into_element(cx),
            ]
        },
    );

    // Frame after selection commit.
    let value_frame_3 = value.clone();
    let open_frame_3 = open.clone();
    let items_frame_3 = items;
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| {
            vec![
                shadcn::Select::new(value_frame_3, open_frame_3)
                    .value(shadcn::SelectValue::new().placeholder("Select"))
                    .a11y_label("Select")
                    .trigger_test_id("select-trigger")
                    .items(items_frame_3)
                    .into_element(cx),
            ]
        },
    );

    assert_eq!(
        app.models().get_cloned(&value).flatten().as_deref(),
        Some("banana"),
        "expected typeahead 'b' to select the first matching item"
    );
    assert_eq!(
        app.models().get_copied(&open),
        Some(false),
        "expected closed after Enter"
    );
}
