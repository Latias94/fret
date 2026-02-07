use fret_core::geometry::{Point, Px, Rect, Size};
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};

use super::{mix_f32, mix_u64};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct PathCacheKey {
    pub(super) commands_hash: u64,
    pub(super) commands_len: u32,
    pub(super) style_key: u64,
    pub(super) scale_factor_bits: u32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CachedPathEntry {
    pub(super) id: fret_core::PathId,
    pub(super) refs: u32,
    pub(super) last_used_epoch: u64,
}

#[derive(Debug)]
pub(super) struct PreparedPath {
    pub(super) metrics: fret_core::PathMetrics,
    pub(super) triangles: Vec<[f32; 2]>,
    pub(super) cache_key: PathCacheKey,
}

fn mix_path_style(state: u64, style: fret_core::PathStyle) -> u64 {
    match style {
        fret_core::PathStyle::Fill(fill) => {
            let state = mix_u64(state, 0xF11);
            let rule = match fill.rule {
                fret_core::FillRule::NonZero => 1u64,
                fret_core::FillRule::EvenOdd => 2u64,
            };
            mix_u64(state, rule)
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

pub(super) fn path_cache_key(
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

pub(super) fn metrics_from_path_commands(
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
        bounds: Rect::new(Point::new(Px(min_x), Px(min_y)), Size::new(Px(w), Px(h))),
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

pub(super) fn tessellate_path_commands(
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
