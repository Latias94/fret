use fret_canvas::view::{PanZoomConstraints2D, clamp_pan_zoom_view};
use fret_core::{Point, Px, Rect, Size};

use crate::core::CanvasPoint;
use crate::ui::canvas::state::ViewSnapshot;

use super::super::{CanvasViewport2D, PanZoom2D};

pub(in super::super) fn viewport_from_pan_zoom(
    bounds: Rect,
    pan: CanvasPoint,
    zoom: f32,
) -> CanvasViewport2D {
    CanvasViewport2D::new(
        bounds,
        PanZoom2D {
            pan: Point::new(Px(pan.x), Px(pan.y)),
            zoom,
        },
    )
}

pub(in super::super) fn viewport_from_snapshot(
    bounds: Rect,
    snapshot: &ViewSnapshot,
) -> CanvasViewport2D {
    viewport_from_pan_zoom(bounds, snapshot.pan, snapshot.zoom)
}

pub(in super::super) fn screen_to_canvas(
    bounds: Rect,
    screen: Point,
    pan: CanvasPoint,
    zoom: f32,
) -> CanvasPoint {
    let viewport = viewport_from_pan_zoom(bounds, pan, zoom);
    let point = viewport.screen_to_canvas(screen);
    CanvasPoint {
        x: point.x.0,
        y: point.y.0,
    }
}

pub(in super::super) fn clamp_pan_to_translate_extent(
    pan: CanvasPoint,
    zoom: f32,
    bounds: Rect,
    extent: crate::core::CanvasRect,
) -> CanvasPoint {
    let extent_rect = Rect::new(
        Point::new(Px(extent.origin.x), Px(extent.origin.y)),
        Size::new(Px(extent.size.width), Px(extent.size.height)),
    );

    let view = clamp_pan_zoom_view(
        bounds,
        PanZoom2D {
            pan: Point::new(Px(pan.x), Px(pan.y)),
            zoom,
        },
        PanZoomConstraints2D {
            min_zoom: zoom,
            max_zoom: zoom,
            translate_extent_canvas: Some(extent_rect),
        },
    );

    CanvasPoint {
        x: view.pan.x.0,
        y: view.pan.y.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::NodeGraphInteractionState;
    use fret_core::{Point, Rect, Size};

    #[test]
    fn viewport_from_snapshot_matches_pan_zoom_constructor() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(300.0), Px(200.0)),
        );
        let snapshot = ViewSnapshot {
            pan: CanvasPoint { x: 12.0, y: -8.0 },
            zoom: 1.5,
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            selected_groups: Vec::new(),
            draw_order: Vec::new(),
            group_draw_order: Vec::new(),
            interaction: NodeGraphInteractionState::default(),
        };

        assert_eq!(
            viewport_from_snapshot(bounds, &snapshot),
            viewport_from_pan_zoom(bounds, snapshot.pan, snapshot.zoom)
        );
    }
}
