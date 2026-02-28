//! Painting routines and render caches for the node graph canvas.
//!
//! The node graph canvas paint path is mostly immediate-mode: each frame it emits `SceneOp`s.
//! However, preparing stroked vector paths can be expensive for large graphs. This module provides
//! a small cache for those prepared paths so that panning (which does not change geometry) does
//! not re-tessellate every edge on every frame.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use fret_canvas::budget::WorkBudget;
use fret_canvas::cache::CacheStats;
use fret_canvas::cache::PathCache;
use fret_canvas::text::TextCache;
use fret_core::scene::DashPatternV1;
use fret_core::{
    FillStyle, PathCommand, PathConstraints, PathId, PathStyle, Point, Px, SceneOp, Size,
    StrokeCapV1, StrokeJoinV1, StrokeStyleV2, TextBlobId, TextConstraints, TextMetrics,
    TextOverflow, TextStyle, TextWrap,
};

use crate::ui::PortShapeHint;
use crate::ui::presenter::{EdgeMarker, EdgeMarkerKind, EdgeRouteKind};

use super::route_math::{edge_route_end_tangent, edge_route_start_tangent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WirePathKey {
    route: EdgeRouteKind,
    from_x: i64,
    from_y: i64,
    to_x: i64,
    to_y: i64,
    zoom: i64,
    scale: i64,
    stroke_width: i64,
    dash: i64,
    gap: i64,
    phase: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MarkerSide {
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MarkerPathKey {
    side: MarkerSide,
    kind: u8,
    route: EdgeRouteKind,
    from_x: i64,
    from_y: i64,
    to_x: i64,
    to_y: i64,
    zoom: i64,
    scale: i64,
    size_screen: i64,
    pin_radius_screen: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MarkerTangentPathKey {
    side: MarkerSide,
    kind: u8,
    endpoint_x: i64,
    endpoint_y: i64,
    dir_x: i64,
    dir_y: i64,
    zoom: i64,
    scale: i64,
    size_screen: i64,
    pin_radius_screen: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PortShapePathKey {
    shape: u8,
    dir: u8,
    w: i64,
    h: i64,
    zoom: i64,
    scale: i64,
    stroke_width_screen: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextMetricsKey {
    text_hash: u64,
    text_len: u32,
    text: Arc<str>,
    font: fret_core::FontId,
    size: i64,
    weight: u16,
    slant: u8,
    line_height: i64,
    letter_spacing_em: i64,
    max_width: i64,
    wrap: TextWrap,
    overflow: TextOverflow,
    scale_factor: i64,
}

impl Hash for TextMetricsKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.text_hash.hash(state);
        self.text_len.hash(state);
        self.font.hash(state);
        self.size.hash(state);
        self.weight.hash(state);
        self.slant.hash(state);
        self.line_height.hash(state);
        self.letter_spacing_em.hash(state);
        self.max_width.hash(state);
        self.wrap.hash(state);
        self.overflow.hash(state);
        self.scale_factor.hash(state);
    }
}

#[derive(Debug, Clone, Copy)]
struct TextMetricsEntry {
    metrics: TextMetrics,
    last_used_frame: u64,
}

#[derive(Debug, Default)]
pub(crate) struct CanvasPaintCache {
    frame: u64,
    paths: PathCache,
    text: TextCache,
    text_metrics: HashMap<TextMetricsKey, TextMetricsEntry>,
}

impl CanvasPaintCache {
    pub(crate) fn begin_frame(&mut self) -> u64 {
        self.frame = self.frame.wrapping_add(1);
        self.paths.begin_frame();
        self.text.begin_frame();
        self.frame
    }

    pub(crate) fn touch_text_blobs_in_scene_ops(&mut self, ops: &[SceneOp]) {
        let _ = self.text.touch_blobs_in_scene_ops(ops);
    }

    pub(crate) fn touch_paths_in_scene_ops(&mut self, ops: &[SceneOp]) {
        let _ = self.paths.touch_paths_in_scene_ops(ops);
    }

    pub(crate) fn diagnostics_path_cache_snapshot(&self) -> (usize, CacheStats) {
        (self.paths.len(), self.paths.stats())
    }

    pub(crate) fn clear(&mut self, services: &mut dyn fret_core::UiServices) {
        self.paths.clear(services);
        self.text.clear(services);
        self.text_metrics.clear();
    }

    pub(crate) fn prune(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        max_age_frames: u64,
        max_entries: usize,
    ) {
        if max_entries == 0 {
            self.clear(services);
            return;
        }

        // Approximate budgets: paths are typically the heaviest and most numerous in node graphs.
        let path_budget = (max_entries.saturating_mul(6)) / 10;
        let text_budget = max_entries.saturating_sub(path_budget);
        let text_blob_budget = (text_budget.saturating_mul(7)) / 10;
        let metrics_budget = text_budget.saturating_sub(text_blob_budget);

        self.paths.prune(services, max_age_frames, path_budget);
        self.text.prune(services, max_age_frames, text_blob_budget);

        let now = self.frame;
        self.text_metrics
            .retain(|_, entry| now.saturating_sub(entry.last_used_frame) <= max_age_frames);

        if metrics_budget > 0 && self.text_metrics.len() > metrics_budget {
            let mut candidates: Vec<(u64, TextMetricsKey)> = self
                .text_metrics
                .iter()
                .map(|(k, v)| (v.last_used_frame, k.clone()))
                .collect();
            candidates.sort_by_key(|(last_used, _)| *last_used);

            let over = self.text_metrics.len().saturating_sub(metrics_budget);
            for (_, key) in candidates.into_iter().take(over) {
                self.text_metrics.remove(&key);
            }
        }
    }

    pub(crate) fn wire_path(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        width_px: f32,
        dash: Option<DashPatternV1>,
    ) -> Option<PathId> {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        if !from.x.0.is_finite()
            || !from.y.0.is_finite()
            || !to.x.0.is_finite()
            || !to.y.0.is_finite()
        {
            return None;
        }

        let stroke_width = width_px / zoom;
        if !stroke_width.is_finite() || stroke_width <= 0.0 {
            return None;
        }

        let dash = dash.and_then(|p| scale_dash_pattern_screen_px_to_canvas_units(p, zoom));

        let q = |v: f32, step: f32| -> i64 {
            if !v.is_finite() {
                return 0;
            }
            (v / step).round() as i64
        };

        let key = WirePathKey {
            route,
            from_x: q(from.x.0, 0.01),
            from_y: q(from.y.0, 0.01),
            to_x: q(to.x.0, 0.01),
            to_y: q(to.y.0, 0.01),
            zoom: q(zoom, 0.0001),
            scale: q(scale_factor * zoom, 0.0001),
            stroke_width: q(stroke_width, 0.0001),
            dash: dash.map(|p| q(p.dash.0, 0.01)).unwrap_or(0),
            gap: dash.map(|p| q(p.gap.0, 0.01)).unwrap_or(0),
            phase: dash.map(|p| q(p.phase.0, 0.01)).unwrap_or(0),
        };

        let cache_key = stable_path_key(1, &key);
        match route {
            EdgeRouteKind::Bezier => {
                let dx = to.x.0 - from.x.0;
                let ctrl = (dx.abs() * 0.5).clamp(40.0 / zoom, 160.0 / zoom);
                let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
                let c1 = Point::new(Px(from.x.0 + dir * ctrl), from.y);
                let c2 = Point::new(Px(to.x.0 - dir * ctrl), to.y);

                let commands = [
                    PathCommand::MoveTo(from),
                    PathCommand::CubicTo {
                        ctrl1: c1,
                        ctrl2: c2,
                        to,
                    },
                ];

                let (id, _metrics) = self.paths.prepare(
                    services,
                    cache_key,
                    &commands,
                    PathStyle::StrokeV2(StrokeStyleV2 {
                        width: Px(width_px / zoom),
                        join: StrokeJoinV1::Round,
                        cap: StrokeCapV1::Round,
                        miter_limit: 4.0,
                        dash,
                    }),
                    PathConstraints {
                        scale_factor: scale_factor * zoom,
                    },
                );
                Some(id)
            }
            EdgeRouteKind::Straight => {
                let commands = [PathCommand::MoveTo(from), PathCommand::LineTo(to)];
                let (id, _metrics) = self.paths.prepare(
                    services,
                    cache_key,
                    &commands,
                    PathStyle::StrokeV2(StrokeStyleV2 {
                        width: Px(width_px / zoom),
                        join: StrokeJoinV1::Round,
                        cap: StrokeCapV1::Round,
                        miter_limit: 4.0,
                        dash,
                    }),
                    PathConstraints {
                        scale_factor: scale_factor * zoom,
                    },
                );
                Some(id)
            }
            EdgeRouteKind::Step => {
                let mx = 0.5 * (from.x.0 + to.x.0);
                let p1 = Point::new(Px(mx), from.y);
                let p2 = Point::new(Px(mx), to.y);

                let commands = [
                    PathCommand::MoveTo(from),
                    PathCommand::LineTo(p1),
                    PathCommand::LineTo(p2),
                    PathCommand::LineTo(to),
                ];

                let (id, _metrics) = self.paths.prepare(
                    services,
                    cache_key,
                    &commands,
                    PathStyle::StrokeV2(StrokeStyleV2 {
                        width: Px(width_px / zoom),
                        join: StrokeJoinV1::Round,
                        cap: StrokeCapV1::Round,
                        miter_limit: 4.0,
                        dash,
                    }),
                    PathConstraints {
                        scale_factor: scale_factor * zoom,
                    },
                );
                Some(id)
            }
        }
    }

    pub(crate) fn wire_path_from_commands(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        cache_key: u64,
        commands: &[PathCommand],
        zoom: f32,
        scale_factor: f32,
        width_px: f32,
        dash: Option<DashPatternV1>,
    ) -> Option<PathId> {
        if commands.is_empty() {
            return None;
        }

        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };

        let stroke_width = width_px / zoom;
        if !stroke_width.is_finite() || stroke_width <= 0.0 {
            return None;
        }

        let dash = dash.and_then(|p| scale_dash_pattern_screen_px_to_canvas_units(p, zoom));

        let cache_key = stable_path_key(3, &cache_key);
        let (id, _metrics) = self.paths.prepare(
            services,
            cache_key,
            commands,
            PathStyle::StrokeV2(StrokeStyleV2 {
                width: Px(width_px / zoom),
                join: StrokeJoinV1::Round,
                cap: StrokeCapV1::Round,
                miter_limit: 4.0,
                dash,
            }),
            PathConstraints {
                scale_factor: scale_factor * zoom,
            },
        );
        Some(id)
    }

    pub(crate) fn port_shape_fill_path(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        shape: PortShapeHint,
        size: Size,
        dir: Option<crate::core::PortDirection>,
        zoom: f32,
        scale_factor: f32,
    ) -> Option<PathId> {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        if !size.width.0.is_finite()
            || !size.height.0.is_finite()
            || size.width.0 <= 0.0
            || size.height.0 <= 0.0
        {
            return None;
        }

        let q = |v: f32, step: f32| -> i64 {
            if !v.is_finite() {
                return 0;
            }
            (v / step).round() as i64
        };

        let dir_tag = match dir {
            Some(crate::core::PortDirection::In) => 1,
            Some(crate::core::PortDirection::Out) => 2,
            None => 0,
        };
        let shape_tag = match shape {
            PortShapeHint::Circle => 0,
            PortShapeHint::Diamond => 1,
            PortShapeHint::Triangle => 2,
        };

        let key = PortShapePathKey {
            shape: shape_tag,
            dir: dir_tag,
            w: q(size.width.0, 0.01),
            h: q(size.height.0, 0.01),
            zoom: q(zoom, 0.0001),
            scale: q(scale_factor * zoom, 0.0001),
            stroke_width_screen: 0,
        };

        let cache_key = stable_path_key(20, &key);
        let constraints = PathConstraints {
            scale_factor: scale_factor * zoom,
        };
        if let Some((id, _metrics)) = self.paths.get(cache_key, constraints) {
            return Some(id);
        }

        let w = size.width.0;
        let h = size.height.0;
        let mx = 0.5 * w;
        let my = 0.5 * h;
        let commands: Vec<PathCommand> = match shape {
            PortShapeHint::Circle => return None,
            PortShapeHint::Diamond => vec![
                PathCommand::MoveTo(Point::new(Px(mx), Px(0.0))),
                PathCommand::LineTo(Point::new(Px(w), Px(my))),
                PathCommand::LineTo(Point::new(Px(mx), Px(h))),
                PathCommand::LineTo(Point::new(Px(0.0), Px(my))),
                PathCommand::Close,
            ],
            PortShapeHint::Triangle => {
                let tip_left = matches!(dir, Some(crate::core::PortDirection::In));
                if tip_left {
                    vec![
                        PathCommand::MoveTo(Point::new(Px(0.0), Px(my))),
                        PathCommand::LineTo(Point::new(Px(w), Px(0.0))),
                        PathCommand::LineTo(Point::new(Px(w), Px(h))),
                        PathCommand::Close,
                    ]
                } else {
                    vec![
                        PathCommand::MoveTo(Point::new(Px(w), Px(my))),
                        PathCommand::LineTo(Point::new(Px(0.0), Px(0.0))),
                        PathCommand::LineTo(Point::new(Px(0.0), Px(h))),
                        PathCommand::Close,
                    ]
                }
            }
        };

        let (id, _metrics) = self.paths.prepare(
            services,
            cache_key,
            &commands,
            PathStyle::Fill(FillStyle::default()),
            constraints,
        );
        Some(id)
    }

    pub(crate) fn port_shape_stroke_path(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        shape: PortShapeHint,
        size: Size,
        dir: Option<crate::core::PortDirection>,
        zoom: f32,
        scale_factor: f32,
        stroke_width_screen_px: f32,
    ) -> Option<PathId> {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        if !size.width.0.is_finite()
            || !size.height.0.is_finite()
            || size.width.0 <= 0.0
            || size.height.0 <= 0.0
        {
            return None;
        }

        let stroke_width_screen_px =
            if stroke_width_screen_px.is_finite() && stroke_width_screen_px > 0.0 {
                stroke_width_screen_px
            } else {
                return None;
            };

        let q = |v: f32, step: f32| -> i64 {
            if !v.is_finite() {
                return 0;
            }
            (v / step).round() as i64
        };

        let dir_tag = match dir {
            Some(crate::core::PortDirection::In) => 1,
            Some(crate::core::PortDirection::Out) => 2,
            None => 0,
        };
        let shape_tag = match shape {
            PortShapeHint::Circle => 0,
            PortShapeHint::Diamond => 1,
            PortShapeHint::Triangle => 2,
        };

        let key = PortShapePathKey {
            shape: shape_tag,
            dir: dir_tag,
            w: q(size.width.0, 0.01),
            h: q(size.height.0, 0.01),
            zoom: q(zoom, 0.0001),
            scale: q(scale_factor * zoom, 0.0001),
            stroke_width_screen: q(stroke_width_screen_px, 0.001),
        };

        let cache_key = stable_path_key(21, &key);
        let constraints = PathConstraints {
            scale_factor: scale_factor * zoom,
        };
        if let Some((id, _metrics)) = self.paths.get(cache_key, constraints) {
            return Some(id);
        }

        let w = size.width.0;
        let h = size.height.0;
        let mx = 0.5 * w;
        let my = 0.5 * h;
        let commands: Vec<PathCommand> = match shape {
            PortShapeHint::Circle => return None,
            PortShapeHint::Diamond => vec![
                PathCommand::MoveTo(Point::new(Px(mx), Px(0.0))),
                PathCommand::LineTo(Point::new(Px(w), Px(my))),
                PathCommand::LineTo(Point::new(Px(mx), Px(h))),
                PathCommand::LineTo(Point::new(Px(0.0), Px(my))),
                PathCommand::Close,
            ],
            PortShapeHint::Triangle => {
                let tip_left = matches!(dir, Some(crate::core::PortDirection::In));
                if tip_left {
                    vec![
                        PathCommand::MoveTo(Point::new(Px(0.0), Px(my))),
                        PathCommand::LineTo(Point::new(Px(w), Px(0.0))),
                        PathCommand::LineTo(Point::new(Px(w), Px(h))),
                        PathCommand::Close,
                    ]
                } else {
                    vec![
                        PathCommand::MoveTo(Point::new(Px(w), Px(my))),
                        PathCommand::LineTo(Point::new(Px(0.0), Px(0.0))),
                        PathCommand::LineTo(Point::new(Px(0.0), Px(h))),
                        PathCommand::Close,
                    ]
                }
            }
        };

        let (id, _metrics) = self.paths.prepare(
            services,
            cache_key,
            &commands,
            PathStyle::StrokeV2(StrokeStyleV2 {
                width: Px(stroke_width_screen_px / zoom),
                join: StrokeJoinV1::Miter,
                cap: StrokeCapV1::Butt,
                miter_limit: 4.0,
                dash: None,
            }),
            constraints,
        );
        Some(id)
    }

    pub(crate) fn edge_end_marker_path_budgeted(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
        self.marker_path_budgeted(
            services,
            MarkerSide::End,
            route,
            from,
            to,
            zoom,
            scale_factor,
            marker,
            pin_radius_screen,
            budget,
        )
    }

    pub(crate) fn edge_end_marker_path_budgeted_with_tangent(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        endpoint: Point,
        tangent: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
        self.marker_path_with_tangent_budgeted(
            services,
            MarkerSide::End,
            endpoint,
            tangent,
            zoom,
            scale_factor,
            marker,
            pin_radius_screen,
            budget,
        )
    }

    pub(crate) fn edge_start_marker_path_budgeted(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
        self.marker_path_budgeted(
            services,
            MarkerSide::Start,
            route,
            from,
            to,
            zoom,
            scale_factor,
            marker,
            pin_radius_screen,
            budget,
        )
    }

    pub(crate) fn edge_start_marker_path_budgeted_with_tangent(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        endpoint: Point,
        tangent: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
        self.marker_path_with_tangent_budgeted(
            services,
            MarkerSide::Start,
            endpoint,
            tangent,
            zoom,
            scale_factor,
            marker,
            pin_radius_screen,
            budget,
        )
    }

    fn marker_path_budgeted(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        side: MarkerSide,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        if !from.x.0.is_finite()
            || !from.y.0.is_finite()
            || !to.x.0.is_finite()
            || !to.y.0.is_finite()
        {
            return (None, false);
        }

        let q = |v: f32, step: f32| -> i64 {
            if !v.is_finite() {
                return 0;
            }
            (v / step).round() as i64
        };

        let kind = match marker.kind {
            EdgeMarkerKind::Arrow => 1,
        };

        let key = MarkerPathKey {
            side,
            kind,
            route,
            from_x: q(from.x.0, 0.01),
            from_y: q(from.y.0, 0.01),
            to_x: q(to.x.0, 0.01),
            to_y: q(to.y.0, 0.01),
            zoom: q(zoom, 0.0001),
            scale: q(scale_factor * zoom, 0.0001),
            size_screen: q(marker.size.max(1.0), 0.01),
            pin_radius_screen: q(pin_radius_screen.max(0.0), 0.01),
        };

        let cache_key = stable_path_key(2, &key);
        let constraints = PathConstraints {
            scale_factor: scale_factor * zoom,
        };
        if let Some((id, _metrics)) = self.paths.get(cache_key, constraints) {
            return (Some(id), false);
        }

        if !budget.try_consume(1) {
            return (None, true);
        }

        let zoom = zoom.max(1.0e-6);
        let dir = match side {
            MarkerSide::Start => edge_route_start_tangent(route, from, to, zoom),
            MarkerSide::End => edge_route_end_tangent(route, from, to, zoom),
        };

        let len = (dir.x.0 * dir.x.0 + dir.y.0 * dir.y.0).sqrt();
        if !len.is_finite() || len <= 1.0e-6 {
            return (None, false);
        }
        let ux = dir.x.0 / len;
        let uy = dir.y.0 / len;
        let nx = -uy;
        let ny = ux;

        let size_screen = marker.size.max(1.0);
        let size = size_screen / zoom;

        let pin_r = pin_radius_screen.max(0.0) / zoom;
        let tip = match side {
            MarkerSide::Start => Point::new(Px(from.x.0 + ux * pin_r), Px(from.y.0 + uy * pin_r)),
            MarkerSide::End => Point::new(Px(to.x.0 - ux * pin_r), Px(to.y.0 - uy * pin_r)),
        };

        match marker.kind {
            EdgeMarkerKind::Arrow => {
                let arrow_len = size;
                let half_w = (0.65 * size).max(0.5 / zoom);
                let base = match side {
                    MarkerSide::Start => {
                        Point::new(Px(tip.x.0 + ux * arrow_len), Px(tip.y.0 + uy * arrow_len))
                    }
                    MarkerSide::End => {
                        Point::new(Px(tip.x.0 - ux * arrow_len), Px(tip.y.0 - uy * arrow_len))
                    }
                };
                let p1 = Point::new(Px(base.x.0 + nx * half_w), Px(base.y.0 + ny * half_w));
                let p2 = Point::new(Px(base.x.0 - nx * half_w), Px(base.y.0 - ny * half_w));

                let commands = [
                    PathCommand::MoveTo(tip),
                    PathCommand::LineTo(p1),
                    PathCommand::LineTo(p2),
                    PathCommand::Close,
                ];

                let (id, _metrics) = self.paths.prepare(
                    services,
                    cache_key,
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    constraints,
                );
                (Some(id), false)
            }
        }
    }

    fn marker_path_with_tangent_budgeted(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        side: MarkerSide,
        endpoint: Point,
        tangent: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
        budget: &mut WorkBudget,
    ) -> (Option<PathId>, bool) {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        if !endpoint.x.0.is_finite() || !endpoint.y.0.is_finite() {
            return (None, false);
        }

        let q = |v: f32, step: f32| -> i64 {
            if !v.is_finite() {
                return 0;
            }
            (v / step).round() as i64
        };

        let kind = match marker.kind {
            EdgeMarkerKind::Arrow => 1,
        };

        let len = (tangent.x.0 * tangent.x.0 + tangent.y.0 * tangent.y.0).sqrt();
        let (ux, uy) = if len.is_finite() && len > 1.0e-6 {
            (tangent.x.0 / len, tangent.y.0 / len)
        } else {
            (1.0, 0.0)
        };

        let key = MarkerTangentPathKey {
            side,
            kind,
            endpoint_x: q(endpoint.x.0, 0.01),
            endpoint_y: q(endpoint.y.0, 0.01),
            dir_x: q(ux, 0.0001),
            dir_y: q(uy, 0.0001),
            zoom: q(zoom, 0.0001),
            scale: q(scale_factor * zoom, 0.0001),
            size_screen: q(marker.size.max(1.0), 0.01),
            pin_radius_screen: q(pin_radius_screen.max(0.0), 0.01),
        };

        let cache_key = stable_path_key(4, &key);
        let constraints = PathConstraints {
            scale_factor: scale_factor * zoom,
        };
        if let Some((id, _metrics)) = self.paths.get(cache_key, constraints) {
            return (Some(id), false);
        }

        if !budget.try_consume(1) {
            return (None, true);
        }

        let nx = -uy;
        let ny = ux;

        let zoom = zoom.max(1.0e-6);
        let size_screen = marker.size.max(1.0);
        let size = size_screen / zoom;

        let pin_r = pin_radius_screen.max(0.0) / zoom;
        let tip = match side {
            MarkerSide::Start => {
                Point::new(Px(endpoint.x.0 + ux * pin_r), Px(endpoint.y.0 + uy * pin_r))
            }
            MarkerSide::End => {
                Point::new(Px(endpoint.x.0 - ux * pin_r), Px(endpoint.y.0 - uy * pin_r))
            }
        };

        match marker.kind {
            EdgeMarkerKind::Arrow => {
                let arrow_len = size;
                let half_w = (0.65 * size).max(0.5 / zoom);
                let base = match side {
                    MarkerSide::Start => {
                        Point::new(Px(tip.x.0 + ux * arrow_len), Px(tip.y.0 + uy * arrow_len))
                    }
                    MarkerSide::End => {
                        Point::new(Px(tip.x.0 - ux * arrow_len), Px(tip.y.0 - uy * arrow_len))
                    }
                };
                let p1 = Point::new(Px(base.x.0 + nx * half_w), Px(base.y.0 + ny * half_w));
                let p2 = Point::new(Px(base.x.0 - nx * half_w), Px(base.y.0 - ny * half_w));

                let commands = [
                    PathCommand::MoveTo(tip),
                    PathCommand::LineTo(p1),
                    PathCommand::LineTo(p2),
                    PathCommand::Close,
                ];

                let (id, _metrics) = self.paths.prepare(
                    services,
                    cache_key,
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    constraints,
                );
                (Some(id), false)
            }
        }
    }

    pub(crate) fn text_blob(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        text: impl Into<Arc<str>>,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let prepared = self
            .text
            .prepare_arc(services, text.into(), style, constraints);
        (prepared.blob, prepared.metrics)
    }

    pub(crate) fn text_metrics(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        text: impl Into<Arc<str>>,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        let text: Arc<str> = text.into();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.as_ref().hash(&mut hasher);
        let text_hash = hasher.finish();

        let q = |v: f32, step: f32| -> i64 {
            if !v.is_finite() {
                return 0;
            }
            (v / step).round() as i64
        };

        let max_width = constraints.max_width.map(|w| w.0.max(0.0)).unwrap_or(0.0);

        let key = TextMetricsKey {
            text_hash,
            text_len: text.len().min(u32::MAX as usize) as u32,
            text: text.clone(),
            font: style.font.clone(),
            size: q(style.size.0.max(0.0), 0.01),
            weight: style.weight.0,
            slant: match style.slant {
                fret_core::TextSlant::Normal => 0,
                fret_core::TextSlant::Italic => 1,
                fret_core::TextSlant::Oblique => 2,
            },
            line_height: q(style.line_height.map(|v| v.0).unwrap_or(0.0).max(0.0), 0.01),
            letter_spacing_em: q(style.letter_spacing_em.unwrap_or(0.0), 0.0001),
            max_width: q(max_width, 0.01),
            wrap: constraints.wrap,
            overflow: constraints.overflow,
            scale_factor: q(constraints.scale_factor.max(0.0), 0.0001),
        };

        let now = self.frame;
        if let Some(entry) = self.text_metrics.get_mut(&key) {
            entry.last_used_frame = now;
            return entry.metrics;
        }

        let metrics = services
            .text()
            .measure_str(text.as_ref(), style, constraints);
        self.text_metrics.insert(
            key,
            TextMetricsEntry {
                metrics,
                last_used_frame: now,
            },
        );
        metrics
    }

    pub(crate) fn text_blob_budgeted(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        text: impl Into<Arc<str>>,
        style: &TextStyle,
        constraints: TextConstraints,
        budget: &mut WorkBudget,
    ) -> (Option<(TextBlobId, TextMetrics)>, bool) {
        let text: Arc<str> = text.into();

        if let Some(prepared) = self.text.get_arc(text.clone(), style, constraints) {
            return (Some((prepared.blob, prepared.metrics)), false);
        }

        if !budget.try_consume(1) {
            return (None, true);
        }

        let prepared = self.text.prepare_arc(services, text, style, constraints);
        (Some((prepared.blob, prepared.metrics)), false)
    }
}

fn stable_path_key<T: Hash>(tag: u8, key: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    tag.hash(&mut hasher);
    key.hash(&mut hasher);
    hasher.finish()
}

fn scale_dash_pattern_screen_px_to_canvas_units(
    pattern: DashPatternV1,
    zoom: f32,
) -> Option<DashPatternV1> {
    let z = zoom.max(1.0e-6);
    let dash = pattern.dash.0 / z;
    let gap = pattern.gap.0 / z;
    let phase = pattern.phase.0 / z;
    let period = dash + gap;
    if !dash.is_finite() || !gap.is_finite() || !phase.is_finite() || dash <= 0.0 || period <= 0.0 {
        return None;
    }
    Some(DashPatternV1::new(Px(dash), Px(gap), Px(phase)))
}
