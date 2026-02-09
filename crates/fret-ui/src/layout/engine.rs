use fret_core::time::Instant;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use fret_core::{FrameId, NodeId, Point, Px, Rect, Size};
use serde_json::json;
use slotmap::SecondaryMap;
use taffy::{TaffyTree, prelude::NodeId as TaffyNodeId};

use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};

mod flow;
pub(crate) use flow::{build_viewport_flow_subtree, layout_children_from_engine_if_solved};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeContext {
    node: NodeId,
    measured: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutId(TaffyNodeId);

#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutEngineMeasureHotspot {
    pub node: NodeId,
    pub total_time: Duration,
    pub calls: u64,
    pub cache_hits: u64,
}

pub struct TaffyLayoutEngine {
    tree: TaffyTree<NodeContext>,
    node_to_layout: SecondaryMap<NodeId, LayoutId>,
    layout_to_node: HashMap<LayoutId, NodeId>,
    styles: SecondaryMap<NodeId, taffy::Style>,
    children: SecondaryMap<NodeId, Vec<NodeId>>,
    parent: SecondaryMap<NodeId, NodeId>,
    seen_generation: u64,
    seen_stamp: SecondaryMap<NodeId, u64>,
    solve_generation: u64,
    node_solved_stamp: SecondaryMap<NodeId, SolvedStamp>,
    root_solve_stamp: SecondaryMap<NodeId, RootSolveStamp>,
    measure_cache_scratch: HashMap<LayoutMeasureKey, taffy::geometry::Size<f32>>,
    solve_scale_factor: f32,
    frame_id: Option<FrameId>,
    last_solve_time: Duration,
    last_solve_root: Option<NodeId>,
    last_solve_elapsed: Duration,
    last_solve_measure_calls: u64,
    last_solve_measure_cache_hits: u64,
    measure_profiling_enabled: bool,
    last_solve_measure_time: Duration,
    last_solve_measure_hotspots: Vec<LayoutEngineMeasureHotspot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RootSolveKey {
    width_bits: u64,
    height_bits: u64,
    scale_bits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LayoutMeasureKey {
    node: NodeId,
    known_w: Option<u32>,
    known_h: Option<u32>,
    avail_w: (u8, u32),
    avail_h: (u8, u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SolvedStamp {
    frame_id: FrameId,
    solve_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RootSolveStamp {
    frame_id: FrameId,
    key: RootSolveKey,
}

impl Default for TaffyLayoutEngine {
    fn default() -> Self {
        let mut tree = TaffyTree::new();
        tree.enable_rounding();
        Self {
            tree,
            node_to_layout: SecondaryMap::new(),
            layout_to_node: HashMap::new(),
            styles: SecondaryMap::new(),
            children: SecondaryMap::new(),
            parent: SecondaryMap::new(),
            seen_generation: 1,
            seen_stamp: SecondaryMap::new(),
            solve_generation: 0,
            node_solved_stamp: SecondaryMap::new(),
            root_solve_stamp: SecondaryMap::new(),
            measure_cache_scratch: HashMap::new(),
            solve_scale_factor: 1.0,
            frame_id: None,
            last_solve_time: Duration::default(),
            last_solve_root: None,
            last_solve_elapsed: Duration::default(),
            last_solve_measure_calls: 0,
            last_solve_measure_cache_hits: 0,
            measure_profiling_enabled: false,
            last_solve_measure_time: Duration::default(),
            last_solve_measure_hotspots: Vec::new(),
        }
    }
}

impl TaffyLayoutEngine {
    fn saw_this_frame(&self, node: NodeId) -> bool {
        self.seen_stamp.get(node).copied() == Some(self.seen_generation)
    }

    fn invalidate_solved_ancestors(&mut self, mut node: NodeId) {
        while let Some(parent) = self.parent.get(node).copied() {
            self.node_solved_stamp.remove(parent);
            self.root_solve_stamp.remove(parent);
            node = parent;
        }
    }

    pub fn begin_frame(&mut self, frame_id: FrameId) {
        if self.frame_id != Some(frame_id) {
            self.frame_id = Some(frame_id);
            self.seen_generation = self.seen_generation.wrapping_add(1);
            if self.seen_generation == 0 {
                // Keep `seen_generation` non-zero to simplify "not seen this frame" checks.
                self.seen_generation = 1;
                self.seen_stamp.clear();
            }
            self.solve_generation = 0;
            self.solve_scale_factor = 1.0;
            self.last_solve_time = Duration::default();
            self.last_solve_root = None;
            self.last_solve_elapsed = Duration::default();
            self.last_solve_measure_calls = 0;
            self.last_solve_measure_cache_hits = 0;
            self.last_solve_measure_time = Duration::default();
            self.last_solve_measure_hotspots.clear();
        }
    }

    pub fn end_frame(&mut self) {
        let stale: Vec<NodeId> = self
            .node_to_layout
            .iter()
            .filter_map(|(node, _)| {
                let seen = self.seen_stamp.get(node).copied() == Some(self.seen_generation);
                (!seen).then_some(node)
            })
            .collect();

        for node in stale {
            let Some(layout_id) = self.node_to_layout.remove(node) else {
                continue;
            };
            self.layout_to_node.remove(&layout_id);
            self.styles.remove(node);
            self.seen_stamp.remove(node);
            if let Some(children) = self.children.remove(node) {
                for child in children {
                    if self.parent.get(child) == Some(&node) {
                        self.parent.remove(child);
                    }
                }
            }
            self.parent.remove(node);
            self.node_solved_stamp.remove(node);
            self.root_solve_stamp.remove(node);
            let _ = self.tree.remove(layout_id.0);
        }
    }

    pub fn layout_id_for_node(&self, node: NodeId) -> Option<LayoutId> {
        self.node_to_layout.get(node).copied()
    }

    pub(crate) fn mark_seen_if_present(&mut self, node: NodeId) {
        if self.node_to_layout.contains_key(node) {
            self.seen_stamp.insert(node, self.seen_generation);
        }
    }

    pub fn node_for_layout_id(&self, id: LayoutId) -> Option<NodeId> {
        self.layout_to_node.get(&id).copied()
    }

    pub fn solve_count(&self) -> u64 {
        self.solve_generation
    }

    pub fn last_solve_time(&self) -> Duration {
        self.last_solve_time
    }

    pub fn last_solve_root(&self) -> Option<NodeId> {
        self.last_solve_root
    }

    pub fn last_solve_elapsed(&self) -> Duration {
        self.last_solve_elapsed
    }

    pub fn last_solve_measure_calls(&self) -> u64 {
        self.last_solve_measure_calls
    }

    pub fn last_solve_measure_cache_hits(&self) -> u64 {
        self.last_solve_measure_cache_hits
    }

    pub fn last_solve_measure_time(&self) -> Duration {
        self.last_solve_measure_time
    }

    pub fn last_solve_measure_hotspots(&self) -> &[LayoutEngineMeasureHotspot] {
        self.last_solve_measure_hotspots.as_slice()
    }

    pub fn set_measure_profiling_enabled(&mut self, enabled: bool) {
        self.measure_profiling_enabled = enabled;
    }

    pub fn child_layout_rect_if_solved(&self, parent: NodeId, child: NodeId) -> Option<Rect> {
        if self.solve_generation == 0 {
            return None;
        }
        let frame_id = self.frame_id?;
        let parent_stamp = self.node_solved_stamp.get(parent).copied()?;
        let child_stamp = self.node_solved_stamp.get(child).copied()?;
        if parent_stamp.frame_id != frame_id || child_stamp.frame_id != frame_id {
            return None;
        }
        if parent_stamp.solve_generation == 0
            || child_stamp.solve_generation == 0
            || parent_stamp.solve_generation != child_stamp.solve_generation
        {
            return None;
        }
        if !self.saw_this_frame(parent) || !self.saw_this_frame(child) {
            return None;
        }
        if !self
            .children
            .get(parent)
            .is_some_and(|children| children.contains(&child))
        {
            return None;
        }
        self.layout_id_for_node(child)
            .map(|id| self.layout_rect(id))
    }

    pub fn root_is_solved_for(
        &self,
        root: NodeId,
        available: LayoutSize<AvailableSpace>,
        scale_factor: f32,
    ) -> bool {
        if !self.saw_this_frame(root) {
            return false;
        }
        let Some(frame_id) = self.frame_id else {
            return false;
        };
        let solved = self
            .node_solved_stamp
            .get(root)
            .copied()
            .is_some_and(|s| s.frame_id == frame_id && s.solve_generation != 0);
        if !solved {
            return false;
        }

        fn key_bits(axis: AvailableSpace) -> u64 {
            match axis {
                AvailableSpace::Definite(px) => px.0.to_bits() as u64,
                AvailableSpace::MinContent => 1u64 << 32,
                AvailableSpace::MaxContent => 2u64 << 32,
            }
        }

        let sf = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        let key = RootSolveKey {
            width_bits: key_bits(available.width),
            height_bits: key_bits(available.height),
            scale_bits: sf.to_bits(),
        };
        self.root_solve_stamp
            .get(root)
            .copied()
            .is_some_and(|s| s.frame_id == frame_id && s.key == key)
    }

    pub fn compute_root_for_node_with_measure_if_needed(
        &mut self,
        root: NodeId,
        available: LayoutSize<AvailableSpace>,
        scale_factor: f32,
        measure: impl FnMut(NodeId, LayoutConstraints) -> Size,
    ) -> Option<LayoutId> {
        let root_id = self.layout_id_for_node(root)?;
        if !self.root_is_solved_for(root, available, scale_factor) {
            self.compute_root_with_measure(root_id, available, scale_factor, measure);
        }
        Some(root_id)
    }

    pub fn request_layout_node(&mut self, node: NodeId) -> LayoutId {
        self.seen_stamp.insert(node, self.seen_generation);
        if let Some(id) = self.node_to_layout.get(node).copied() {
            return id;
        }

        let taffy_id = self
            .tree
            .new_leaf_with_context(
                Default::default(),
                NodeContext {
                    node,
                    measured: false,
                },
            )
            .expect("taffy new_leaf");
        let id = LayoutId(taffy_id);
        self.node_to_layout.insert(node, id);
        self.layout_to_node.insert(id, node);
        id
    }

    pub fn set_measured(&mut self, node: NodeId, measured: bool) {
        let id = self.request_layout_node(node).0;
        let ctx = self
            .tree
            .get_node_context(id)
            .copied()
            .unwrap_or(NodeContext { node, measured });
        if ctx.node == node && ctx.measured == measured {
            return;
        }
        self.node_solved_stamp.remove(node);
        self.root_solve_stamp.remove(node);
        self.invalidate_solved_ancestors(node);
        let _ = self
            .tree
            .set_node_context(id, Some(NodeContext { node, measured }));
        let _ = self.tree.mark_dirty(id);
    }

    pub fn set_style(&mut self, node: NodeId, style: taffy::Style) {
        let id = self.request_layout_node(node).0;
        if self.styles.get(node) == Some(&style) {
            return;
        }
        self.node_solved_stamp.remove(node);
        self.root_solve_stamp.remove(node);
        self.invalidate_solved_ancestors(node);
        if self.tree.set_style(id, style.clone()).is_ok() {
            self.styles.insert(node, style);
            let _ = self.tree.mark_dirty(id);
        }
    }

    pub fn set_children(&mut self, node: NodeId, children: &[NodeId]) {
        let parent = self.request_layout_node(node).0;

        let prev_children = self.children.get(node).map(|v| v.as_slice()).unwrap_or(&[]);
        if prev_children == children {
            return;
        }
        self.node_solved_stamp.remove(node);
        self.root_solve_stamp.remove(node);
        self.invalidate_solved_ancestors(node);

        if let Some(prev_children) = self.children.get(node) {
            for &child in prev_children.iter() {
                if self.parent.get(child) == Some(&node) {
                    self.parent.remove(child);
                }
            }
        }

        let mut child_nodes: Vec<TaffyNodeId> = Vec::with_capacity(children.len());
        for &child in children {
            let child_id = self.request_layout_node(child).0;
            child_nodes.push(child_id);
            self.parent.insert(child, node);
        }

        if self.tree.set_children(parent, &child_nodes).is_ok() {
            self.children.insert(node, children.to_vec());
            let _ = self.tree.mark_dirty(parent);
        }
    }

    #[stacksafe::stacksafe]
    pub fn compute_root_with_measure(
        &mut self,
        root: LayoutId,
        available: LayoutSize<AvailableSpace>,
        scale_factor: f32,
        mut measure: impl FnMut(NodeId, LayoutConstraints) -> Size,
    ) {
        fn quantize_size_key_bits(value: f32) -> u32 {
            if !value.is_finite() || value <= 0.0 {
                return 0;
            }
            let quantum = 64.0f32;
            let quantized = (value * quantum).round() / quantum;
            quantized.to_bits()
        }

        fn avail_key(avail: taffy::style::AvailableSpace) -> (u8, u32) {
            match avail {
                taffy::style::AvailableSpace::Definite(v) => (0, quantize_size_key_bits(v)),
                taffy::style::AvailableSpace::MinContent => (1, 0),
                taffy::style::AvailableSpace::MaxContent => (2, 0),
            }
        }

        let started = Instant::now();
        let sf = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        self.solve_scale_factor = sf;

        let span = if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace_span!(
                "fret.ui.layout_engine.solve",
                root = tracing::field::Empty,
                frame_id = self.frame_id.map(|f| f.0).unwrap_or(0),
                scale_factor = sf,
                elapsed_us = tracing::field::Empty,
                measure_calls = tracing::field::Empty,
                measure_cache_hits = tracing::field::Empty,
                measure_us = tracing::field::Empty,
            )
        } else {
            tracing::Span::none()
        };
        let _span_guard = span.enter();

        let taffy_available = taffy::geometry::Size {
            width: match available.width {
                AvailableSpace::Definite(px) => taffy::style::AvailableSpace::Definite(px.0 * sf),
                AvailableSpace::MinContent => taffy::style::AvailableSpace::MinContent,
                AvailableSpace::MaxContent => taffy::style::AvailableSpace::MaxContent,
            },
            height: match available.height {
                AvailableSpace::Definite(px) => taffy::style::AvailableSpace::Definite(px.0 * sf),
                AvailableSpace::MinContent => taffy::style::AvailableSpace::MinContent,
                AvailableSpace::MaxContent => taffy::style::AvailableSpace::MaxContent,
            },
        };

        let mut measure_calls: u64 = 0;
        let mut measure_cache_hits: u64 = 0;
        self.measure_cache_scratch.clear();
        let measure_cache = &mut self.measure_cache_scratch;
        let enable_profile = self.measure_profiling_enabled;
        let mut measure_time = Duration::default();

        #[derive(Debug, Clone, Copy, Default)]
        struct MeasureNodeProfile {
            total_time: Duration,
            calls: u64,
            cache_hits: u64,
        }

        let mut by_node: HashMap<NodeId, MeasureNodeProfile> = HashMap::new();
        self.tree
            .compute_layout_with_measure(
                root.0,
                taffy_available,
                |known, avail, _id, ctx, _style| {
                    let Some(ctx) = ctx else {
                        return taffy::geometry::Size::default();
                    };
                    if !ctx.measured {
                        return taffy::geometry::Size::default();
                    }

                    measure_calls = measure_calls.saturating_add(1);
                    let key = LayoutMeasureKey {
                        node: ctx.node,
                        known_w: known.width.map(quantize_size_key_bits),
                        known_h: known.height.map(quantize_size_key_bits),
                        avail_w: avail_key(avail.width),
                        avail_h: avail_key(avail.height),
                    };
                    if let Some(size) = measure_cache.get(&key) {
                        measure_cache_hits = measure_cache_hits.saturating_add(1);
                        if enable_profile {
                            let profile = by_node.entry(ctx.node).or_default();
                            profile.cache_hits = profile.cache_hits.saturating_add(1);
                        }
                        return *size;
                    }

                    let constraints = LayoutConstraints::new(
                        LayoutSize::new(
                            known.width.map(|w| Px(w / sf)),
                            known.height.map(|h| Px(h / sf)),
                        ),
                        LayoutSize::new(
                            match avail.width {
                                taffy::style::AvailableSpace::Definite(w) => {
                                    AvailableSpace::Definite(Px(w / sf))
                                }
                                taffy::style::AvailableSpace::MinContent => {
                                    AvailableSpace::MinContent
                                }
                                taffy::style::AvailableSpace::MaxContent => {
                                    AvailableSpace::MaxContent
                                }
                            },
                            match avail.height {
                                taffy::style::AvailableSpace::Definite(h) => {
                                    AvailableSpace::Definite(Px(h / sf))
                                }
                                taffy::style::AvailableSpace::MinContent => {
                                    AvailableSpace::MinContent
                                }
                                taffy::style::AvailableSpace::MaxContent => {
                                    AvailableSpace::MaxContent
                                }
                            },
                        ),
                    );

                    let (s, elapsed) = if enable_profile {
                        let measure_started = Instant::now();
                        let size = measure(ctx.node, constraints);
                        (size, measure_started.elapsed())
                    } else {
                        (measure(ctx.node, constraints), Duration::default())
                    };

                    if enable_profile {
                        measure_time += elapsed;
                        let profile = by_node.entry(ctx.node).or_default();
                        profile.total_time += elapsed;
                        profile.calls = profile.calls.saturating_add(1);
                    }
                    let out = taffy::geometry::Size {
                        width: s.width.0 * sf,
                        height: s.height.0 * sf,
                    };
                    measure_cache.insert(key, out);
                    out
                },
            )
            .ok();

        self.solve_generation = self.solve_generation.saturating_add(1);
        self.last_solve_measure_calls = measure_calls;
        self.last_solve_measure_cache_hits = measure_cache_hits;
        self.last_solve_measure_time = measure_time;
        if enable_profile {
            const MAX_HOTSPOTS: usize = 8;
            let mut hotspots: Vec<LayoutEngineMeasureHotspot> = by_node
                .into_iter()
                .map(|(node, p)| LayoutEngineMeasureHotspot {
                    node,
                    total_time: p.total_time,
                    calls: p.calls,
                    cache_hits: p.cache_hits,
                })
                .collect();
            hotspots.sort_by_key(|h| std::cmp::Reverse(h.total_time));
            hotspots.truncate(MAX_HOTSPOTS);
            self.last_solve_measure_hotspots = hotspots;
        } else {
            self.last_solve_measure_hotspots.clear();
        }
        if let Some(root_node) = self.node_for_layout_id(root) {
            span.record("root", tracing::field::debug(root_node));
            self.last_solve_root = Some(root_node);
            self.mark_solved_subtree(root_node);
            fn key_bits(axis: AvailableSpace) -> u64 {
                match axis {
                    AvailableSpace::Definite(px) => px.0.to_bits() as u64,
                    AvailableSpace::MinContent => 1u64 << 32,
                    AvailableSpace::MaxContent => 2u64 << 32,
                }
            }
            if let Some(frame_id) = self.frame_id {
                self.root_solve_stamp.insert(
                    root_node,
                    RootSolveStamp {
                        frame_id,
                        key: RootSolveKey {
                            width_bits: key_bits(available.width),
                            height_bits: key_bits(available.height),
                            scale_bits: self.solve_scale_factor.to_bits(),
                        },
                    },
                );
            }
        } else {
            self.last_solve_root = None;
        }
        self.last_solve_elapsed = started.elapsed();
        span.record("elapsed_us", self.last_solve_elapsed.as_micros() as u64);
        span.record("measure_calls", measure_calls);
        span.record("measure_cache_hits", measure_cache_hits);
        span.record("measure_us", measure_time.as_micros() as u64);
        self.last_solve_time += self.last_solve_elapsed;
    }

    pub fn compute_root(
        &mut self,
        root: LayoutId,
        available: LayoutSize<AvailableSpace>,
        scale_factor: f32,
    ) {
        self.compute_root_with_measure(root, available, scale_factor, |_node, _constraints| {
            Size::default()
        });
    }

    pub fn layout_rect(&self, id: LayoutId) -> Rect {
        let Ok(layout) = self.tree.layout(id.0) else {
            return Rect::new(Point::new(Px(0.0), Px(0.0)), Size::default());
        };
        let sf = if self.solve_scale_factor.is_finite() && self.solve_scale_factor > 0.0 {
            self.solve_scale_factor
        } else {
            1.0
        };
        let min_x_dp = layout.location.x;
        let min_y_dp = layout.location.y;
        let max_x_dp = layout.location.x + layout.size.width;
        let max_y_dp = layout.location.y + layout.size.height;

        let snap_edge_dp = |v: f32| if v.is_finite() { v.round() } else { 0.0 };

        let snapped_min_x_dp = snap_edge_dp(min_x_dp);
        let snapped_min_y_dp = snap_edge_dp(min_y_dp);
        let snapped_max_x_dp = snap_edge_dp(max_x_dp);
        let snapped_max_y_dp = snap_edge_dp(max_y_dp);

        Rect::new(
            Point::new(Px(snapped_min_x_dp / sf), Px(snapped_min_y_dp / sf)),
            Size::new(
                Px(((snapped_max_x_dp - snapped_min_x_dp) / sf).max(0.0)),
                Px(((snapped_max_y_dp - snapped_min_y_dp) / sf).max(0.0)),
            ),
        )
    }

    pub fn debug_dump_subtree_json(
        &self,
        root: NodeId,
        mut label_for_node: impl FnMut(NodeId) -> Option<String>,
    ) -> serde_json::Value {
        fn sanitize_for_filename(s: &str) -> String {
            s.chars()
                .map(|ch| match ch {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
                    _ => '_',
                })
                .collect()
        }

        fn abs_rect_for_node(
            engine: &TaffyLayoutEngine,
            node: NodeId,
            abs_cache: &mut HashMap<NodeId, Rect>,
        ) -> Rect {
            if let Some(rect) = abs_cache.get(&node).copied() {
                return rect;
            }

            let local = engine
                .layout_id_for_node(node)
                .map(|id| engine.layout_rect(id))
                .unwrap_or_else(|| Rect::new(Point::new(Px(0.0), Px(0.0)), Size::default()));
            let abs = if let Some(parent) = engine.parent.get(node).copied() {
                let parent_abs = abs_rect_for_node(engine, parent, abs_cache);
                Rect::new(
                    Point::new(
                        Px(parent_abs.origin.x.0 + local.origin.x.0),
                        Px(parent_abs.origin.y.0 + local.origin.y.0),
                    ),
                    local.size,
                )
            } else {
                local
            };
            abs_cache.insert(node, abs);
            abs
        }

        let root_layout_id = self.layout_id_for_node(root);
        let mut stack: Vec<NodeId> = vec![root];
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut abs_cache: HashMap<NodeId, Rect> = HashMap::new();
        let mut nodes: Vec<serde_json::Value> = Vec::new();

        while let Some(node) = stack.pop() {
            if !visited.insert(node) {
                continue;
            }

            let children = self.children.get(node).cloned().unwrap_or_default();
            for &child in children.iter().rev() {
                stack.push(child);
            }

            let layout_id = self.layout_id_for_node(node);
            let local = layout_id
                .map(|id| self.layout_rect(id))
                .unwrap_or_else(|| Rect::new(Point::new(Px(0.0), Px(0.0)), Size::default()));
            let abs = abs_rect_for_node(self, node, &mut abs_cache);

            let measured = layout_id
                .and_then(|id| self.tree.get_node_context(id.0).copied())
                .map(|ctx| ctx.measured)
                .unwrap_or(false);

            let style_dbg = self
                .styles
                .get(node)
                .map(|s| format!("{s:?}"))
                .unwrap_or_else(|| "<missing>".to_string());

            let label_dbg = label_for_node(node).unwrap_or_else(|| "<unknown>".to_string());

            nodes.push(json!({
                "node": format!("{node:?}"),
                "label": label_dbg,
                "measured": measured,
                "parent": self.parent.get(node).map(|p| format!("{p:?}")),
                "children": children.iter().map(|c| format!("{c:?}")).collect::<Vec<_>>(),
                "layout_id": layout_id.map(|id| format!("{:?}", id.0)).unwrap_or_else(|| "<missing>".to_string()),
                "local_rect": {
                    "x": local.origin.x.0,
                    "y": local.origin.y.0,
                    "w": local.size.width.0,
                    "h": local.size.height.0,
                },
                "abs_rect": {
                    "x": abs.origin.x.0,
                    "y": abs.origin.y.0,
                    "w": abs.size.width.0,
                    "h": abs.size.height.0,
                },
                "style": style_dbg,
            }));
        }

        let filename = sanitize_for_filename(&format!("{root:?}"));
        json!({
            "meta": {
                "root": format!("{root:?}"),
                "root_layout_id": root_layout_id.map(|id| format!("{:?}", id.0)),
                "frame_id": self.frame_id.map(|id| id.0),
                "solve_generation": self.solve_generation,
                "solve_scale_factor": self.solve_scale_factor,
                "last_solve_time_ms": self.last_solve_time.as_millis(),
                "suggested_filename": format!("taffy_{filename}.json"),
            },
            "nodes": nodes,
        })
    }

    pub fn debug_write_subtree_json(
        &self,
        root: NodeId,
        dir: impl AsRef<Path>,
        filename: impl AsRef<Path>,
        label_for_node: impl FnMut(NodeId) -> Option<String>,
    ) -> std::io::Result<PathBuf> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;
        let path = dir.join(filename);
        let dump = self.debug_dump_subtree_json(root, label_for_node);
        let bytes = serde_json::to_vec_pretty(&dump).expect("serialize taffy debug json");
        std::fs::write(&path, bytes)?;
        Ok(path)
    }

    fn mark_solved_subtree(&mut self, root: NodeId) {
        let Some(frame_id) = self.frame_id else {
            return;
        };
        let mut stack: Vec<NodeId> = vec![root];
        while let Some(node) = stack.pop() {
            self.node_solved_stamp.insert(
                node,
                SolvedStamp {
                    frame_id,
                    solve_generation: self.solve_generation,
                },
            );
            if let Some(children) = self.children.get(node) {
                stack.extend(children.iter().copied());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::SlotMap;

    fn fresh_node_ids(count: usize) -> Vec<NodeId> {
        let mut map: SlotMap<NodeId, ()> = SlotMap::with_key();
        (0..count).map(|_| map.insert(())).collect()
    }

    #[test]
    fn multiple_roots_do_not_couple_layout_results() {
        let [root_a, root_b, child_a, child_b] = fresh_node_ids(4).try_into().unwrap();

        let mut engine = TaffyLayoutEngine::default();
        engine.begin_frame(FrameId(1));

        engine.set_children(root_a, &[child_a]);
        engine.set_children(root_b, &[child_b]);

        engine.set_style(
            root_a,
            taffy::Style {
                display: taffy::style::Display::Block,
                size: taffy::geometry::Size {
                    width: taffy::style::Dimension::length(100.0),
                    height: taffy::style::Dimension::length(10.0),
                },
                ..Default::default()
            },
        );
        engine.set_style(
            root_b,
            taffy::Style {
                display: taffy::style::Display::Block,
                size: taffy::geometry::Size {
                    width: taffy::style::Dimension::length(200.0),
                    height: taffy::style::Dimension::length(10.0),
                },
                ..Default::default()
            },
        );

        let fill = taffy::Style {
            display: taffy::style::Display::Block,
            size: taffy::geometry::Size {
                width: taffy::style::Dimension::percent(1.0),
                height: taffy::style::Dimension::percent(1.0),
            },
            ..Default::default()
        };
        engine.set_style(child_a, fill.clone());
        engine.set_style(child_b, fill);

        let root_a_id = engine.layout_id_for_node(root_a).unwrap();
        let root_b_id = engine.layout_id_for_node(root_b).unwrap();

        engine.compute_root(
            root_a_id,
            LayoutSize::new(
                AvailableSpace::Definite(Px(100.0)),
                AvailableSpace::Definite(Px(10.0)),
            ),
            1.0,
        );

        let child_a_id = engine.layout_id_for_node(child_a).unwrap();
        let a_before = engine.layout_rect(child_a_id);
        assert!((a_before.size.width.0 - 100.0).abs() < 0.01);
        assert!((a_before.size.height.0 - 10.0).abs() < 0.01);

        engine.compute_root(
            root_b_id,
            LayoutSize::new(
                AvailableSpace::Definite(Px(200.0)),
                AvailableSpace::Definite(Px(10.0)),
            ),
            1.0,
        );

        let child_b_id = engine.layout_id_for_node(child_b).unwrap();
        let b = engine.layout_rect(child_b_id);
        assert!((b.size.width.0 - 200.0).abs() < 0.01);
        assert!((b.size.height.0 - 10.0).abs() < 0.01);

        let a_after = engine.layout_rect(child_a_id);
        assert_eq!(a_before, a_after);

        assert_eq!(
            engine.child_layout_rect_if_solved(root_a, child_a),
            Some(a_after),
            "solved subtree rects should remain readable after solving an unrelated root"
        );
    }

    #[test]
    fn layout_rect_snaps_edges_not_location_and_size() {
        let [root, child] = fresh_node_ids(2).try_into().unwrap();

        let mut engine = TaffyLayoutEngine::default();
        engine.begin_frame(FrameId(1));

        engine.set_children(root, &[child]);

        engine.set_style(
            root,
            taffy::Style {
                display: taffy::style::Display::Block,
                ..Default::default()
            },
        );

        engine.set_style(
            child,
            taffy::Style {
                display: taffy::style::Display::Block,
                size: taffy::geometry::Size {
                    width: taffy::style::Dimension::length(0.5),
                    height: taffy::style::Dimension::length(0.5),
                },
                margin: taffy::geometry::Rect {
                    left: taffy::style::LengthPercentageAuto::length(0.5),
                    right: taffy::style::LengthPercentageAuto::auto(),
                    top: taffy::style::LengthPercentageAuto::auto(),
                    bottom: taffy::style::LengthPercentageAuto::auto(),
                },
                ..Default::default()
            },
        );

        let root_id = engine.layout_id_for_node(root).unwrap();
        engine.compute_root(
            root_id,
            LayoutSize::new(
                AvailableSpace::Definite(Px(10.0)),
                AvailableSpace::Definite(Px(10.0)),
            ),
            2.0,
        );

        let child_id = engine.layout_id_for_node(child).unwrap();
        let rect = engine.layout_rect(child_id);
        assert!((rect.origin.x.0 - 0.5).abs() < 0.0001);
        assert!((rect.size.width.0 - 0.0).abs() < 0.0001);
    }
}
