mod finite;
mod rects;
mod triangles;

pub(super) use finite::{points_are_finite, uv_points_are_finite};
pub(super) use rects::{effective_rect_rounding, rect_is_empty, rect_is_finite, rect_quad_points};
pub(super) use triangles::{
    indexed_triangle, sequential_triangle_indices, triangle_is_degenerate,
    triangle_vertices_are_drawable,
};
