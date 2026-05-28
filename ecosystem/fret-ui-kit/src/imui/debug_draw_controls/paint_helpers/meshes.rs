use fret_core::{DrawOrder, ImageId};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::geometry::{
    indexed_triangle, triangle_vertices_are_drawable,
};
use crate::imui::debug_draw_controls::{DebugDrawImageMeshOptions, DebugDrawVertex};

pub(in crate::imui::debug_draw_controls) fn paint_triangle_mesh(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    vertices: &[DebugDrawVertex],
    indices: &[u32],
) {
    for triangle_indices in indices.chunks_exact(3) {
        let Some(triangle) = indexed_triangle(vertices, triangle_indices) else {
            continue;
        };
        if !triangle_vertices_are_drawable(&triangle) {
            continue;
        }
        painter
            .scene()
            .push(fret_core::SceneOp::VertexColorTriangle {
                order,
                vertices: triangle.map(DebugDrawVertex::scene_vertex),
            });
    }
}

pub(in crate::imui::debug_draw_controls) fn paint_image_triangle_mesh(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    image: ImageId,
    vertices: &[DebugDrawVertex],
    indices: &[u32],
    options: DebugDrawImageMeshOptions,
) {
    if !options.opacity.is_finite() || options.opacity <= 0.0 {
        return;
    }
    for triangle_indices in indices.chunks_exact(3) {
        let Some(triangle) = indexed_triangle(vertices, triangle_indices) else {
            continue;
        };
        if !triangle_vertices_are_drawable(&triangle) {
            continue;
        }
        painter.scene().push(fret_core::SceneOp::ImageTriangle {
            order,
            image,
            vertices: triangle.map(DebugDrawVertex::scene_vertex),
            sampling: options.sampling,
            opacity: options.opacity,
        });
    }
}
