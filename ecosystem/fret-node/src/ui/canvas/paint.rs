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

mod paint_markers;
mod paint_ports;
mod paint_text;
mod paint_wire;

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
