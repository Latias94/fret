use bytemuck::{Pod, Zeroable};
use fret_core::{
    geometry::{Corners, Edges, Rect},
    scene::{Color, Scene, SceneOp, UvRect},
};
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};
use slotmap::SlotMap;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::images::{ImageColorSpace, ImageDescriptor, ImageRegistry};
use crate::svg::{
    SMOOTH_SVG_SCALE_FACTOR, SvgAlphaMask, SvgRenderer, upload_alpha_mask, upload_rgba_image,
};
use crate::targets::{RenderTargetDescriptor, RenderTargetRegistry};
use crate::text::TextSystem;

#[derive(Debug, Clone, Copy)]
pub struct ClearColor(pub wgpu::Color);

impl Default for ClearColor {
    fn default() -> Self {
        Self(wgpu::Color {
            r: 0.08,
            g: 0.09,
            b: 0.10,
            a: 1.0,
        })
    }
}

const MAX_CLIPS: usize = 8;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ClipRRectUniform {
    rect: [f32; 4],
    corner_radii: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ViewportUniform {
    viewport_size: [f32; 2],
    clip_count: u32,
    _pad0: u32,
    clips: [ClipRRectUniform; MAX_CLIPS],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct QuadInstance {
    rect: [f32; 4],
    color: [f32; 4],
    corner_radii: [f32; 4],
    border: [f32; 4],
    border_color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ViewportVertex {
    pos_px: [f32; 2],
    uv: [f32; 2],
    opacity: f32,
    _pad: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TextVertex {
    pos_px: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PathVertex {
    pos_px: [f32; 2],
    color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScissorRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SvgRasterKind {
    AlphaMask,
    Rgba,
}

const SVG_MASK_ATLAS_PAGE_SIZE_PX: u32 = 1024;
const SVG_MASK_ATLAS_PADDING_PX: u32 = 1;

#[derive(Debug, Default, Clone, Copy)]
pub struct SvgPerfSnapshot {
    pub frames: u64,
    pub prepare_svg_ops_us: u64,

    pub cache_hits: u64,
    pub cache_misses: u64,

    pub alpha_raster_count: u64,
    pub alpha_raster_us: u64,
    pub rgba_raster_count: u64,
    pub rgba_raster_us: u64,

    pub alpha_atlas_inserts: u64,
    pub alpha_atlas_write_us: u64,
    pub alpha_standalone_uploads: u64,
    pub alpha_standalone_upload_us: u64,
    pub rgba_uploads: u64,
    pub rgba_upload_us: u64,

    pub atlas_pages_live: usize,
    pub svg_rasters_live: usize,
    pub svg_standalone_bytes_live: u64,
    pub svg_mask_atlas_bytes_live: u64,
    pub svg_mask_atlas_used_px: u64,
    pub svg_mask_atlas_capacity_px: u64,
}

#[derive(Debug, Default)]
struct SvgPerfStats {
    frames: u64,
    prepare_svg_ops: Duration,

    cache_hits: u64,
    cache_misses: u64,

    alpha_raster_count: u64,
    alpha_raster: Duration,
    rgba_raster_count: u64,
    rgba_raster: Duration,

    alpha_atlas_inserts: u64,
    alpha_atlas_write: Duration,
    alpha_standalone_uploads: u64,
    alpha_standalone_upload: Duration,
    rgba_uploads: u64,
    rgba_upload: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SvgRasterKey {
    svg: fret_core::SvgId,
    target_w: u32,
    target_h: u32,
    smooth_scale_bits: u32,
    kind: SvgRasterKind,
    fit: fret_core::SvgFit,
}

enum SvgRasterStorage {
    Standalone { _texture: wgpu::Texture },
    MaskAtlas { page_index: usize },
}

struct SvgMaskAtlasPage {
    image: fret_core::ImageId,
    size_px: (u32, u32),
    cursor_x: u32,
    cursor_y: u32,
    row_h: u32,
    entries: usize,
    last_used_epoch: u64,
    _texture: wgpu::Texture,
}

struct SvgRasterEntry {
    image: fret_core::ImageId,
    uv: UvRect,
    size_px: (u32, u32),
    approx_bytes: u64,
    last_used_epoch: u64,
    storage: SvgRasterStorage,
}

impl SvgMaskAtlasPage {
    fn bytes(&self) -> u64 {
        u64::from(self.size_px.0).saturating_mul(u64::from(self.size_px.1))
    }

    fn try_alloc(&mut self, size_px: (u32, u32)) -> Option<(u32, u32)> {
        let (w, h) = size_px;
        if w == 0 || h == 0 {
            return None;
        }
        let (page_w, page_h) = self.size_px;
        if w > page_w || h > page_h {
            return None;
        }

        let mut x = self.cursor_x;
        let mut y = self.cursor_y;
        let mut row_h = self.row_h;

        if x.saturating_add(w) > page_w {
            x = 0;
            y = y.saturating_add(row_h);
            row_h = 0;
        }
        if y.saturating_add(h) > page_h {
            return None;
        }

        self.cursor_x = x.saturating_add(w);
        self.cursor_y = y;
        self.row_h = row_h.max(h);
        Some((x, y))
    }
}

impl ScissorRect {
    fn full(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            w: width,
            h: height,
        }
    }
}

fn write_r8_texture_region(
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

fn color_to_linear_rgba_premul(color: Color) -> [f32; 4] {
    let a = color.a;
    [color.r * a, color.g * a, color.b * a, a]
}

fn corners_to_vec4(c: Corners) -> [f32; 4] {
    [
        c.top_left.0,
        c.top_right.0,
        c.bottom_right.0,
        c.bottom_left.0,
    ]
}

fn clamp_corner_radii_for_rect(rect_w: f32, rect_h: f32, corner_radii: [f32; 4]) -> [f32; 4] {
    let mut max_radius = if rect_w.is_finite() && rect_h.is_finite() {
        rect_w.min(rect_h) * 0.5
    } else {
        0.0
    };
    if !max_radius.is_finite() || max_radius <= 0.0 {
        max_radius = 0.0;
    }

    corner_radii.map(|r| {
        if !r.is_finite() || r <= 0.0 {
            0.0
        } else if max_radius == 0.0 {
            0.0
        } else {
            r.min(max_radius)
        }
    })
}

fn edges_to_vec4(e: Edges) -> [f32; 4] {
    [e.left.0, e.top.0, e.right.0, e.bottom.0]
}

fn rect_to_pixels(rect: Rect, scale_factor: f32) -> (f32, f32, f32, f32) {
    (
        rect.origin.x.0 * scale_factor,
        rect.origin.y.0 * scale_factor,
        rect.size.width.0 * scale_factor,
        rect.size.height.0 * scale_factor,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PathCacheKey {
    commands_hash: u64,
    commands_len: u32,
    style_key: u64,
    scale_factor_bits: u32,
}

#[derive(Debug, Clone, Copy)]
struct CachedPathEntry {
    id: fret_core::PathId,
    refs: u32,
    last_used_epoch: u64,
}

fn mix_u64(mut state: u64, value: u64) -> u64 {
    state ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = state.rotate_left(7);
    state = state.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    state
}

fn mix_f32(state: u64, value: f32) -> u64 {
    mix_u64(state, u64::from(value.to_bits()))
}

fn mix_path_style(state: u64, style: fret_core::PathStyle) -> u64 {
    match style {
        fret_core::PathStyle::Fill(fill) => {
            let mut state = mix_u64(state, 0xF11);
            let rule = match fill.rule {
                fret_core::FillRule::NonZero => 1u64,
                fret_core::FillRule::EvenOdd => 2u64,
            };
            state = mix_u64(state, rule);
            state
        }
        fret_core::PathStyle::Stroke(stroke) => {
            let mut state = mix_u64(state, 0x570);
            state = mix_f32(state, stroke.width.0);
            state
        }
    }
}

fn hash_path_commands(commands: &[fret_core::PathCommand]) -> u64 {
    let mut state = 0u64;
    for cmd in commands {
        match *cmd {
            fret_core::PathCommand::MoveTo(p) => {
                state = mix_u64(state, 1);
                state = mix_f32(state, p.x.0);
                state = mix_f32(state, p.y.0);
            }
            fret_core::PathCommand::LineTo(p) => {
                state = mix_u64(state, 2);
                state = mix_f32(state, p.x.0);
                state = mix_f32(state, p.y.0);
            }
            fret_core::PathCommand::QuadTo { ctrl, to } => {
                state = mix_u64(state, 3);
                state = mix_f32(state, ctrl.x.0);
                state = mix_f32(state, ctrl.y.0);
                state = mix_f32(state, to.x.0);
                state = mix_f32(state, to.y.0);
            }
            fret_core::PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                state = mix_u64(state, 4);
                state = mix_f32(state, ctrl1.x.0);
                state = mix_f32(state, ctrl1.y.0);
                state = mix_f32(state, ctrl2.x.0);
                state = mix_f32(state, ctrl2.y.0);
                state = mix_f32(state, to.x.0);
                state = mix_f32(state, to.y.0);
            }
            fret_core::PathCommand::Close => {
                state = mix_u64(state, 5);
            }
        }
    }
    state
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut state = 0u64;
    for &b in bytes {
        state = mix_u64(state, u64::from(b));
    }
    state
}

fn path_cache_key(
    commands: &[fret_core::PathCommand],
    style: fret_core::PathStyle,
    constraints: fret_core::PathConstraints,
) -> PathCacheKey {
    PathCacheKey {
        commands_hash: hash_path_commands(commands),
        commands_len: commands.len().min(u32::MAX as usize) as u32,
        style_key: mix_path_style(0, style),
        scale_factor_bits: constraints.scale_factor.to_bits(),
    }
}

fn metrics_from_path_commands(
    commands: &[fret_core::PathCommand],
    style: fret_core::PathStyle,
) -> fret_core::PathMetrics {
    let mut min_x: Option<f32> = None;
    let mut min_y: Option<f32> = None;
    let mut max_x: Option<f32> = None;
    let mut max_y: Option<f32> = None;

    let mut include_point = |p: fret_core::Point| {
        let x = p.x.0;
        let y = p.y.0;
        min_x = Some(min_x.map_or(x, |v| v.min(x)));
        min_y = Some(min_y.map_or(y, |v| v.min(y)));
        max_x = Some(max_x.map_or(x, |v| v.max(x)));
        max_y = Some(max_y.map_or(y, |v| v.max(y)));
    };

    // Keep bounds semantics aligned with `build_lyon_path`: if a segment appears before a `MoveTo`,
    // treat it as an implicit `MoveTo(end)` (control points are ignored because there is no
    // well-defined start point).
    let mut active = false;
    for cmd in commands {
        match *cmd {
            fret_core::PathCommand::MoveTo(p) => {
                include_point(p);
                active = true;
            }
            fret_core::PathCommand::LineTo(p) => {
                include_point(p);
                active = true;
            }
            fret_core::PathCommand::QuadTo { ctrl, to } => {
                if active {
                    include_point(ctrl);
                }
                include_point(to);
                active = true;
            }
            fret_core::PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                if active {
                    include_point(ctrl1);
                    include_point(ctrl2);
                }
                include_point(to);
                active = true;
            }
            fret_core::PathCommand::Close => {
                active = false;
            }
        }
    }

    let (Some(mut min_x), Some(mut min_y), Some(mut max_x), Some(mut max_y)) =
        (min_x, min_y, max_x, max_y)
    else {
        return fret_core::PathMetrics::default();
    };

    if let fret_core::PathStyle::Stroke(stroke) = style {
        let half = stroke.width.0.max(0.0) * 0.5;
        min_x -= half;
        min_y -= half;
        max_x += half;
        max_y += half;
    }

    let w = (max_x - min_x).max(0.0);
    let h = (max_y - min_y).max(0.0);
    fret_core::PathMetrics {
        bounds: Rect::new(
            fret_core::Point::new(fret_core::Px(min_x), fret_core::Px(min_y)),
            fret_core::Size::new(fret_core::Px(w), fret_core::Px(h)),
        ),
    }
}

fn build_lyon_path(commands: &[fret_core::PathCommand]) -> lyon::path::Path {
    use lyon::math::point;

    let mut builder = lyon::path::Path::builder();
    let mut active = false;

    for cmd in commands {
        match *cmd {
            fret_core::PathCommand::MoveTo(p) => {
                if active {
                    builder.end(false);
                }
                builder.begin(point(p.x.0, p.y.0));
                active = true;
            }
            fret_core::PathCommand::LineTo(p) => {
                let to = point(p.x.0, p.y.0);
                if !active {
                    builder.begin(to);
                    active = true;
                } else {
                    builder.line_to(to);
                }
            }
            fret_core::PathCommand::QuadTo { ctrl, to } => {
                let ctrl = point(ctrl.x.0, ctrl.y.0);
                let to = point(to.x.0, to.y.0);
                if !active {
                    builder.begin(to);
                    active = true;
                } else {
                    builder.quadratic_bezier_to(ctrl, to);
                }
            }
            fret_core::PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                let ctrl1 = point(ctrl1.x.0, ctrl1.y.0);
                let ctrl2 = point(ctrl2.x.0, ctrl2.y.0);
                let to = point(to.x.0, to.y.0);
                if !active {
                    builder.begin(to);
                    active = true;
                } else {
                    builder.cubic_bezier_to(ctrl1, ctrl2, to);
                }
            }
            fret_core::PathCommand::Close => {
                if active {
                    builder.end(true);
                    active = false;
                }
            }
        }
    }

    if active {
        builder.end(false);
    }

    builder.build()
}

fn tessellate_path_commands(
    commands: &[fret_core::PathCommand],
    style: fret_core::PathStyle,
    constraints: fret_core::PathConstraints,
) -> Vec<[f32; 2]> {
    if commands.is_empty() {
        return Vec::new();
    }

    let path = build_lyon_path(commands);

    let scale = constraints.scale_factor.max(1.0);
    let tolerance = (0.25 / scale).clamp(0.01, 1.0);

    let mut buffers: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    match style {
        fret_core::PathStyle::Fill(fill) => {
            let fill_rule = match fill.rule {
                fret_core::FillRule::NonZero => lyon::tessellation::FillRule::NonZero,
                fret_core::FillRule::EvenOdd => lyon::tessellation::FillRule::EvenOdd,
            };
            let opts = FillOptions::default()
                .with_tolerance(tolerance)
                .with_fill_rule(fill_rule);
            let mut tessellator = FillTessellator::new();
            let _ = tessellator.tessellate_path(
                &path,
                &opts,
                &mut BuffersBuilder::new(&mut buffers, |v: FillVertex| {
                    let p = v.position();
                    [p.x, p.y]
                }),
            );
        }
        fret_core::PathStyle::Stroke(stroke) => {
            let width = stroke.width.0.max(0.0);
            let opts = StrokeOptions::default()
                .with_line_width(width)
                .with_tolerance(tolerance)
                .with_line_join(lyon::tessellation::LineJoin::Round)
                .with_start_cap(lyon::tessellation::LineCap::Round)
                .with_end_cap(lyon::tessellation::LineCap::Round);
            let mut tessellator = StrokeTessellator::new();
            let _ = tessellator.tessellate_path(
                &path,
                &opts,
                &mut BuffersBuilder::new(&mut buffers, |v: StrokeVertex| {
                    let p = v.position();
                    [p.x, p.y]
                }),
            );
        }
    }

    let mut out = Vec::with_capacity(buffers.indices.len());
    for idx in buffers.indices {
        if let Some(v) = buffers.vertices.get(idx as usize) {
            out.push(*v);
        }
    }
    out
}

fn svg_draw_rect_px(
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

fn scissor_from_rect(rect: Rect, scale_factor: f32, viewport: (u32, u32)) -> Option<ScissorRect> {
    let (vw, vh) = viewport;
    if vw == 0 || vh == 0 {
        return None;
    }

    let (x, y, w, h) = rect_to_pixels(rect, scale_factor);
    let x0 = x.floor().clamp(0.0, vw as f32) as i32;
    let y0 = y.floor().clamp(0.0, vh as f32) as i32;
    let x1 = (x + w).ceil().clamp(0.0, vw as f32) as i32;
    let y1 = (y + h).ceil().clamp(0.0, vh as f32) as i32;

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

fn intersect_scissor(a: ScissorRect, b: ScissorRect) -> ScissorRect {
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

fn union_scissor(a: ScissorRect, b: ScissorRect) -> ScissorRect {
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

struct DrawCall {
    scissor: ScissorRect,
    uniform_index: u32,
    first_instance: u32,
    instance_count: u32,
}

struct ViewportDraw {
    scissor: ScissorRect,
    uniform_index: u32,
    first_vertex: u32,
    vertex_count: u32,
    target: fret_core::RenderTargetId,
}

#[derive(Clone, Copy)]
struct ImageDraw {
    scissor: ScissorRect,
    uniform_index: u32,
    first_vertex: u32,
    vertex_count: u32,
    image: fret_core::ImageId,
}

#[derive(Clone, Copy)]
struct MaskDraw {
    scissor: ScissorRect,
    uniform_index: u32,
    first_vertex: u32,
    vertex_count: u32,
    image: fret_core::ImageId,
}

struct TextDraw {
    scissor: ScissorRect,
    uniform_index: u32,
    first_vertex: u32,
    vertex_count: u32,
}

#[derive(Clone, Copy)]
struct PathDraw {
    scissor: ScissorRect,
    uniform_index: u32,
    first_vertex: u32,
    vertex_count: u32,
}

struct PathIntermediate {
    size: (u32, u32),
    format: wgpu::TextureFormat,
    sample_count: u32,
    resolved_view: wgpu::TextureView,
    msaa_view: Option<wgpu::TextureView>,
    bind_group: wgpu::BindGroup,
}

enum OrderedDraw {
    Quad(DrawCall),
    Viewport(ViewportDraw),
    Image(ImageDraw),
    Mask(MaskDraw),
    Text(TextDraw),
    Path(PathDraw),
}

pub struct Renderer {
    adapter: wgpu::Adapter,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_stride: u64,
    uniform_capacity: usize,

    quad_pipeline_format: Option<wgpu::TextureFormat>,
    quad_pipeline: Option<wgpu::RenderPipeline>,

    viewport_pipeline_format: Option<wgpu::TextureFormat>,
    viewport_pipeline: Option<wgpu::RenderPipeline>,
    viewport_bind_group_layout: wgpu::BindGroupLayout,
    viewport_sampler: wgpu::Sampler,

    instance_buffers: Vec<wgpu::Buffer>,
    instance_buffer_index: usize,
    instance_capacity: usize,

    viewport_vertex_buffers: Vec<wgpu::Buffer>,
    viewport_vertex_buffer_index: usize,
    viewport_vertex_capacity: usize,

    text_pipeline_format: Option<wgpu::TextureFormat>,
    text_pipeline: Option<wgpu::RenderPipeline>,

    mask_pipeline_format: Option<wgpu::TextureFormat>,
    mask_pipeline: Option<wgpu::RenderPipeline>,

    text_vertex_buffers: Vec<wgpu::Buffer>,
    text_vertex_buffer_index: usize,
    text_vertex_capacity: usize,

    path_pipeline_format: Option<wgpu::TextureFormat>,
    path_pipeline: Option<wgpu::RenderPipeline>,

    path_msaa_pipeline_format: Option<wgpu::TextureFormat>,
    path_msaa_pipeline: Option<wgpu::RenderPipeline>,
    path_msaa_pipeline_sample_count: Option<u32>,

    composite_pipeline_format: Option<wgpu::TextureFormat>,
    composite_pipeline: Option<wgpu::RenderPipeline>,

    path_vertex_buffers: Vec<wgpu::Buffer>,
    path_vertex_buffer_index: usize,
    path_vertex_capacity: usize,

    path_intermediate: Option<PathIntermediate>,
    path_composite_vertices: wgpu::Buffer,

    text_system: TextSystem,

    paths: SlotMap<fret_core::PathId, PreparedPath>,
    path_cache: HashMap<PathCacheKey, CachedPathEntry>,
    path_cache_capacity: usize,
    path_cache_epoch: u64,

    svg_renderer: SvgRenderer,
    svgs: SlotMap<fret_core::SvgId, Arc<[u8]>>,
    svg_hash_index: HashMap<u64, Vec<fret_core::SvgId>>,
    svg_rasters: HashMap<SvgRasterKey, SvgRasterEntry>,
    svg_mask_atlas_pages: Vec<Option<SvgMaskAtlasPage>>,
    svg_mask_atlas_free: Vec<usize>,
    // Bytes used by standalone SVG rasters (not atlas-backed).
    svg_raster_bytes: u64,
    // Bytes reserved by SVG alpha-mask atlas pages.
    svg_mask_atlas_bytes: u64,
    svg_raster_budget_bytes: u64,
    svg_raster_epoch: u64,
    svg_perf_enabled: bool,
    svg_perf: SvgPerfStats,

    path_msaa_samples: u32,

    render_targets: RenderTargetRegistry,
    images: ImageRegistry,

    viewport_bind_groups: HashMap<fret_core::RenderTargetId, (u64, wgpu::BindGroup)>,
    render_target_revisions: HashMap<fret_core::RenderTargetId, u64>,
    render_targets_generation: u64,

    image_bind_groups: HashMap<fret_core::ImageId, (u64, wgpu::BindGroup)>,
    image_revisions: HashMap<fret_core::ImageId, u64>,
    images_generation: u64,

    scene_encoding_cache_key: Option<SceneEncodingCacheKey>,
    scene_encoding_cache: SceneEncoding,
    scene_encoding_scratch: SceneEncoding,
}

#[derive(Debug)]
struct PreparedPath {
    metrics: fret_core::PathMetrics,
    triangles: Vec<[f32; 2]>,
    cache_key: PathCacheKey,
}

pub struct RenderSceneParams<'a> {
    pub format: wgpu::TextureFormat,
    pub target_view: &'a wgpu::TextureView,
    pub scene: &'a Scene,
    pub clear: ClearColor,
    pub scale_factor: f32,
    pub viewport_size: (u32, u32),
}

#[derive(Default)]
struct SceneEncoding {
    instances: Vec<QuadInstance>,
    viewport_vertices: Vec<ViewportVertex>,
    text_vertices: Vec<TextVertex>,
    path_vertices: Vec<PathVertex>,
    uniforms: Vec<ViewportUniform>,
    ordered_draws: Vec<OrderedDraw>,
}

impl SceneEncoding {
    fn clear(&mut self) {
        self.instances.clear();
        self.viewport_vertices.clear();
        self.text_vertices.clear();
        self.path_vertices.clear();
        self.uniforms.clear();
        self.ordered_draws.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SceneEncodingCacheKey {
    format: wgpu::TextureFormat,
    viewport_size: (u32, u32),
    scale_factor_bits: u32,
    scene_fingerprint: u64,
    scene_ops_len: usize,
    render_targets_generation: u64,
    images_generation: u64,
}

impl Renderer {
    pub fn set_svg_perf_enabled(&mut self, enabled: bool) {
        self.svg_perf_enabled = enabled;
        self.svg_perf = SvgPerfStats::default();
    }

    /// Drop all cached SVG rasterizations (standalone rasters and alpha-mask atlas pages) while
    /// keeping the underlying registered SVG bytes (`SvgId`) intact.
    ///
    /// This is the GPUI-style explicit lifecycle knob: apps can decide when to reclaim memory and
    /// accept the cost of re-rasterizing later.
    pub fn clear_svg_raster_cache(&mut self) {
        let rasters = std::mem::take(&mut self.svg_rasters);
        for (_, entry) in rasters {
            if matches!(entry.storage, SvgRasterStorage::Standalone { .. }) {
                let _ = self.unregister_image(entry.image);
            }
        }
        self.svg_raster_bytes = 0;

        for idx in 0..self.svg_mask_atlas_pages.len() {
            self.evict_svg_mask_atlas_page(idx);
        }
        self.svg_mask_atlas_pages.clear();
        self.svg_mask_atlas_free.clear();
        self.svg_mask_atlas_bytes = 0;
    }

    pub fn take_svg_perf_snapshot(&mut self) -> Option<SvgPerfSnapshot> {
        if !self.svg_perf_enabled {
            return None;
        }

        let pages_live = self
            .svg_mask_atlas_pages
            .iter()
            .filter(|p| p.is_some())
            .count();
        let rasters_live = self.svg_rasters.len();
        let standalone_bytes_live = self.svg_raster_bytes;
        let atlas_bytes_live = self.svg_mask_atlas_bytes;
        let atlas_capacity_px = u64::from(pages_live as u32)
            .saturating_mul(u64::from(SVG_MASK_ATLAS_PAGE_SIZE_PX))
            .saturating_mul(u64::from(SVG_MASK_ATLAS_PAGE_SIZE_PX));
        let atlas_used_px = self
            .svg_rasters
            .values()
            .filter_map(|e| match e.storage {
                SvgRasterStorage::MaskAtlas { page_index } => Some((page_index, e.size_px)),
                SvgRasterStorage::Standalone { .. } => None,
            })
            .filter(|(page_index, _)| {
                self.svg_mask_atlas_pages
                    .get(*page_index)
                    .is_some_and(|p| p.is_some())
            })
            .fold(0u64, |acc, (_, (w, h))| {
                let pad = u64::from(SVG_MASK_ATLAS_PADDING_PX.saturating_mul(2));
                let w_pad = u64::from(w).saturating_add(pad);
                let h_pad = u64::from(h).saturating_add(pad);
                acc.saturating_add(w_pad.saturating_mul(h_pad))
            });

        let snap = SvgPerfSnapshot {
            frames: self.svg_perf.frames,
            prepare_svg_ops_us: self.svg_perf.prepare_svg_ops.as_micros() as u64,

            cache_hits: self.svg_perf.cache_hits,
            cache_misses: self.svg_perf.cache_misses,

            alpha_raster_count: self.svg_perf.alpha_raster_count,
            alpha_raster_us: self.svg_perf.alpha_raster.as_micros() as u64,
            rgba_raster_count: self.svg_perf.rgba_raster_count,
            rgba_raster_us: self.svg_perf.rgba_raster.as_micros() as u64,

            alpha_atlas_inserts: self.svg_perf.alpha_atlas_inserts,
            alpha_atlas_write_us: self.svg_perf.alpha_atlas_write.as_micros() as u64,
            alpha_standalone_uploads: self.svg_perf.alpha_standalone_uploads,
            alpha_standalone_upload_us: self.svg_perf.alpha_standalone_upload.as_micros() as u64,
            rgba_uploads: self.svg_perf.rgba_uploads,
            rgba_upload_us: self.svg_perf.rgba_upload.as_micros() as u64,

            atlas_pages_live: pages_live,
            svg_rasters_live: rasters_live,
            svg_standalone_bytes_live: standalone_bytes_live,
            svg_mask_atlas_bytes_live: atlas_bytes_live,
            svg_mask_atlas_used_px: atlas_used_px,
            svg_mask_atlas_capacity_px: atlas_capacity_px,
        };

        self.svg_perf = SvgPerfStats::default();
        Some(snap)
    }

    pub fn svg_raster_budget_bytes(&self) -> u64 {
        self.svg_raster_budget_bytes
    }

    pub fn set_svg_raster_budget_bytes(&mut self, bytes: u64) {
        // Keep a small non-zero floor so callers can't accidentally force unbounded thrash.
        self.svg_raster_budget_bytes = bytes.max(1024);
    }

    pub fn path_msaa_samples(&self) -> u32 {
        self.path_msaa_samples
    }

    pub fn set_path_msaa_samples(&mut self, samples: u32) {
        let samples = samples.max(1);
        let samples = samples.min(16);
        if samples == 1 {
            self.path_msaa_samples = 1;
            return;
        }

        // wgpu requires sample counts to be powers of two. Prefer a conservative downgrade to the
        // nearest supported-shape value (rather than rounding up to a potentially unsupported count).
        let pow2_floor = 1u32 << (31 - samples.leading_zeros());
        self.path_msaa_samples = pow2_floor.max(1);
    }

    fn effective_path_msaa_samples(&self, format: wgpu::TextureFormat) -> u32 {
        let requested = self.path_msaa_samples.max(1);
        if requested == 1 {
            return 1;
        }

        let features = self.adapter.get_texture_format_features(format);
        if !features
            .allowed_usages
            .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            return 1;
        }

        // When MSAA is enabled we render into an intermediate and then sample from the resolved
        // texture in the composite pass, so the format must be sampleable and support resolves.
        if !features
            .allowed_usages
            .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            || !features
                .flags
                .contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE)
        {
            return 1;
        }

        for candidate in [16u32, 8, 4, 2] {
            if candidate <= requested && features.flags.sample_count_supported(candidate) {
                return candidate;
            }
        }
        1
    }

    fn svg_target_box_px(rect: Rect, scale_factor: f32) -> (u32, u32) {
        let w = (rect.size.width.0 * scale_factor).ceil().max(1.0);
        let h = (rect.size.height.0 * scale_factor).ceil().max(1.0);
        (w as u32, h as u32)
    }

    fn svg_raster_key(
        svg: fret_core::SvgId,
        rect: Rect,
        scale_factor: f32,
        kind: SvgRasterKind,
        fit: fret_core::SvgFit,
    ) -> SvgRasterKey {
        let (target_w, target_h) = Self::svg_target_box_px(rect, scale_factor);
        SvgRasterKey {
            svg,
            target_w,
            target_h,
            smooth_scale_bits: SMOOTH_SVG_SCALE_FACTOR.to_bits(),
            kind,
            fit,
        }
    }

    fn prepare_svg_ops(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        scale_factor: f32,
    ) {
        for op in scene.ops() {
            match op {
                SceneOp::SvgMaskIcon { rect, svg, fit, .. } => {
                    let _ = self.ensure_svg_raster(
                        device,
                        queue,
                        *svg,
                        *rect,
                        scale_factor,
                        SvgRasterKind::AlphaMask,
                        *fit,
                    );
                }
                SceneOp::SvgImage { rect, svg, fit, .. } => {
                    let _ = self.ensure_svg_raster(
                        device,
                        queue,
                        *svg,
                        *rect,
                        scale_factor,
                        SvgRasterKind::Rgba,
                        *fit,
                    );
                }
                _ => {}
            }
        }

        self.prune_svg_rasters();
    }

    fn bump_svg_raster_epoch(&mut self) -> u64 {
        self.svg_raster_epoch = self.svg_raster_epoch.wrapping_add(1);
        self.svg_raster_epoch
    }

    fn prune_svg_rasters(&mut self) {
        if self.svg_raster_bytes <= self.svg_raster_budget_bytes {
            return;
        }

        // Best-effort eviction: never evict entries used in the current frame.
        let cur_epoch = self.svg_raster_epoch;

        while self.svg_raster_bytes > self.svg_raster_budget_bytes {
            let mut victim_standalone: Option<(SvgRasterKey, u64)> = None;
            for (k, v) in &self.svg_rasters {
                if v.last_used_epoch == cur_epoch {
                    continue;
                }
                if !matches!(&v.storage, SvgRasterStorage::Standalone { .. }) {
                    continue;
                }
                match victim_standalone {
                    None => victim_standalone = Some((*k, v.last_used_epoch)),
                    Some((_, best_epoch)) => {
                        if v.last_used_epoch < best_epoch {
                            victim_standalone = Some((*k, v.last_used_epoch));
                        }
                    }
                }
            }

            let Some((victim_key, _)) = victim_standalone else {
                // Cache is over budget but all standalone entries were used this frame (or there
                // are no standalone entries). Keep correctness and allow a temporary overshoot.
                break;
            };

            if let Some(entry) = self.svg_rasters.remove(&victim_key) {
                self.drop_svg_raster_entry(entry);
            } else {
                break;
            }
        }
    }

    fn drop_svg_raster_entry(&mut self, entry: SvgRasterEntry) {
        match entry.storage {
            SvgRasterStorage::Standalone { .. } => {
                self.svg_raster_bytes = self.svg_raster_bytes.saturating_sub(entry.approx_bytes);
                let _ = self.unregister_image(entry.image);
            }
            SvgRasterStorage::MaskAtlas { page_index } => {
                let Some(page) = self
                    .svg_mask_atlas_pages
                    .get_mut(page_index)
                    .and_then(|p| p.as_mut())
                else {
                    return;
                };
                page.entries = page.entries.saturating_sub(1);
                if page.entries == 0 {
                    self.evict_svg_mask_atlas_page(page_index);
                }
            }
        }
    }

    fn ensure_svg_mask_atlas_page(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> usize {
        let size_px = (SVG_MASK_ATLAS_PAGE_SIZE_PX, SVG_MASK_ATLAS_PAGE_SIZE_PX);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret svg mask atlas"),
            size: wgpu::Extent3d {
                width: size_px.0,
                height: size_px.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let image = self.register_image(ImageDescriptor {
            view: view.clone(),
            size: size_px,
            format: wgpu::TextureFormat::R8Unorm,
            color_space: ImageColorSpace::Linear,
        });

        let zeros = vec![0u8; (size_px.0 as usize) * (size_px.1 as usize)];
        write_r8_texture_region(queue, &texture, (0, 0), size_px, &zeros);

        let page = SvgMaskAtlasPage {
            image,
            size_px,
            cursor_x: 0,
            cursor_y: 0,
            row_h: 0,
            entries: 0,
            last_used_epoch: self.svg_raster_epoch,
            _texture: texture,
        };

        let idx = self.svg_mask_atlas_free.pop().unwrap_or_else(|| {
            self.svg_mask_atlas_pages.push(None);
            self.svg_mask_atlas_pages.len() - 1
        });
        self.svg_mask_atlas_pages[idx] = Some(page);

        self.svg_mask_atlas_bytes = self
            .svg_mask_atlas_bytes
            .saturating_add(u64::from(size_px.0).saturating_mul(u64::from(size_px.1)));
        idx
    }

    fn insert_svg_alpha_mask_into_atlas(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mask: &SvgAlphaMask,
    ) -> Option<(fret_core::ImageId, UvRect, (u32, u32), usize)> {
        let (w, h) = mask.size_px;
        if w == 0 || h == 0 {
            return None;
        }

        let pad = SVG_MASK_ATLAS_PADDING_PX;
        let w_pad = w.saturating_add(pad.saturating_mul(2));
        let h_pad = h.saturating_add(pad.saturating_mul(2));
        if w_pad == 0
            || h_pad == 0
            || w_pad > SVG_MASK_ATLAS_PAGE_SIZE_PX
            || h_pad > SVG_MASK_ATLAS_PAGE_SIZE_PX
        {
            return None;
        }

        let mut alloc: Option<(usize, u32, u32)> = None;
        for (idx, page) in self.svg_mask_atlas_pages.iter_mut().enumerate() {
            let Some(page) = page.as_mut() else {
                continue;
            };
            if let Some((x, y)) = page.try_alloc((w_pad, h_pad)) {
                alloc = Some((idx, x, y));
                break;
            }
        }
        if alloc.is_none() {
            let page_index = self.ensure_svg_mask_atlas_page(device, queue);
            let page = self.svg_mask_atlas_pages[page_index]
                .as_mut()
                .expect("atlas page exists");
            let (x, y) = page.try_alloc((w_pad, h_pad))?;
            alloc = Some((page_index, x, y));
        }

        let (page_index, x, y) = alloc?;
        let page = self.svg_mask_atlas_pages[page_index]
            .as_mut()
            .expect("atlas page exists");

        let mut padded = vec![0u8; (w_pad as usize) * (h_pad as usize)];
        let max_x = w.saturating_sub(1);
        let max_y = h.saturating_sub(1);
        for yy in 0..h_pad {
            let src_y = yy.saturating_sub(pad).min(max_y);
            for xx in 0..w_pad {
                let src_x = xx.saturating_sub(pad).min(max_x);
                let dst = (yy as usize) * (w_pad as usize) + (xx as usize);
                let src = (src_y as usize) * (w as usize) + (src_x as usize);
                padded[dst] = mask.alpha[src];
            }
        }

        write_r8_texture_region(queue, &page._texture, (x, y), (w_pad, h_pad), &padded);

        let page_w = page.size_px.0 as f32;
        let page_h = page.size_px.1 as f32;
        let u0 = (x + pad) as f32 / page_w;
        let v0 = (y + pad) as f32 / page_h;
        let u1 = (x + pad + w) as f32 / page_w;
        let v1 = (y + pad + h) as f32 / page_h;

        page.entries += 1;
        page.last_used_epoch = self.svg_raster_epoch;

        Some((page.image, UvRect { u0, v0, u1, v1 }, (w, h), page_index))
    }

    fn evict_svg_mask_atlas_page(&mut self, page_index: usize) {
        let Some(page) = self
            .svg_mask_atlas_pages
            .get_mut(page_index)
            .and_then(|p| p.take())
        else {
            return;
        };

        let mut keys_to_remove: Vec<SvgRasterKey> = Vec::new();
        for (k, v) in &self.svg_rasters {
            let is_page = match &v.storage {
                SvgRasterStorage::MaskAtlas { page_index: idx } => *idx == page_index,
                SvgRasterStorage::Standalone { .. } => false,
            };
            if is_page {
                keys_to_remove.push(*k);
            }
        }
        for k in keys_to_remove {
            let _ = self.svg_rasters.remove(&k);
        }

        self.svg_mask_atlas_bytes = self.svg_mask_atlas_bytes.saturating_sub(page.bytes());
        let _ = self.unregister_image(page.image);

        self.svg_mask_atlas_free.push(page_index);
    }

    fn ensure_svg_raster(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        svg: fret_core::SvgId,
        rect: Rect,
        scale_factor: f32,
        kind: SvgRasterKind,
        fit: fret_core::SvgFit,
    ) -> Option<(fret_core::ImageId, fret_core::UvRect, (u32, u32))> {
        let key = Self::svg_raster_key(svg, rect, scale_factor, kind, fit);
        if self.svg_rasters.contains_key(&key) {
            if self.svg_perf_enabled {
                self.svg_perf.cache_hits = self.svg_perf.cache_hits.saturating_add(1);
            }
            let (image, uv, size_px, page_index) = {
                let e = self.svg_rasters.get_mut(&key)?;
                e.last_used_epoch = self.svg_raster_epoch;
                let page_index = match &e.storage {
                    SvgRasterStorage::MaskAtlas { page_index } => Some(*page_index),
                    SvgRasterStorage::Standalone { .. } => None,
                };
                (e.image, e.uv, e.size_px, page_index)
            };
            if let Some(page_index) = page_index {
                if let Some(Some(page)) = self.svg_mask_atlas_pages.get_mut(page_index) {
                    page.last_used_epoch = self.svg_raster_epoch;
                }
            }
            return Some((image, uv, size_px));
        }
        if self.svg_perf_enabled {
            self.svg_perf.cache_misses = self.svg_perf.cache_misses.saturating_add(1);
        }

        let bytes = self.svgs.get(svg)?;
        let target_box_px = (key.target_w, key.target_h);

        let (image, uv, size_px, approx_bytes, storage) = match kind {
            SvgRasterKind::AlphaMask => {
                let t_raster = Instant::now();
                let mask = self
                    .svg_renderer
                    .render_alpha_mask_fit_mode(bytes, target_box_px, SMOOTH_SVG_SCALE_FACTOR, fit)
                    .ok()?;
                if self.svg_perf_enabled {
                    self.svg_perf.alpha_raster_count =
                        self.svg_perf.alpha_raster_count.saturating_add(1);
                    self.svg_perf.alpha_raster += t_raster.elapsed();
                }

                let t_insert = Instant::now();
                if let Some((image, uv, size_px, page_index)) =
                    self.insert_svg_alpha_mask_into_atlas(device, queue, &mask)
                {
                    if self.svg_perf_enabled {
                        self.svg_perf.alpha_atlas_inserts =
                            self.svg_perf.alpha_atlas_inserts.saturating_add(1);
                        self.svg_perf.alpha_atlas_write += t_insert.elapsed();
                    }
                    (
                        image,
                        uv,
                        size_px,
                        0,
                        SvgRasterStorage::MaskAtlas {
                            page_index: page_index,
                        },
                    )
                } else {
                    if self.svg_perf_enabled {
                        self.svg_perf.alpha_atlas_write += t_insert.elapsed();
                    }
                    let t_upload = Instant::now();
                    let uploaded = upload_alpha_mask(device, queue, &mask);
                    if self.svg_perf_enabled {
                        self.svg_perf.alpha_standalone_uploads =
                            self.svg_perf.alpha_standalone_uploads.saturating_add(1);
                        self.svg_perf.alpha_standalone_upload += t_upload.elapsed();
                    }
                    let image = self.register_image(ImageDescriptor {
                        view: uploaded.view.clone(),
                        size: uploaded.size_px,
                        format: wgpu::TextureFormat::R8Unorm,
                        color_space: ImageColorSpace::Linear,
                    });
                    let approx_bytes =
                        u64::from(uploaded.size_px.0).saturating_mul(u64::from(uploaded.size_px.1));
                    (
                        image,
                        UvRect::FULL,
                        uploaded.size_px,
                        approx_bytes,
                        SvgRasterStorage::Standalone {
                            _texture: uploaded.texture,
                        },
                    )
                }
            }
            SvgRasterKind::Rgba => {
                let t_raster = Instant::now();
                let rgba = self
                    .svg_renderer
                    .render_rgba_fit_mode(bytes, target_box_px, SMOOTH_SVG_SCALE_FACTOR, fit)
                    .ok()?;
                if self.svg_perf_enabled {
                    self.svg_perf.rgba_raster_count =
                        self.svg_perf.rgba_raster_count.saturating_add(1);
                    self.svg_perf.rgba_raster += t_raster.elapsed();
                }
                let t_upload = Instant::now();
                let uploaded = upload_rgba_image(device, queue, &rgba);
                if self.svg_perf_enabled {
                    self.svg_perf.rgba_uploads = self.svg_perf.rgba_uploads.saturating_add(1);
                    self.svg_perf.rgba_upload += t_upload.elapsed();
                }
                let image = self.register_image(ImageDescriptor {
                    view: uploaded.view.clone(),
                    size: uploaded.size_px,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    color_space: ImageColorSpace::Srgb,
                });
                let approx_bytes = u64::from(uploaded.size_px.0)
                    .saturating_mul(u64::from(uploaded.size_px.1))
                    .saturating_mul(4);
                (
                    image,
                    UvRect::FULL,
                    uploaded.size_px,
                    approx_bytes,
                    SvgRasterStorage::Standalone {
                        _texture: uploaded.texture,
                    },
                )
            }
        };

        let is_standalone = matches!(&storage, SvgRasterStorage::Standalone { .. });
        self.svg_rasters.insert(
            key,
            SvgRasterEntry {
                image,
                uv,
                size_px,
                approx_bytes,
                last_used_epoch: self.svg_raster_epoch,
                storage,
            },
        );
        if is_standalone {
            self.svg_raster_bytes = self.svg_raster_bytes.saturating_add(approx_bytes);
        }

        Some((image, uv, size_px))
    }

    fn ensure_path_intermediate(
        &mut self,
        device: &wgpu::Device,
        viewport_size: (u32, u32),
        format: wgpu::TextureFormat,
        sample_count: u32,
    ) {
        let needs_rebuild = match &self.path_intermediate {
            Some(cur) => {
                cur.size != viewport_size
                    || cur.format != format
                    || cur.sample_count != sample_count
            }
            None => true,
        };
        if !needs_rebuild {
            return;
        }

        let resolved_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret path intermediate resolved"),
            size: wgpu::Extent3d {
                width: viewport_size.0,
                height: viewport_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let resolved_view = resolved_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let msaa_view = if sample_count > 1 {
            let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fret path intermediate msaa"),
                size: wgpu::Extent3d {
                    width: viewport_size.0,
                    height: viewport_size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            Some(msaa_texture.create_view(&wgpu::TextureViewDescriptor::default()))
        } else {
            None
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret path intermediate bind group"),
            layout: &self.viewport_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.viewport_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&resolved_view),
                },
            ],
        });

        self.path_intermediate = Some(PathIntermediate {
            size: viewport_size,
            format,
            sample_count,
            resolved_view,
            msaa_view,
            bind_group,
        });
    }
    pub fn new(adapter: &wgpu::Adapter, device: &wgpu::Device) -> Self {
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;
        // wgpu requires uniform dynamic offsets to be aligned to 256 bytes.
        let uniform_stride = ((uniform_size + 255) / 256) * 256;
        let uniform_capacity = 256usize;

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret quad uniforms layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(std::num::NonZeroU64::new(uniform_size).unwrap()),
                    },
                    count: None,
                }],
            });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret quad uniforms buffer"),
            size: uniform_stride * uniform_capacity as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret quad uniforms bind group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: Some(std::num::NonZeroU64::new(uniform_size).unwrap()),
                }),
            }],
        });

        let viewport_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret viewport texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
            });

        let viewport_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fret viewport sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let text_system = TextSystem::new(device);

        const FRAMES_IN_FLIGHT: usize = 3;
        let instance_capacity = 1024;
        let instance_buffers = (0..FRAMES_IN_FLIGHT)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret quad instances #{i}")),
                    size: (instance_capacity * std::mem::size_of::<QuadInstance>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

        let viewport_vertex_capacity = 64 * 6;
        let viewport_vertex_buffers = (0..FRAMES_IN_FLIGHT)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret viewport vertices #{i}")),
                    size: (viewport_vertex_capacity * std::mem::size_of::<ViewportVertex>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

        let text_vertex_capacity = 512 * 6;
        let text_vertex_buffers = (0..FRAMES_IN_FLIGHT)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret text vertices #{i}")),
                    size: (text_vertex_capacity * std::mem::size_of::<TextVertex>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

        let path_vertex_capacity = 1024;
        let path_vertex_buffers = (0..FRAMES_IN_FLIGHT)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret path vertices #{i}")),
                    size: (path_vertex_capacity * std::mem::size_of::<PathVertex>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

        let path_composite_vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret path composite vertices"),
            size: (6 * std::mem::size_of::<ViewportVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            adapter: adapter.clone(),
            uniform_buffer,
            uniform_bind_group,
            uniform_bind_group_layout,
            uniform_stride,
            uniform_capacity,
            quad_pipeline_format: None,
            quad_pipeline: None,
            viewport_pipeline_format: None,
            viewport_pipeline: None,
            viewport_bind_group_layout,
            viewport_sampler,
            instance_buffers,
            instance_buffer_index: 0,
            instance_capacity,
            viewport_vertex_buffers,
            viewport_vertex_buffer_index: 0,
            viewport_vertex_capacity,
            text_pipeline_format: None,
            text_pipeline: None,
            mask_pipeline_format: None,
            mask_pipeline: None,
            text_vertex_buffers,
            text_vertex_buffer_index: 0,
            text_vertex_capacity,
            path_pipeline_format: None,
            path_pipeline: None,
            path_msaa_pipeline_format: None,
            path_msaa_pipeline: None,
            path_msaa_pipeline_sample_count: None,
            composite_pipeline_format: None,
            composite_pipeline: None,
            path_vertex_buffers,
            path_vertex_buffer_index: 0,
            path_vertex_capacity,
            path_intermediate: None,
            path_composite_vertices,
            text_system,
            paths: SlotMap::with_key(),
            path_cache: HashMap::new(),
            path_cache_capacity: 2048,
            path_cache_epoch: 0,
            svg_renderer: SvgRenderer::new(),
            svgs: SlotMap::with_key(),
            svg_hash_index: HashMap::new(),
            svg_rasters: HashMap::new(),
            svg_mask_atlas_pages: Vec::new(),
            svg_mask_atlas_free: Vec::new(),
            svg_raster_bytes: 0,
            svg_mask_atlas_bytes: 0,
            svg_raster_budget_bytes: 64 * 1024 * 1024,
            svg_raster_epoch: 0,
            svg_perf_enabled: false,
            svg_perf: SvgPerfStats::default(),
            path_msaa_samples: 4,
            render_targets: RenderTargetRegistry::default(),
            images: ImageRegistry::default(),
            viewport_bind_groups: HashMap::new(),
            render_target_revisions: HashMap::new(),
            render_targets_generation: 0,
            image_bind_groups: HashMap::new(),
            image_revisions: HashMap::new(),
            images_generation: 0,
            scene_encoding_cache_key: None,
            scene_encoding_cache: SceneEncoding::default(),
            scene_encoding_scratch: SceneEncoding::default(),
        }
    }

    fn bump_path_cache_epoch(&mut self) -> u64 {
        self.path_cache_epoch = self.path_cache_epoch.wrapping_add(1);
        self.path_cache_epoch
    }

    fn prune_path_cache(&mut self) {
        if self.path_cache.len() <= self.path_cache_capacity {
            return;
        }

        // Simple O(n) eviction: drop least-recently-used entries with refs == 0.
        // This keeps the implementation small and deterministic for MVP-PATH-2.
        while self.path_cache.len() > self.path_cache_capacity {
            let mut victim: Option<(PathCacheKey, CachedPathEntry)> = None;
            for (k, v) in &self.path_cache {
                if v.refs != 0 {
                    continue;
                }
                let replace = match victim {
                    None => true,
                    Some((_, cur)) => v.last_used_epoch < cur.last_used_epoch,
                };
                if replace {
                    victim = Some((*k, *v));
                }
            }

            let Some((key, entry)) = victim else {
                break;
            };

            self.path_cache.remove(&key);
            self.paths.remove(entry.id);
        }
    }

    pub fn register_render_target(
        &mut self,
        desc: RenderTargetDescriptor,
    ) -> fret_core::RenderTargetId {
        let id = self.render_targets.register(desc);
        self.render_target_revisions.insert(id, 1);
        self.render_targets_generation = self.render_targets_generation.saturating_add(1);
        id
    }

    pub fn register_image(&mut self, desc: ImageDescriptor) -> fret_core::ImageId {
        let id = self.images.register(desc);
        self.image_revisions.insert(id, 1);
        self.images_generation = self.images_generation.saturating_add(1);
        id
    }

    pub fn update_image(&mut self, id: fret_core::ImageId, desc: ImageDescriptor) -> bool {
        if !self.images.update(id, desc) {
            return false;
        }
        let next = self.image_revisions.get(&id).copied().unwrap_or(0) + 1;
        self.image_revisions.insert(id, next);
        self.image_bind_groups.remove(&id);
        self.images_generation = self.images_generation.saturating_add(1);
        true
    }

    pub fn unregister_image(&mut self, id: fret_core::ImageId) -> bool {
        if !self.images.unregister(id) {
            return false;
        }
        self.image_revisions.remove(&id);
        self.image_bind_groups.remove(&id);
        self.images_generation = self.images_generation.saturating_add(1);
        true
    }

    pub fn update_render_target(
        &mut self,
        id: fret_core::RenderTargetId,
        desc: RenderTargetDescriptor,
    ) -> bool {
        if !self.render_targets.update(id, desc) {
            return false;
        }
        let next = self.render_target_revisions.get(&id).copied().unwrap_or(0) + 1;
        self.render_target_revisions.insert(id, next);
        self.viewport_bind_groups.remove(&id);
        self.render_targets_generation = self.render_targets_generation.saturating_add(1);
        true
    }

    pub fn unregister_render_target(&mut self, id: fret_core::RenderTargetId) -> bool {
        if !self.render_targets.unregister(id) {
            return false;
        }
        self.render_target_revisions.remove(&id);
        self.viewport_bind_groups.remove(&id);
        self.render_targets_generation = self.render_targets_generation.saturating_add(1);
        true
    }

    fn ensure_viewport_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.viewport_pipeline_format == Some(format) && self.viewport_pipeline.is_some() {
            return;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret viewport shader"),
            source: wgpu::ShaderSource::Wgsl(VIEWPORT_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret viewport pipeline layout"),
            bind_group_layouts: &[
                &self.uniform_bind_group_layout,
                &self.viewport_bind_group_layout,
            ],
            immediate_size: 0,
        });

        let vertex_stride = std::mem::size_of::<ViewportVertex>() as wgpu::BufferAddress;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret viewport pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        self.viewport_pipeline_format = Some(format);
        self.viewport_pipeline = Some(pipeline);
    }

    fn ensure_text_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.text_pipeline_format == Some(format) && self.text_pipeline.is_some() {
            return;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret text shader"),
            source: wgpu::ShaderSource::Wgsl(TEXT_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret text pipeline layout"),
            bind_group_layouts: &[
                &self.uniform_bind_group_layout,
                self.text_system.atlas_bind_group_layout(),
            ],
            immediate_size: 0,
        });

        let vertex_stride = std::mem::size_of::<TextVertex>() as wgpu::BufferAddress;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret text pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        self.text_pipeline_format = Some(format);
        self.text_pipeline = Some(pipeline);
    }

    fn ensure_mask_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.mask_pipeline_format == Some(format) && self.mask_pipeline.is_some() {
            return;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret mask shader"),
            source: wgpu::ShaderSource::Wgsl(MASK_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret mask pipeline layout"),
            bind_group_layouts: &[
                &self.uniform_bind_group_layout,
                &self.viewport_bind_group_layout,
            ],
            immediate_size: 0,
        });

        let vertex_stride = std::mem::size_of::<TextVertex>() as wgpu::BufferAddress;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret mask pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        self.mask_pipeline_format = Some(format);
        self.mask_pipeline = Some(pipeline);
    }

    fn ensure_composite_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.composite_pipeline_format == Some(format) && self.composite_pipeline.is_some() {
            return;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret composite premul shader"),
            source: wgpu::ShaderSource::Wgsl(COMPOSITE_PREMUL_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret composite premul pipeline layout"),
            bind_group_layouts: &[
                &self.uniform_bind_group_layout,
                &self.viewport_bind_group_layout,
            ],
            immediate_size: 0,
        });

        let vertex_stride = std::mem::size_of::<ViewportVertex>() as wgpu::BufferAddress;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret composite premul pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        self.composite_pipeline_format = Some(format);
        self.composite_pipeline = Some(pipeline);
    }

    fn ensure_path_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.path_pipeline_format == Some(format) && self.path_pipeline.is_some() {
            return;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret path shader"),
            source: wgpu::ShaderSource::Wgsl(PATH_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret path pipeline layout"),
            bind_group_layouts: &[&self.uniform_bind_group_layout],
            immediate_size: 0,
        });

        let vertex_stride = std::mem::size_of::<PathVertex>() as wgpu::BufferAddress;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret path pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        self.path_pipeline_format = Some(format);
        self.path_pipeline = Some(pipeline);
    }

    fn ensure_path_msaa_pipeline(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        sample_count: u32,
    ) {
        if self.path_msaa_pipeline_format == Some(format)
            && self.path_msaa_pipeline.is_some()
            && self.path_msaa_pipeline_sample_count == Some(sample_count)
        {
            return;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret path msaa shader"),
            source: wgpu::ShaderSource::Wgsl(PATH_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret path msaa pipeline layout"),
            bind_group_layouts: &[&self.uniform_bind_group_layout],
            immediate_size: 0,
        });

        let vertex_stride = std::mem::size_of::<PathVertex>() as wgpu::BufferAddress;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret path msaa pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        self.path_msaa_pipeline_format = Some(format);
        self.path_msaa_pipeline_sample_count = Some(sample_count);
        self.path_msaa_pipeline = Some(pipeline);
    }

    fn ensure_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.quad_pipeline_format == Some(format) && self.quad_pipeline.is_some() {
            return;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret quad shader"),
            source: wgpu::ShaderSource::Wgsl(QUAD_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret quad pipeline layout"),
            bind_group_layouts: &[&self.uniform_bind_group_layout],
            immediate_size: 0,
        });

        let instance_stride = std::mem::size_of::<QuadInstance>() as wgpu::BufferAddress;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret quad pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: instance_stride,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 48,
                            shader_location: 3,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 64,
                            shader_location: 4,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        self.quad_pipeline_format = Some(format);
        self.quad_pipeline = Some(pipeline);
    }

    fn ensure_instance_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.instance_capacity {
            return;
        }
        let new_capacity = needed.next_power_of_two().max(self.instance_capacity * 2);
        self.instance_buffers = (0..self.instance_buffers.len())
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret quad instances (resized) #{i}")),
                    size: (new_capacity * std::mem::size_of::<QuadInstance>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();
        self.instance_buffer_index = 0;
        self.instance_capacity = new_capacity;
    }

    fn ensure_viewport_vertex_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.viewport_vertex_capacity {
            return;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.viewport_vertex_capacity * 2);
        self.viewport_vertex_buffers = (0..self.viewport_vertex_buffers.len())
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret viewport vertices (resized) #{i}")),
                    size: (new_capacity * std::mem::size_of::<ViewportVertex>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();
        self.viewport_vertex_buffer_index = 0;
        self.viewport_vertex_capacity = new_capacity;
    }

    fn ensure_text_vertex_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.text_vertex_capacity {
            return;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.text_vertex_capacity * 2);
        self.text_vertex_buffers = (0..self.text_vertex_buffers.len())
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret text vertices (resized) #{i}")),
                    size: (new_capacity * std::mem::size_of::<TextVertex>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();
        self.text_vertex_buffer_index = 0;
        self.text_vertex_capacity = new_capacity;
    }

    fn ensure_path_vertex_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.path_vertex_capacity {
            return;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.path_vertex_capacity * 2);
        self.path_vertex_buffers = (0..self.path_vertex_buffers.len())
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret path vertices (resized) #{i}")),
                    size: (new_capacity * std::mem::size_of::<PathVertex>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();
        self.path_vertex_buffer_index = 0;
        self.path_vertex_capacity = new_capacity;
    }

    fn ensure_uniform_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.uniform_capacity {
            return;
        }

        let new_capacity = needed
            .next_power_of_two()
            .max(self.uniform_capacity.saturating_mul(2).max(1));
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret uniforms buffer (resized)"),
            size: self.uniform_stride * new_capacity as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret uniforms bind group (resized)"),
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: Some(std::num::NonZeroU64::new(uniform_size).unwrap()),
                }),
            }],
        });

        self.uniform_buffer = uniform_buffer;
        self.uniform_bind_group = uniform_bind_group;
        self.uniform_capacity = new_capacity;
    }

    pub fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        params: RenderSceneParams<'_>,
    ) -> wgpu::CommandBuffer {
        let RenderSceneParams {
            format,
            target_view,
            scene,
            clear,
            scale_factor,
            viewport_size,
        } = params;
        self.ensure_viewport_pipeline(device, format);
        self.ensure_pipeline(device, format);
        self.ensure_text_pipeline(device, format);
        self.ensure_mask_pipeline(device, format);
        self.ensure_path_pipeline(device, format);
        let path_samples = self.effective_path_msaa_samples(format);
        if path_samples > 1 {
            self.ensure_composite_pipeline(device, format);
            self.ensure_path_msaa_pipeline(device, format, path_samples);
            self.ensure_path_intermediate(device, viewport_size, format, path_samples);
        }

        self.text_system.flush_uploads(queue);
        if self.svg_perf_enabled {
            self.svg_perf.frames = self.svg_perf.frames.saturating_add(1);
        }
        self.bump_svg_raster_epoch();
        let svg_prepare_start = Instant::now();
        self.prepare_svg_ops(device, queue, scene, scale_factor);
        if self.svg_perf_enabled {
            self.svg_perf.prepare_svg_ops += svg_prepare_start.elapsed();
        }

        let key = SceneEncodingCacheKey {
            format,
            viewport_size,
            scale_factor_bits: scale_factor.to_bits(),
            scene_fingerprint: scene.fingerprint(),
            scene_ops_len: scene.ops_len(),
            render_targets_generation: self.render_targets_generation,
            images_generation: self.images_generation,
        };

        let cache_hit = self.scene_encoding_cache_key == Some(key);
        let encoding = if cache_hit {
            std::mem::take(&mut self.scene_encoding_cache)
        } else {
            let mut encoding = std::mem::take(&mut self.scene_encoding_scratch);
            encoding.clear();
            self.encode_scene_ops_into(scene, scale_factor, viewport_size, &mut encoding);

            // Preserve the old cache's allocations for reuse.
            self.scene_encoding_scratch = std::mem::take(&mut self.scene_encoding_cache);
            self.scene_encoding_cache_key = Some(key);
            encoding
        };

        self.ensure_uniform_capacity(device, encoding.uniforms.len());
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;
        let mut uniform_bytes = vec![0u8; (self.uniform_stride * encoding.uniforms.len() as u64) as usize];
        for (i, u) in encoding.uniforms.iter().enumerate() {
            let offset = (self.uniform_stride * i as u64) as usize;
            uniform_bytes[offset..offset + uniform_size as usize]
                .copy_from_slice(bytemuck::bytes_of(u));
        }
        queue.write_buffer(&self.uniform_buffer, 0, &uniform_bytes);

        self.prepare_viewport_bind_groups(device, &encoding.ordered_draws);
        self.prepare_image_bind_groups(device, &encoding.ordered_draws);

        let instances = &encoding.instances;
        let viewport_vertices = &encoding.viewport_vertices;
        let text_vertices = &encoding.text_vertices;
        let path_vertices = &encoding.path_vertices;

        self.ensure_instance_capacity(device, instances.len());
        self.ensure_viewport_vertex_capacity(device, viewport_vertices.len());
        self.ensure_text_vertex_capacity(device, text_vertices.len());
        self.ensure_path_vertex_capacity(device, path_vertices.len());

        let instance_buffer_index = self.instance_buffer_index;
        self.instance_buffer_index = (self.instance_buffer_index + 1) % self.instance_buffers.len();
        let instance_buffer = &self.instance_buffers[instance_buffer_index];
        if !instances.is_empty() {
            queue.write_buffer(instance_buffer, 0, bytemuck::cast_slice(&instances));
        }

        let viewport_vertex_buffer_index = self.viewport_vertex_buffer_index;
        self.viewport_vertex_buffer_index =
            (self.viewport_vertex_buffer_index + 1) % self.viewport_vertex_buffers.len();
        let viewport_vertex_buffer = &self.viewport_vertex_buffers[viewport_vertex_buffer_index];
        if !viewport_vertices.is_empty() {
            queue.write_buffer(
                viewport_vertex_buffer,
                0,
                bytemuck::cast_slice(&viewport_vertices),
            );
        }

        let text_vertex_buffer_index = self.text_vertex_buffer_index;
        self.text_vertex_buffer_index =
            (self.text_vertex_buffer_index + 1) % self.text_vertex_buffers.len();
        let text_vertex_buffer = &self.text_vertex_buffers[text_vertex_buffer_index];
        if !text_vertices.is_empty() {
            queue.write_buffer(text_vertex_buffer, 0, bytemuck::cast_slice(&text_vertices));
        }

        let path_vertex_buffer_index = self.path_vertex_buffer_index;
        self.path_vertex_buffer_index =
            (self.path_vertex_buffer_index + 1) % self.path_vertex_buffers.len();
        let path_vertex_buffer = &self.path_vertex_buffers[path_vertex_buffer_index];
        if !path_vertices.is_empty() {
            queue.write_buffer(path_vertex_buffer, 0, bytemuck::cast_slice(path_vertices));
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fret renderer encoder"),
        });

        {
            enum ActivePipeline {
                None,
                Quad,
                Viewport,
                Text,
                Mask,
                Composite,
                Path,
            }

            let quad_pipeline = self
                .quad_pipeline
                .as_ref()
                .expect("quad pipeline must exist");
            let viewport_pipeline = self
                .viewport_pipeline
                .as_ref()
                .expect("viewport pipeline must exist");
            let text_pipeline = self
                .text_pipeline
                .as_ref()
                .expect("text pipeline must exist");
            let mask_pipeline = self
                .mask_pipeline
                .as_ref()
                .expect("mask pipeline must exist");
            let composite_pipeline = self
                .composite_pipeline
                .as_ref()
                .expect("composite pipeline must exist");
            let path_pipeline = self
                .path_pipeline
                .as_ref()
                .expect("path pipeline must exist");
            let path_msaa_pipeline = self.path_msaa_pipeline.as_ref();

            let mut active_pipeline = ActivePipeline::None;

            fn begin_main_pass<'a>(
                encoder: &'a mut wgpu::CommandEncoder,
                target_view: &'a wgpu::TextureView,
                load: wgpu::LoadOp<wgpu::Color>,
            ) -> wgpu::RenderPass<'a> {
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("fret renderer pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
            }

            let mut pass = begin_main_pass(&mut encoder, target_view, wgpu::LoadOp::Clear(clear.0));
            let mut active_uniform_offset: Option<u32> = None;

            let mut i = 0usize;
            while i < encoding.ordered_draws.len() {
                let item = &encoding.ordered_draws[i];

                if let OrderedDraw::Path(first) = item
                    && path_samples > 1
                {
                    let mut union = first.scissor;
                    let batch_uniform_index = first.uniform_index;
                    let mut end = i + 1;
                    while end < encoding.ordered_draws.len() {
                        match &encoding.ordered_draws[end] {
                            OrderedDraw::Path(d) if d.uniform_index == batch_uniform_index => {
                                union = union_scissor(union, d.scissor);
                                end += 1;
                            }
                            _ => break,
                        }
                    }

                    // Render the path batch to an intermediate MSAA target, then composite into the
                    // main pass to preserve strict op ordering.
                    drop(pass);

                    let Some(intermediate) = &self.path_intermediate else {
                        pass = begin_main_pass(&mut encoder, target_view, wgpu::LoadOp::Load);
                        i = end;
                        continue;
                    };

                    {
                        let Some(path_msaa_pipeline) = path_msaa_pipeline else {
                            pass = begin_main_pass(&mut encoder, target_view, wgpu::LoadOp::Load);
                            i = end;
                            continue;
                        };

                        let mut path_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("fret path intermediate pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: intermediate
                                        .msaa_view
                                        .as_ref()
                                        .unwrap_or(&intermediate.resolved_view),
                                    depth_slice: None,
                                    resolve_target: if intermediate.sample_count > 1 {
                                        Some(&intermediate.resolved_view)
                                    } else {
                                        None
                                    },
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                        store: if intermediate.sample_count > 1 {
                                            wgpu::StoreOp::Discard
                                        } else {
                                            wgpu::StoreOp::Store
                                        },
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                                multiview_mask: None,
                            });

                        path_pass.set_pipeline(path_msaa_pipeline);
                        path_pass.set_vertex_buffer(0, path_vertex_buffer.slice(..));

                        let mut active_uniform_offset: Option<u32> = None;
                        for j in i..end {
                            let OrderedDraw::Path(draw) = &encoding.ordered_draws[j] else {
                                unreachable!();
                            };
                            if draw.scissor.w == 0 || draw.scissor.h == 0 {
                                continue;
                            }
                            path_pass.set_scissor_rect(
                                draw.scissor.x,
                                draw.scissor.y,
                                draw.scissor.w,
                                draw.scissor.h,
                            );
                            let uniform_offset =
                                (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                            if active_uniform_offset != Some(uniform_offset) {
                                path_pass.set_bind_group(
                                    0,
                                    &self.uniform_bind_group,
                                    &[uniform_offset],
                                );
                                active_uniform_offset = Some(uniform_offset);
                            }
                            path_pass.draw(
                                draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                                0..1,
                            );
                        }
                    }

                    pass = begin_main_pass(&mut encoder, target_view, wgpu::LoadOp::Load);
                    active_pipeline = ActivePipeline::None;
                    active_uniform_offset = None;

                    if union.w > 0 && union.h > 0 {
                        let x0 = union.x as f32;
                        let y0 = union.y as f32;
                        let x1 = (union.x + union.w) as f32;
                        let y1 = (union.y + union.h) as f32;

                        let vw = viewport_size.0.max(1) as f32;
                        let vh = viewport_size.1.max(1) as f32;
                        let u0 = x0 / vw;
                        let v0 = y0 / vh;
                        let u1 = x1 / vw;
                        let v1 = y1 / vh;

                        let vertices: [ViewportVertex; 6] = [
                            ViewportVertex {
                                pos_px: [x0, y0],
                                uv: [u0, v0],
                                opacity: 1.0,
                                _pad: [0.0; 3],
                            },
                            ViewportVertex {
                                pos_px: [x1, y0],
                                uv: [u1, v0],
                                opacity: 1.0,
                                _pad: [0.0; 3],
                            },
                            ViewportVertex {
                                pos_px: [x1, y1],
                                uv: [u1, v1],
                                opacity: 1.0,
                                _pad: [0.0; 3],
                            },
                            ViewportVertex {
                                pos_px: [x0, y0],
                                uv: [u0, v0],
                                opacity: 1.0,
                                _pad: [0.0; 3],
                            },
                            ViewportVertex {
                                pos_px: [x1, y1],
                                uv: [u1, v1],
                                opacity: 1.0,
                                _pad: [0.0; 3],
                            },
                            ViewportVertex {
                                pos_px: [x0, y1],
                                uv: [u0, v1],
                                opacity: 1.0,
                                _pad: [0.0; 3],
                            },
                        ];
                        queue.write_buffer(
                            &self.path_composite_vertices,
                            0,
                            bytemuck::cast_slice(&vertices),
                        );

                        pass.set_pipeline(composite_pipeline);
                        let uniform_offset =
                            (u64::from(batch_uniform_index) * self.uniform_stride) as u32;
                        pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                        pass.set_bind_group(1, &intermediate.bind_group, &[]);
                        pass.set_vertex_buffer(0, self.path_composite_vertices.slice(..));
                        pass.set_scissor_rect(union.x, union.y, union.w, union.h);
                        pass.draw(0..6, 0..1);
                        active_pipeline = ActivePipeline::Composite;
                    }

                    i = end;
                    continue;
                }

                match item {
                    OrderedDraw::Quad(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Quad) {
                            pass.set_pipeline(quad_pipeline);
                            pass.set_vertex_buffer(0, instance_buffer.slice(..));
                            active_pipeline = ActivePipeline::Quad;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            0..6,
                            draw.first_instance..(draw.first_instance + draw.instance_count),
                        );
                    }
                    OrderedDraw::Viewport(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Viewport) {
                            pass.set_pipeline(viewport_pipeline);
                            pass.set_vertex_buffer(0, viewport_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Viewport;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        let Some((_, bind_group)) = self.viewport_bind_groups.get(&draw.target)
                        else {
                            // Missing bind group should only happen if the target vanished
                            // between encoding and rendering.
                            i += 1;
                            continue;
                        };
                        pass.set_bind_group(1, bind_group, &[]);
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                    }
                    OrderedDraw::Image(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Viewport) {
                            pass.set_pipeline(viewport_pipeline);
                            pass.set_vertex_buffer(0, viewport_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Viewport;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        let Some((_, bind_group)) = self.image_bind_groups.get(&draw.image) else {
                            i += 1;
                            continue;
                        };
                        pass.set_bind_group(1, bind_group, &[]);
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                    }
                    OrderedDraw::Mask(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Mask) {
                            pass.set_pipeline(mask_pipeline);
                            pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Mask;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        let Some((_, bind_group)) = self.image_bind_groups.get(&draw.image) else {
                            i += 1;
                            continue;
                        };
                        pass.set_bind_group(1, bind_group, &[]);
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                    }
                    OrderedDraw::Text(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Text) {
                            pass.set_pipeline(text_pipeline);
                            pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                            pass.set_bind_group(1, self.text_system.atlas_bind_group(), &[]);
                            active_pipeline = ActivePipeline::Text;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                    }
                    OrderedDraw::Path(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Path) {
                            pass.set_pipeline(path_pipeline);
                            pass.set_vertex_buffer(0, path_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Path;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                    }
                }

                i += 1;
            }
        }

        let cmd = encoder.finish();

        // Keep the most recent encoding for potential reuse on the next frame.
        if cache_hit {
            self.scene_encoding_cache_key = Some(key);
        }
        self.scene_encoding_cache = encoding;
        cmd
    }

    fn prepare_viewport_bind_groups(&mut self, device: &wgpu::Device, draws: &[OrderedDraw]) {
        for item in draws {
            let OrderedDraw::Viewport(draw) = item else {
                continue;
            };

            let target = draw.target;
            let Some(view) = self.render_targets.get(target) else {
                continue;
            };

            let revision = self
                .render_target_revisions
                .get(&target)
                .copied()
                .unwrap_or(0);
            let needs_rebuild = match self.viewport_bind_groups.get(&target) {
                Some((cached, _)) => *cached != revision,
                None => true,
            };
            if !needs_rebuild {
                continue;
            }

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret viewport texture bind group"),
                layout: &self.viewport_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.viewport_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                ],
            });

            self.viewport_bind_groups
                .insert(target, (revision, bind_group));
        }
    }

    fn prepare_image_bind_groups(&mut self, device: &wgpu::Device, draws: &[OrderedDraw]) {
        for item in draws {
            let image = match item {
                OrderedDraw::Image(draw) => draw.image,
                OrderedDraw::Mask(draw) => draw.image,
                _ => continue,
            };
            let Some(view) = self.images.get(image) else {
                continue;
            };

            let revision = self.image_revisions.get(&image).copied().unwrap_or(0);
            let needs_rebuild = match self.image_bind_groups.get(&image) {
                Some((cached, _)) => *cached != revision,
                None => true,
            };
            if !needs_rebuild {
                continue;
            }

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fret image texture bind group"),
                layout: &self.viewport_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.viewport_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                ],
            });

            self.image_bind_groups.insert(image, (revision, bind_group));
        }
    }

    fn encode_scene_ops_into(
        &mut self,
        scene: &Scene,
        scale_factor: f32,
        viewport_size: (u32, u32),
        encoding: &mut SceneEncoding,
    ) {
        encoding.clear();
        let instances = &mut encoding.instances;
        let viewport_vertices = &mut encoding.viewport_vertices;
        let text_vertices = &mut encoding.text_vertices;
        let path_vertices = &mut encoding.path_vertices;
        let uniforms = &mut encoding.uniforms;
        let ordered_draws = &mut encoding.ordered_draws;

        let mut scissor_stack: Vec<ScissorRect> =
            vec![ScissorRect::full(viewport_size.0, viewport_size.1)];

        let mut current_scissor = *scissor_stack
            .last()
            .expect("scissor stack must be non-empty");

        let mut active_rounded_clips: Vec<ClipRRectUniform> = Vec::new();
        let mut clip_kind_stack: Vec<bool> = Vec::new();

        uniforms.push(ViewportUniform {
            viewport_size: [viewport_size.0 as f32, viewport_size.1 as f32],
            clip_count: 0,
            _pad0: 0,
            clips: [ClipRRectUniform::zeroed(); MAX_CLIPS],
        });
        let mut current_uniform_index: u32 = 0;

        let mut quad_batch: Option<(ScissorRect, u32, u32)> = None;

        macro_rules! flush_quad_batch {
            () => {{
                if let Some((scissor, uniform_index, first_instance)) = quad_batch.take() {
                    let instance_count = (instances.len() as u32).saturating_sub(first_instance);
                    if instance_count > 0 {
                        ordered_draws.push(OrderedDraw::Quad(DrawCall {
                            scissor,
                            uniform_index,
                            first_instance,
                            instance_count,
                        }));
                    }
                }
            }};
        }

        for op in scene.ops() {
            match op {
                SceneOp::PushClipRect { rect } => {
                    let Some(new_scissor) = scissor_from_rect(*rect, scale_factor, viewport_size)
                    else {
                        continue;
                    };

                    let combined = intersect_scissor(current_scissor, new_scissor);
                    if combined != current_scissor {
                        flush_quad_batch!();
                    }

                    current_scissor = combined;
                    scissor_stack.push(current_scissor);
                    clip_kind_stack.push(false);
                }
                SceneOp::PushClipRRect { rect, corner_radii } => {
                    let Some(new_scissor) = scissor_from_rect(*rect, scale_factor, viewport_size)
                    else {
                        continue;
                    };

                    let combined = intersect_scissor(current_scissor, new_scissor);
                    if combined != current_scissor {
                        flush_quad_batch!();
                    }

                    current_scissor = combined;
                    scissor_stack.push(current_scissor);

                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    let radii = clamp_corner_radii_for_rect(w, h, corners_to_vec4(*corner_radii));
                    let is_rrect = radii.iter().any(|v| *v > 0.0);
                    if is_rrect && active_rounded_clips.len() < MAX_CLIPS {
                        flush_quad_batch!();

                        active_rounded_clips.push(ClipRRectUniform {
                            rect: [x, y, w, h],
                            corner_radii: radii,
                        });

                        let mut clips = [ClipRRectUniform::zeroed(); MAX_CLIPS];
                        for (i, clip) in active_rounded_clips.iter().enumerate() {
                            clips[i] = *clip;
                        }
                        current_uniform_index = uniforms.len() as u32;
                        uniforms.push(ViewportUniform {
                            viewport_size: [viewport_size.0 as f32, viewport_size.1 as f32],
                            clip_count: active_rounded_clips.len() as u32,
                            _pad0: 0,
                            clips,
                        });
                        clip_kind_stack.push(true);
                    } else {
                        clip_kind_stack.push(false);
                    }
                }
                SceneOp::PopClip => {
                    if scissor_stack.len() > 1 {
                        scissor_stack.pop();
                        let new_scissor = *scissor_stack
                            .last()
                            .expect("scissor stack must be non-empty");
                        if new_scissor != current_scissor {
                            flush_quad_batch!();
                            current_scissor = new_scissor;
                        }
                    }

                    if let Some(was_rrect) = clip_kind_stack.pop()
                        && was_rrect
                    {
                        flush_quad_batch!();
                        active_rounded_clips.pop();

                        let mut clips = [ClipRRectUniform::zeroed(); MAX_CLIPS];
                        for (i, clip) in active_rounded_clips.iter().enumerate() {
                            clips[i] = *clip;
                        }
                        current_uniform_index = uniforms.len() as u32;
                        uniforms.push(ViewportUniform {
                            viewport_size: [viewport_size.0 as f32, viewport_size.1 as f32],
                            clip_count: active_rounded_clips.len() as u32,
                            _pad0: 0,
                            clips,
                        });
                    }
                }
                SceneOp::Quad {
                    rect,
                    background,
                    border,
                    border_color,
                    corner_radii,
                    ..
                } => {
                    if background.a <= 0.0 && border_color.a <= 0.0 {
                        continue;
                    }
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }

                    let needs_new_batch = match quad_batch {
                        Some((scissor, uniform_index, _)) => {
                            scissor != current_scissor || uniform_index != current_uniform_index
                        }
                        None => true,
                    };

                    if needs_new_batch {
                        flush_quad_batch!();
                        quad_batch = Some((
                            current_scissor,
                            current_uniform_index,
                            instances.len() as u32,
                        ));
                    }

                    let corner_radii =
                        clamp_corner_radii_for_rect(w, h, corners_to_vec4(*corner_radii));
                    instances.push(QuadInstance {
                        rect: [x, y, w, h],
                        color: color_to_linear_rgba_premul(*background),
                        corner_radii,
                        border: edges_to_vec4(*border),
                        border_color: color_to_linear_rgba_premul(*border_color),
                    });
                }
                SceneOp::Image { .. } => {
                    flush_quad_batch!();
                    let SceneOp::Image {
                        rect,
                        image,
                        opacity,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    if *opacity <= 0.0 {
                        continue;
                    }
                    if self.images.get(*image).is_none() {
                        continue;
                    }
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }

                    let first_vertex = viewport_vertices.len() as u32;
                    let o = opacity.clamp(0.0, 1.0);

                    let x0 = x;
                    let y0 = y;
                    let x1 = x + w;
                    let y1 = y + h;

                    viewport_vertices.extend_from_slice(&[
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y0],
                            uv: [1.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y1],
                            uv: [0.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                    ]);

                    ordered_draws.push(OrderedDraw::Image(ImageDraw {
                        scissor: current_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        image: *image,
                    }));
                }
                SceneOp::ImageRegion { .. } => {
                    flush_quad_batch!();
                    let SceneOp::ImageRegion {
                        rect,
                        image,
                        uv,
                        opacity,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    if *opacity <= 0.0 {
                        continue;
                    }
                    if self.images.get(*image).is_none() {
                        continue;
                    }
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }

                    let first_vertex = viewport_vertices.len() as u32;
                    let o = opacity.clamp(0.0, 1.0);

                    let x0 = x;
                    let y0 = y;
                    let x1 = x + w;
                    let y1 = y + h;

                    let (u0, v0, u1, v1) = (uv.u0, uv.v0, uv.u1, uv.v1);
                    viewport_vertices.extend_from_slice(&[
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [u0, v0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y0],
                            uv: [u1, v0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [u1, v1],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [u0, v0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [u1, v1],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y1],
                            uv: [u0, v1],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                    ]);

                    ordered_draws.push(OrderedDraw::Image(ImageDraw {
                        scissor: current_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        image: *image,
                    }));
                }
                SceneOp::MaskImage { .. } => {
                    flush_quad_batch!();
                    let SceneOp::MaskImage {
                        rect,
                        image,
                        uv,
                        color,
                        opacity,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    if *opacity <= 0.0 || color.a <= 0.0 {
                        continue;
                    }
                    if self.images.get(*image).is_none() {
                        continue;
                    }
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }

                    let first_vertex = text_vertices.len() as u32;
                    let o = opacity.clamp(0.0, 1.0);
                    let mut premul = color_to_linear_rgba_premul(*color);
                    premul = premul.map(|c| c * o);

                    let x0 = x;
                    let y0 = y;
                    let x1 = x + w;
                    let y1 = y + h;

                    let (u0, v0, u1, v1) = (uv.u0, uv.v0, uv.u1, uv.v1);
                    text_vertices.extend_from_slice(&[
                        TextVertex {
                            pos_px: [x0, y0],
                            uv: [u0, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [x1, y0],
                            uv: [u1, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [x1, y1],
                            uv: [u1, v1],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [x0, y0],
                            uv: [u0, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [x1, y1],
                            uv: [u1, v1],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [x0, y1],
                            uv: [u0, v1],
                            color: premul,
                        },
                    ]);

                    ordered_draws.push(OrderedDraw::Mask(MaskDraw {
                        scissor: current_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        image: *image,
                    }));
                }
                SceneOp::SvgMaskIcon { .. } => {
                    flush_quad_batch!();
                    let SceneOp::SvgMaskIcon {
                        rect,
                        svg,
                        fit,
                        color,
                        opacity,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    if *opacity <= 0.0 || color.a <= 0.0 {
                        continue;
                    }

                    let key = Self::svg_raster_key(
                        *svg,
                        *rect,
                        scale_factor,
                        SvgRasterKind::AlphaMask,
                        *fit,
                    );
                    let Some(entry) = self.svg_rasters.get(&key) else {
                        continue;
                    };
                    if self.images.get(entry.image).is_none() {
                        continue;
                    }

                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }

                    let first_vertex = text_vertices.len() as u32;
                    let o = opacity.clamp(0.0, 1.0);
                    let mut premul = color_to_linear_rgba_premul(*color);
                    premul = premul.map(|c| c * o);

                    let (x0, y0, x1, y1) =
                        svg_draw_rect_px(x, y, w, h, entry.size_px, SMOOTH_SVG_SCALE_FACTOR, *fit);

                    let (u0, v0, u1, v1) = (entry.uv.u0, entry.uv.v0, entry.uv.u1, entry.uv.v1);
                    text_vertices.extend_from_slice(&[
                        TextVertex {
                            pos_px: [x0, y0],
                            uv: [u0, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [x1, y0],
                            uv: [u1, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [x1, y1],
                            uv: [u1, v1],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [x0, y0],
                            uv: [u0, v0],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [x1, y1],
                            uv: [u1, v1],
                            color: premul,
                        },
                        TextVertex {
                            pos_px: [x0, y1],
                            uv: [u0, v1],
                            color: premul,
                        },
                    ]);

                    let svg_scissor = scissor_from_rect(*rect, scale_factor, viewport_size)
                        .map(|s| intersect_scissor(current_scissor, s))
                        .unwrap_or(current_scissor);
                    ordered_draws.push(OrderedDraw::Mask(MaskDraw {
                        scissor: svg_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        image: entry.image,
                    }));
                }
                SceneOp::SvgImage { .. } => {
                    flush_quad_batch!();
                    let SceneOp::SvgImage {
                        rect,
                        svg,
                        fit,
                        opacity,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    if *opacity <= 0.0 {
                        continue;
                    }

                    let key =
                        Self::svg_raster_key(*svg, *rect, scale_factor, SvgRasterKind::Rgba, *fit);
                    let Some(entry) = self.svg_rasters.get(&key) else {
                        continue;
                    };
                    if self.images.get(entry.image).is_none() {
                        continue;
                    }

                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }

                    let first_vertex = viewport_vertices.len() as u32;
                    let o = opacity.clamp(0.0, 1.0);

                    let (x0, y0, x1, y1) =
                        svg_draw_rect_px(x, y, w, h, entry.size_px, SMOOTH_SVG_SCALE_FACTOR, *fit);

                    viewport_vertices.extend_from_slice(&[
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y0],
                            uv: [1.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y1],
                            uv: [0.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                    ]);

                    let svg_scissor = scissor_from_rect(*rect, scale_factor, viewport_size)
                        .map(|s| intersect_scissor(current_scissor, s))
                        .unwrap_or(current_scissor);
                    ordered_draws.push(OrderedDraw::Image(ImageDraw {
                        scissor: svg_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        image: entry.image,
                    }));
                }
                SceneOp::Text {
                    origin,
                    text,
                    color,
                    ..
                } => {
                    flush_quad_batch!();

                    let Some(blob) = self.text_system.blob(*text) else {
                        continue;
                    };

                    let first_vertex = text_vertices.len() as u32;

                    let base_x = origin.x.0 * scale_factor;
                    let base_y = origin.y.0 * scale_factor;
                    let premul = color_to_linear_rgba_premul(*color);

                    for g in &blob.glyphs {
                        let x0 = base_x + g.rect[0] * scale_factor;
                        let y0 = base_y + g.rect[1] * scale_factor;
                        let x1 = x0 + g.rect[2] * scale_factor;
                        let y1 = y0 + g.rect[3] * scale_factor;

                        let (u0, v0, u1, v1) = (g.uv[0], g.uv[1], g.uv[2], g.uv[3]);

                        text_vertices.extend_from_slice(&[
                            TextVertex {
                                pos_px: [x0, y0],
                                uv: [u0, v0],
                                color: premul,
                            },
                            TextVertex {
                                pos_px: [x1, y0],
                                uv: [u1, v0],
                                color: premul,
                            },
                            TextVertex {
                                pos_px: [x1, y1],
                                uv: [u1, v1],
                                color: premul,
                            },
                            TextVertex {
                                pos_px: [x0, y0],
                                uv: [u0, v0],
                                color: premul,
                            },
                            TextVertex {
                                pos_px: [x1, y1],
                                uv: [u1, v1],
                                color: premul,
                            },
                            TextVertex {
                                pos_px: [x0, y1],
                                uv: [u0, v1],
                                color: premul,
                            },
                        ]);
                    }

                    let vertex_count = (text_vertices.len() as u32).saturating_sub(first_vertex);
                    if vertex_count > 0 {
                        ordered_draws.push(OrderedDraw::Text(TextDraw {
                            scissor: current_scissor,
                            uniform_index: current_uniform_index,
                            first_vertex,
                            vertex_count,
                        }));
                    }
                }
                SceneOp::Path { .. } => {
                    flush_quad_batch!();
                    let SceneOp::Path {
                        origin,
                        path,
                        color,
                        ..
                    } = op
                    else {
                        unreachable!();
                    };
                    if color.a <= 0.0 {
                        continue;
                    }
                    let Some(prepared) = self.paths.get(*path) else {
                        continue;
                    };
                    if prepared.triangles.is_empty() {
                        continue;
                    }

                    let path_bounds = Rect::new(
                        fret_core::Point::new(
                            origin.x + prepared.metrics.bounds.origin.x,
                            origin.y + prepared.metrics.bounds.origin.y,
                        ),
                        prepared.metrics.bounds.size,
                    );
                    let Some(bounds_scissor) =
                        scissor_from_rect(path_bounds, scale_factor, viewport_size)
                    else {
                        continue;
                    };
                    let clipped_scissor = intersect_scissor(current_scissor, bounds_scissor);
                    if clipped_scissor.w == 0 || clipped_scissor.h == 0 {
                        continue;
                    }

                    let first_vertex = path_vertices.len() as u32;
                    let ox = origin.x.0 * scale_factor;
                    let oy = origin.y.0 * scale_factor;
                    let premul = color_to_linear_rgba_premul(*color);

                    for p in &prepared.triangles {
                        path_vertices.push(PathVertex {
                            pos_px: [ox + p[0] * scale_factor, oy + p[1] * scale_factor],
                            color: premul,
                        });
                    }

                    let vertex_count = (path_vertices.len() as u32).saturating_sub(first_vertex);
                    if vertex_count > 0 {
                        ordered_draws.push(OrderedDraw::Path(PathDraw {
                            scissor: clipped_scissor,
                            uniform_index: current_uniform_index,
                            first_vertex,
                            vertex_count,
                        }));
                    }
                }
                SceneOp::ViewportSurface {
                    rect,
                    target,
                    opacity,
                    ..
                } => {
                    flush_quad_batch!();
                    if *opacity <= 0.0 {
                        continue;
                    }
                    if self.render_targets.get(*target).is_none() {
                        continue;
                    }
                    let (x, y, w, h) = rect_to_pixels(*rect, scale_factor);
                    if w <= 0.0 || h <= 0.0 {
                        continue;
                    }

                    let first_vertex = viewport_vertices.len() as u32;
                    let o = opacity.clamp(0.0, 1.0);

                    let x0 = x;
                    let y0 = y;
                    let x1 = x + w;
                    let y1 = y + h;

                    viewport_vertices.extend_from_slice(&[
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y0],
                            uv: [1.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [0.0, 0.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [1.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y1],
                            uv: [0.0, 1.0],
                            opacity: o,
                            _pad: [0.0; 3],
                        },
                    ]);

                    ordered_draws.push(OrderedDraw::Viewport(ViewportDraw {
                        scissor: current_scissor,
                        uniform_index: current_uniform_index,
                        first_vertex,
                        vertex_count: 6,
                        target: *target,
                    }));
                }
            }
        }

        flush_quad_batch!();
    }
}

impl fret_core::TextService for Renderer {
    fn prepare(
        &mut self,
        text: &str,
        style: fret_core::TextStyle,
        constraints: fret_core::TextConstraints,
    ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
        self.text_system.prepare(text, style, constraints)
    }

    fn measure(
        &mut self,
        text: &str,
        style: fret_core::TextStyle,
        constraints: fret_core::TextConstraints,
    ) -> fret_core::TextMetrics {
        self.text_system.measure(text, style, constraints)
    }

    fn caret_x(&mut self, blob: fret_core::TextBlobId, index: usize) -> fret_core::Px {
        self.text_system
            .caret_x(blob, index)
            .unwrap_or(fret_core::Px(0.0))
    }

    fn hit_test_x(&mut self, blob: fret_core::TextBlobId, x: fret_core::Px) -> usize {
        self.text_system.hit_test_x(blob, x).unwrap_or(0)
    }

    fn selection_rects(
        &mut self,
        blob: fret_core::TextBlobId,
        range: (usize, usize),
        out: &mut Vec<fret_core::Rect>,
    ) {
        let _ = self.text_system.selection_rects(blob, range, out);
    }

    fn caret_stops(&mut self, blob: fret_core::TextBlobId, out: &mut Vec<(usize, fret_core::Px)>) {
        out.clear();
        if let Some(stops) = self.text_system.caret_stops(blob) {
            out.extend_from_slice(stops);
        }
    }

    fn caret_rect(
        &mut self,
        blob: fret_core::TextBlobId,
        index: usize,
        affinity: fret_core::CaretAffinity,
    ) -> fret_core::Rect {
        self.text_system
            .caret_rect(blob, index, affinity)
            .unwrap_or_default()
    }

    fn hit_test_point(
        &mut self,
        blob: fret_core::TextBlobId,
        point: fret_core::Point,
    ) -> fret_core::HitTestResult {
        self.text_system
            .hit_test_point(blob, point)
            .unwrap_or(fret_core::HitTestResult {
                index: 0,
                affinity: fret_core::CaretAffinity::Downstream,
            })
    }

    fn release(&mut self, blob: fret_core::TextBlobId) {
        self.text_system.release(blob);
    }
}

impl fret_core::PathService for Renderer {
    fn prepare(
        &mut self,
        commands: &[fret_core::PathCommand],
        style: fret_core::PathStyle,
        constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        let key = path_cache_key(commands, style, constraints);
        let epoch = self.bump_path_cache_epoch();

        match self.path_cache.entry(key) {
            Entry::Occupied(mut e) => {
                let entry = e.get_mut();
                entry.refs = entry.refs.saturating_add(1);
                entry.last_used_epoch = epoch;
                let id = entry.id;

                if let Some(prepared) = self.paths.get(id) {
                    return (id, prepared.metrics);
                }

                // Cache entry is stale (should be rare). Rebuild it.
                e.remove();
            }
            Entry::Vacant(_) => {}
        }

        let metrics = metrics_from_path_commands(commands, style);
        let triangles = tessellate_path_commands(commands, style, constraints);
        let id = self.paths.insert(PreparedPath {
            metrics,
            triangles,
            cache_key: key,
        });
        self.path_cache.insert(
            key,
            CachedPathEntry {
                id,
                refs: 1,
                last_used_epoch: epoch,
            },
        );
        self.prune_path_cache();
        (id, metrics)
    }

    fn release(&mut self, path: fret_core::PathId) {
        let Some(cache_key) = self.paths.get(path).map(|p| p.cache_key) else {
            return;
        };

        if let Some(entry) = self.path_cache.get_mut(&cache_key) {
            if entry.refs > 0 {
                entry.refs -= 1;
            }
        }

        self.prune_path_cache();
    }
}

impl fret_core::SvgService for Renderer {
    fn register_svg(&mut self, bytes: &[u8]) -> fret_core::SvgId {
        let h = hash_bytes(bytes);
        if let Some(ids) = self.svg_hash_index.get(&h) {
            for &id in ids {
                let Some(existing) = self.svgs.get(id) else {
                    continue;
                };
                if existing.as_ref() == bytes {
                    return id;
                }
            }
        }

        let id = self.svgs.insert(Arc::<[u8]>::from(bytes));
        self.svg_hash_index.entry(h).or_default().push(id);
        id
    }

    fn unregister_svg(&mut self, svg: fret_core::SvgId) -> bool {
        let Some(bytes) = self.svgs.remove(svg) else {
            return false;
        };

        let h = hash_bytes(&bytes);
        if let Some(list) = self.svg_hash_index.get_mut(&h) {
            list.retain(|id| *id != svg);
            if list.is_empty() {
                self.svg_hash_index.remove(&h);
            }
        }

        // Drop any cached rasterizations for this SVG.
        let mut keys_to_remove: Vec<SvgRasterKey> = Vec::new();
        for k in self.svg_rasters.keys() {
            if k.svg == svg {
                keys_to_remove.push(*k);
            }
        }
        for k in keys_to_remove {
            if let Some(entry) = self.svg_rasters.remove(&k) {
                self.drop_svg_raster_entry(entry);
            }
        }

        true
    }
}

const QUAD_SHADER: &str = r#"
const MAX_CLIPS: u32 = 8u;

struct ClipRRect {
  rect: vec4<f32>,
  corner_radii: vec4<f32>,
};

struct Viewport {
  viewport_size: vec2<f32>,
  clip_count: u32,
  _pad0: u32,
  clips: array<ClipRRect, MAX_CLIPS>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

struct QuadInstance {
  rect: vec4<f32>,
  color: vec4<f32>,
  corner_radii: vec4<f32>,
  border: vec4<f32>,
  border_color: vec4<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) pixel_pos: vec2<f32>,
  @location(1) rect_origin: vec2<f32>,
  @location(2) rect_size: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) corner_radii: vec4<f32>,
  @location(5) border: vec4<f32>,
  @location(6) border_color: vec4<f32>,
};

fn quad_vertex_xy(vertex_index: u32) -> vec2<f32> {
  switch vertex_index {
    case 0u: { return vec2<f32>(0.0, 0.0); }
    case 1u: { return vec2<f32>(1.0, 0.0); }
    case 2u: { return vec2<f32>(1.0, 1.0); }
    case 3u: { return vec2<f32>(0.0, 0.0); }
    case 4u: { return vec2<f32>(1.0, 1.0); }
    default: { return vec2<f32>(0.0, 1.0); }
  }
}

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

@vertex
fn vs_main(
  @builtin(vertex_index) vertex_index: u32,
  @location(0) rect: vec4<f32>,
  @location(1) color: vec4<f32>,
  @location(2) corner_radii: vec4<f32>,
  @location(3) border: vec4<f32>,
  @location(4) border_color: vec4<f32>,
) -> VsOut {
  let uv = quad_vertex_xy(vertex_index);
  let pixel_pos = rect.xy + uv * rect.zw;
  let clip_xy = to_clip_space(pixel_pos);

  var out: VsOut;
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.pixel_pos = pixel_pos;
  out.rect_origin = rect.xy;
  out.rect_size = rect.zw;
  out.color = color;
  out.corner_radii = corner_radii;
  out.border = border;
  out.border_color = border_color;
  return out;
}

fn pick_corner_radius(center_to_point: vec2<f32>, radii: vec4<f32>) -> f32 {
  if (center_to_point.x < 0.0) {
    if (center_to_point.y < 0.0) { return radii.x; }
    return radii.w;
  }
  if (center_to_point.y < 0.0) { return radii.y; }
  return radii.z;
}

fn quad_sdf_impl(corner_center_to_point: vec2<f32>, corner_radius: f32) -> f32 {
  if (corner_radius == 0.0) {
    return max(corner_center_to_point.x, corner_center_to_point.y);
  }
  let signed_distance_to_inset_quad =
    length(max(vec2<f32>(0.0), corner_center_to_point)) +
    min(0.0, max(corner_center_to_point.x, corner_center_to_point.y));
  return signed_distance_to_inset_quad - corner_radius;
}

fn quad_sdf(point: vec2<f32>, rect_origin: vec2<f32>, rect_size: vec2<f32>, corner_radii: vec4<f32>) -> f32 {
  let center = rect_origin + rect_size * 0.5;
  let center_to_point = point - center;
  let half_size = rect_size * 0.5;
  let corner_radius = pick_corner_radius(center_to_point, corner_radii);
  let corner_to_point = abs(center_to_point) - half_size;
  let corner_center_to_point = corner_to_point + corner_radius;
  return quad_sdf_impl(corner_center_to_point, corner_radius);
}

fn saturate(x: f32) -> f32 {
  return clamp(x, 0.0, 1.0);
}

fn clip_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  for (var i = 0u; i < MAX_CLIPS; i = i + 1u) {
    if (i >= viewport.clip_count) {
      break;
    }
    let clip = viewport.clips[i];
    let sdf = quad_sdf(pixel_pos, clip.rect.xy, clip.rect.zw, clip.corner_radii);
    let aa = max(fwidth(sdf), 1e-4);
    let a = 1.0 - smoothstep(-aa, aa, sdf);
    alpha = alpha * a;
    if (alpha <= 0.0) {
      break;
    }
  }
  return alpha;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  if (clip <= 0.0) {
    discard;
  }

  let outer_sdf = quad_sdf(input.pixel_pos, input.rect_origin, input.rect_size, input.corner_radii);

  // NOTE: AA must scale with derivatives. A fixed threshold (e.g. 0.5) breaks under DPI changes
  // and transforms. See ADR 0030.
  let aa_outer = max(fwidth(outer_sdf), 1e-4);
  let alpha_outer = 1.0 - smoothstep(-aa_outer, aa_outer, outer_sdf);

  let border_sum = input.border.x + input.border.y + input.border.z + input.border.w;
  if (border_sum <= 0.0) {
    return vec4<f32>(input.color.rgb, input.color.a) * (alpha_outer * clip);
  }

  // Border alignment: inside. Inner radii are derived by subtracting adjacent border widths.
  let inner_origin = input.rect_origin + vec2<f32>(input.border.x, input.border.y);
  let inner_size = input.rect_size - vec2<f32>(input.border.x + input.border.z, input.border.y + input.border.w);

  let inner_radii = max(
    vec4<f32>(0.0),
    vec4<f32>(
      input.corner_radii.x - max(input.border.x, input.border.y), // TL
      input.corner_radii.y - max(input.border.z, input.border.y), // TR
      input.corner_radii.z - max(input.border.z, input.border.w), // BR
      input.corner_radii.w - max(input.border.x, input.border.w)  // BL
    )
  );

  var alpha_inner = 0.0;
  if (inner_size.x > 0.0 && inner_size.y > 0.0) {
    let inner_sdf = quad_sdf(input.pixel_pos, inner_origin, inner_size, inner_radii);
    let aa_inner = max(fwidth(inner_sdf), 1e-4);
    alpha_inner = 1.0 - smoothstep(-aa_inner, aa_inner, inner_sdf);
  }

  let border_cov = saturate(alpha_outer - alpha_inner);
  let fill = vec4<f32>(input.color.rgb, input.color.a) * alpha_inner;
  let border = vec4<f32>(input.border_color.rgb, input.border_color.a) * border_cov;

  return (fill + border) * clip;
}
"#;

const VIEWPORT_SHADER: &str = r#"
const MAX_CLIPS: u32 = 8u;

struct ClipRRect {
  rect: vec4<f32>,
  corner_radii: vec4<f32>,
};

struct Viewport {
  viewport_size: vec2<f32>,
  clip_count: u32,
  _pad0: u32,
  clips: array<ClipRRect, MAX_CLIPS>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

@group(1) @binding(0) var viewport_sampler: sampler;
@group(1) @binding(1) var viewport_texture: texture_2d<f32>;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) opacity: f32,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) opacity: f32,
  @location(2) pixel_pos: vec2<f32>,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

fn pick_corner_radius(center_to_point: vec2<f32>, radii: vec4<f32>) -> f32 {
  if (center_to_point.x < 0.0) {
    if (center_to_point.y < 0.0) { return radii.x; }
    return radii.w;
  }
  if (center_to_point.y < 0.0) { return radii.y; }
  return radii.z;
}

fn quad_sdf_impl(corner_center_to_point: vec2<f32>, corner_radius: f32) -> f32 {
  if (corner_radius == 0.0) {
    return max(corner_center_to_point.x, corner_center_to_point.y);
  }
  let signed_distance_to_inset_quad =
    length(max(vec2<f32>(0.0), corner_center_to_point)) +
    min(0.0, max(corner_center_to_point.x, corner_center_to_point.y));
  return signed_distance_to_inset_quad - corner_radius;
}

fn quad_sdf(point: vec2<f32>, rect_origin: vec2<f32>, rect_size: vec2<f32>, corner_radii: vec4<f32>) -> f32 {
  let center = rect_origin + rect_size * 0.5;
  let center_to_point = point - center;
  let half_size = rect_size * 0.5;
  let corner_radius = pick_corner_radius(center_to_point, corner_radii);
  let corner_to_point = abs(center_to_point) - half_size;
  let corner_center_to_point = corner_to_point + corner_radius;
  return quad_sdf_impl(corner_center_to_point, corner_radius);
}

fn clip_alpha(pixel_pos: vec2<f32>) -> f32 {
  var alpha = 1.0;
  for (var i = 0u; i < MAX_CLIPS; i = i + 1u) {
    if (i >= viewport.clip_count) {
      break;
    }
    let clip = viewport.clips[i];
    let sdf = quad_sdf(pixel_pos, clip.rect.xy, clip.rect.zw, clip.corner_radii);
    let aa = max(fwidth(sdf), 1e-4);
    let a = 1.0 - smoothstep(-aa, aa, sdf);
    alpha = alpha * a;
    if (alpha <= 0.0) {
      break;
    }
  }
  return alpha;
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.opacity = input.opacity;
  out.pixel_pos = input.pos_px;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let clip = clip_alpha(input.pixel_pos);
  if (clip <= 0.0) {
    discard;
  }
  let tex = textureSample(viewport_texture, viewport_sampler, input.uv);
  let a = tex.a * input.opacity * clip;
  return vec4<f32>(tex.rgb * a, a);
}
"#;

const COMPOSITE_PREMUL_SHADER: &str = r#"
struct Viewport {
  viewport_size: vec2<f32>,
  _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

@group(1) @binding(0) var tex_sampler: sampler;
@group(1) @binding(1) var tex: texture_2d<f32>;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) opacity: f32,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) opacity: f32,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.opacity = input.opacity;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let sample = textureSample(tex, tex_sampler, input.uv);
  let o = clamp(input.opacity, 0.0, 1.0);
  return vec4<f32>(sample.rgb * o, sample.a * o);
}
"#;

const PATH_SHADER: &str = r#"
struct Viewport {
  viewport_size: vec2<f32>,
  _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) color: vec4<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) color: vec4<f32>,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.color = input.color;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  return input.color;
}
"#;

const TEXT_SHADER: &str = r#"
struct Viewport {
  viewport_size: vec2<f32>,
  _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

@group(1) @binding(0) var glyph_sampler: sampler;
@group(1) @binding(1) var glyph_atlas: texture_2d<f32>;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) color: vec4<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.color = input.color;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let tex = textureSample(glyph_atlas, glyph_sampler, input.uv);
  let coverage = tex.r;
  return vec4<f32>(input.color.rgb * coverage, input.color.a * coverage);
}
"#;

const MASK_SHADER: &str = r#"
struct Viewport {
  viewport_size: vec2<f32>,
  _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> viewport: Viewport;

@group(1) @binding(0) var mask_sampler: sampler;
@group(1) @binding(1) var mask_texture: texture_2d<f32>;

struct VsIn {
  @location(0) pos_px: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) color: vec4<f32>,
};

struct VsOut {
  @builtin(position) clip_pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
};

fn to_clip_space(pixel_pos: vec2<f32>) -> vec2<f32> {
  let ndc_x = (pixel_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
  let ndc_y = 1.0 - (pixel_pos.y / viewport.viewport_size.y) * 2.0;
  return vec2<f32>(ndc_x, ndc_y);
}

@vertex
fn vs_main(input: VsIn) -> VsOut {
  var out: VsOut;
  let clip_xy = to_clip_space(input.pos_px);
  out.clip_pos = vec4<f32>(clip_xy, 0.0, 1.0);
  out.uv = input.uv;
  out.color = input.color;
  return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
  let tex = textureSample(mask_texture, mask_sampler, input.uv);
  let coverage = tex.r;
  return vec4<f32>(input.color.rgb * coverage, input.color.a * coverage);
}
"#;

#[cfg(test)]
mod tests {
    use super::{
        PATH_SHADER, QUAD_SHADER, TEXT_SHADER, VIEWPORT_SHADER, clamp_corner_radii_for_rect,
        svg_draw_rect_px,
    };

    #[test]
    fn shaders_parse_as_wgsl() {
        for (name, src) in [
            ("viewport", VIEWPORT_SHADER),
            ("quad", QUAD_SHADER),
            ("path", PATH_SHADER),
            ("text", TEXT_SHADER),
        ] {
            naga::front::wgsl::parse_str(src)
                .unwrap_or_else(|err| panic!("WGSL parse failed for {name} shader: {err}"));
        }
    }

    #[test]
    fn corner_radii_are_clamped_to_half_min_rect_dim() {
        let radii = clamp_corner_radii_for_rect(100.0, 6.0, [999.0, 999.0, 999.0, 999.0]);
        assert_eq!(radii, [3.0, 3.0, 3.0, 3.0]);
    }

    #[test]
    fn corner_radii_clamp_is_nan_safe() {
        let radii = clamp_corner_radii_for_rect(f32::NAN, 6.0, [999.0, -1.0, f32::NAN, 0.0]);
        assert_eq!(radii, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn svg_draw_rect_centers_contained_raster() {
        // target 100x50, raster 100x100 at smooth=2 => draw 50x50 centered.
        let (x0, y0, x1, y1) = svg_draw_rect_px(
            0.0,
            0.0,
            100.0,
            50.0,
            (100, 100),
            2.0,
            fret_core::SvgFit::Contain,
        );
        assert_eq!((x0, y0, x1, y1), (25.0, 0.0, 75.0, 50.0));
    }

    #[test]
    fn svg_draw_rect_width_can_overflow_height() {
        // target 50x50, raster 100x200 at smooth=2 => draw 50x100, centered (overflows vertically).
        let (x0, y0, x1, y1) = svg_draw_rect_px(
            0.0,
            0.0,
            50.0,
            50.0,
            (100, 200),
            2.0,
            fret_core::SvgFit::Width,
        );
        assert_eq!((x0, y0, x1, y1), (0.0, -25.0, 50.0, 75.0));
    }
}
