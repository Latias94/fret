//! Internal drag routing helpers.
//!
//! This module exposes a small, mechanism-only API for routing **internal drag** events to a
//! stable "anchor" node. Higher-level policy layers (e.g. docking tear-off drags) can refresh
//! these routes each frame so cross-window hover/update stays deterministic.
//!
//! The runtime owns no policy for *which* drags should be routed: it only exposes an override
//! table keyed by `(window, drag_kind)`.

use crate::UiHost;
use fret_core::{AppWindowId, NodeId};
use fret_runtime::DragKindId;

pub fn route<H: UiHost>(app: &H, window: AppWindowId, kind: DragKindId) -> Option<NodeId> {
    let routes = app.global::<crate::drag_route::InternalDragRouteService>()?;
    routes.route(window, kind)
}

pub fn set_route<H: UiHost>(app: &mut H, window: AppWindowId, kind: DragKindId, node: NodeId) {
    app.with_global_mut(
        crate::drag_route::InternalDragRouteService::default,
        |routes, _app| {
            routes.set(window, kind, node);
        },
    );
}

pub fn remove_route<H: UiHost>(app: &mut H, window: AppWindowId, kind: DragKindId) {
    app.with_global_mut(
        crate::drag_route::InternalDragRouteService::default,
        |routes, _app| {
            routes.remove(window, kind);
        },
    );
}

pub fn clear_window<H: UiHost>(app: &mut H, window: AppWindowId) {
    app.with_global_mut(
        crate::drag_route::InternalDragRouteService::default,
        |routes, _app| {
            routes.clear_window(window);
        },
    );
}
