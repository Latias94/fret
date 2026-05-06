use std::sync::Arc;

use fret_core::{Point, Px, Rect, UvPoint};

use super::{DebugDrawRoundCorners, DebugDrawVertex};

pub(super) fn rect_is_empty(rect: Rect) -> bool {
    rect.size.width.0 <= 0.0 || rect.size.height.0 <= 0.0
}

pub(super) fn rect_is_finite(rect: Rect) -> bool {
    rect.origin.x.0.is_finite()
        && rect.origin.y.0.is_finite()
        && rect.size.width.0.is_finite()
        && rect.size.height.0.is_finite()
}

pub(super) fn point_is_finite(point: Point) -> bool {
    point.x.0.is_finite() && point.y.0.is_finite()
}

pub(super) fn points_are_finite(points: &[Point; 4]) -> bool {
    points.iter().copied().all(point_is_finite)
}

pub(super) fn uv_points_are_finite(uvs: &[UvPoint; 4]) -> bool {
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

pub(super) fn triangle_is_degenerate(p1: Point, p2: Point, p3: Point) -> bool {
    let ax = p2.x.0 - p1.x.0;
    let ay = p2.y.0 - p1.y.0;
    let bx = p3.x.0 - p1.x.0;
    let by = p3.y.0 - p1.y.0;
    (ax * by - ay * bx).abs() <= f32::EPSILON
}

pub(super) fn triangle_vertices_are_drawable(vertices: &[DebugDrawVertex; 3]) -> bool {
    vertices.iter().copied().all(debug_draw_vertex_is_finite)
        && vertices.iter().any(|vertex| vertex.color.a > 0.0)
        && !triangle_is_degenerate(
            vertices[0].position,
            vertices[1].position,
            vertices[2].position,
        )
}

pub(super) fn indexed_triangle(
    vertices: &[DebugDrawVertex],
    indices: &[u32],
) -> Option<[DebugDrawVertex; 3]> {
    let i0 = usize::try_from(indices[0]).ok()?;
    let i1 = usize::try_from(indices[1]).ok()?;
    let i2 = usize::try_from(indices[2]).ok()?;
    Some([*vertices.get(i0)?, *vertices.get(i1)?, *vertices.get(i2)?])
}

pub(super) fn sequential_triangle_indices(len: usize) -> Arc<[u32]> {
    let capped_len = len.min(u32::MAX as usize);
    Arc::from(
        (0..capped_len)
            .map(|index| index as u32)
            .collect::<Vec<_>>(),
    )
}

pub(super) fn rect_quad_points(rect: Rect) -> [Point; 4] {
    let x0 = rect.origin.x;
    let y0 = rect.origin.y;
    let x1 = Px(rect.origin.x.0 + rect.size.width.0);
    let y1 = Px(rect.origin.y.0 + rect.size.height.0);
    [
        Point::new(x0, y0),
        Point::new(x1, y0),
        Point::new(x1, y1),
        Point::new(x0, y1),
    ]
}

pub(super) fn effective_rect_rounding(
    rect: Rect,
    rounding: Px,
    corners: DebugDrawRoundCorners,
) -> Px {
    if rect_is_empty(rect)
        || !rect_is_finite(rect)
        || !rounding.0.is_finite()
        || rounding.0 < 0.5
        || corners.is_empty()
    {
        return Px(0.0);
    }

    let width = rect.size.width.0.abs();
    let height = rect.size.height.0.abs();
    let x_scale = if corners.contains(DebugDrawRoundCorners::TOP)
        || corners.contains(DebugDrawRoundCorners::BOTTOM)
    {
        0.5
    } else {
        1.0
    };
    let y_scale = if corners.contains(DebugDrawRoundCorners::LEFT)
        || corners.contains(DebugDrawRoundCorners::RIGHT)
    {
        0.5
    } else {
        1.0
    };
    let rounding = rounding
        .0
        .min(width * x_scale - 1.0)
        .min(height * y_scale - 1.0);
    if rounding >= 0.5 {
        Px(rounding)
    } else {
        Px(0.0)
    }
}
