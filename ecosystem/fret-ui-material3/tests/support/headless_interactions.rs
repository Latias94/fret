use fret_core::{KeyCode, NodeId, Point, PointerId, Px, Rect, UiServices};
use fret_ui::{UiHost, UiTree};

use super::events::{key_down, key_up, pointer_move};

#[derive(Debug, Clone, Copy)]
pub(crate) struct HeadlessHoverTarget {
    pub(crate) node_id: NodeId,
    pub(crate) bounds: Rect,
    pub(crate) center: Point,
}

pub(crate) fn node_id_by_test_id<H: UiHost>(
    ui: &UiTree<H>,
    test_id: &str,
    context: &str,
) -> NodeId {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find_map(|node| (node.test_id.as_deref() == Some(test_id)).then_some(node.id))
        })
        .unwrap_or_else(|| panic!("expected {test_id} in semantics snapshot ({context})"))
}

pub(crate) fn hover_test_id<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn UiServices,
    test_id: &str,
    context: &str,
) -> HeadlessHoverTarget {
    let node_id = node_id_by_test_id(ui, test_id, context);
    let bounds = ui
        .debug_node_visual_bounds(node_id)
        .unwrap_or_else(|| panic!("expected {test_id} bounds ({context})"));
    let center = Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(app, services, &pointer_move(PointerId(1), center));
    HeadlessHoverTarget {
        node_id,
        bounds,
        center,
    }
}

pub(crate) fn focus_test_id<H: UiHost>(ui: &mut UiTree<H>, test_id: &str, context: &str) -> NodeId {
    let node_id = node_id_by_test_id(ui, test_id, context);
    ui.set_focus(Some(node_id));
    node_id
}

pub(crate) fn dispatch_idle_pointer<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn UiServices,
) {
    ui.dispatch_event(
        app,
        services,
        &pointer_move(PointerId(1), Point::new(Px(1.0), Px(1.0))),
    );
}

pub(crate) fn dispatch_key_tap<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn UiServices,
    key: KeyCode,
) {
    ui.dispatch_event(app, services, &key_down(key));
    ui.dispatch_event(app, services, &key_up(key));
}
