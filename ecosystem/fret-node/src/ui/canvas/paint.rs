//! Painting routines and render caches for the node graph canvas.
//!
//! The node graph canvas paint path is mostly immediate-mode: each frame it emits `SceneOp`s.
//! However, preparing stroked vector paths can be expensive for large graphs. This module provides
//! a small cache for those prepared paths so that panning (which does not change geometry) does
//! not re-tessellate every edge on every frame.

use std::collections::HashMap;
use std::hash::Hash;

use fret_core::{
    FillStyle, PathCommand, PathConstraints, PathId, PathStyle, Point, Px, StrokeStyle,
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

#[derive(Debug, Default)]
pub(crate) struct CanvasPaintCache {
    frame: u64,
    wire_paths: HashMap<WirePathKey, PathCacheEntry>,
    marker_paths: HashMap<MarkerPathKey, PathCacheEntry>,
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

        let total = self
            .wire_paths
            .len()
            .saturating_add(self.marker_paths.len());
        if total <= max_entries {
            return;
        }

        #[derive(Debug, Clone, Copy)]
        enum EvictKey {
            Wire(WirePathKey),
            Marker(MarkerPathKey),
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
