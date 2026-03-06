//! Viewport helper API (UI-only).
//!
//! XyFlow exposes a "viewport helper" surface via hooks (`useReactFlow`) that lets apps drive the
//! viewport without directly referencing a specific widget instance. In Fret this is expressed as
//! a small wrapper around the `NodeGraphViewQueue` + persisted `NodeGraphViewState` models.

use fret_core::Rect;
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::core::{CanvasPoint, NodeId};
use crate::io::NodeGraphViewState;

use super::view_queue::{NodeGraphFitViewOptions, NodeGraphSetViewportOptions, NodeGraphViewQueue};

#[derive(Debug, Clone)]
pub struct NodeGraphViewportHelper {
    view_state: Model<NodeGraphViewState>,
    view_queue: Model<NodeGraphViewQueue>,
}

impl NodeGraphViewportHelper {
    pub fn new(
        view_state: Model<NodeGraphViewState>,
        view_queue: Model<NodeGraphViewQueue>,
    ) -> Self {
        Self {
            view_state,
            view_queue,
        }
    }

    pub fn viewport<H: UiHost>(&self, host: &H) -> (CanvasPoint, f32) {
        self.view_state
            .read_ref(host, |s| (s.pan, s.zoom))
            .ok()
            .unwrap_or((CanvasPoint::default(), 1.0))
    }

    pub fn set_viewport<H: UiHost>(&self, host: &mut H, pan: CanvasPoint, zoom: f32) {
        let _ = self.view_queue.update(host, |q, _cx| {
            q.push_set_viewport(pan, zoom);
        });
    }

    pub fn set_viewport_with_options<H: UiHost>(
        &self,
        host: &mut H,
        pan: CanvasPoint,
        zoom: f32,
        options: NodeGraphSetViewportOptions,
    ) {
        let _ = self.view_queue.update(host, |q, _cx| {
            q.push_set_viewport_with_options(pan, zoom, options);
        });
    }

    pub fn fit_view_nodes<H: UiHost>(&self, host: &mut H, nodes: Vec<NodeId>) {
        let _ = self.view_queue.update(host, |q, _cx| {
            q.push_frame_nodes(nodes);
        });
    }

    pub fn fit_view_nodes_with_options<H: UiHost>(
        &self,
        host: &mut H,
        nodes: Vec<NodeId>,
        options: NodeGraphFitViewOptions,
    ) {
        let _ = self.view_queue.update(host, |q, _cx| {
            q.push_frame_nodes_with_options(nodes, options);
        });
    }

    pub fn set_center_in_bounds<H: UiHost>(&self, host: &mut H, bounds: Rect, center: CanvasPoint) {
        self.set_center_in_bounds_with_options(
            host,
            bounds,
            center,
            None,
            NodeGraphSetViewportOptions::default(),
        );
    }

    pub fn set_center_in_bounds_with_options<H: UiHost>(
        &self,
        host: &mut H,
        bounds: Rect,
        center: CanvasPoint,
        zoom: Option<f32>,
        options: NodeGraphSetViewportOptions,
    ) {
        let current = self.viewport(&*host);
        let zoom = zoom.unwrap_or(current.1);
        let pan = pan_for_center(bounds, center, zoom);
        self.set_viewport_with_options(host, pan, zoom, options);
    }
}

pub(super) fn pan_for_center(bounds: Rect, center: CanvasPoint, zoom: f32) -> CanvasPoint {
    let z = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    let w = bounds.size.width.0;
    let h = bounds.size.height.0;

    CanvasPoint {
        x: w / (2.0 * z) - center.x,
        y: h / (2.0 * z) - center.y,
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px, Rect, Size};

    use super::pan_for_center;
    use crate::core::CanvasPoint;

    #[test]
    fn pan_for_center_aligns_canvas_point_to_window_center() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let center = CanvasPoint { x: 10.0, y: 20.0 };
        let pan = pan_for_center(bounds, center, 2.0);

        assert!((pan.x - (800.0 / 4.0 - 10.0)).abs() <= 1.0e-6);
        assert!((pan.y - (600.0 / 4.0 - 20.0)).abs() <= 1.0e-6);
    }
}
