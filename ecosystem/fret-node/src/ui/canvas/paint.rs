//! Painting routines and render caches for the node graph canvas.
//!
//! The node graph canvas paint path is mostly immediate-mode: each frame it emits `SceneOp`s.
//! However, preparing stroked vector paths can be expensive for large graphs. This module provides
//! a small cache for those prepared paths so that panning (which does not change geometry) does
//! not re-tessellate every edge on every frame.

use std::collections::HashMap;

use fret_core::{PathCommand, PathConstraints, PathId, PathStyle, Point, Px, StrokeStyle};

use crate::ui::presenter::EdgeRouteKind;

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
struct WirePathEntry {
    id: PathId,
    last_used_frame: u64,
}

#[derive(Debug, Default)]
pub(crate) struct CanvasPaintCache {
    frame: u64,
    wire_paths: HashMap<WirePathKey, WirePathEntry>,
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
    }

    pub(crate) fn prune(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        max_age_frames: u64,
        max_entries: usize,
    ) {
        if self.wire_paths.len() <= max_entries {
            // Still prune by age (keeps memory bounded when panning across large graphs).
            let now = self.frame;
            self.wire_paths.retain(|_, entry| {
                let keep = now.saturating_sub(entry.last_used_frame) <= max_age_frames;
                if !keep {
                    services.path().release(entry.id);
                }
                keep
            });
            return;
        }

        // If over budget, apply age pruning first, then drop oldest entries until within budget.
        let now = self.frame;
        self.wire_paths.retain(|_, entry| {
            let keep = now.saturating_sub(entry.last_used_frame) <= max_age_frames;
            if !keep {
                services.path().release(entry.id);
            }
            keep
        });

        if self.wire_paths.len() <= max_entries {
            return;
        }

        let mut entries: Vec<(WirePathKey, u64)> = self
            .wire_paths
            .iter()
            .map(|(k, v)| (*k, v.last_used_frame))
            .collect();
        entries.sort_by_key(|(_k, last)| *last);

        let over = self.wire_paths.len().saturating_sub(max_entries);
        for (key, _last) in entries.into_iter().take(over) {
            if let Some(entry) = self.wire_paths.remove(&key) {
                services.path().release(entry.id);
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
            WirePathEntry {
                id,
                last_used_frame: now,
            },
        );

        Some(id)
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
