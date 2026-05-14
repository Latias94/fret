use super::*;

#[test]
fn ui_writer_imui_facade_ext_compiles() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-ui-writer-facade-ext",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui_writer_imui_facade_ext_smoke(ui);
            })
        },
    );

    assert_eq!(ui.children(root).len(), 3);
}

#[test]
fn ui_kit_builder_can_be_rendered_from_imui() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-ui-kit-bridge",
        |cx| {
            crate::imui_raw(cx, |ui| {
                use fret_ui_kit::imui::UiWriterUiKitExt as _;

                let builder = fret_ui_kit::ui::text("Hello").text_sm();
                ui.add_ui(builder);
            })
        },
    );

    assert_eq!(ui.children(root).len(), 1);
}

#[test]
fn imui_default_mounts_with_stacked_host() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-default-stacked-host",
        |cx| {
            crate::imui(cx, |ui| {
                ui.menu_item_with_options(
                    "First",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-default.first")),
                        ..Default::default()
                    },
                );
                ui.menu_item_with_options(
                    "Second",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-default.second")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    assert_eq!(ui.children(root).len(), 1);

    let host = ui.children(root)[0];
    assert_eq!(ui.children(host).len(), 2);

    let host_bounds = ui.debug_node_bounds(host).expect("host bounds");
    assert_eq!(host_bounds.size.width, bounds.size.width);
    assert_eq!(host_bounds.size.height, bounds.size.height);

    let first = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-default.first",
    );
    let second = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-default.second",
    );
    assert!(second.y.0 > first.y.0);
}

#[test]
fn imui_default_mount_paints_text_on_top_of_control_chrome() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-default-text-paint-order",
        |cx| {
            crate::imui(cx, |ui| {
                ui.text("Count: 0");
                ui.button("Increment");
            })
        },
    );

    services.prepared.clear();
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let ops = scene.ops();
    let text_count = ops
        .iter()
        .filter(|op| matches!(op, fret_core::SceneOp::Text { .. }))
        .count();
    assert!(
        text_count >= 2,
        "expected IMUI text and button label text to paint, got scene ops: {ops:?}"
    );

    let first_chrome = ops
        .iter()
        .position(|op| matches!(op, fret_core::SceneOp::Quad { .. }))
        .expect("expected button chrome quad to paint");
    assert!(
        ops.iter()
            .skip(first_chrome + 1)
            .any(|op| matches!(op, fret_core::SceneOp::Text { .. })),
        "expected button label text to paint after control chrome, got scene ops: {ops:?}"
    );
}

#[test]
fn imui_raw_preserves_direct_sibling_emission() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-raw-direct-siblings",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.menu_item_with_options(
                    "First",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-raw.first")),
                        ..Default::default()
                    },
                );
                ui.menu_item_with_options(
                    "Second",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-raw.second")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    assert_eq!(ui.children(root).len(), 2);
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-raw.first",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-raw.second",
    ));
}
