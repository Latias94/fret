use super::state::{
    EncodeState, apply_transform_px, bounds_of_quad_points, transform_quad_points_px,
    transform_rows,
};
use super::*;

use crate::svg::SMOOTH_SVG_SCALE_FACTOR;
use fret_core::geometry::{Corners, Edges};

pub(super) fn encode_quad(
    state: &mut EncodeState<'_>,
    rect: Rect,
    background: Color,
    border: Edges,
    border_color: Color,
    corner_radii: Corners,
) {
    let opacity = state.current_opacity();

    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    let background = EncodeState::color_with_opacity(background, opacity);
    let border_color = EncodeState::color_with_opacity(border_color, opacity);

    if background.a <= 0.0 && border_color.a <= 0.0 {
        return;
    }
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let needs_new_batch = match state.quad_batch {
        Some((scissor, uniform_index, _)) => {
            scissor != state.current_scissor || uniform_index != state.current_uniform_index
        }
        None => true,
    };

    if needs_new_batch {
        state.flush_quad_batch();
        state.quad_batch = Some((
            state.current_scissor,
            state.current_uniform_index,
            state.instances.len() as u32,
        ));
    }

    let t_px = state.to_physical_px(state.current_transform());
    let (transform0, transform1) = transform_rows(t_px);

    let corner_radii = corners_to_vec4(corner_radii).map(|r| r * state.scale_factor);
    let corner_radii = clamp_corner_radii_for_rect(w, h, corner_radii);
    let border = edges_to_vec4(border).map(|e| e * state.scale_factor);
    state.instances.push(QuadInstance {
        rect: [x, y, w, h],
        transform0,
        transform1,
        color: color_to_linear_rgba_premul(background),
        corner_radii,
        border,
        border_color: color_to_linear_rgba_premul(border_color),
    });
}

pub(super) fn encode_image(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    image: fret_core::ImageId,
    opacity: f32,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || group_opacity <= 0.0 {
        return;
    }
    if renderer.images.get(image).is_none() {
        return;
    }
    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let t_px = state.current_transform_px();
    let quad = transform_quad_points_px(t_px, x, y, w, h);

    let first_vertex = state.viewport_vertices.len() as u32;
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);

    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [0.0, 0.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [1.0, 0.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [1.0, 1.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [0.0, 0.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [1.0, 1.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [0.0, 1.0],
            opacity: o,
            _pad: [0.0; 3],
        },
    ]);

    state.ordered_draws.push(OrderedDraw::Image(ImageDraw {
        scissor: state.current_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 6,
        image,
    }));
}

pub(super) fn encode_image_region(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    image: fret_core::ImageId,
    uv: UvRect,
    opacity: f32,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || group_opacity <= 0.0 {
        return;
    }
    if renderer.images.get(image).is_none() {
        return;
    }
    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let t_px = state.current_transform_px();
    let quad = transform_quad_points_px(t_px, x, y, w, h);

    let first_vertex = state.viewport_vertices.len() as u32;
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);

    let (u0, v0, u1, v1) = (uv.u0, uv.v0, uv.u1, uv.v1);
    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [u1, v0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [u0, v1],
            opacity: o,
            _pad: [0.0; 3],
        },
    ]);

    state.ordered_draws.push(OrderedDraw::Image(ImageDraw {
        scissor: state.current_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 6,
        image,
    }));
}

pub(super) fn encode_mask_image(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    image: fret_core::ImageId,
    uv: UvRect,
    color: Color,
    opacity: f32,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || group_opacity <= 0.0 || color.a <= 0.0 {
        return;
    }
    if renderer.images.get(image).is_none() {
        return;
    }
    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let t_px = state.current_transform_px();
    let quad = transform_quad_points_px(t_px, x, y, w, h);

    let first_vertex = state.text_vertices.len() as u32;
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);
    let mut premul = color_to_linear_rgba_premul(color);
    premul = premul.map(|c| c * o);

    let (u0, v0, u1, v1) = (uv.u0, uv.v0, uv.u1, uv.v1);
    state.text_vertices.extend_from_slice(&[
        TextVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [u1, v0],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [u0, v1],
            color: premul,
        },
    ]);

    state.ordered_draws.push(OrderedDraw::Mask(MaskDraw {
        scissor: state.current_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 6,
        image,
    }));
}

pub(super) fn encode_svg_mask_icon(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    svg: fret_core::SvgId,
    fit: fret_core::SvgFit,
    color: Color,
    opacity: f32,
) {
    state.flush_quad_batch();
    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || group_opacity <= 0.0 || color.a <= 0.0 {
        return;
    }

    let t = state.current_transform();
    let s = EncodeState::current_transform_max_scale(t);
    let key_rect = Rect::new(
        rect.origin,
        Size::new(Px(rect.size.width.0 * s), Px(rect.size.height.0 * s)),
    );
    let key = Renderer::svg_raster_key(
        svg,
        key_rect,
        state.scale_factor,
        SvgRasterKind::AlphaMask,
        fit,
    );
    let Some(entry) = renderer.svg_rasters.get(&key) else {
        return;
    };
    if renderer.images.get(entry.image).is_none() {
        return;
    }

    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let first_vertex = state.text_vertices.len() as u32;
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);
    let mut premul = color_to_linear_rgba_premul(color);
    premul = premul.map(|c| c * o);

    let (lx0, ly0, lx1, ly1) =
        svg_draw_rect_px(x, y, w, h, entry.size_px, SMOOTH_SVG_SCALE_FACTOR, fit);
    let t_px = state.current_transform_px();
    let quad = [
        apply_transform_px(t_px, lx0, ly0),
        apply_transform_px(t_px, lx1, ly0),
        apply_transform_px(t_px, lx1, ly1),
        apply_transform_px(t_px, lx0, ly1),
    ];

    let (u0, v0, u1, v1) = (entry.uv.u0, entry.uv.v0, entry.uv.u1, entry.uv.v1);
    state.text_vertices.extend_from_slice(&[
        TextVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [u1, v0],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [u0, v0],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [u1, v1],
            color: premul,
        },
        TextVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [u0, v1],
            color: premul,
        },
    ]);

    let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
    let svg_scissor = scissor_from_bounds_px(min_x, min_y, max_x, max_y, state.viewport_size)
        .map(|s| intersect_scissor(state.current_scissor, s))
        .unwrap_or(state.current_scissor);
    state.ordered_draws.push(OrderedDraw::Mask(MaskDraw {
        scissor: svg_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 6,
        image: entry.image,
    }));
}

pub(super) fn encode_svg_image(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    svg: fret_core::SvgId,
    fit: fret_core::SvgFit,
    opacity: f32,
) {
    state.flush_quad_batch();
    let group_opacity = state.current_opacity();
    if opacity <= 0.0 || group_opacity <= 0.0 {
        return;
    }

    let t = state.current_transform();
    let s = EncodeState::current_transform_max_scale(t);
    let key_rect = Rect::new(
        rect.origin,
        Size::new(Px(rect.size.width.0 * s), Px(rect.size.height.0 * s)),
    );

    let key = Renderer::svg_raster_key(svg, key_rect, state.scale_factor, SvgRasterKind::Rgba, fit);
    let Some(entry) = renderer.svg_rasters.get(&key) else {
        return;
    };
    if renderer.images.get(entry.image).is_none() {
        return;
    }

    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let first_vertex = state.viewport_vertices.len() as u32;
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);

    let (lx0, ly0, lx1, ly1) =
        svg_draw_rect_px(x, y, w, h, entry.size_px, SMOOTH_SVG_SCALE_FACTOR, fit);
    let t_px = state.current_transform_px();
    let quad = [
        apply_transform_px(t_px, lx0, ly0),
        apply_transform_px(t_px, lx1, ly0),
        apply_transform_px(t_px, lx1, ly1),
        apply_transform_px(t_px, lx0, ly1),
    ];

    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [0.0, 0.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [1.0, 0.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [1.0, 1.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [0.0, 0.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [1.0, 1.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [0.0, 1.0],
            opacity: o,
            _pad: [0.0; 3],
        },
    ]);

    let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
    let svg_scissor = scissor_from_bounds_px(min_x, min_y, max_x, max_y, state.viewport_size)
        .map(|s| intersect_scissor(state.current_scissor, s))
        .unwrap_or(state.current_scissor);
    state.ordered_draws.push(OrderedDraw::Image(ImageDraw {
        scissor: svg_scissor,
        uniform_index: state.current_uniform_index,
        first_vertex,
        vertex_count: 6,
        image: entry.image,
    }));
}

pub(super) fn encode_text(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    origin: Point,
    blob_id: fret_core::TextBlobId,
    color: Color,
) {
    state.flush_quad_batch();

    let Some(blob) = renderer.text_system.blob(blob_id) else {
        return;
    };

    let group_opacity = state.current_opacity();
    if group_opacity <= 0.0 || color.a <= 0.0 {
        return;
    }

    let t_px = state.current_transform_px();

    let base_x = origin.x.0 * state.scale_factor;
    let base_y = origin.y.0 * state.scale_factor;
    let premul = color_to_linear_rgba_premul(EncodeState::color_with_opacity(color, group_opacity));

    let mut active_kind: Option<TextDrawKind> = None;
    let mut group_first_vertex = state.text_vertices.len() as u32;

    for g in &blob.glyphs {
        let kind = match g.kind {
            crate::text::GlyphQuadKind::Mask => TextDrawKind::Mask,
            crate::text::GlyphQuadKind::Color => TextDrawKind::Color,
        };

        if active_kind != Some(kind) {
            if let Some(prev) = active_kind {
                let vertex_count =
                    (state.text_vertices.len() as u32).saturating_sub(group_first_vertex);
                if vertex_count > 0 {
                    state.ordered_draws.push(OrderedDraw::Text(TextDraw {
                        scissor: state.current_scissor,
                        uniform_index: state.current_uniform_index,
                        first_vertex: group_first_vertex,
                        vertex_count,
                        kind: prev,
                    }));
                }
            }
            active_kind = Some(kind);
            group_first_vertex = state.text_vertices.len() as u32;
        }

        let vertex_color = match kind {
            TextDrawKind::Mask => premul,
            // Preserve emoji/color glyph RGB; only apply overall alpha.
            TextDrawKind::Color => [1.0, 1.0, 1.0, premul[3]],
        };

        let lx0 = base_x + g.rect[0] * state.scale_factor;
        let ly0 = base_y + g.rect[1] * state.scale_factor;
        let lx1 = lx0 + g.rect[2] * state.scale_factor;
        let ly1 = ly0 + g.rect[3] * state.scale_factor;
        let quad = [
            apply_transform_px(t_px, lx0, ly0),
            apply_transform_px(t_px, lx1, ly0),
            apply_transform_px(t_px, lx1, ly1),
            apply_transform_px(t_px, lx0, ly1),
        ];

        let (u0, v0, u1, v1) = (g.uv[0], g.uv[1], g.uv[2], g.uv[3]);

        state.text_vertices.extend_from_slice(&[
            TextVertex {
                pos_px: [quad[0].0, quad[0].1],
                uv: [u0, v0],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[1].0, quad[1].1],
                uv: [u1, v0],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[2].0, quad[2].1],
                uv: [u1, v1],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[0].0, quad[0].1],
                uv: [u0, v0],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[2].0, quad[2].1],
                uv: [u1, v1],
                color: vertex_color,
            },
            TextVertex {
                pos_px: [quad[3].0, quad[3].1],
                uv: [u0, v1],
                color: vertex_color,
            },
        ]);
    }

    if let Some(kind) = active_kind {
        let vertex_count = (state.text_vertices.len() as u32).saturating_sub(group_first_vertex);
        if vertex_count > 0 {
            state.ordered_draws.push(OrderedDraw::Text(TextDraw {
                scissor: state.current_scissor,
                uniform_index: state.current_uniform_index,
                first_vertex: group_first_vertex,
                vertex_count,
                kind,
            }));
        }
    }
}

pub(super) fn encode_path(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    origin: Point,
    path: fret_core::PathId,
    color: Color,
) {
    state.flush_quad_batch();

    let group_opacity = state.current_opacity();
    if color.a <= 0.0 || group_opacity <= 0.0 {
        return;
    }
    let Some(prepared) = renderer.paths.get(path) else {
        return;
    };
    if prepared.triangles.is_empty() {
        return;
    }
    let t_px = state.current_transform_px();

    let local_bounds = Rect::new(
        Point::new(
            origin.x + prepared.metrics.bounds.origin.x,
            origin.y + prepared.metrics.bounds.origin.y,
        ),
        prepared.metrics.bounds.size,
    );
    let (bx, by, bw, bh) = rect_to_pixels(local_bounds, state.scale_factor);
    let bounds_quad = transform_quad_points_px(t_px, bx, by, bw, bh);
    let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&bounds_quad);
    let Some(bounds_scissor) =
        scissor_from_bounds_px(min_x, min_y, max_x, max_y, state.viewport_size)
    else {
        return;
    };
    let clipped_scissor = intersect_scissor(state.current_scissor, bounds_scissor);
    if clipped_scissor.w == 0 || clipped_scissor.h == 0 {
        return;
    }

    let first_vertex = state.path_vertices.len() as u32;
    let ox = origin.x.0 * state.scale_factor;
    let oy = origin.y.0 * state.scale_factor;
    let premul = color_to_linear_rgba_premul(EncodeState::color_with_opacity(color, group_opacity));

    for p in &prepared.triangles {
        let lx = ox + p[0] * state.scale_factor;
        let ly = oy + p[1] * state.scale_factor;
        let (wx, wy) = apply_transform_px(t_px, lx, ly);
        state.path_vertices.push(PathVertex {
            pos_px: [wx, wy],
            color: premul,
        });
    }

    let vertex_count = (state.path_vertices.len() as u32).saturating_sub(first_vertex);
    if vertex_count > 0 {
        state.ordered_draws.push(OrderedDraw::Path(PathDraw {
            scissor: clipped_scissor,
            uniform_index: state.current_uniform_index,
            first_vertex,
            vertex_count,
        }));
    }
}

pub(super) fn encode_viewport_surface(
    renderer: &Renderer,
    state: &mut EncodeState<'_>,
    rect: Rect,
    target: fret_core::RenderTargetId,
    opacity: f32,
) {
    state.flush_quad_batch();
    let group_opacity = state.current_opacity();

    if opacity <= 0.0 || group_opacity <= 0.0 {
        return;
    }
    if renderer.render_targets.get(target).is_none() {
        return;
    }
    let (x, y, w, h) = rect_to_pixels(rect, state.scale_factor);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let t_px = state.current_transform_px();
    let quad = transform_quad_points_px(t_px, x, y, w, h);

    let first_vertex = state.viewport_vertices.len() as u32;
    let o = (opacity.clamp(0.0, 1.0) * group_opacity).clamp(0.0, 1.0);

    state.viewport_vertices.extend_from_slice(&[
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [0.0, 0.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[1].0, quad[1].1],
            uv: [1.0, 0.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [1.0, 1.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[0].0, quad[0].1],
            uv: [0.0, 0.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[2].0, quad[2].1],
            uv: [1.0, 1.0],
            opacity: o,
            _pad: [0.0; 3],
        },
        ViewportVertex {
            pos_px: [quad[3].0, quad[3].1],
            uv: [0.0, 1.0],
            opacity: o,
            _pad: [0.0; 3],
        },
    ]);

    state
        .ordered_draws
        .push(OrderedDraw::Viewport(ViewportDraw {
            scissor: state.current_scissor,
            uniform_index: state.current_uniform_index,
            first_vertex,
            vertex_count: 6,
            target,
        }));
}
