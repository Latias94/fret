use fret_core::geometry::{Corners, Edges, Rect};
use fret_core::scene::Color;

use super::ScissorRect;
use crate::upload_counters::record_svg_upload;

pub(super) fn write_r8_texture_region(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    origin: (u32, u32),
    size_px: (u32, u32),
    data: &[u8],
) {
    let (w, h) = size_px;
    debug_assert_eq!(data.len(), (w as usize) * (h as usize));
    if w == 0 || h == 0 {
        return;
    }

    let bytes_per_row = w;
    let aligned_bytes_per_row = bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let aligned_bytes_per_row = aligned_bytes_per_row.max(bytes_per_row);

    let mut owned: Vec<u8> = Vec::new();
    let bytes: &[u8] = if aligned_bytes_per_row == bytes_per_row {
        data
    } else {
        owned.resize((aligned_bytes_per_row * h) as usize, 0);
        for row in 0..h as usize {
            let src0 = row * w as usize;
            let src1 = src0 + w as usize;
            let dst0 = row * aligned_bytes_per_row as usize;
            let dst1 = dst0 + w as usize;
            owned[dst0..dst1].copy_from_slice(&data[src0..src1]);
        }
        &owned
    };

    record_svg_upload(bytes.len());
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: origin.0,
                y: origin.1,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(aligned_bytes_per_row),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

pub(super) fn color_to_linear_rgba_premul(color: Color) -> [f32; 4] {
    let a = color.a;
    [color.r * a, color.g * a, color.b * a, a]
}

pub(super) fn corners_to_vec4(c: Corners) -> [f32; 4] {
    [
        c.top_left.0,
        c.top_right.0,
        c.bottom_right.0,
        c.bottom_left.0,
    ]
}

pub(super) fn clamp_corner_radii_for_rect(
    rect_w: f32,
    rect_h: f32,
    corner_radii: [f32; 4],
) -> [f32; 4] {
    let mut max_radius = if rect_w.is_finite() && rect_h.is_finite() {
        rect_w.min(rect_h) * 0.5
    } else {
        0.0
    };
    if !max_radius.is_finite() || max_radius <= 0.0 {
        max_radius = 0.0;
    }

    corner_radii.map(|r| {
        if !r.is_finite() || r <= 0.0 || max_radius == 0.0 {
            0.0
        } else {
            r.min(max_radius)
        }
    })
}

pub(super) fn edges_to_vec4(e: Edges) -> [f32; 4] {
    [e.left.0, e.top.0, e.right.0, e.bottom.0]
}

pub(super) fn rect_to_pixels(rect: Rect, scale_factor: f32) -> (f32, f32, f32, f32) {
    (
        rect.origin.x.0 * scale_factor,
        rect.origin.y.0 * scale_factor,
        rect.size.width.0 * scale_factor,
        rect.size.height.0 * scale_factor,
    )
}

pub(super) fn svg_draw_rect_px(
    target_x: f32,
    target_y: f32,
    target_w: f32,
    target_h: f32,
    raster_size_px: (u32, u32),
    smooth_scale_factor: f32,
    fit: fret_core::SvgFit,
) -> (f32, f32, f32, f32) {
    let smooth = smooth_scale_factor.max(1.0);
    match fit {
        fret_core::SvgFit::Contain => {
            let draw_w = (raster_size_px.0 as f32 / smooth).min(target_w.max(0.0));
            let draw_h = (raster_size_px.1 as f32 / smooth).min(target_h.max(0.0));
            let x0 = target_x + ((target_w - draw_w).max(0.0) * 0.5);
            let y0 = target_y + ((target_h - draw_h).max(0.0) * 0.5);
            (x0, y0, x0 + draw_w, y0 + draw_h)
        }
        fret_core::SvgFit::Width => {
            let draw_w = raster_size_px.0 as f32 / smooth;
            let draw_h = raster_size_px.1 as f32 / smooth;
            let x0 = target_x + (target_w - draw_w) * 0.5;
            let y0 = target_y + (target_h - draw_h) * 0.5;
            (x0, y0, x0 + draw_w, y0 + draw_h)
        }
        fret_core::SvgFit::Stretch => {
            (target_x, target_y, target_x + target_w, target_y + target_h)
        }
    }
}

pub(super) fn scissor_from_bounds_px(
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    viewport: (u32, u32),
) -> Option<ScissorRect> {
    let (vw, vh) = viewport;
    if vw == 0 || vh == 0 {
        return None;
    }

    let x0 = min_x.floor().clamp(0.0, vw as f32) as i32;
    let y0 = min_y.floor().clamp(0.0, vh as f32) as i32;
    let x1 = max_x.ceil().clamp(0.0, vw as f32) as i32;
    let y1 = max_y.ceil().clamp(0.0, vh as f32) as i32;

    let w = (x1 - x0).max(0) as u32;
    let h = (y1 - y0).max(0) as u32;
    if w == 0 || h == 0 {
        return Some(ScissorRect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        });
    }

    Some(ScissorRect {
        x: x0 as u32,
        y: y0 as u32,
        w,
        h,
    })
}

pub(super) fn intersect_scissor(a: ScissorRect, b: ScissorRect) -> ScissorRect {
    let ax1 = a.x.saturating_add(a.w);
    let ay1 = a.y.saturating_add(a.h);
    let bx1 = b.x.saturating_add(b.w);
    let by1 = b.y.saturating_add(b.h);

    let x0 = a.x.max(b.x);
    let y0 = a.y.max(b.y);
    let x1 = ax1.min(bx1);
    let y1 = ay1.min(by1);

    let w = x1.saturating_sub(x0);
    let h = y1.saturating_sub(y0);
    ScissorRect { x: x0, y: y0, w, h }
}

pub(super) fn union_scissor(a: ScissorRect, b: ScissorRect) -> ScissorRect {
    let ax1 = a.x.saturating_add(a.w);
    let ay1 = a.y.saturating_add(a.h);
    let bx1 = b.x.saturating_add(b.w);
    let by1 = b.y.saturating_add(b.h);

    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = ax1.max(bx1);
    let y1 = ay1.max(by1);

    let w = x1.saturating_sub(x0);
    let h = y1.saturating_sub(y0);
    ScissorRect { x: x0, y: y0, w, h }
}

pub(super) fn mix_u64(mut state: u64, value: u64) -> u64 {
    state ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = state.rotate_left(7);
    state = state.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    state
}

pub(super) fn mix_f32(state: u64, value: f32) -> u64 {
    mix_u64(state, u64::from(value.to_bits()))
}

pub(super) fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut state = 0u64;
    for &b in bytes {
        state = mix_u64(state, u64::from(b));
    }
    state
}
