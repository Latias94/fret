use super::super::state::{EncodeState, apply_transform_px, bounds_of_quad_points};
use super::super::*;

use crate::svg::SMOOTH_SVG_SCALE_FACTOR;

pub(in super::super) fn encode_svg_mask_icon(
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

pub(in super::super) fn encode_svg_image(
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
