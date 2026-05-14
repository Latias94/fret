use super::*;

pub(crate) fn first_child_point(ui: &UiTree<TestHost>, root: fret_core::NodeId) -> Point {
    let child = ui.children(root)[0];
    let bounds = ui.debug_node_bounds(child).expect("child bounds");
    Point::new(Px(bounds.origin.x.0 + 1.0), Px(bounds.origin.y.0 + 1.0))
}

pub(crate) fn bounds_for_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Rect {
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(test_id))
        .unwrap_or_else(|| panic!("expected node with test_id={test_id}"));
    node.bounds
}

pub(crate) fn ui_writer_imui_facade_ext_smoke<H: fret_ui::UiHost>(
    ui: &mut impl fret_authoring::UiWriter<H>,
) {
    use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;

    ui.text("Hello");
    ui.separator();
    let _ = ui.button("OK");
}

pub(crate) fn floating_window_nodes(
    ui: &UiTree<TestHost>,
    root: fret_core::NodeId,
) -> (fret_core::NodeId, fret_core::NodeId) {
    let window = ui.children(root)[0];
    let chrome = ui.children(window)[0];
    let col = ui.children(chrome)[0];
    let title_bar = ui.children(col)[0];
    (window, title_bar)
}

pub(crate) fn point_for_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
    test_id: &str,
) -> Point {
    let node = node_for_test_id(ui, app, services, bounds, test_id);
    let bounds = ui.debug_node_bounds(node).expect("node bounds");
    Point::new(Px(bounds.origin.x.0 + 1.0), Px(bounds.origin.y.0 + 1.0))
}

pub(crate) fn node_for_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
    test_id: &str,
) -> fret_core::NodeId {
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(test_id))
        .unwrap_or_else(|| panic!("expected semantics node with test_id {test_id:?}"))
        .id
}

pub(crate) fn focus_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
    test_id: &str,
) -> fret_core::NodeId {
    let node = node_for_test_id(ui, app, services, bounds, test_id);
    ui.set_focus(Some(node));
    assert_eq!(ui.focus(), Some(node));
    node
}

pub(crate) fn has_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
    test_id: &str,
) -> bool {
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    ui.semantics_snapshot()
        .expect("semantics snapshot")
        .nodes
        .iter()
        .any(|n| n.test_id.as_deref() == Some(test_id))
}
