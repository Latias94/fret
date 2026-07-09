use super::*;

#[allow(dead_code)]
pub(super) fn render_code_editor_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut FakeServices,
    window: AppWindowId,
    handle: CodeEditorHandle,
    bounds: Rect,
) -> fret_core::Scene {
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "code-editor-frame",
        |cx| {
            vec![
                CodeEditor::new(handle.clone())
                    .key(0)
                    .overscan(8)
                    .into_element(cx),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(app, services, bounds, 1.0);

    let mut scene = fret_core::Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
    scene
}

pub(super) fn editor_ui_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(360.0)),
    )
}

pub(super) fn render_editor_scroll_audit_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut FakeServices,
    window: AppWindowId,
    handle: CodeEditorHandle,
) {
    let bounds = editor_ui_bounds();
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "code-editor-scroll-audit",
        |cx| {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.size.width = fret_ui::element::Length::Fill;
            layout.size.height = fret_ui::element::Length::Px(Px(180.0));
            vec![cx.container(
                fret_ui::element::ContainerProps {
                    layout,
                    ..Default::default()
                },
                |cx| {
                    vec![
                        CodeEditor::new(handle.clone())
                            .key(0)
                            .overscan(8)
                            .viewport_test_id("code-editor-scroll-audit-viewport")
                            .into_element(cx),
                    ]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

pub(super) fn node_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    test_id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .unwrap_or_else(|| panic!("missing semantics test_id={test_id}"))
}

pub(super) fn bounds_by_test_id(
    ui: &UiTree<App>,
    snap: &fret_core::SemanticsSnapshot,
    test_id: &str,
) -> Rect {
    let node = node_by_test_id(snap, test_id);
    ui.debug_node_visual_bounds(node.id)
        .or_else(|| ui.debug_node_bounds(node.id))
        .unwrap_or(node.bounds)
}

pub(super) fn center_of(bounds: Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

pub(super) fn windowed_rows_telemetry(
    app: &App,
    window: AppWindowId,
) -> WindowedRowsSurfaceWindowTelemetry {
    let store = app
        .global::<WindowedRowsSurfaceDiagnosticsStore>()
        .expect("windowed rows diagnostics store");
    let windows = store
        .windows_for_window(window, app.frame_id())
        .expect("windowed rows telemetry for frame");
    assert_eq!(
        windows.len(),
        1,
        "expected one windowed rows surface telemetry entry"
    );
    windows[0]
}

pub(super) fn row_geom_key_for_tests(text: &Arc<str>) -> geom::RowGeomKey {
    geom::RowGeomKey::for_plain(
        text,
        &TextStyle::default(),
        (
            None,
            TextWrap::None,
            TextOverflow::Clip,
            fret_core::TextAlign::Start,
            1.0,
        ),
        fret_runtime::TextFontStackKey(0),
    )
}
