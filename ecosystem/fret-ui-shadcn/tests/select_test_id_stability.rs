use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

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
        "select-test-id-stability",
        root,
    );
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn assert_has_test_id(snap: &fret_core::SemanticsSnapshot, id: &str) {
    let found = snap.nodes.iter().any(|n| n.test_id.as_deref() == Some(id));
    assert!(found, "missing semantics node with test_id={id:?}");
}

#[test]
fn select_test_ids_survive_open_close_open() {
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

    let value: Model<Option<Arc<str>>> = app.models_mut().insert(None);
    let open: Model<bool> = app.models_mut().insert(false);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let items: Vec<fret_ui_shadcn::SelectItem> = (0..40)
        .map(|i| fret_ui_shadcn::SelectItem::new(format!("item-{i}"), format!("Item {i}")))
        .collect();

    // Frame 1: mount closed (stable trigger).
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
                fret_ui_shadcn::Select::new(value_frame_1.clone(), open_frame_1.clone())
                    .placeholder("Select an item")
                    .a11y_label("Select")
                    .trigger_test_id("select-trigger")
                    .items(items_frame_1.clone())
                    .into_element(cx),
            ]
        },
    );
    {
        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert_has_test_id(&snap, "select-trigger");
    }

    // Frame 2: open.
    let _ = app.models_mut().update(&open, |v| *v = true);
    let value_frame_2 = value.clone();
    let open_frame_2 = open.clone();
    let items_frame_2 = items.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| {
            vec![
                fret_ui_shadcn::Select::new(value_frame_2.clone(), open_frame_2.clone())
                    .placeholder("Select an item")
                    .a11y_label("Select")
                    .trigger_test_id("select-trigger")
                    .items(items_frame_2.clone())
                    .into_element(cx),
            ]
        },
    );
    {
        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert_has_test_id(&snap, "select-trigger");
        assert_has_test_id(&snap, "select-scroll-viewport");
    }

    // Frame 3: close.
    let _ = app.models_mut().update(&open, |v| *v = false);
    let value_frame_3 = value.clone();
    let open_frame_3 = open.clone();
    let items_frame_3 = items.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| {
            vec![
                fret_ui_shadcn::Select::new(value_frame_3.clone(), open_frame_3.clone())
                    .placeholder("Select an item")
                    .a11y_label("Select")
                    .trigger_test_id("select-trigger")
                    .items(items_frame_3.clone())
                    .into_element(cx),
            ]
        },
    );
    {
        let snap = ui
            .semantics_snapshot()
            .cloned()
            .expect("expected semantics snapshot");
        assert_has_test_id(&snap, "select-trigger");
    }

    // Frame 4: open again, viewport id still present.
    let _ = app.models_mut().update(&open, |v| *v = true);
    let value_frame_4 = value.clone();
    let open_frame_4 = open.clone();
    let items_frame_4 = items;
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| {
            vec![
                fret_ui_shadcn::Select::new(value_frame_4.clone(), open_frame_4.clone())
                    .placeholder("Select an item")
                    .a11y_label("Select")
                    .trigger_test_id("select-trigger")
                    .items(items_frame_4.clone())
                    .into_element(cx),
            ]
        },
    );
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert_has_test_id(&snap, "select-trigger");
    assert_has_test_id(&snap, "select-scroll-viewport");
}
