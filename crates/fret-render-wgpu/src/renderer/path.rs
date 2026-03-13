use fret_core::geometry::{Point, Px, Rect, Size};
use lyon::path::iterator::PathIterator;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};
use slotmap::SlotMap;
use std::collections::HashMap;

use super::{mix_f32, mix_u64};

pub(super) struct PathState {
    paths: SlotMap<fret_core::PathId, PreparedPath>,
    path_cache: HashMap<PathCacheKey, CachedPathEntry>,
    path_cache_capacity: usize,
    path_cache_epoch: u64,
}

impl Default for PathState {
    fn default() -> Self {
        Self {
            paths: SlotMap::with_key(),
            path_cache: HashMap::new(),
            path_cache_capacity: 2048,
            path_cache_epoch: 0,
        }
    }
}

impl PathState {
    pub(super) fn prepared(&self, path: fret_core::PathId) -> Option<&PreparedPath> {
        self.paths.get(path)
    }

    pub(super) fn prepare_path(
        &mut self,
        commands: &[fret_core::PathCommand],
        style: fret_core::PathStyle,
        constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        let key = path_cache_key(commands, style, constraints);
        let epoch = self.bump_cache_epoch();

        match self.path_cache.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let cached = entry.get_mut();
                cached.refs = cached.refs.saturating_add(1);
                cached.last_used_epoch = epoch;
                let id = cached.id;

                if let Some(prepared) = self.paths.get(id) {
                    return (id, prepared.metrics);
                }

                entry.remove();
            }
            std::collections::hash_map::Entry::Vacant(_) => {}
        }

        let metrics = metrics_from_path_commands(commands, style);
        let (triangles, stroke_s01_mode) = tessellate_path_commands(commands, style, constraints);
        let id = self.paths.insert(PreparedPath {
            metrics,
            triangles,
            stroke_s01_mode,
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
        self.prune_cache();
        (id, metrics)
    }

    pub(super) fn release_path(&mut self, path: fret_core::PathId) {
        let Some(cache_key) = self.paths.get(path).map(|prepared| prepared.cache_key) else {
            return;
        };

        if let Some(entry) = self.path_cache.get_mut(&cache_key)
            && entry.refs > 0
        {
            entry.refs -= 1;
        }

        self.prune_cache();
    }

    fn bump_cache_epoch(&mut self) -> u64 {
        self.path_cache_epoch = self.path_cache_epoch.wrapping_add(1);
        self.path_cache_epoch
    }

    fn prune_cache(&mut self) {
        if self.path_cache.len() <= self.path_cache_capacity {
            return;
        }

        while self.path_cache.len() > self.path_cache_capacity {
            let mut victim: Option<(PathCacheKey, CachedPathEntry)> = None;
            for (key, value) in &self.path_cache {
                if value.refs != 0 {
                    continue;
                }
                let replace = match victim {
                    None => true,
                    Some((_, current)) => value.last_used_epoch < current.last_used_epoch,
                };
                if replace {
                    victim = Some((*key, *value));
                }
            }

            let Some((key, entry)) = victim else {
                break;
            };

            self.path_cache.remove(&key);
            self.paths.remove(entry.id);
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StrokeS01ModeV1 {
    None,
    /// Stroke-space arclength (`s01`) is continuous across the entire stroke.
    Continuous,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PathTriangleVertex {
    pub(super) pos: [f32; 2],
    pub(super) stroke_s01: f32,
}

#[derive(Debug)]
pub(super) struct PreparedPath {
    pub(super) metrics: fret_core::PathMetrics,
    pub(super) triangles: Vec<PathTriangleVertex>,
    pub(super) stroke_s01_mode: StrokeS01ModeV1,
    pub(super) cache_key: PathCacheKey,
}

fn mix_path_style(state: u64, style: fret_core::PathStyle) -> u64 {
    fn stable_join_key(join: fret_core::StrokeJoinV1) -> u64 {
        match join {
            fret_core::StrokeJoinV1::Miter => 1,
            fret_core::StrokeJoinV1::Bevel => 2,
            fret_core::StrokeJoinV1::Round => 3,
        }
    }

    fn stable_cap_key(cap: fret_core::StrokeCapV1) -> u64 {
        match cap {
            fret_core::StrokeCapV1::Butt => 1,
            fret_core::StrokeCapV1::Square => 2,
            fret_core::StrokeCapV1::Round => 3,
        }
    }

    fn stable_f32(x: f32) -> f32 {
        if x.is_finite() { x } else { 0.0 }
    }

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
        fret_core::PathStyle::StrokeV2(stroke) => {
            let mut state = mix_u64(state, 0x572);
            state = mix_f32(state, stroke.width.0);
            state = mix_u64(state, stable_join_key(stroke.join));
            state = mix_u64(state, stable_cap_key(stroke.cap));
            state = mix_f32(state, stable_f32(stroke.miter_limit));
            match stroke.dash {
                None => {
                    state = mix_u64(state, 0);
                }
                Some(dash) => {
                    state = mix_u64(state, 1);
                    state = mix_f32(state, stable_f32(dash.dash.0));
                    state = mix_f32(state, stable_f32(dash.gap.0));
                    state = mix_f32(state, stable_f32(dash.phase.0));
                }
            }
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

    let expand = match style {
        fret_core::PathStyle::Fill(_) => 0.0,
        fret_core::PathStyle::Stroke(stroke) => stroke.width.0.max(0.0) * 0.5,
        fret_core::PathStyle::StrokeV2(stroke) => {
            let half = stroke.width.0.max(0.0) * 0.5;
            if stroke.join == fret_core::StrokeJoinV1::Miter {
                let miter_limit = if stroke.miter_limit.is_finite() {
                    stroke.miter_limit.clamp(1.0, 64.0)
                } else {
                    4.0
                };
                half * miter_limit
            } else {
                half
            }
        }
    };
    if expand > 0.0 {
        min_x -= expand;
        min_y -= expand;
        max_x += expand;
        max_y += expand;
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

#[derive(Debug, Clone, Copy)]
struct DashState {
    dash: f32,
    gap: f32,
    period: f32,
    phase: f32,
    on: bool,
    remaining: f32,
}

impl DashState {
    fn new(pattern: fret_core::scene::DashPatternV1) -> Option<Self> {
        let dash = pattern.dash.0;
        let gap = pattern.gap.0;
        let phase = pattern.phase.0;
        if !dash.is_finite() || !gap.is_finite() || !phase.is_finite() {
            return None;
        }
        let period = dash + gap;
        if dash <= 0.0 || period <= 0.0 {
            return None;
        }
        let phase = phase.rem_euclid(period);
        let on = phase < dash;
        let remaining = if on { dash - phase } else { period - phase };
        Some(Self {
            dash,
            gap,
            period,
            phase,
            on,
            remaining,
        })
    }

    fn reset(&mut self) {
        let phase = self.phase.rem_euclid(self.period);
        self.on = phase < self.dash;
        self.remaining = if self.on {
            self.dash - phase
        } else {
            self.period - phase
        };
        self.normalize_remaining();
    }

    fn normalize_remaining(&mut self) {
        // Handle zero-length phases (e.g. gap == 0) without getting stuck toggling forever.
        for _ in 0..4 {
            if self.remaining > 1e-6 {
                break;
            }
            self.on = !self.on;
            self.remaining = if self.on { self.dash } else { self.gap };
        }
        if self.remaining <= 0.0 {
            self.remaining = 1e-6;
        }
    }

    fn consume(&mut self, amount: f32) {
        self.remaining -= amount;
        if self.remaining <= 1e-6 {
            self.on = !self.on;
            self.remaining = if self.on { self.dash } else { self.gap };
            self.normalize_remaining();
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FlatLineSegment {
    from: lyon::math::Point,
    to: lyon::math::Point,
    len: f32,
}

#[derive(Debug, Default, Clone)]
struct FlatSubpath {
    segments: Vec<FlatLineSegment>,
    total_len: f32,
}

fn flatten_lyon_path_to_subpaths(path: &lyon::path::Path, tolerance: f32) -> Vec<FlatSubpath> {
    use lyon::path::Event;

    fn dist(a: lyon::math::Point, b: lyon::math::Point) -> f32 {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        (dx * dx + dy * dy).sqrt()
    }

    let mut out: Vec<FlatSubpath> = Vec::new();
    let mut cur: FlatSubpath = FlatSubpath::default();

    for evt in path.iter().flattened(tolerance) {
        match evt {
            Event::Begin { .. } => {
                if !cur.segments.is_empty() {
                    out.push(cur);
                }
                cur = FlatSubpath::default();
            }
            Event::Line { from, to } => {
                let len = dist(from, to);
                if len.is_finite() && len > 0.0 {
                    cur.segments.push(FlatLineSegment { from, to, len });
                    cur.total_len += len;
                }
            }
            Event::End { last, first, close } => {
                if close {
                    let len = dist(last, first);
                    if len.is_finite() && len > 0.0 {
                        cur.segments.push(FlatLineSegment {
                            from: last,
                            to: first,
                            len,
                        });
                        cur.total_len += len;
                    }
                }
                if !cur.segments.is_empty() {
                    out.push(cur);
                }
                cur = FlatSubpath::default();
            }
            Event::Quadratic { .. } | Event::Cubic { .. } => {}
        }
    }

    if !cur.segments.is_empty() {
        out.push(cur);
    }

    out
}

#[derive(Debug, Clone)]
struct DashedSubpath {
    path: lyon::path::Path,
    start_adv: f32,
}

fn build_dashed_lyon_subpaths(
    path: &lyon::path::Path,
    pattern: fret_core::scene::DashPatternV1,
    tolerance: f32,
) -> Option<(Vec<DashedSubpath>, f32)> {
    use lyon::math::point;
    use std::cmp::Ordering;

    let subpaths = flatten_lyon_path_to_subpaths(path, tolerance);
    if subpaths.is_empty() {
        return None;
    }
    let denom = subpaths
        .iter()
        .fold(0.0_f32, |acc, sp| acc.max(sp.total_len));
    if !denom.is_finite() || denom <= 1e-6 {
        return None;
    }

    let mut out: Vec<DashedSubpath> = Vec::new();

    for sp in subpaths {
        let mut dash = DashState::new(pattern)?;
        dash.reset();

        let mut s = 0.0f32;
        let mut active_builder: Option<lyon::path::path::Builder> = None;
        let mut active_start_adv = 0.0f32;

        for seg in &sp.segments {
            let dx = seg.to.x - seg.from.x;
            let dy = seg.to.y - seg.from.y;
            let len = seg.len;
            if len.partial_cmp(&0.0) != Some(Ordering::Greater) {
                continue;
            }

            let inv_len = 1.0 / len;
            let mut traveled = 0.0f32;
            let mut cur = seg.from;

            while traveled < len {
                let step = dash.remaining.min(len - traveled);
                if step.partial_cmp(&0.0) != Some(Ordering::Greater) {
                    break;
                }

                let next_s = s + step;
                traveled += step;
                let t = traveled * inv_len;
                let next = point(seg.from.x + dx * t, seg.from.y + dy * t);

                if dash.on {
                    if active_builder.is_none() {
                        let mut b = lyon::path::Path::builder();
                        b.begin(cur);
                        active_start_adv = s;
                        active_builder = Some(b);
                    }
                    if let Some(b) = active_builder.as_mut() {
                        b.line_to(next);
                    }
                } else if let Some(mut b) = active_builder.take() {
                    b.end(false);
                    out.push(DashedSubpath {
                        path: b.build(),
                        start_adv: active_start_adv,
                    });
                }

                cur = next;
                s = next_s;
                dash.consume(step);
            }
        }

        if let Some(mut b) = active_builder.take() {
            b.end(false);
            out.push(DashedSubpath {
                path: b.build(),
                start_adv: active_start_adv,
            });
        }
    }

    if out.is_empty() {
        return None;
    }
    Some((out, denom))
}

#[derive(Clone, Copy)]
struct PathTessVertex {
    pos: [f32; 2],
    advancement: f32,
}

pub(super) fn tessellate_path_commands(
    commands: &[fret_core::PathCommand],
    style: fret_core::PathStyle,
    constraints: fret_core::PathConstraints,
) -> (Vec<PathTriangleVertex>, StrokeS01ModeV1) {
    if commands.is_empty() {
        return (Vec::new(), StrokeS01ModeV1::None);
    }

    let path = build_lyon_path(commands);

    let scale = constraints.scale_factor.max(1.0);
    let tolerance = (0.25 / scale).clamp(0.01, 1.0);

    let mut buffers: VertexBuffers<PathTessVertex, u32> = VertexBuffers::new();
    let mut stroke_s01_mode = StrokeS01ModeV1::None;
    let mut stroke_s01_denominator_override: Option<f32> = None;
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
                    PathTessVertex {
                        pos: [p.x, p.y],
                        advancement: 0.0,
                    }
                }),
            );
        }
        fret_core::PathStyle::Stroke(stroke) => {
            stroke_s01_mode = StrokeS01ModeV1::Continuous;
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
                    PathTessVertex {
                        pos: [p.x, p.y],
                        advancement: v.advancement(),
                    }
                }),
            );
        }
        fret_core::PathStyle::StrokeV2(stroke) => {
            stroke_s01_mode = StrokeS01ModeV1::Continuous;
            let width = stroke.width.0.max(0.0);

            let join = match stroke.join {
                fret_core::StrokeJoinV1::Miter => lyon::tessellation::LineJoin::Miter,
                fret_core::StrokeJoinV1::Bevel => lyon::tessellation::LineJoin::Bevel,
                fret_core::StrokeJoinV1::Round => lyon::tessellation::LineJoin::Round,
            };
            let cap = match stroke.cap {
                fret_core::StrokeCapV1::Butt => lyon::tessellation::LineCap::Butt,
                fret_core::StrokeCapV1::Square => lyon::tessellation::LineCap::Square,
                fret_core::StrokeCapV1::Round => lyon::tessellation::LineCap::Round,
            };

            let miter_limit = if stroke.miter_limit.is_finite() {
                stroke.miter_limit.clamp(1.0, 64.0)
            } else {
                4.0
            };

            let opts = StrokeOptions::default()
                .with_line_width(width)
                .with_tolerance(tolerance)
                .with_line_join(join)
                .with_miter_limit(miter_limit)
                .with_start_cap(cap)
                .with_end_cap(cap);
            let mut tessellator = StrokeTessellator::new();
            if let Some(pattern) = stroke.dash
                && let Some((dashed, denom)) = build_dashed_lyon_subpaths(&path, pattern, tolerance)
            {
                stroke_s01_denominator_override = Some(denom);
                for sub in dashed {
                    let adv_offset = sub.start_adv;
                    let _ = tessellator.tessellate_path(
                        &sub.path,
                        &opts,
                        &mut BuffersBuilder::new(&mut buffers, move |v: StrokeVertex| {
                            let p = v.position();
                            PathTessVertex {
                                pos: [p.x, p.y],
                                advancement: adv_offset + v.advancement(),
                            }
                        }),
                    );
                }
            } else {
                let _ = tessellator.tessellate_path(
                    &path,
                    &opts,
                    &mut BuffersBuilder::new(&mut buffers, |v: StrokeVertex| {
                        let p = v.position();
                        PathTessVertex {
                            pos: [p.x, p.y],
                            advancement: v.advancement(),
                        }
                    }),
                );
            }
        }
    }

    let max_adv = buffers
        .vertices
        .iter()
        .fold(0.0_f32, |acc, v| acc.max(v.advancement));
    let denom = stroke_s01_denominator_override.unwrap_or(max_adv);
    let inv_denom = if denom.is_finite() && denom > 1e-6 {
        1.0 / denom
    } else {
        0.0
    };

    let mut out = Vec::with_capacity(buffers.indices.len());
    for idx in buffers.indices {
        if let Some(v) = buffers.vertices.get(idx as usize) {
            out.push(PathTriangleVertex {
                pos: v.pos,
                stroke_s01: (v.advancement * inv_denom).clamp(0.0, 1.0),
            });
        }
    }
    (out, stroke_s01_mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect_commands(origin: (f32, f32), size: (f32, f32)) -> Vec<fret_core::PathCommand> {
        let (x, y) = origin;
        let (w, h) = size;
        vec![
            fret_core::PathCommand::MoveTo(Point::new(Px(x), Px(y))),
            fret_core::PathCommand::LineTo(Point::new(Px(x + w), Px(y))),
            fret_core::PathCommand::LineTo(Point::new(Px(x + w), Px(y + h))),
            fret_core::PathCommand::LineTo(Point::new(Px(x), Px(y + h))),
            fret_core::PathCommand::Close,
        ]
    }

    #[test]
    fn path_state_deduplicates_and_evicts_unreferenced_entries() {
        let mut state = PathState::default();
        state.path_cache_capacity = 1;
        let constraints = fret_core::PathConstraints { scale_factor: 1.0 };
        let style = fret_core::PathStyle::Fill(fret_core::FillStyle::default());

        let commands_a = rect_commands((0.0, 0.0), (10.0, 10.0));
        let (path_a0, _) = state.prepare_path(&commands_a, style, constraints);
        let (path_a1, _) = state.prepare_path(&commands_a, style, constraints);
        assert_eq!(path_a0, path_a1);
        assert!(state.prepared(path_a0).is_some());

        state.release_path(path_a0);
        state.release_path(path_a1);

        let commands_b = rect_commands((20.0, 20.0), (8.0, 8.0));
        let (path_b, _) = state.prepare_path(&commands_b, style, constraints);

        assert!(state.prepared(path_a0).is_none());
        assert!(state.prepared(path_b).is_some());
    }
}
