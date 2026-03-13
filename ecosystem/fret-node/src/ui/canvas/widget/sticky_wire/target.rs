use std::sync::Arc;

use fret_core::Point;
use fret_ui::UiHost;

use crate::core::PortId;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{
    HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith,
};
use crate::ui::canvas::{CanvasGeometry, CanvasSpatialDerived};

pub(super) fn sticky_wire_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> (
    Arc<CanvasGeometry>,
    Arc<CanvasSpatialDerived>,
    Option<PortId>,
) {
    let (geom, index) = canvas.canvas_derived(&*host, snapshot);
    let mut scratch = HitTestScratch::default();
    let mut hit_test = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
    let hit_port = canvas.hit_port(&mut hit_test, position);
    let target = super::super::sticky_wire_connect::connectable_sticky_wire_target(
        canvas, host, snapshot, hit_port,
    );
    (geom, index, target)
}
