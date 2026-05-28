use fret_core::{Point, UvPoint};

use super::super::DebugDrawVertex;

pub(super) fn point_is_finite(point: Point) -> bool {
    point.x.0.is_finite() && point.y.0.is_finite()
}

pub(in crate::imui::debug_draw_controls) fn points_are_finite(points: &[Point; 4]) -> bool {
    points.iter().copied().all(point_is_finite)
}

pub(in crate::imui::debug_draw_controls) fn uv_points_are_finite(uvs: &[UvPoint; 4]) -> bool {
    uvs.iter().all(|uv| uv.u.is_finite() && uv.v.is_finite())
}

pub(super) fn debug_draw_vertex_is_finite(vertex: DebugDrawVertex) -> bool {
    point_is_finite(vertex.position)
        && vertex.uv.u.is_finite()
        && vertex.uv.v.is_finite()
        && vertex.color.r.is_finite()
        && vertex.color.g.is_finite()
        && vertex.color.b.is_finite()
        && vertex.color.a.is_finite()
}
