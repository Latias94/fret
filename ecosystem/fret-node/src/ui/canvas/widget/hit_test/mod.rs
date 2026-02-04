use super::*;

mod edges;
mod focus_anchor;
mod geom;
mod ports;
mod score;

#[derive(Debug, Default, Clone)]
pub(super) struct HitTestScratch {
    edges: Vec<EdgeId>,
    ports: Vec<PortId>,
}

impl HitTestScratch {
    pub(super) fn edges_mut(&mut self) -> &mut Vec<EdgeId> {
        &mut self.edges
    }

    pub(super) fn ports_mut(&mut self) -> &mut Vec<PortId> {
        &mut self.ports
    }
}

#[derive(Debug)]
pub(super) struct HitTestCtx<'a> {
    pub(super) geom: &'a CanvasGeometry,
    pub(super) index: &'a CanvasSpatialIndex,
    pub(super) zoom: f32,
    pub(super) scratch: &'a mut HitTestScratch,
}

impl<'a> HitTestCtx<'a> {
    pub(super) fn new(
        geom: &'a CanvasGeometry,
        index: &'a CanvasSpatialIndex,
        zoom: f32,
        scratch: &'a mut HitTestScratch,
    ) -> Self {
        Self {
            geom,
            index,
            zoom,
            scratch,
        }
    }
}

fn zoom_z(zoom: f32) -> f32 {
    if zoom.is_finite() && zoom > 0.0 {
        zoom.max(1.0e-6)
    } else {
        1.0
    }
}

fn zoom_eps(zoom: f32) -> f32 {
    (1.0e-3 / zoom_z(zoom)).max(1.0e-6)
}

fn hit_test_canvas_units_from_screen_px(screen_px: f32, zoom: f32) -> f32 {
    let z = zoom_z(zoom);
    let v = canvas_units_from_screen_px(screen_px, z);
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}
