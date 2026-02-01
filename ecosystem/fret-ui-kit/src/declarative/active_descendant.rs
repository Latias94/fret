use fret_core::{NodeId, Point, Px, Rect};
use fret_ui::elements::GlobalElementId;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveOption {
    pub element: GlobalElementId,
    pub node: NodeId,
}

/// Resolve an active descendant `NodeId` from a list of element IDs and an active index.
///
/// This is a small helper for cmdk/listbox-like composite widgets where:
/// - focus stays on an owner node (often a `TextField`), and
/// - the highlighted option is exposed via `active_descendant` (ADR 0073).
pub fn active_descendant_for_index<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    elements: &[GlobalElementId],
    active_index: Option<usize>,
) -> Option<NodeId> {
    active_option_for_index(cx, elements, active_index).map(|opt| opt.node)
}

pub fn active_option_for_index<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    elements: &[GlobalElementId],
    active_index: Option<usize>,
) -> Option<ActiveOption> {
    let element = active_index.and_then(|idx| elements.get(idx).copied())?;
    let node = cx.node_for_element(element)?;
    Some(ActiveOption { element, node })
}

/// Scroll a child rectangle into view within a viewport, using the same conservative behavior as
/// the runtime's focus-traversal scroll-into-view contract.
pub fn scroll_handle_into_view_y(handle: &ScrollHandle, viewport: Rect, child: Rect) -> bool {
    let viewport_h = viewport.size.height.0.max(0.0);
    if viewport_h <= 0.0 {
        return false;
    }

    let view_top = viewport.origin.y.0;
    let view_bottom = view_top + viewport_h;
    let child_top = child.origin.y.0;
    let child_h = child.size.height.0.max(0.0);
    let child_bottom = child_top + child_h;

    // If the child is taller than the viewport, we cannot make it fully visible. Match the
    // common DOM `scrollIntoView({ block: "nearest" })` outcome: only ensure the top edge is
    // visible, and avoid runaway scrolling that would try to "fit" the bottom edge.
    if child_h >= viewport_h - 0.01 {
        let delta = child_top - view_top;
        if delta.abs() <= 0.01 {
            return false;
        }

        let prev = handle.offset();
        handle.set_offset(Point::new(prev.x, Px(prev.y.0 + delta)));

        let next = handle.offset();
        return (prev.y.0 - next.y.0).abs() > 0.01;
    }

    let delta = if child_top < view_top {
        child_top - view_top
    } else if child_bottom > view_bottom {
        child_bottom - view_bottom
    } else {
        0.0
    };

    if delta.abs() <= 0.01 {
        return false;
    }

    let prev = handle.offset();
    handle.set_offset(Point::new(prev.x, Px(prev.y.0 + delta)));

    let next = handle.offset();
    (prev.y.0 - next.y.0).abs() > 0.01
}

/// Align a child rectangle's top edge with the viewport top edge by adjusting the scroll handle.
///
/// This is intentionally stronger than [`scroll_handle_into_view_y`]: it scrolls even when the
/// child is already visible.
pub fn scroll_handle_align_top_y(handle: &ScrollHandle, viewport: Rect, child: Rect) -> bool {
    let viewport_h = viewport.size.height.0.max(0.0);
    if viewport_h <= 0.0 {
        return false;
    }

    let view_top = viewport.origin.y.0;
    let child_top = child.origin.y.0;
    let delta = child_top - view_top;

    if delta.abs() <= 0.01 {
        return false;
    }

    let prev = handle.offset();
    handle.set_offset(Point::new(prev.x, Px(prev.y.0 + delta)));

    let next = handle.offset();
    (prev.y.0 - next.y.0).abs() > 0.01
}

/// Best-effort "scroll active option into view" helper for cmdk/listbox-like widgets.
///
/// This uses last-frame bounds for both the scroll viewport element and the active item element.
/// When either bound is missing, it does nothing.
pub fn scroll_active_element_into_view_y<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    handle: &ScrollHandle,
    viewport_element: GlobalElementId,
    active_element: GlobalElementId,
) -> bool {
    let Some(viewport) = cx.last_bounds_for_element(viewport_element) else {
        return false;
    };
    let Some(child) = cx.last_bounds_for_element(active_element) else {
        return false;
    };

    scroll_handle_into_view_y(handle, viewport, child)
}

/// Best-effort "scroll active option to top edge" helper for cmdk/listbox-like widgets.
///
/// This uses last-frame bounds for both the scroll viewport element and the active item element.
/// When either bound is missing, it does nothing.
pub fn scroll_active_element_align_top_y<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    handle: &ScrollHandle,
    viewport_element: GlobalElementId,
    active_element: GlobalElementId,
) -> bool {
    let Some(viewport) = cx.last_bounds_for_element(viewport_element) else {
        return false;
    };
    let Some(child) = cx.last_bounds_for_element(active_element) else {
        return false;
    };

    scroll_handle_align_top_y(handle, viewport, child)
}
