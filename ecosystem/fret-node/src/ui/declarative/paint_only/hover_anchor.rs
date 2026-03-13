use fret_canvas::view::PanZoom2D;
use fret_core::{Point, Px, Rect};
use fret_runtime::Model;

use super::{
    NodeDragState, NodeRectDraw, node_drag_contains, node_drag_delta_canvas, rect_approx_eq,
};

#[derive(Debug, Default, Clone)]
pub(super) struct HoverAnchorStore {
    /// Last-known hovered node id (paint-only).
    pub(super) hovered_id: Option<crate::core::NodeId>,
    /// Best-effort hovered node bounds in canvas space.
    ///
    /// This is independent of portal hosting caps so hover-driven overlays remain stable even when
    /// portals are throttled.
    pub(super) hovered_canvas_bounds: Option<Rect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HoverTooltipAnchorSource {
    PortalBoundsStore,
    HoverAnchorStore,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct HoverTooltipAnchor {
    pub(super) origin_screen: Point,
    pub(super) width_screen: Px,
    pub(super) source: HoverTooltipAnchorSource,
}

pub(super) fn resolve_hover_tooltip_anchor(
    bounds: Rect,
    view: PanZoom2D,
    portals_disabled: bool,
    portal_canvas_bounds: Option<Rect>,
    hover_anchor_canvas_bounds: Option<Rect>,
) -> Option<HoverTooltipAnchor> {
    if !bounds.size.width.0.is_finite()
        || !bounds.size.height.0.is_finite()
        || bounds.size.width.0 <= 0.0
        || bounds.size.height.0 <= 0.0
    {
        return None;
    }

    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let candidate = if !portals_disabled {
        portal_canvas_bounds.map(|rect| (rect, HoverTooltipAnchorSource::PortalBoundsStore))
    } else {
        None
    }
    .or_else(|| {
        hover_anchor_canvas_bounds.map(|rect| (rect, HoverTooltipAnchorSource::HoverAnchorStore))
    })?;

    let (canvas_rect, source) = candidate;
    let origin_screen = view.canvas_to_screen(bounds, canvas_rect.origin);
    let width_screen = Px((canvas_rect.size.width.0 * zoom).max(0.0));
    if !origin_screen.x.0.is_finite()
        || !origin_screen.y.0.is_finite()
        || !width_screen.0.is_finite()
        || width_screen.0 <= 0.0
    {
        return None;
    }

    Some(HoverTooltipAnchor {
        origin_screen,
        width_screen,
        source,
    })
}

pub(super) fn hovered_canvas_anchor_rect_for_surface(
    hovered_id: crate::core::NodeId,
    draws: Option<&[NodeRectDraw]>,
    view: PanZoom2D,
    node_drag: Option<&NodeDragState>,
) -> Option<Rect> {
    let draw = draws?.iter().find(|draw| draw.id == hovered_id)?;
    let mut rect = draw.rect;
    let drag_active = node_drag.is_some_and(NodeDragState::is_active);
    if drag_active && node_drag.is_some_and(|drag| node_drag_contains(drag, hovered_id)) {
        let drag = node_drag.expect("checked active node drag");
        let (ddx, ddy) = node_drag_delta_canvas(view, drag);
        rect.origin = Point::new(Px(rect.origin.x.0 + ddx), Px(rect.origin.y.0 + ddy));
    }
    Some(rect)
}

pub(super) fn sync_hover_anchor_store_in_models(
    models: &mut fret_runtime::ModelStore,
    hover_anchor_store: &Model<HoverAnchorStore>,
    hovered_id: Option<crate::core::NodeId>,
    draws: Option<&[NodeRectDraw]>,
    view: PanZoom2D,
    node_drag: Option<&NodeDragState>,
) -> bool {
    if let Some(hovered_id) = hovered_id {
        let Some(rect) = hovered_canvas_anchor_rect_for_surface(hovered_id, draws, view, node_drag)
        else {
            return false;
        };

        let should_update = models
            .read(hover_anchor_store, |st| {
                if st.hovered_id != Some(hovered_id) {
                    return true;
                }
                let Some(prev) = st.hovered_canvas_bounds else {
                    return true;
                };
                !rect_approx_eq(prev, rect, 0.25)
            })
            .unwrap_or(true);
        if !should_update {
            return false;
        }

        let _ = models.update(hover_anchor_store, |st| {
            st.hovered_id = Some(hovered_id);
            st.hovered_canvas_bounds = Some(rect);
        });
        return true;
    }

    let should_clear = models
        .read(hover_anchor_store, |st| st.hovered_id.is_some())
        .unwrap_or(false);
    if !should_clear {
        return false;
    }

    let _ = models.update(hover_anchor_store, |st| {
        st.hovered_id = None;
        st.hovered_canvas_bounds = None;
    });
    true
}
