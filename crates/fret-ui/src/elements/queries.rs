use fret_core::{AppWindowId, NodeId, Rect};

use super::with_window_state;
use super::{ElementContext, ElementRuntime, GlobalElementId};
use crate::UiHost;

pub fn with_element_cx<H: UiHost, R>(
    app: &mut H,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    app.with_global_mut_untracked(ElementRuntime::new, |runtime, app| {
        let mut cx = ElementContext::new_for_root_name(app, runtime, window, bounds, root_name);
        f(&mut cx)
    })
}

pub fn root_bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<Rect> {
    app.with_global_mut_untracked(ElementRuntime::new, |runtime, _app| {
        let state = runtime.for_window_mut(window);
        let root = state.node_entry(element).map(|e| e.root)?;
        state.root_bounds(root)
    })
}

pub fn node_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<NodeId> {
    with_window_state(app, window, |st| st.node_entry(element).map(|e| e.node))
}

/// Returns the most recent `NodeId` mapping for `element` without preparing element runtime for the
/// current frame.
///
/// This is intended for callers that run *during* tree construction (e.g. overlay policy hooks)
/// where calling `prepare_window_for_frame` can reset `*_next` state.
///
/// Prefer `node_for_element` for normal post-frame queries.
pub fn peek_node_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<NodeId> {
    app.with_global_mut_untracked(ElementRuntime::new, |runtime, _app| {
        let state = runtime.for_window(window)?;
        state.node_entry(element).map(|e| e.node)
    })
}

/// Returns whether `element` is known to be mounted in the **current frame**.
///
/// `node_for_element` may return a stale mapping by design (cross-frame queries). For policies
/// that need a liveness gate (e.g. cached overlay request synthesis), prefer this check.
pub fn element_is_live_in_current_frame<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> bool {
    let frame_id = app.frame_id();
    with_window_state(app, window, |st| {
        st.node_entry(element)
            .map(|e| e.last_seen_frame == frame_id)
            .unwrap_or(false)
    })
}

/// Returns the most recent recorded bounds for a declarative element, if available.
///
/// This is a cross-frame geometry query intended for component-layer policies (e.g. anchored
/// overlays) that need a stable anchor rect. The value is updated during layout.
pub fn bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<Rect> {
    with_window_state(app, window, |st| {
        st.last_bounds(element)
            .or_else(|| st.current_bounds(element))
    })
}

/// Returns the most recent recorded **visual** bounds (post-`render_transform` AABB) for a
/// declarative element, if available.
///
/// This is a cross-frame geometry query intended for component-layer anchored overlay policies
/// that must track render transforms (ADR 0083) while keeping layout authoritative.
pub fn visual_bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<Rect> {
    with_window_state(app, window, |st| {
        st.last_visual_bounds(element)
            .or_else(|| st.current_visual_bounds(element))
    })
}

pub(crate) fn record_bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
    bounds: Rect,
) {
    with_window_state(app, window, |st| st.record_bounds(element, bounds));
}

pub(crate) fn record_visual_bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
    bounds: Rect,
) {
    with_window_state(app, window, |st| st.record_visual_bounds(element, bounds));
}
