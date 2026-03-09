use std::collections::HashMap;
use std::sync::Arc;

use fret_core::Rect;

use crate::core::{CanvasPoint, NodeId as GraphNodeId, PortId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DrawOrderFingerprint {
    pub(crate) lo: u64,
    pub(crate) hi: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DerivedBaseKey {
    pub(crate) graph_rev: u64,
    pub(crate) zoom_bits: u32,
    pub(crate) node_origin_x_bits: u32,
    pub(crate) node_origin_y_bits: u32,
    pub(crate) draw_order: DrawOrderFingerprint,
    pub(crate) presenter_rev: u64,
    pub(crate) edge_types_rev: u64,
    pub(crate) overrides_rev: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GeometryCacheKey {
    pub(crate) base: DerivedBaseKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SpatialIndexCacheKey {
    pub(crate) geom: GeometryCacheKey,
    pub(crate) cell_size_screen_bits: u32,
    pub(crate) min_cell_size_screen_bits: u32,
    pub(crate) edge_aabb_pad_screen_bits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InternalsViewKey {
    pub(crate) pan_x_bits: u32,
    pub(crate) pan_y_bits: u32,
    pub(crate) bounds_x_bits: u32,
    pub(crate) bounds_y_bits: u32,
    pub(crate) bounds_w_bits: u32,
    pub(crate) bounds_h_bits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InternalsCacheKey {
    pub(crate) base: DerivedBaseKey,
    pub(crate) view: InternalsViewKey,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DerivedBuildCounters {
    pub(crate) geom_rebuilds: u64,
    pub(crate) index_rebuilds: u64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct GeometryCache {
    pub(crate) geom_key: Option<GeometryCacheKey>,
    pub(crate) index_key: Option<SpatialIndexCacheKey>,
    pub(crate) geom: Arc<super::super::geometry::CanvasGeometry>,
    pub(crate) index: Arc<super::super::spatial::CanvasSpatialDerived>,
    pub(crate) drag_preview: Option<DragPreviewCache>,
    pub(crate) counters: DerivedBuildCounters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragPreviewKind {
    NodeDrag,
    GroupDrag,
    NodeResize,
}

#[derive(Debug, Clone)]
pub(crate) struct DragPreviewCache {
    pub(crate) kind: DragPreviewKind,
    pub(crate) base_index_key: SpatialIndexCacheKey,
    pub(crate) preview_rev: u64,
    pub(crate) geom: Arc<super::super::geometry::CanvasGeometry>,
    pub(crate) index: Arc<super::super::spatial::CanvasSpatialDerived>,
    pub(crate) node_positions: HashMap<GraphNodeId, CanvasPoint>,
    pub(crate) node_rects: HashMap<GraphNodeId, Rect>,
    pub(crate) node_ports: HashMap<GraphNodeId, Vec<PortId>>,
}

pub(crate) struct DragPreviewCacheMetaMut<'a> {
    pub(crate) node_positions: &'a mut HashMap<GraphNodeId, CanvasPoint>,
    pub(crate) node_rects: &'a mut HashMap<GraphNodeId, Rect>,
    pub(crate) node_ports: &'a HashMap<GraphNodeId, Vec<PortId>>,
}
