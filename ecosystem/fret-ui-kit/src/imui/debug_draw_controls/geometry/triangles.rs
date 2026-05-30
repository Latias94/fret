use std::sync::Arc;

use fret_core::Point;

use super::super::DebugDrawVertex;
use super::finite::debug_draw_vertex_is_finite;

pub(in crate::imui::debug_draw_controls) fn triangle_is_degenerate(
    p1: Point,
    p2: Point,
    p3: Point,
) -> bool {
    let ax = p2.x.0 - p1.x.0;
    let ay = p2.y.0 - p1.y.0;
    let bx = p3.x.0 - p1.x.0;
    let by = p3.y.0 - p1.y.0;
    (ax * by - ay * bx).abs() <= f32::EPSILON
}

pub(in crate::imui::debug_draw_controls) fn triangle_vertices_are_drawable(
    vertices: &[DebugDrawVertex; 3],
) -> bool {
    vertices.iter().copied().all(debug_draw_vertex_is_finite)
        && vertices.iter().any(|vertex| vertex.color.a > 0.0)
        && !triangle_is_degenerate(
            vertices[0].position,
            vertices[1].position,
            vertices[2].position,
        )
}

pub(in crate::imui::debug_draw_controls) fn indexed_triangle(
    vertices: &[DebugDrawVertex],
    indices: &[u32],
) -> Option<[DebugDrawVertex; 3]> {
    let i0 = usize::try_from(indices[0]).ok()?;
    let i1 = usize::try_from(indices[1]).ok()?;
    let i2 = usize::try_from(indices[2]).ok()?;
    Some([*vertices.get(i0)?, *vertices.get(i1)?, *vertices.get(i2)?])
}

pub(in crate::imui::debug_draw_controls) fn sequential_triangle_indices(len: usize) -> Arc<[u32]> {
    let capped_len = len.min(u32::MAX as usize);
    Arc::from(
        (0..capped_len)
            .map(|index| index as u32)
            .collect::<Vec<_>>(),
    )
}
