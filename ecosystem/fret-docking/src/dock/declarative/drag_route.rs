use fret_core::{AppWindowId, NodeId};
use fret_runtime::{DRAG_KIND_DOCK_PANEL, DRAG_KIND_DOCK_TABS, DragKindId};
use fret_ui::UiHost;

use super::super::manager::DockManager;

pub(super) fn keep_internal_drag_route_alive<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    host_node: NodeId,
) {
    fret_ui::internal_drag::set_route(app, window, DRAG_KIND_DOCK_PANEL, host_node);
    fret_ui::internal_drag::set_route(app, window, DRAG_KIND_DOCK_TABS, host_node);
    if app.global::<DockManager>().is_some() {
        app.with_global_mut_untracked(DockManager::default, |dock, _app| {
            dock.register_dock_space_node(window, host_node);
        });
    }
}

pub(super) fn dock_dragging_affects_window<H: UiHost>(app: &H, window: AppWindowId) -> bool {
    app.any_drag_session(|drag| {
        is_dock_drag_kind(drag.kind)
            && (drag.source_window == window || drag.current_window == window)
            && drag.dragging
    })
}

pub(super) fn is_dock_drag_kind(kind: DragKindId) -> bool {
    kind == DRAG_KIND_DOCK_PANEL || kind == DRAG_KIND_DOCK_TABS
}
