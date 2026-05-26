use fret_core::{Corners, DrawOrder, ImageId, Px, Rect, UvRect};
use fret_ui::canvas::CanvasPainter;

use super::geometry::{effective_rect_rounding, indexed_triangle, triangle_vertices_are_drawable};
use super::{
    DebugDrawImageMeshOptions, DebugDrawImageOptions, DebugDrawRoundCorners, DebugDrawVertex,
};

pub(super) fn normalized_opacity(opacity: f32) -> f32 {
    if opacity.is_finite() {
        opacity.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

pub(super) fn uv_rect_is_valid(uv: UvRect) -> bool {
    uv.u0.is_finite()
        && uv.v0.is_finite()
        && uv.u1.is_finite()
        && uv.v1.is_finite()
        && uv.u1 > uv.u0
        && uv.v1 > uv.v0
}

pub(super) fn corner_radii_are_visible(radii: Corners) -> bool {
    radii.top_left.0 > 0.0
        || radii.top_right.0 > 0.0
        || radii.bottom_right.0 > 0.0
        || radii.bottom_left.0 > 0.0
}

pub(super) fn rounded_rect_corner_radii(
    rect: Rect,
    rounding: Px,
    corners: DebugDrawRoundCorners,
) -> Corners {
    let rounding = effective_rect_rounding(rect, rounding, corners);
    Corners {
        top_left: if corners.contains(DebugDrawRoundCorners::TOP_LEFT) {
            rounding
        } else {
            Px(0.0)
        },
        top_right: if corners.contains(DebugDrawRoundCorners::TOP_RIGHT) {
            rounding
        } else {
            Px(0.0)
        },
        bottom_right: if corners.contains(DebugDrawRoundCorners::BOTTOM_RIGHT) {
            rounding
        } else {
            Px(0.0)
        },
        bottom_left: if corners.contains(DebugDrawRoundCorners::BOTTOM_LEFT) {
            rounding
        } else {
            Px(0.0)
        },
    }
}

pub(super) fn paint_triangle_mesh(
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

pub(super) fn paint_image_triangle_mesh(
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

pub(super) fn paint_image(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    rect: Rect,
    image: ImageId,
    options: DebugDrawImageOptions,
    opacity: f32,
) {
    painter.scene().push(fret_core::SceneOp::Image {
        order,
        rect,
        image,
        fit: options.fit,
        sampling: options.sampling,
        opacity,
    });
}

pub(super) fn paint_image_region(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    rect: Rect,
    image: ImageId,
    uv: UvRect,
    options: DebugDrawImageOptions,
    opacity: f32,
) {
    painter.scene().push(fret_core::SceneOp::ImageRegion {
        order,
        rect,
        image,
        uv,
        sampling: options.sampling,
        opacity,
    });
}
