use fret_core::ImageId;

use super::super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};

pub(super) fn image_triangle_mesh_summary(
    image: ImageId,
    vertex_count: usize,
    index_count: usize,
) -> DebugDrawCommandSummary {
    let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::ImageTriangleMesh);
    summary.image = Some(image);
    summary.vertex_count = vertex_count;
    summary.index_count = index_count;
    summary.triangle_count = index_count / 3;
    summary
}

pub(super) fn image_rect_summary(
    kind: DebugDrawCommandKind,
    image: ImageId,
) -> DebugDrawCommandSummary {
    let mut summary = DebugDrawCommandSummary::new(kind);
    summary.image = Some(image);
    summary.point_count = 4;
    summary
}

pub(super) fn image_quad_summary(image: ImageId) -> DebugDrawCommandSummary {
    let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::ImageQuad);
    summary.image = Some(image);
    summary.point_count = 4;
    summary.vertex_count = 4;
    summary.index_count = 6;
    summary.triangle_count = 2;
    summary
}

pub(super) fn svg_rect_summary(kind: DebugDrawCommandKind) -> DebugDrawCommandSummary {
    let mut summary = DebugDrawCommandSummary::new(kind);
    summary.point_count = 4;
    summary
}
