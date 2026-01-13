//! Painting routines and render caches for the node graph canvas.
//!
//! The node graph canvas paint path is mostly immediate-mode: each frame it emits `SceneOp`s.
//! However, preparing stroked vector paths can be expensive for large graphs. This module provides
//! a small cache for those prepared paths so that panning (which does not change geometry) does
//! not re-tessellate every edge on every frame.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use fret_core::{
    FillStyle, PathCommand, PathConstraints, PathId, PathStyle, Point, Px, StrokeStyle, TextBlobId,
    TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};

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
}

#[derive(Debug)]
struct PathCacheEntry {
    id: PathId,
    last_used_frame: u64,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextBlobKey {
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

impl Hash for TextBlobKey {
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
struct TextBlobEntry {
    id: TextBlobId,
    metrics: TextMetrics,
    last_used_frame: u64,
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
    wire_paths: HashMap<WirePathKey, PathCacheEntry>,
    marker_paths: HashMap<MarkerPathKey, PathCacheEntry>,
    text_blobs: HashMap<TextBlobKey, TextBlobEntry>,
    text_metrics: HashMap<TextMetricsKey, TextMetricsEntry>,
}

impl CanvasPaintCache {
    pub(crate) fn begin_frame(&mut self) -> u64 {
        self.frame = self.frame.wrapping_add(1);
        self.frame
    }

    pub(crate) fn clear(&mut self, services: &mut dyn fret_core::UiServices) {
        for entry in self.wire_paths.drain().map(|(_, e)| e) {
            services.path().release(entry.id);
        }
        for entry in self.marker_paths.drain().map(|(_, e)| e) {
            services.path().release(entry.id);
        }
        for entry in self.text_blobs.drain().map(|(_, e)| e) {
            services.text().release(entry.id);
        }
        self.text_metrics.clear();
    }

    pub(crate) fn prune(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        max_age_frames: u64,
        max_entries: usize,
    ) {
        fn prune_map_by_age<K: Copy + Eq + Hash>(
            map: &mut HashMap<K, PathCacheEntry>,
            services: &mut dyn fret_core::UiServices,
            now: u64,
            max_age_frames: u64,
        ) {
            map.retain(|_, entry| {
                let keep = now.saturating_sub(entry.last_used_frame) <= max_age_frames;
                if !keep {
                    services.path().release(entry.id);
                }
                keep
            });
        }

        let now = self.frame;
        prune_map_by_age(&mut self.wire_paths, services, now, max_age_frames);
        prune_map_by_age(&mut self.marker_paths, services, now, max_age_frames);

        self.text_blobs.retain(|_, entry| {
            let keep = now.saturating_sub(entry.last_used_frame) <= max_age_frames;
            if !keep {
                services.text().release(entry.id);
            }
            keep
        });

        self.text_metrics
            .retain(|_, entry| now.saturating_sub(entry.last_used_frame) <= max_age_frames);

        let total = self
            .wire_paths
            .len()
            .saturating_add(self.marker_paths.len());
        let total = total.saturating_add(self.text_blobs.len());
        let total = total.saturating_add(self.text_metrics.len());
        if total <= max_entries {
            return;
        }

        #[derive(Debug, Clone)]
        enum EvictKey {
            Wire(WirePathKey),
            Marker(MarkerPathKey),
            Text(TextBlobKey),
            Metrics(TextMetricsKey),
        }

        let mut entries: Vec<(EvictKey, u64)> = Vec::with_capacity(total);
        entries.extend(
            self.wire_paths
                .iter()
                .map(|(k, v)| (EvictKey::Wire(*k), v.last_used_frame)),
        );
        entries.extend(
            self.marker_paths
                .iter()
                .map(|(k, v)| (EvictKey::Marker(*k), v.last_used_frame)),
        );
        entries.extend(
            self.text_blobs
                .iter()
                .map(|(k, v)| (EvictKey::Text(k.clone()), v.last_used_frame)),
        );
        entries.extend(
            self.text_metrics
                .iter()
                .map(|(k, v)| (EvictKey::Metrics(k.clone()), v.last_used_frame)),
        );
        entries.sort_by_key(|(_k, last)| *last);

        let over = total.saturating_sub(max_entries);
        for (key, _last) in entries.into_iter().take(over) {
            match key {
                EvictKey::Wire(key) => {
                    if let Some(entry) = self.wire_paths.remove(&key) {
                        services.path().release(entry.id);
                    }
                }
                EvictKey::Marker(key) => {
                    if let Some(entry) = self.marker_paths.remove(&key) {
                        services.path().release(entry.id);
                    }
                }
                EvictKey::Text(key) => {
                    if let Some(entry) = self.text_blobs.remove(&key) {
                        services.text().release(entry.id);
                    }
                }
                EvictKey::Metrics(key) => {
                    self.text_metrics.remove(&key);
                }
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
        };

        let now = self.frame;
        if let Some(entry) = self.wire_paths.get_mut(&key) {
            entry.last_used_frame = now;
            return Some(entry.id);
        }

        let id = match route {
            EdgeRouteKind::Bezier => {
                prepare_bezier_wire_path(services, from, to, zoom, scale_factor, width_px)?
            }
            EdgeRouteKind::Straight => {
                prepare_straight_path(services, from, to, zoom, scale_factor, width_px)?
            }
            EdgeRouteKind::Step => {
                prepare_step_path(services, from, to, zoom, scale_factor, width_px)?
            }
        };

        self.wire_paths.insert(
            key,
            PathCacheEntry {
                id,
                last_used_frame: now,
            },
        );

        Some(id)
    }

    pub(crate) fn edge_end_marker_path(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
    ) -> Option<PathId> {
        self.marker_path(
            services,
            MarkerSide::End,
            route,
            from,
            to,
            zoom,
            scale_factor,
            marker,
            pin_radius_screen,
        )
    }

    pub(crate) fn edge_start_marker_path(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
        scale_factor: f32,
        marker: &EdgeMarker,
        pin_radius_screen: f32,
    ) -> Option<PathId> {
        self.marker_path(
            services,
            MarkerSide::Start,
            route,
            from,
            to,
            zoom,
            scale_factor,
            marker,
            pin_radius_screen,
        )
    }

    fn marker_path(
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

        let now = self.frame;
        if let Some(entry) = self.marker_paths.get_mut(&key) {
            entry.last_used_frame = now;
            return Some(entry.id);
        }

        let id = prepare_marker_path(
            services,
            side,
            route,
            from,
            to,
            zoom,
            scale_factor,
            marker,
            pin_radius_screen,
        )?;
        self.marker_paths.insert(
            key,
            PathCacheEntry {
                id,
                last_used_frame: now,
            },
        );
        Some(id)
    }

    pub(crate) fn text_blob(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        text: impl Into<Arc<str>>,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
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

        let key = TextBlobKey {
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
        if let Some(entry) = self.text_blobs.get_mut(&key) {
            entry.last_used_frame = now;
            return (entry.id, entry.metrics);
        }

        let (id, metrics) = services
            .text()
            .prepare_str(text.as_ref(), style, constraints);
        self.text_blobs.insert(
            key,
            TextBlobEntry {
                id,
                metrics,
                last_used_frame: now,
            },
        );
        (id, metrics)
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
}

fn prepare_marker_path(
    services: &mut dyn fret_core::UiServices,
    side: MarkerSide,
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
    scale_factor: f32,
    marker: &EdgeMarker,
    pin_radius_screen: f32,
) -> Option<PathId> {
    let zoom = zoom.max(1.0e-6);
    let dir = match side {
        MarkerSide::Start => edge_route_start_tangent(route, from, to, zoom),
        MarkerSide::End => edge_route_end_tangent(route, from, to, zoom),
    };

    let len = (dir.x.0 * dir.x.0 + dir.y.0 * dir.y.0).sqrt();
    if !len.is_finite() || len <= 1.0e-6 {
        return None;
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

            let (id, _metrics) = services.path().prepare(
                &commands,
                PathStyle::Fill(FillStyle::default()),
                PathConstraints {
                    scale_factor: scale_factor * zoom,
                },
            );
            Some(id)
        }
    }
}

fn prepare_bezier_wire_path(
    services: &mut dyn fret_core::UiServices,
    from: Point,
    to: Point,
    zoom: f32,
    scale_factor: f32,
    width_px: f32,
) -> Option<PathId> {
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

    let (id, _metrics) = services.path().prepare(
        &commands,
        PathStyle::Stroke(StrokeStyle {
            width: Px(width_px / zoom),
        }),
        PathConstraints {
            scale_factor: scale_factor * zoom,
        },
    );

    Some(id)
}

fn prepare_step_path(
    services: &mut dyn fret_core::UiServices,
    from: Point,
    to: Point,
    zoom: f32,
    scale_factor: f32,
    width_px: f32,
) -> Option<PathId> {
    let mx = 0.5 * (from.x.0 + to.x.0);
    let p1 = Point::new(Px(mx), from.y);
    let p2 = Point::new(Px(mx), to.y);

    let commands = [
        PathCommand::MoveTo(from),
        PathCommand::LineTo(p1),
        PathCommand::LineTo(p2),
        PathCommand::LineTo(to),
    ];

    let (id, _metrics) = services.path().prepare(
        &commands,
        PathStyle::Stroke(StrokeStyle {
            width: Px(width_px / zoom),
        }),
        PathConstraints {
            scale_factor: scale_factor * zoom,
        },
    );

    Some(id)
}

fn prepare_straight_path(
    services: &mut dyn fret_core::UiServices,
    from: Point,
    to: Point,
    zoom: f32,
    scale_factor: f32,
    width_px: f32,
) -> Option<PathId> {
    let commands = [PathCommand::MoveTo(from), PathCommand::LineTo(to)];

    let (id, _metrics) = services.path().prepare(
        &commands,
        PathStyle::Stroke(StrokeStyle {
            width: Px(width_px / zoom),
        }),
        PathConstraints {
            scale_factor: scale_factor * zoom,
        },
    );

    Some(id)
}
