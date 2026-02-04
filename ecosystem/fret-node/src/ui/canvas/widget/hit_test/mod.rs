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

fn zoom_eps(zoom: f32) -> f32 {
    (1.0e-3 / zoom.max(1.0e-6)).max(1.0e-6)
}
