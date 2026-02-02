use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::util::{
    now_unix_ms, read_pick_result, read_pick_result_run_id, read_script_result,
    read_script_result_run_id, touch, write_json_value, write_script,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum BundleStatsSort {
    #[default]
    Invalidation,
    Time,
}

impl BundleStatsSort {
    pub(super) fn parse(s: &str) -> Result<Self, String> {
        match s.trim() {
            "invalidation" => Ok(Self::Invalidation),
            "time" => Ok(Self::Time),
            other => Err(format!(
                "invalid --sort value: {other} (expected: invalidation|time)"
            )),
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Invalidation => "invalidation",
            Self::Time => "time",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsReport {
    sort: BundleStatsSort,
    warmup_frames: u64,
    pub(super) windows: u32,
    pub(super) snapshots: u32,
    snapshots_considered: u32,
    snapshots_skipped_warmup: u32,
    pub(super) snapshots_with_model_changes: u32,
    pub(super) snapshots_with_global_changes: u32,
    snapshots_with_propagated_model_changes: u32,
    snapshots_with_propagated_global_changes: u32,
    pub(super) snapshots_with_hover_layout_invalidations: u32,
    sum_layout_time_us: u64,
    sum_prepaint_time_us: u64,
    sum_paint_time_us: u64,
    sum_total_time_us: u64,
    sum_cache_roots: u64,
    sum_cache_roots_reused: u64,
    sum_cache_replayed_ops: u64,
    pub(super) sum_invalidation_walk_calls: u64,
    pub(super) sum_invalidation_walk_nodes: u64,
    sum_model_change_invalidation_roots: u64,
    sum_global_change_invalidation_roots: u64,
    pub(super) sum_hover_layout_invalidations: u64,
    max_layout_time_us: u64,
    max_prepaint_time_us: u64,
    max_paint_time_us: u64,
    max_total_time_us: u64,
    pub(super) max_invalidation_walk_calls: u32,
    pub(super) max_invalidation_walk_nodes: u32,
    max_model_change_invalidation_roots: u32,
    max_global_change_invalidation_roots: u32,
    pub(super) max_hover_layout_invalidations: u32,
    worst_hover_layout: Option<BundleStatsWorstHoverLayout>,
    global_type_hotspots: Vec<BundleStatsGlobalTypeHotspot>,
    model_source_hotspots: Vec<BundleStatsModelSourceHotspot>,
    pub(super) top: Vec<BundleStatsSnapshotRow>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsSnapshotRow {
    pub(super) window: u64,
    pub(super) tick_id: u64,
    pub(super) frame_id: u64,
    pub(super) timestamp_unix_ms: Option<u64>,
    pub(super) layout_time_us: u64,
    pub(super) prepaint_time_us: u64,
    pub(super) paint_time_us: u64,
    pub(super) total_time_us: u64,
    pub(super) layout_nodes_performed: u32,
    pub(super) paint_nodes_performed: u32,
    pub(super) paint_cache_misses: u32,
    pub(super) layout_engine_solves: u64,
    pub(super) layout_engine_solve_time_us: u64,
    pub(super) changed_models: u32,
    pub(super) changed_globals: u32,
    pub(super) changed_global_types_sample: Vec<String>,
    pub(super) propagated_model_change_models: u32,
    pub(super) propagated_model_change_observation_edges: u32,
    pub(super) propagated_model_change_unobserved_models: u32,
    pub(super) propagated_global_change_globals: u32,
    pub(super) propagated_global_change_observation_edges: u32,
    pub(super) propagated_global_change_unobserved_globals: u32,
    pub(super) invalidation_walk_calls: u32,
    pub(super) invalidation_walk_nodes: u32,
    pub(super) model_change_invalidation_roots: u32,
    pub(super) global_change_invalidation_roots: u32,
    pub(super) invalidation_walk_calls_model_change: u32,
    pub(super) invalidation_walk_nodes_model_change: u32,
    pub(super) invalidation_walk_calls_global_change: u32,
    pub(super) invalidation_walk_nodes_global_change: u32,
    pub(super) invalidation_walk_calls_hover: u32,
    pub(super) invalidation_walk_nodes_hover: u32,
    pub(super) invalidation_walk_calls_focus: u32,
    pub(super) invalidation_walk_nodes_focus: u32,
    pub(super) invalidation_walk_calls_other: u32,
    pub(super) invalidation_walk_nodes_other: u32,
    pub(super) top_invalidation_walks: Vec<BundleStatsInvalidationWalk>,
    pub(super) hover_pressable_target_changes: u32,
    pub(super) hover_hover_region_target_changes: u32,
    pub(super) hover_declarative_instance_changes: u32,
    pub(super) hover_declarative_hit_test_invalidations: u32,
    pub(super) hover_declarative_layout_invalidations: u32,
    pub(super) hover_declarative_paint_invalidations: u32,
    pub(super) top_hover_declarative_invalidations:
        Vec<BundleStatsHoverDeclarativeInvalidationHotspot>,
    pub(super) cache_roots: u32,
    pub(super) cache_roots_reused: u32,
    pub(super) cache_roots_contained_relayout: u32,
    pub(super) cache_replayed_ops: u64,
    pub(super) view_cache_contained_relayouts: u32,
    pub(super) set_children_barrier_writes: u32,
    pub(super) barrier_relayouts_scheduled: u32,
    pub(super) barrier_relayouts_performed: u32,
    pub(super) virtual_list_visible_range_checks: u32,
    pub(super) virtual_list_visible_range_refreshes: u32,
    pub(super) top_cache_roots: Vec<BundleStatsCacheRoot>,
    pub(super) top_contained_relayout_cache_roots: Vec<BundleStatsCacheRoot>,
    pub(super) top_layout_engine_solves: Vec<BundleStatsLayoutEngineSolve>,
    pub(super) model_change_hotspots: Vec<BundleStatsModelChangeHotspot>,
    pub(super) model_change_unobserved: Vec<BundleStatsModelChangeUnobserved>,
    pub(super) global_change_hotspots: Vec<BundleStatsGlobalChangeHotspot>,
    pub(super) global_change_unobserved: Vec<BundleStatsGlobalChangeUnobserved>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsHoverDeclarativeInvalidationHotspot {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) hit_test: u32,
    pub(super) layout: u32,
    pub(super) paint: u32,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsWorstHoverLayout {
    window: u64,
    tick_id: u64,
    frame_id: u64,
    hover_declarative_layout_invalidations: u32,
    hotspots: Vec<BundleStatsHoverDeclarativeInvalidationHotspot>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsInvalidationWalk {
    pub(super) root_node: u64,
    pub(super) root_element: Option<u64>,
    pub(super) kind: Option<String>,
    pub(super) source: Option<String>,
    pub(super) detail: Option<String>,
    pub(super) walked_nodes: u32,
    pub(super) truncated_at: Option<u64>,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsCacheRoot {
    pub(super) root_node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_path: Option<String>,
    pub(super) reused: bool,
    pub(super) contained_layout: bool,
    pub(super) contained_relayout_in_frame: bool,
    pub(super) paint_replayed_ops: u32,
    pub(super) reuse_reason: Option<String>,
    pub(super) root_in_semantics: Option<bool>,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutEngineSolve {
    pub(super) root_node: u64,
    pub(super) solve_time_us: u64,
    pub(super) measure_calls: u64,
    pub(super) measure_cache_hits: u64,
    pub(super) measure_time_us: u64,
    pub(super) top_measures: Vec<BundleStatsLayoutEngineMeasureHotspot>,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutEngineMeasureHotspot {
    pub(super) node: u64,
    pub(super) measure_time_us: u64,
    pub(super) calls: u64,
    pub(super) cache_hits: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) top_children: Vec<BundleStatsLayoutEngineMeasureChildHotspot>,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutEngineMeasureChildHotspot {
    pub(super) child: u64,
    pub(super) measure_time_us: u64,
    pub(super) calls: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsModelChangeHotspot {
    model: u64,
    observation_edges: u32,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsModelChangeUnobserved {
    model: u64,
    created_type: Option<String>,
    created_at: Option<String>,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsGlobalChangeHotspot {
    type_name: String,
    observation_edges: u32,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsGlobalChangeUnobserved {
    type_name: String,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsGlobalTypeHotspot {
    type_name: String,
    count: u64,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsModelSourceHotspot {
    source: String,
    count: u64,
}

impl BundleStatsReport {
    pub(super) fn print_human(&self, bundle_path: &Path) {
        println!("bundle: {}", bundle_path.display());
        println!(
            "windows={} snapshots={} considered={} warmup_skipped={} model_changes={} global_changes={} propagated_model_changes={} propagated_global_changes={}",
            self.windows,
            self.snapshots,
            self.snapshots_considered,
            self.snapshots_skipped_warmup,
            self.snapshots_with_model_changes,
            self.snapshots_with_global_changes,
            self.snapshots_with_propagated_model_changes,
            self.snapshots_with_propagated_global_changes
        );
        if self.warmup_frames > 0 {
            println!("warmup_frames={}", self.warmup_frames);
        }
        println!("sort={}", self.sort.as_str());
        println!(
            "time sum (us): total={} layout={} prepaint={} paint={}",
            self.sum_total_time_us,
            self.sum_layout_time_us,
            self.sum_prepaint_time_us,
            self.sum_paint_time_us
        );
        println!(
            "time max (us): total={} layout={} prepaint={} paint={}",
            self.max_total_time_us,
            self.max_layout_time_us,
            self.max_prepaint_time_us,
            self.max_paint_time_us
        );
        println!(
            "cache roots sum: roots={} reused={} replayed_ops={}",
            self.sum_cache_roots, self.sum_cache_roots_reused, self.sum_cache_replayed_ops
        );
        println!(
            "invalidation sum: calls={} nodes={}",
            self.sum_invalidation_walk_calls, self.sum_invalidation_walk_nodes
        );
        println!(
            "invalidation max: calls={} nodes={}",
            self.max_invalidation_walk_calls, self.max_invalidation_walk_nodes
        );
        println!(
            "roots sum: model={} global={}",
            self.sum_model_change_invalidation_roots, self.sum_global_change_invalidation_roots
        );
        println!(
            "roots max: model={} global={}",
            self.max_model_change_invalidation_roots, self.max_global_change_invalidation_roots
        );
        if self.sum_hover_layout_invalidations > 0 || self.max_hover_layout_invalidations > 0 {
            println!(
                "hover decl layout invalidations: sum={} max_per_frame={} frames_with_hover_layout={}",
                self.sum_hover_layout_invalidations,
                self.max_hover_layout_invalidations,
                self.snapshots_with_hover_layout_invalidations
            );
        }

        if !self.global_type_hotspots.is_empty() {
            let items: Vec<String> = self
                .global_type_hotspots
                .iter()
                .map(|h| format!("{}={}", h.type_name, h.count))
                .collect();
            println!("changed_globals_top: {}", items.join(" | "));
        }
        if !self.model_source_hotspots.is_empty() {
            let items: Vec<String> = self
                .model_source_hotspots
                .iter()
                .map(|h| format!("{}={}", h.source, h.count))
                .collect();
            println!("changed_models_top: {}", items.join(" | "));
        }

        if self.top.is_empty() {
            return;
        }

        println!("top (sort={}):", self.sort.as_str());
        for row in &self.top {
            let ts = row
                .timestamp_unix_ms
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string());
            println!(
                "  window={} tick={} frame={} ts={} time.us(total/layout/prepaint/paint)={}/{}/{}/{} layout.solve_us={} paint.cache_misses={} layout.nodes={} paint.nodes={} cache_roots={} cache.reused={} cache.replayed_ops={} contained_relayouts={} cache.contained_relayout_roots={} barrier(set_children/scheduled/performed)={}/{}/{} vlist(range_checks/refreshes)={}/{} inv.calls={} inv.nodes={} by_src.calls(hover/focus/other)={}/{}/{} by_src.nodes(hover/focus/other)={}/{}/{} hover.decl_inv(layout/hit/paint)={}/{}/{} roots.model={} roots.global={} changed.models={} changed.globals={} propagated.models={} propagated.edges={} unobs.models={} propagated.globals={} propagated.global_edges={} unobs.globals={}",
                row.window,
                row.tick_id,
                row.frame_id,
                ts,
                row.total_time_us,
                row.layout_time_us,
                row.prepaint_time_us,
                row.paint_time_us,
                row.layout_engine_solve_time_us,
                row.paint_cache_misses,
                row.layout_nodes_performed,
                row.paint_nodes_performed,
                row.cache_roots,
                row.cache_roots_reused,
                row.cache_replayed_ops,
                row.view_cache_contained_relayouts,
                row.cache_roots_contained_relayout,
                row.set_children_barrier_writes,
                row.barrier_relayouts_scheduled,
                row.barrier_relayouts_performed,
                row.virtual_list_visible_range_checks,
                row.virtual_list_visible_range_refreshes,
                row.invalidation_walk_calls,
                row.invalidation_walk_nodes,
                row.invalidation_walk_calls_hover,
                row.invalidation_walk_calls_focus,
                row.invalidation_walk_calls_other,
                row.invalidation_walk_nodes_hover,
                row.invalidation_walk_nodes_focus,
                row.invalidation_walk_nodes_other,
                row.hover_declarative_layout_invalidations,
                row.hover_declarative_hit_test_invalidations,
                row.hover_declarative_paint_invalidations,
                row.model_change_invalidation_roots,
                row.global_change_invalidation_roots,
                row.changed_models,
                row.changed_globals,
                row.propagated_model_change_models,
                row.propagated_model_change_observation_edges,
                row.propagated_model_change_unobserved_models,
                row.propagated_global_change_globals,
                row.propagated_global_change_observation_edges,
                row.propagated_global_change_unobserved_globals
            );
            if !row.top_invalidation_walks.is_empty() {
                let items: Vec<String> = row
                    .top_invalidation_walks
                    .iter()
                    .take(3)
                    .map(|w| {
                        let mut s = format!(
                            "nodes={} src={} kind={} root={}",
                            w.walked_nodes,
                            w.source.as_deref().unwrap_or("?"),
                            w.kind.as_deref().unwrap_or("?"),
                            w.root_node
                        );
                        if let Some(detail) = w.detail.as_deref()
                            && !detail.is_empty()
                        {
                            s.push_str(&format!(" detail={detail}"));
                        }
                        if let Some(test_id) = w.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={}", test_id));
                        }
                        if let Some(role) = w.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={}", role));
                        }
                        if let Some(el) = w.root_element {
                            s.push_str(&format!(" element={}", el));
                        }
                        if let Some(trunc) = w.truncated_at {
                            s.push_str(&format!(" trunc_at={}", trunc));
                        }
                        s
                    })
                    .collect();
                println!("    top_walks: {}", items.join(" | "));
            }
            if !row.top_cache_roots.is_empty() {
                let items: Vec<String> = row
                    .top_cache_roots
                    .iter()
                    .take(3)
                    .map(|c| {
                        let mut s = format!(
                            "ops={} reused={} root={} reason={}",
                            c.paint_replayed_ops,
                            c.reused,
                            c.root_node,
                            c.reuse_reason.as_deref().unwrap_or("?")
                        );
                        if let Some(test_id) = c.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = c.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = c.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = c.element_path.as_deref()
                            && !path.is_empty()
                        {
                            s.push_str(&format!(" path={path}"));
                        }
                        if let Some(in_sem) = c.root_in_semantics {
                            s.push_str(&format!(" root_in_semantics={in_sem}"));
                        }
                        s
                    })
                    .collect();
                println!("    top_cache_roots: {}", items.join(" | "));
            }
            if !row.top_contained_relayout_cache_roots.is_empty() {
                let items: Vec<String> = row
                    .top_contained_relayout_cache_roots
                    .iter()
                    .take(3)
                    .map(|c| {
                        let mut s = format!(
                            "ops={} reused={} root={} reason={}",
                            c.paint_replayed_ops,
                            c.reused,
                            c.root_node,
                            c.reuse_reason.as_deref().unwrap_or("?")
                        );
                        if let Some(test_id) = c.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = c.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = c.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = c.element_path.as_deref()
                            && !path.is_empty()
                        {
                            s.push_str(&format!(" path={path}"));
                        }
                        if let Some(in_sem) = c.root_in_semantics {
                            s.push_str(&format!(" root_in_semantics={in_sem}"));
                        }
                        s
                    })
                    .collect();
                println!(
                    "    top_contained_relayout_cache_roots: {}",
                    items.join(" | ")
                );
            }
            if row.hover_declarative_layout_invalidations > 0
                && !row.top_hover_declarative_invalidations.is_empty()
            {
                let items: Vec<String> = row
                    .top_hover_declarative_invalidations
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!(
                            "layout={} hit={} paint={} node={}",
                            h.layout, h.hit_test, h.paint, h.node
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = h.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        s
                    })
                    .collect();
                println!("    hover_layout_hotspots: {}", items.join(" | "));
            }
            if !row.top_layout_engine_solves.is_empty() {
                let items: Vec<String> = row
                    .top_layout_engine_solves
                    .iter()
                    .take(3)
                    .map(|s| {
                        let mut out = format!(
                            "us={} measure.us={} measure.calls={} hits={} root={}",
                            s.solve_time_us,
                            s.measure_time_us,
                            s.measure_calls,
                            s.measure_cache_hits,
                            s.root_node
                        );
                        if let Some(test_id) = s.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            out.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = s.root_role.as_deref()
                            && !role.is_empty()
                        {
                            out.push_str(&format!(" role={role}"));
                        }
                        if let Some(m) = s.top_measures.first() {
                            if m.measure_time_us > 0 && m.node != 0 {
                                out.push_str(&format!(
                                    " top_measure.us={} node={}",
                                    m.measure_time_us, m.node
                                ));
                                if let Some(kind) = m.element_kind.as_deref()
                                    && !kind.is_empty()
                                {
                                    out.push_str(&format!(" kind={kind}"));
                                }
                                if let Some(el) = m.element {
                                    out.push_str(&format!(" element={el}"));
                                }
                                if let Some(test_id) = m.test_id.as_deref()
                                    && !test_id.is_empty()
                                {
                                    out.push_str(&format!(" test_id={test_id}"));
                                }
                                if let Some(role) = m.role.as_deref()
                                    && !role.is_empty()
                                {
                                    out.push_str(&format!(" role={role}"));
                                }
                                if let Some(c) = m.top_children.first() {
                                    if c.measure_time_us > 0 && c.child != 0 {
                                        out.push_str(&format!(
                                            " child.us={} child={}",
                                            c.measure_time_us, c.child
                                        ));
                                        if let Some(kind) = c.element_kind.as_deref()
                                            && !kind.is_empty()
                                        {
                                            out.push_str(&format!(" child.kind={kind}"));
                                        }
                                        if let Some(el) = c.element {
                                            out.push_str(&format!(" child.element={el}"));
                                        }
                                        if let Some(test_id) = c.test_id.as_deref()
                                            && !test_id.is_empty()
                                        {
                                            out.push_str(&format!(" child.test_id={test_id}"));
                                        }
                                        if let Some(role) = c.role.as_deref()
                                            && !role.is_empty()
                                        {
                                            out.push_str(&format!(" child.role={role}"));
                                        }
                                    }
                                }
                            }
                        }
                        out
                    })
                    .collect();
                println!("    top_layout_engine_solves: {}", items.join(" | "));
            }
            if !row.model_change_hotspots.is_empty() {
                let items: Vec<String> = row
                    .model_change_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!("{}={}", h.model, h.observation_edges);
                        if let Some(at) = h.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    hot_models: {}", items.join(" | "));
            }
            if !row.model_change_unobserved.is_empty() {
                let items: Vec<String> = row
                    .model_change_unobserved
                    .iter()
                    .take(3)
                    .map(|u| {
                        let mut s = format!("{}", u.model);
                        if let Some(ty) = u.created_type.as_deref() {
                            s.push_str(&format!("={}", ty));
                        }
                        if let Some(at) = u.created_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        if let Some(at) = u.changed_at.as_deref() {
                            s.push_str(&format!(" changed@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    unobs_models: {}", items.join(" | "));
            }
            if !row.global_change_hotspots.is_empty() {
                let items: Vec<String> = row
                    .global_change_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!("{}={}", h.type_name, h.observation_edges);
                        if let Some(at) = h.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    hot_globals: {}", items.join(" | "));
            }
            if !row.global_change_unobserved.is_empty() {
                let items: Vec<String> = row
                    .global_change_unobserved
                    .iter()
                    .take(3)
                    .map(|u| {
                        let mut s = u.type_name.clone();
                        if let Some(at) = u.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    unobs_globals: {}", items.join(" | "));
            }
            if !row.changed_global_types_sample.is_empty() {
                println!(
                    "    changed_globals: {}",
                    row.changed_global_types_sample.join(" | ")
                );
            }
        }
    }

    pub(super) fn to_json(&self) -> serde_json::Value {
        use serde_json::{Map, Value};

        let mut root = Map::new();
        root.insert("schema_version".to_string(), Value::from(1));
        root.insert("sort".to_string(), Value::from(self.sort.as_str()));
        root.insert("warmup_frames".to_string(), Value::from(self.warmup_frames));
        root.insert("windows".to_string(), Value::from(self.windows));
        root.insert("snapshots".to_string(), Value::from(self.snapshots));
        root.insert(
            "snapshots_considered".to_string(),
            Value::from(self.snapshots_considered),
        );
        root.insert(
            "snapshots_skipped_warmup".to_string(),
            Value::from(self.snapshots_skipped_warmup),
        );
        root.insert(
            "snapshots_with_model_changes".to_string(),
            Value::from(self.snapshots_with_model_changes),
        );
        root.insert(
            "snapshots_with_global_changes".to_string(),
            Value::from(self.snapshots_with_global_changes),
        );
        root.insert(
            "snapshots_with_propagated_model_changes".to_string(),
            Value::from(self.snapshots_with_propagated_model_changes),
        );
        root.insert(
            "snapshots_with_propagated_global_changes".to_string(),
            Value::from(self.snapshots_with_propagated_global_changes),
        );
        root.insert(
            "snapshots_with_hover_layout_invalidations".to_string(),
            Value::from(self.snapshots_with_hover_layout_invalidations),
        );

        let mut sum = Map::new();
        sum.insert(
            "layout_time_us".to_string(),
            Value::from(self.sum_layout_time_us),
        );
        sum.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.sum_prepaint_time_us),
        );
        sum.insert(
            "paint_time_us".to_string(),
            Value::from(self.sum_paint_time_us),
        );
        sum.insert(
            "total_time_us".to_string(),
            Value::from(self.sum_total_time_us),
        );
        sum.insert("cache_roots".to_string(), Value::from(self.sum_cache_roots));
        sum.insert(
            "cache_roots_reused".to_string(),
            Value::from(self.sum_cache_roots_reused),
        );
        sum.insert(
            "cache_replayed_ops".to_string(),
            Value::from(self.sum_cache_replayed_ops),
        );
        sum.insert(
            "invalidation_walk_calls".to_string(),
            Value::from(self.sum_invalidation_walk_calls),
        );
        sum.insert(
            "invalidation_walk_nodes".to_string(),
            Value::from(self.sum_invalidation_walk_nodes),
        );
        sum.insert(
            "model_change_invalidation_roots".to_string(),
            Value::from(self.sum_model_change_invalidation_roots),
        );
        sum.insert(
            "global_change_invalidation_roots".to_string(),
            Value::from(self.sum_global_change_invalidation_roots),
        );
        sum.insert(
            "hover_layout_invalidations".to_string(),
            Value::from(self.sum_hover_layout_invalidations),
        );
        root.insert("sum".to_string(), Value::Object(sum));

        let mut max = Map::new();
        max.insert(
            "layout_time_us".to_string(),
            Value::from(self.max_layout_time_us),
        );
        max.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.max_prepaint_time_us),
        );
        max.insert(
            "paint_time_us".to_string(),
            Value::from(self.max_paint_time_us),
        );
        max.insert(
            "total_time_us".to_string(),
            Value::from(self.max_total_time_us),
        );
        max.insert(
            "invalidation_walk_calls".to_string(),
            Value::from(self.max_invalidation_walk_calls),
        );
        max.insert(
            "invalidation_walk_nodes".to_string(),
            Value::from(self.max_invalidation_walk_nodes),
        );
        max.insert(
            "model_change_invalidation_roots".to_string(),
            Value::from(self.max_model_change_invalidation_roots),
        );
        max.insert(
            "global_change_invalidation_roots".to_string(),
            Value::from(self.max_global_change_invalidation_roots),
        );
        max.insert(
            "hover_layout_invalidations".to_string(),
            Value::from(self.max_hover_layout_invalidations),
        );
        root.insert("max".to_string(), Value::Object(max));

        let global_type_hotspots = self
            .global_type_hotspots
            .iter()
            .map(|h| {
                let mut obj = Map::new();
                obj.insert("type_name".to_string(), Value::from(h.type_name.clone()));
                obj.insert("count".to_string(), Value::from(h.count));
                Value::Object(obj)
            })
            .collect::<Vec<_>>();
        root.insert(
            "global_type_hotspots".to_string(),
            Value::Array(global_type_hotspots),
        );
        let model_source_hotspots = self
            .model_source_hotspots
            .iter()
            .map(|h| {
                let mut obj = Map::new();
                obj.insert("source".to_string(), Value::from(h.source.clone()));
                obj.insert("count".to_string(), Value::from(h.count));
                Value::Object(obj)
            })
            .collect::<Vec<_>>();
        root.insert(
            "model_source_hotspots".to_string(),
            Value::Array(model_source_hotspots),
        );

        let top = self
            .top
            .iter()
            .map(|row| {
                let mut obj = Map::new();
                obj.insert("window".to_string(), Value::from(row.window));
                obj.insert("tick_id".to_string(), Value::from(row.tick_id));
                obj.insert("frame_id".to_string(), Value::from(row.frame_id));
                obj.insert(
                    "timestamp_unix_ms".to_string(),
                    row.timestamp_unix_ms
                        .map(Value::from)
                        .unwrap_or(Value::Null),
                );
                obj.insert(
                    "layout_time_us".to_string(),
                    Value::from(row.layout_time_us),
                );
                obj.insert(
                    "prepaint_time_us".to_string(),
                    Value::from(row.prepaint_time_us),
                );
                obj.insert("paint_time_us".to_string(), Value::from(row.paint_time_us));
                obj.insert("total_time_us".to_string(), Value::from(row.total_time_us));
                obj.insert(
                    "layout_nodes_performed".to_string(),
                    Value::from(row.layout_nodes_performed),
                );
                obj.insert(
                    "paint_nodes_performed".to_string(),
                    Value::from(row.paint_nodes_performed),
                );
                obj.insert(
                    "paint_cache_misses".to_string(),
                    Value::from(row.paint_cache_misses),
                );
                obj.insert(
                    "layout_engine_solves".to_string(),
                    Value::from(row.layout_engine_solves),
                );
                obj.insert(
                    "layout_engine_solve_time_us".to_string(),
                    Value::from(row.layout_engine_solve_time_us),
                );
                obj.insert("cache_roots".to_string(), Value::from(row.cache_roots));
                obj.insert(
                    "cache_roots_reused".to_string(),
                    Value::from(row.cache_roots_reused),
                );
                obj.insert(
                    "cache_roots_contained_relayout".to_string(),
                    Value::from(row.cache_roots_contained_relayout),
                );
                obj.insert(
                    "cache_replayed_ops".to_string(),
                    Value::from(row.cache_replayed_ops),
                );
                obj.insert(
                    "changed_models".to_string(),
                    Value::from(row.changed_models),
                );
                obj.insert(
                    "changed_globals".to_string(),
                    Value::from(row.changed_globals),
                );
                obj.insert(
                    "changed_global_types_sample".to_string(),
                    Value::Array(
                        row.changed_global_types_sample
                            .iter()
                            .cloned()
                            .map(Value::from)
                            .collect(),
                    ),
                );
                obj.insert(
                    "propagated_model_change_models".to_string(),
                    Value::from(row.propagated_model_change_models),
                );
                obj.insert(
                    "propagated_model_change_observation_edges".to_string(),
                    Value::from(row.propagated_model_change_observation_edges),
                );
                obj.insert(
                    "propagated_model_change_unobserved_models".to_string(),
                    Value::from(row.propagated_model_change_unobserved_models),
                );
                obj.insert(
                    "propagated_global_change_globals".to_string(),
                    Value::from(row.propagated_global_change_globals),
                );
                obj.insert(
                    "propagated_global_change_observation_edges".to_string(),
                    Value::from(row.propagated_global_change_observation_edges),
                );
                obj.insert(
                    "propagated_global_change_unobserved_globals".to_string(),
                    Value::from(row.propagated_global_change_unobserved_globals),
                );
                obj.insert(
                    "invalidation_walk_calls".to_string(),
                    Value::from(row.invalidation_walk_calls),
                );
                obj.insert(
                    "invalidation_walk_nodes".to_string(),
                    Value::from(row.invalidation_walk_nodes),
                );
                obj.insert(
                    "model_change_invalidation_roots".to_string(),
                    Value::from(row.model_change_invalidation_roots),
                );
                obj.insert(
                    "global_change_invalidation_roots".to_string(),
                    Value::from(row.global_change_invalidation_roots),
                );
                obj.insert(
                    "invalidation_walk_calls_model_change".to_string(),
                    Value::from(row.invalidation_walk_calls_model_change),
                );
                obj.insert(
                    "invalidation_walk_nodes_model_change".to_string(),
                    Value::from(row.invalidation_walk_nodes_model_change),
                );
                obj.insert(
                    "invalidation_walk_calls_global_change".to_string(),
                    Value::from(row.invalidation_walk_calls_global_change),
                );
                obj.insert(
                    "invalidation_walk_nodes_global_change".to_string(),
                    Value::from(row.invalidation_walk_nodes_global_change),
                );
                obj.insert(
                    "invalidation_walk_calls_hover".to_string(),
                    Value::from(row.invalidation_walk_calls_hover),
                );
                obj.insert(
                    "invalidation_walk_nodes_hover".to_string(),
                    Value::from(row.invalidation_walk_nodes_hover),
                );
                obj.insert(
                    "invalidation_walk_calls_focus".to_string(),
                    Value::from(row.invalidation_walk_calls_focus),
                );
                obj.insert(
                    "invalidation_walk_nodes_focus".to_string(),
                    Value::from(row.invalidation_walk_nodes_focus),
                );
                obj.insert(
                    "invalidation_walk_calls_other".to_string(),
                    Value::from(row.invalidation_walk_calls_other),
                );
                obj.insert(
                    "invalidation_walk_nodes_other".to_string(),
                    Value::from(row.invalidation_walk_nodes_other),
                );
                obj.insert(
                    "hover_pressable_target_changes".to_string(),
                    Value::from(row.hover_pressable_target_changes),
                );
                obj.insert(
                    "hover_hover_region_target_changes".to_string(),
                    Value::from(row.hover_hover_region_target_changes),
                );
                obj.insert(
                    "hover_declarative_instance_changes".to_string(),
                    Value::from(row.hover_declarative_instance_changes),
                );
                obj.insert(
                    "hover_declarative_hit_test_invalidations".to_string(),
                    Value::from(row.hover_declarative_hit_test_invalidations),
                );
                obj.insert(
                    "hover_declarative_layout_invalidations".to_string(),
                    Value::from(row.hover_declarative_layout_invalidations),
                );
                obj.insert(
                    "hover_declarative_paint_invalidations".to_string(),
                    Value::from(row.hover_declarative_paint_invalidations),
                );

                let top_invalidation_walks = row
                    .top_invalidation_walks
                    .iter()
                    .map(|w| {
                        let mut w_obj = Map::new();
                        w_obj.insert("root_node".to_string(), Value::from(w.root_node));
                        w_obj.insert(
                            "root_element".to_string(),
                            w.root_element.map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "kind".to_string(),
                            w.kind.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "source".to_string(),
                            w.source.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "detail".to_string(),
                            w.detail.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert("walked_nodes".to_string(), Value::from(w.walked_nodes));
                        w_obj.insert(
                            "truncated_at".to_string(),
                            w.truncated_at.map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "root_role".to_string(),
                            w.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "root_test_id".to_string(),
                            w.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(w_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_invalidation_walks".to_string(),
                    Value::Array(top_invalidation_walks),
                );

                let top_hover_declarative_invalidations = row
                    .top_hover_declarative_invalidations
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("node".to_string(), Value::from(h.node));
                        h_obj.insert(
                            "element".to_string(),
                            h.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert("hit_test".to_string(), Value::from(h.hit_test));
                        h_obj.insert("layout".to_string(), Value::from(h.layout));
                        h_obj.insert("paint".to_string(), Value::from(h.paint));
                        h_obj.insert(
                            "role".to_string(),
                            h.role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "test_id".to_string(),
                            h.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_hover_declarative_invalidations".to_string(),
                    Value::Array(top_hover_declarative_invalidations),
                );

                let top_cache_roots = row
                    .top_cache_roots
                    .iter()
                    .map(|c| {
                        let mut c_obj = Map::new();
                        c_obj.insert("root_node".to_string(), Value::from(c.root_node));
                        c_obj.insert(
                            "element".to_string(),
                            c.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "element_path".to_string(),
                            c.element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert("reused".to_string(), Value::from(c.reused));
                        c_obj.insert(
                            "contained_layout".to_string(),
                            Value::from(c.contained_layout),
                        );
                        c_obj.insert(
                            "contained_relayout_in_frame".to_string(),
                            Value::from(c.contained_relayout_in_frame),
                        );
                        c_obj.insert(
                            "paint_replayed_ops".to_string(),
                            Value::from(c.paint_replayed_ops),
                        );
                        c_obj.insert(
                            "reuse_reason".to_string(),
                            c.reuse_reason
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_in_semantics".to_string(),
                            c.root_in_semantics.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_role".to_string(),
                            c.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_test_id".to_string(),
                            c.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(c_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert("top_cache_roots".to_string(), Value::Array(top_cache_roots));

                let top_contained_relayout_cache_roots = row
                    .top_contained_relayout_cache_roots
                    .iter()
                    .map(|c| {
                        let mut c_obj = Map::new();
                        c_obj.insert("root_node".to_string(), Value::from(c.root_node));
                        c_obj.insert(
                            "element".to_string(),
                            c.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "element_path".to_string(),
                            c.element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert("reused".to_string(), Value::from(c.reused));
                        c_obj.insert(
                            "contained_layout".to_string(),
                            Value::from(c.contained_layout),
                        );
                        c_obj.insert(
                            "contained_relayout_in_frame".to_string(),
                            Value::from(c.contained_relayout_in_frame),
                        );
                        c_obj.insert(
                            "paint_replayed_ops".to_string(),
                            Value::from(c.paint_replayed_ops),
                        );
                        c_obj.insert(
                            "reuse_reason".to_string(),
                            c.reuse_reason
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_in_semantics".to_string(),
                            c.root_in_semantics.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_role".to_string(),
                            c.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_test_id".to_string(),
                            c.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(c_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_contained_relayout_cache_roots".to_string(),
                    Value::Array(top_contained_relayout_cache_roots),
                );

                let top_layout_engine_solves = row
                    .top_layout_engine_solves
                    .iter()
                    .map(|s| {
                        let mut s_obj = Map::new();
                        s_obj.insert("root_node".to_string(), Value::from(s.root_node));
                        s_obj.insert("solve_time_us".to_string(), Value::from(s.solve_time_us));
                        s_obj.insert("measure_calls".to_string(), Value::from(s.measure_calls));
                        s_obj.insert(
                            "measure_cache_hits".to_string(),
                            Value::from(s.measure_cache_hits),
                        );
                        s_obj.insert(
                            "measure_time_us".to_string(),
                            Value::from(s.measure_time_us),
                        );
                        let top_measures = s
                            .top_measures
                            .iter()
                            .map(|m| {
                                let mut m_obj = Map::new();
                                m_obj.insert("node".to_string(), Value::from(m.node));
                                m_obj.insert(
                                    "measure_time_us".to_string(),
                                    Value::from(m.measure_time_us),
                                );
                                m_obj.insert("calls".to_string(), Value::from(m.calls));
                                m_obj.insert("cache_hits".to_string(), Value::from(m.cache_hits));
                                m_obj.insert(
                                    "element".to_string(),
                                    m.element.map(Value::from).unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "element_kind".to_string(),
                                    m.element_kind
                                        .clone()
                                        .map(Value::from)
                                        .unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "role".to_string(),
                                    m.role.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "test_id".to_string(),
                                    m.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                let top_children = m
                                    .top_children
                                    .iter()
                                    .map(|c| {
                                        let mut c_obj = Map::new();
                                        c_obj.insert("child".to_string(), Value::from(c.child));
                                        c_obj.insert(
                                            "measure_time_us".to_string(),
                                            Value::from(c.measure_time_us),
                                        );
                                        c_obj.insert("calls".to_string(), Value::from(c.calls));
                                        c_obj.insert(
                                            "element".to_string(),
                                            c.element.map(Value::from).unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "element_kind".to_string(),
                                            c.element_kind
                                                .clone()
                                                .map(Value::from)
                                                .unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "role".to_string(),
                                            c.role.clone().map(Value::from).unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "test_id".to_string(),
                                            c.test_id
                                                .clone()
                                                .map(Value::from)
                                                .unwrap_or(Value::Null),
                                        );
                                        Value::Object(c_obj)
                                    })
                                    .collect::<Vec<_>>();
                                m_obj
                                    .insert("top_children".to_string(), Value::Array(top_children));
                                Value::Object(m_obj)
                            })
                            .collect::<Vec<_>>();
                        s_obj.insert("top_measures".to_string(), Value::Array(top_measures));
                        s_obj.insert(
                            "root_role".to_string(),
                            s.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        s_obj.insert(
                            "root_test_id".to_string(),
                            s.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(s_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_layout_engine_solves".to_string(),
                    Value::Array(top_layout_engine_solves),
                );

                let model_change_hotspots = row
                    .model_change_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("model".to_string(), Value::from(h.model));
                        h_obj.insert(
                            "observation_edges".to_string(),
                            Value::from(h.observation_edges),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "model_change_hotspots".to_string(),
                    Value::Array(model_change_hotspots),
                );

                let model_change_unobserved = row
                    .model_change_unobserved
                    .iter()
                    .map(|u| {
                        let mut u_obj = Map::new();
                        u_obj.insert("model".to_string(), Value::from(u.model));
                        u_obj.insert(
                            "created_type".to_string(),
                            u.created_type
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        u_obj.insert(
                            "created_at".to_string(),
                            u.created_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(u_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "model_change_unobserved".to_string(),
                    Value::Array(model_change_unobserved),
                );

                let global_change_hotspots = row
                    .global_change_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("type_name".to_string(), Value::from(h.type_name.clone()));
                        h_obj.insert(
                            "observation_edges".to_string(),
                            Value::from(h.observation_edges),
                        );
                        h_obj.insert(
                            "changed_at".to_string(),
                            h.changed_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "global_change_hotspots".to_string(),
                    Value::Array(global_change_hotspots),
                );

                let global_change_unobserved = row
                    .global_change_unobserved
                    .iter()
                    .map(|u| {
                        let mut u_obj = Map::new();
                        u_obj.insert("type_name".to_string(), Value::from(u.type_name.clone()));
                        u_obj.insert(
                            "changed_at".to_string(),
                            u.changed_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(u_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "global_change_unobserved".to_string(),
                    Value::Array(global_change_unobserved),
                );

                Value::Object(obj)
            })
            .collect::<Vec<_>>();

        root.insert("top".to_string(), Value::Array(top));
        Value::Object(root)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct BundleStatsOptions {
    pub(super) warmup_frames: u64,
}

pub(super) fn bundle_stats_from_path(
    bundle_path: &Path,
    top: usize,
    sort: BundleStatsSort,
    opts: BundleStatsOptions,
) -> Result<BundleStatsReport, String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    bundle_stats_from_json_with_options(&bundle, top, sort, opts)
}

pub(super) fn check_bundle_for_stale_paint(
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_stale_paint_json(&bundle, bundle_path, test_id, eps)
}

pub(super) fn check_bundle_for_stale_paint_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut suspicious: Vec<String> = Vec::new();
    let mut missing_scene_fingerprint = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_y: Option<f64> = None;
        let mut prev_fp: Option<u64> = None;
        for s in snaps {
            let y = semantics_node_y_for_test_id(s, test_id);
            let fp = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if fp.is_none() {
                missing_scene_fingerprint = true;
            }
            let (Some(y), Some(fp)) = (y, fp) else {
                prev_y = y;
                prev_fp = fp;
                continue;
            };

            if let (Some(prev_y), Some(prev_fp)) = (prev_y, prev_fp) {
                if (y - prev_y).abs() >= eps as f64 && fp == prev_fp {
                    let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let paint_nodes_performed = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_nodes_performed"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let paint_replayed_ops = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_cache_replayed_ops"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    suspicious.push(format!(
                        "window={window_id} tick={tick_id} frame={frame_id} test_id={test_id} delta_y={:.2} scene_fingerprint=0x{:016x} paint_nodes_performed={paint_nodes_performed} paint_cache_replayed_ops={paint_replayed_ops}",
                        y - prev_y,
                        fp
                    ));
                    if suspicious.len() >= 8 {
                        break;
                    }
                }
            }

            prev_y = Some(y);
            prev_fp = Some(fp);
        }
    }

    if missing_scene_fingerprint {
        return Err(format!(
            "stale paint check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if suspicious.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "stale paint suspected (semantics bounds moved but scene fingerprint did not change)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in suspicious {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn check_bundle_for_stale_scene(
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_stale_scene_json(&bundle, bundle_path, test_id, eps)
}

#[derive(Debug, Clone, Default)]
pub(super) struct SemanticsChangedRepaintedScan {
    missing_scene_fingerprint: bool,
    missing_semantics_fingerprint: bool,
    suspicious_lines: Vec<String>,
    pub(super) findings: Vec<serde_json::Value>,
}

pub(super) fn check_bundle_for_semantics_changed_repainted(
    bundle_path: &Path,
    warmup_frames: u64,
    dump_json: bool,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let scan = scan_semantics_changed_repainted_json(&bundle, warmup_frames);
    if dump_json && !scan.findings.is_empty() {
        let out_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
        let out_path = out_dir.join("check.semantics_changed_repainted.json");
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "semantics_changed_repainted",
            "bundle_json": bundle_path.display().to_string(),
            "warmup_frames": warmup_frames,
            "findings": scan.findings,
        });
        let _ = write_json_value(&out_path, &payload);
    }

    check_bundle_for_semantics_changed_repainted_json(&bundle, bundle_path, warmup_frames)
}

pub(super) fn check_bundle_for_semantics_changed_repainted_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let scan = scan_semantics_changed_repainted_json(bundle, warmup_frames);

    if scan.missing_scene_fingerprint {
        return Err(format!(
            "semantics repaint check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if scan.missing_semantics_fingerprint {
        return Err(format!(
            "semantics repaint check requires `semantics_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if scan.suspicious_lines.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "missing repaint suspected (semantics fingerprint changed but scene fingerprint did not)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in scan.suspicious_lines {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn scan_semantics_changed_repainted_json(
    bundle: &serde_json::Value,
    warmup_frames: u64,
) -> SemanticsChangedRepaintedScan {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    if windows.is_empty() {
        return SemanticsChangedRepaintedScan::default();
    }

    let mut scan = SemanticsChangedRepaintedScan::default();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_scene_fingerprint: Option<u64> = None;
        let mut prev_semantics_fingerprint: Option<u64> = None;
        let mut prev_tick_id: u64 = 0;
        let mut prev_frame_id: u64 = 0;
        let mut prev_snapshot: Option<&serde_json::Value> = None;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let scene_fingerprint = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if scene_fingerprint.is_none() {
                scan.missing_scene_fingerprint = true;
            }

            let semantics_fingerprint = s.get("semantics_fingerprint").and_then(|v| v.as_u64());
            if semantics_fingerprint.is_none() {
                scan.missing_semantics_fingerprint = true;
            }

            let (Some(scene_fingerprint), Some(semantics_fingerprint)) =
                (scene_fingerprint, semantics_fingerprint)
            else {
                prev_scene_fingerprint = None;
                prev_semantics_fingerprint = None;
                prev_tick_id = tick_id;
                prev_frame_id = frame_id;
                prev_snapshot = Some(s);
                continue;
            };

            if let (Some(prev_scene), Some(prev_sem)) =
                (prev_scene_fingerprint, prev_semantics_fingerprint)
            {
                let semantics_changed = semantics_fingerprint != prev_sem;
                let scene_unchanged = scene_fingerprint == prev_scene;
                if semantics_changed && scene_unchanged {
                    let paint_nodes_performed = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_nodes_performed"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let paint_cache_replayed_ops = s
                        .get("debug")
                        .and_then(|v| v.get("stats"))
                        .and_then(|v| v.get("paint_cache_replayed_ops"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    let diff_detail = prev_snapshot
                        .map(|prev| semantics_diff_detail(prev, s))
                        .unwrap_or(serde_json::Value::Null);

                    scan.findings.push(serde_json::json!({
                        "window": window_id,
                        "prev": {
                            "tick_id": prev_tick_id,
                            "frame_id": prev_frame_id,
                            "scene_fingerprint": prev_scene,
                            "semantics_fingerprint": prev_sem,
                        },
                        "now": {
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "scene_fingerprint": scene_fingerprint,
                            "semantics_fingerprint": semantics_fingerprint,
                        },
                        "paint_nodes_performed": paint_nodes_performed,
                        "paint_cache_replayed_ops": paint_cache_replayed_ops,
                        "semantics_diff": diff_detail,
                    }));

                    let mut detail = String::new();
                    if let Some(prev) = prev_snapshot {
                        let diff = semantics_diff_summary(prev, s);
                        if !diff.is_empty() {
                            detail.push(' ');
                            detail.push_str(&diff);
                        }
                    }

                    scan.suspicious_lines.push(format!(
                        "window={window_id} tick={tick_id} frame={frame_id} prev_tick={prev_tick_id} prev_frame={prev_frame_id} semantics_fingerprint=0x{semantics_fingerprint:016x} prev_semantics_fingerprint=0x{prev_sem:016x} scene_fingerprint=0x{scene_fingerprint:016x} paint_nodes_performed={paint_nodes_performed} paint_cache_replayed_ops={paint_cache_replayed_ops}{detail}"
                    ));
                    if scan.suspicious_lines.len() >= 8 {
                        break;
                    }
                }
            }

            prev_scene_fingerprint = Some(scene_fingerprint);
            prev_semantics_fingerprint = Some(semantics_fingerprint);
            prev_tick_id = tick_id;
            prev_frame_id = frame_id;
            prev_snapshot = Some(s);
        }
    }

    scan
}

fn semantics_diff_detail(
    before: &serde_json::Value,
    after: &serde_json::Value,
) -> serde_json::Value {
    use serde_json::json;
    use std::collections::{HashMap, HashSet};

    let before_nodes = before
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let after_nodes = after
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());

    let (Some(before_nodes), Some(after_nodes)) = (before_nodes, after_nodes) else {
        return serde_json::Value::Null;
    };

    let mut before_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in before_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        before_by_id.insert(id, node);
    }

    let mut after_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in after_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        after_by_id.insert(id, node);
    }

    let before_ids: HashSet<u64> = before_by_id.keys().copied().collect();
    let after_ids: HashSet<u64> = after_by_id.keys().copied().collect();

    let mut added: Vec<u64> = after_ids.difference(&before_ids).copied().collect();
    let mut removed: Vec<u64> = before_ids.difference(&after_ids).copied().collect();
    added.sort_unstable();
    removed.sort_unstable();

    let mut changed: Vec<(u64, u64)> = Vec::new(); // (score, id)
    for id in before_ids.intersection(&after_ids).copied() {
        let Some(a) = after_by_id.get(&id).copied() else {
            continue;
        };
        let Some(b) = before_by_id.get(&id).copied() else {
            continue;
        };
        let fp_a = semantics_node_fingerprint_json(a);
        let fp_b = semantics_node_fingerprint_json(b);
        if fp_a != fp_b {
            let score = semantics_node_score_json(a);
            changed.push((score, id));
        }
    }
    changed.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    let sample_len = 6usize;

    let added_nodes = added
        .iter()
        .take(sample_len)
        .map(|id| semantics_node_summary_json(*id, after_by_id.get(id).copied()))
        .collect::<Vec<_>>();
    let removed_nodes = removed
        .iter()
        .take(sample_len)
        .map(|id| semantics_node_summary_json(*id, before_by_id.get(id).copied()))
        .collect::<Vec<_>>();
    let changed_nodes = changed
        .iter()
        .take(sample_len)
        .map(|(_score, id)| {
            let before = semantics_node_summary_json(*id, before_by_id.get(id).copied());
            let after = semantics_node_summary_json(*id, after_by_id.get(id).copied());
            json!({ "id": id, "before": before, "after": after })
        })
        .collect::<Vec<_>>();

    json!({
        "counts": {
            "added": added.len(),
            "removed": removed.len(),
            "changed": changed.len(),
        },
        "samples": {
            "added_nodes": added_nodes,
            "removed_nodes": removed_nodes,
            "changed_nodes": changed_nodes,
        }
    })
}

fn semantics_node_summary_json(id: u64, node: Option<&serde_json::Value>) -> serde_json::Value {
    use serde_json::json;
    let Some(node) = node else {
        return json!({ "id": id });
    };

    let role = node.get("role").and_then(|v| v.as_str());
    let parent = node.get("parent").and_then(|v| v.as_u64());
    let test_id = node.get("test_id").and_then(|v| v.as_str());
    let label = node.get("label").and_then(|v| v.as_str());
    let value = node.get("value").and_then(|v| v.as_str());

    let bounds = node.get("bounds").and_then(|b| {
        Some(json!({
            "x": b.get("x").and_then(|v| v.as_f64()),
            "y": b.get("y").and_then(|v| v.as_f64()),
            "w": b.get("w").and_then(|v| v.as_f64()),
            "h": b.get("h").and_then(|v| v.as_f64()),
        }))
    });

    json!({
        "id": id,
        "parent": parent,
        "role": role,
        "test_id": test_id,
        "label": label,
        "value": value,
        "bounds": bounds,
    })
}

fn semantics_diff_summary(before: &serde_json::Value, after: &serde_json::Value) -> String {
    let before_nodes = before
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let after_nodes = after
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());

    let (Some(before_nodes), Some(after_nodes)) = (before_nodes, after_nodes) else {
        return String::new();
    };

    use std::collections::{HashMap, HashSet};

    let mut before_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in before_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        before_by_id.insert(id, node);
    }

    let mut after_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in after_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        after_by_id.insert(id, node);
    }

    let before_ids: HashSet<u64> = before_by_id.keys().copied().collect();
    let after_ids: HashSet<u64> = after_by_id.keys().copied().collect();

    let mut added: Vec<u64> = after_ids.difference(&before_ids).copied().collect();
    let mut removed: Vec<u64> = before_ids.difference(&after_ids).copied().collect();
    added.sort_unstable();
    removed.sort_unstable();

    let mut changed: Vec<(u64, u64, u64)> = Vec::new(); // (score, id, fp_after)
    for id in before_ids.intersection(&after_ids).copied() {
        let Some(a) = after_by_id.get(&id).copied() else {
            continue;
        };
        let Some(b) = before_by_id.get(&id).copied() else {
            continue;
        };
        let fp_a = semantics_node_fingerprint_json(a);
        let fp_b = semantics_node_fingerprint_json(b);
        if fp_a != fp_b {
            // Score heuristic: test_id changes are the most useful to report.
            let score = semantics_node_score_json(a);
            changed.push((score, id, fp_a));
        }
    }

    if added.is_empty() && removed.is_empty() && changed.is_empty() {
        return String::new();
    }

    changed.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    let mut out = String::new();
    out.push_str("semantics_diff={");
    out.push_str(&format!(
        "added={} removed={} changed={}",
        added.len(),
        removed.len(),
        changed.len()
    ));

    let sample_len = 6usize;
    if !changed.is_empty() {
        out.push_str(" changed_nodes=[");
        for (i, (_score, id, _fp)) in changed.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = after_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if changed.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    if !added.is_empty() {
        out.push_str(" added_nodes=[");
        for (i, id) in added.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = after_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if added.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    if !removed.is_empty() {
        out.push_str(" removed_nodes=[");
        for (i, id) in removed.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = before_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if removed.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    out.push('}');
    out
}

fn semantics_node_score_json(node: &serde_json::Value) -> u64 {
    // Higher is “more useful for debugging”.
    let mut score: u64 = 0;
    if node.get("test_id").and_then(|v| v.as_str()).is_some() {
        score += 10_000;
    }
    if node.get("label").and_then(|v| v.as_str()).is_some() {
        score += 1_000;
    }
    if node.get("value").and_then(|v| v.as_str()).is_some() {
        score += 500;
    }
    score
}

fn semantics_node_label_json(id: u64, node: Option<&serde_json::Value>) -> String {
    let Some(node) = node else {
        return format!("id={id}");
    };
    let role = node
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let test_id = node
        .get("test_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());
    let label = node
        .get("label")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());
    let value = node
        .get("value")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());

    let mut out = format!("id={id} role={role}");
    if let Some(v) = test_id {
        out.push_str(" test_id=");
        out.push_str(v);
    }
    if let Some(v) = label {
        out.push_str(" label=");
        out.push_str(v);
    }
    if let Some(v) = value {
        out.push_str(" value=");
        out.push_str(v);
    }
    out
}

fn semantics_node_fingerprint_json(node: &serde_json::Value) -> u64 {
    use std::hash::{Hash, Hasher};

    // Use a stable hash for a curated subset of fields.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    node.get("id").and_then(|v| v.as_u64()).hash(&mut hasher);
    node.get("parent")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("role").and_then(|v| v.as_str()).hash(&mut hasher);

    if let Some(bounds) = node.get("bounds") {
        if let Some(v) = bounds.get("x").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("y").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("w").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("h").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
    }

    if let Some(flags) = node.get("flags") {
        flags
            .get("focused")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("captured")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("disabled")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("selected")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("expanded")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("checked")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
    }

    node.get("test_id")
        .and_then(|v| v.as_str())
        .hash(&mut hasher);
    node.get("active_descendant")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("pos_in_set")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("set_size")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("label").and_then(|v| v.as_str()).hash(&mut hasher);
    node.get("value").and_then(|v| v.as_str()).hash(&mut hasher);

    if let Some(actions) = node.get("actions") {
        actions
            .get("focus")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("invoke")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("set_value")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("set_text_selection")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
    }

    hasher.finish()
}

pub(super) fn check_bundle_for_stale_scene_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut suspicious: Vec<String> = Vec::new();
    let mut missing_scene_fingerprint = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut prev_y: Option<f64> = None;
        let mut prev_label: Option<String> = None;
        let mut prev_value: Option<String> = None;
        let mut prev_fp: Option<u64> = None;

        for s in snaps {
            let (y, label, value) = semantics_node_fields_for_test_id(s, test_id);
            let fp = s.get("scene_fingerprint").and_then(|v| v.as_u64());
            if fp.is_none() {
                missing_scene_fingerprint = true;
            }

            let Some(fp) = fp else {
                prev_y = y;
                prev_label = label;
                prev_value = value;
                prev_fp = None;
                continue;
            };

            if let (Some(prev_fp), Some(prev_y)) = (prev_fp, prev_y) {
                let moved = y
                    .zip(Some(prev_y))
                    .is_some_and(|(y, prev_y)| (y - prev_y).abs() >= eps as f64);
                let label_changed = label.as_deref() != prev_label.as_deref();
                let value_changed = value.as_deref() != prev_value.as_deref();
                let changed = moved || label_changed || value_changed;

                if changed && fp == prev_fp {
                    let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    let label_len_prev = prev_label.as_deref().map(|s| s.len()).unwrap_or(0);
                    let label_len_now = label.as_deref().map(|s| s.len()).unwrap_or(0);
                    let value_len_prev = prev_value.as_deref().map(|s| s.len()).unwrap_or(0);
                    let value_len_now = value.as_deref().map(|s| s.len()).unwrap_or(0);
                    let delta_y = y
                        .zip(Some(prev_y))
                        .map(|(y, prev_y)| y - prev_y)
                        .unwrap_or(0.0);
                    suspicious.push(format!(
                        "window={window_id} tick={tick_id} frame={frame_id} test_id={test_id} changed={{moved={moved} label={label_changed} value={value_changed}}} delta_y={delta_y:.2} label_len={label_len_prev}->{label_len_now} value_len={value_len_prev}->{value_len_now} scene_fingerprint=0x{fp:016x}",
                    ));
                    if suspicious.len() >= 8 {
                        break;
                    }
                }
            }

            prev_y = y;
            prev_label = label;
            prev_value = value;
            prev_fp = Some(fp);
        }
    }

    if missing_scene_fingerprint {
        return Err(format!(
            "stale scene check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if suspicious.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "stale scene suspected (semantics changed but scene fingerprint did not change)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in suspicious {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

fn semantics_node_y_for_test_id(snapshot: &serde_json::Value, test_id: &str) -> Option<f64> {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())?;
    let node = nodes.iter().find(|n| {
        n.get("test_id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id == test_id)
    })?;
    node.get("bounds")
        .and_then(|v| v.get("y"))
        .and_then(|v| v.as_f64())
}

fn semantics_node_fields_for_test_id(
    snapshot: &serde_json::Value,
    test_id: &str,
) -> (Option<f64>, Option<String>, Option<String>) {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let Some(nodes) = nodes else {
        return (None, None, None);
    };
    let node = nodes.iter().find(|n| {
        n.get("test_id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id == test_id)
    });
    let Some(node) = node else {
        return (None, None, None);
    };
    let y = node
        .get("bounds")
        .and_then(|v| v.get("y"))
        .and_then(|v| v.as_f64());
    let label = node
        .get("label")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let value = node
        .get("value")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    (y, label, value)
}

fn first_wheel_frame_id_for_window(window: &serde_json::Value) -> Option<u64> {
    window
        .get("events")
        .and_then(|v| v.as_array())?
        .iter()
        .filter(|e| e.get("kind").and_then(|v| v.as_str()) == Some("pointer.wheel"))
        .filter_map(|e| e.get("frame_id").and_then(|v| v.as_u64()))
        .min()
}

fn semantics_node_id_for_test_id(snapshot: &serde_json::Value, test_id: &str) -> Option<u64> {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())?;
    nodes
        .iter()
        .find(|n| {
            n.get("test_id")
                .and_then(|v| v.as_str())
                .is_some_and(|id| id == test_id)
        })?
        .get("id")
        .and_then(|v| v.as_u64())
}

fn hit_test_node_id(snapshot: &serde_json::Value) -> Option<u64> {
    snapshot
        .get("debug")
        .and_then(|v| v.get("hit_test"))
        .and_then(|v| v.get("hit"))
        .and_then(|v| v.as_u64())
}

fn is_descendant(
    mut node: u64,
    ancestor: u64,
    parents: &std::collections::HashMap<u64, u64>,
) -> bool {
    if node == ancestor {
        return true;
    }
    while let Some(parent) = parents.get(&node).copied() {
        if parent == ancestor {
            return true;
        }
        node = parent;
    }
    false
}

fn semantics_parent_map(snapshot: &serde_json::Value) -> std::collections::HashMap<u64, u64> {
    let mut parents = std::collections::HashMap::new();
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let Some(nodes) = nodes else {
        return parents;
    };
    for node in nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(parent) = node.get("parent").and_then(|v| v.as_u64()) else {
            continue;
        };
        parents.insert(id, parent);
    }
    parents
}

pub(super) fn check_bundle_for_wheel_scroll(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_wheel_scroll_json(&bundle, bundle_path, test_id, warmup_frames)
}

pub(super) fn check_bundle_for_wheel_scroll_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_wheel = false;
    let mut failures: Vec<String> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut before: Option<&serde_json::Value> = None;
        let mut before_frame: u64 = 0;
        let mut after: Option<&serde_json::Value> = None;
        let mut after_frame_id: u64 = 0;
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                if frame_id >= before_frame && frame_id < after_frame {
                    before = Some(s);
                    before_frame = frame_id;
                }
                continue;
            }
            after = Some(s);
            after_frame_id = frame_id;
            break;
        }

        let (Some(before), Some(after)) = (before, after) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_before_or_after_snapshot"
            ));
            continue;
        };

        let Some(hit_before) = hit_test_node_id(before) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_hit_before"
            ));
            continue;
        };
        let Some(hit_after) = hit_test_node_id(after) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} error=missing_hit_after"
            ));
            continue;
        };

        let Some(target_before) = semantics_node_id_for_test_id(before, test_id) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(target_after) = semantics_node_id_for_test_id(after, test_id) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        let before_parents = semantics_parent_map(before);
        let after_parents = semantics_parent_map(after);

        if !is_descendant(hit_before, target_before, &before_parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=hit_not_within_target_before hit={hit_before} target={target_before}"
            ));
            continue;
        }

        if is_descendant(hit_after, target_after, &after_parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=hit_still_within_target_after hit={hit_after} target={target_after}"
            ));
        }
    }

    if !any_wheel {
        return Err(format!(
            "wheel scroll check requires at least one pointer.wheel event in the bundle: {}",
            bundle_path.display()
        ));
    }

    if failures.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("wheel scroll check failed (expected hit-test result to move after wheel)\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in failures {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_max(
    bundle_path: &Path,
    out_dir: &Path,
    max_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_visible_range_refreshes_max_json(
        &bundle,
        bundle_path,
        out_dir,
        max_total_refreshes,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_window_shifts_explainable(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_window_shifts_explainable_json(
        &bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_window_shifts_non_retained_max(
    bundle_path: &Path,
    out_dir: &Path,
    max_total_non_retained_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_window_shifts_non_retained_max_json(
        &bundle,
        bundle_path,
        out_dir,
        max_total_non_retained_shifts,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_visible_range_refreshes_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_total_refreshes,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_wheel = false;
    let mut total_refreshes: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let refreshes = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("virtual_list_visible_range_refreshes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if refreshes == 0 {
                continue;
            }
            total_refreshes = total_refreshes.saturating_add(refreshes);
            if samples.len() < 32 {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "refreshes": refreshes,
                }));
            }
        }
    }

    let out_path = out_dir.join("check.vlist_visible_range_refreshes_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_visible_range_refreshes_min",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_total_refreshes": min_total_refreshes,
        "any_wheel": any_wheel,
        "total_refreshes": total_refreshes,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if !any_wheel {
        return Err(format!(
            "vlist visible-range refresh gate requires wheel events, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if min_total_refreshes > 0 && total_refreshes < min_total_refreshes {
        return Err(format!(
            "expected virtual list visible-range refreshes to occur after wheel events, but total_refreshes={total_refreshes} was below min_total_refreshes={min_total_refreshes} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_vlist_window_shifts_explainable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_signal = false;
    let mut total_shifts: u64 = 0;
    let mut offenders: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let wheel_frame = first_wheel_frame_id_for_window(w);
        let after_frame = wheel_frame.unwrap_or(warmup_frames).max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let list = s
                .get("debug")
                .and_then(|v| v.get("virtual_list_windows"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if list.is_empty() {
                continue;
            }
            any_signal = true;

            for win in list {
                let mismatch = win
                    .get("window_mismatch")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let kind = win
                    .get("window_shift_kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or(if mismatch { "escape" } else { "none" });
                if kind == "none" {
                    continue;
                }
                total_shifts = total_shifts.saturating_add(1);

                let reason = win.get("window_shift_reason").and_then(|v| v.as_str());
                let mode = win.get("window_shift_apply_mode").and_then(|v| v.as_str());
                let invalidation_detail = win
                    .get("window_shift_invalidation_detail")
                    .and_then(|v| v.as_str());
                if reason.is_some() && mode.is_some() {
                    if mode == Some("non_retained_rerender") {
                        let expected_detail = match reason {
                            Some("scroll_to_item") => {
                                Some("scroll_handle_scroll_to_item_window_update")
                            }
                            Some("viewport_resize") => {
                                Some("scroll_handle_viewport_resize_window_update")
                            }
                            Some("items_revision") => {
                                Some("scroll_handle_items_revision_window_update")
                            }
                            _ => match kind {
                                "escape" => Some("scroll_handle_window_update"),
                                "prefetch" => Some("scroll_handle_prefetch_window_update"),
                                _ => None,
                            },
                        };
                        if invalidation_detail.is_none() {
                            offenders = offenders.saturating_add(1);
                            failures.push(format!(
                                "window={window_id} tick_id={tick_id} frame_id={frame_id} error=missing_shift_invalidation_detail kind={kind} apply_mode={mode:?}"
                            ));
                            if samples.len() < 64 {
                                samples.push(serde_json::json!({
                                    "window": window_id,
                                    "tick_id": tick_id,
                                    "frame_id": frame_id,
                                    "kind": kind,
                                    "reason": reason,
                                    "apply_mode": mode,
                                    "invalidation_detail": invalidation_detail,
                                    "expected_invalidation_detail": expected_detail,
                                    "node": win.get("node").and_then(|v| v.as_u64()),
                                    "element": win.get("element").and_then(|v| v.as_u64()),
                                    "policy_key": win.get("policy_key").and_then(|v| v.as_u64()),
                                    "inputs_key": win.get("inputs_key").and_then(|v| v.as_u64()),
                                }));
                            }
                        } else if expected_detail.is_some()
                            && invalidation_detail != expected_detail
                        {
                            offenders = offenders.saturating_add(1);
                            failures.push(format!(
                                "window={window_id} tick_id={tick_id} frame_id={frame_id} error=unexpected_shift_invalidation_detail kind={kind} got={invalidation_detail:?} expected={expected_detail:?}"
                            ));
                            if samples.len() < 64 {
                                samples.push(serde_json::json!({
                                    "window": window_id,
                                    "tick_id": tick_id,
                                    "frame_id": frame_id,
                                    "kind": kind,
                                    "reason": reason,
                                    "apply_mode": mode,
                                    "invalidation_detail": invalidation_detail,
                                    "expected_invalidation_detail": expected_detail,
                                    "node": win.get("node").and_then(|v| v.as_u64()),
                                    "element": win.get("element").and_then(|v| v.as_u64()),
                                    "policy_key": win.get("policy_key").and_then(|v| v.as_u64()),
                                    "inputs_key": win.get("inputs_key").and_then(|v| v.as_u64()),
                                }));
                            }
                        }
                    }
                    continue;
                }

                offenders = offenders.saturating_add(1);
                failures.push(format!(
                    "window={window_id} tick_id={tick_id} frame_id={frame_id} error=missing_shift_explainability kind={kind} reason={reason:?} apply_mode={mode:?} invalidation_detail={invalidation_detail:?}"
                ));

                if samples.len() < 64 {
                    samples.push(serde_json::json!({
                        "window": window_id,
                        "tick_id": tick_id,
                        "frame_id": frame_id,
                        "kind": kind,
                        "reason": reason,
                        "apply_mode": mode,
                        "invalidation_detail": invalidation_detail,
                        "node": win.get("node").and_then(|v| v.as_u64()),
                        "element": win.get("element").and_then(|v| v.as_u64()),
                        "policy_key": win.get("policy_key").and_then(|v| v.as_u64()),
                        "inputs_key": win.get("inputs_key").and_then(|v| v.as_u64()),
                        "window_range": win.get("window_range"),
                        "prev_window_range": win.get("prev_window_range"),
                        "render_window_range": win.get("render_window_range"),
                        "deferred_scroll_to_item": win.get("deferred_scroll_to_item").and_then(|v| v.as_bool()),
                        "deferred_scroll_consumed": win.get("deferred_scroll_consumed").and_then(|v| v.as_bool()),
                    }));
                }
            }
        }
    }

    let out_path = out_dir.join("check.vlist_window_shifts_explainable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_window_shifts_explainable",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "total_shifts": total_shifts,
        "offenders": offenders,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if !any_signal {
        return Err(format!(
            "vlist window-shift explainability gate requires debug.virtual_list_windows after warmup, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if offenders == 0 {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("vlist window-shift explainability gate failed (expected every window shift to have reason + apply_mode)\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!("evidence: {}\n", out_path.display()));
    for line in failures.into_iter().take(12) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn check_bundle_for_prepaint_actions_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_snapshots: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_prepaint_actions_min_json(
        &bundle,
        bundle_path,
        out_dir,
        min_snapshots,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_prepaint_actions_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    min_snapshots: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let after_frame = warmup_frames.saturating_add(1);
    let mut snapshots_with_actions: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let actions = s
                .get("debug")
                .and_then(|v| v.get("prepaint_actions"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if actions.is_empty() {
                continue;
            }

            snapshots_with_actions = snapshots_with_actions.saturating_add(1);
            if samples.len() < 32 {
                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "actions_len": actions.len(),
                }));
            }
        }
    }

    let out_path = out_dir.join("check.prepaint_actions_min.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "prepaint_actions_min",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_snapshots": min_snapshots,
        "snapshots_with_actions": snapshots_with_actions,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if min_snapshots > 0 && snapshots_with_actions < min_snapshots {
        return Err(format!(
            "expected prepaint actions to be recorded in at least min_snapshots={min_snapshots}, but snapshots_with_actions={snapshots_with_actions} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_vlist_window_shifts_non_retained_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    max_total_non_retained_shifts: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut snapshots_examined: u64 = 0;
    let mut total_non_retained_shifts: u64 = 0;
    let mut total_shifts: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            snapshots_examined = snapshots_examined.saturating_add(1);

            let debug_stats = s.get("debug").and_then(|v| v.get("stats"));
            let window_shifts_total = debug_stats
                .and_then(|v| v.get("virtual_list_window_shifts_total"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let window_shifts_non_retained = debug_stats
                .and_then(|v| v.get("virtual_list_window_shifts_non_retained"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            total_shifts = total_shifts.saturating_add(window_shifts_total);
            if window_shifts_non_retained == 0 {
                continue;
            }
            total_non_retained_shifts =
                total_non_retained_shifts.saturating_add(window_shifts_non_retained);

            if samples.len() < 64 {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                let shift_samples = s
                    .get("debug")
                    .and_then(|v| v.get("virtual_list_window_shift_samples"))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().take(8).cloned().collect::<Vec<_>>())
                    .unwrap_or_default();

                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "non_retained_shifts": window_shifts_non_retained,
                    "window_shifts_total": window_shifts_total,
                    "shift_samples": shift_samples,
                }));
            }
        }
    }

    let out_path = out_dir.join("check.vlist_window_shifts_non_retained_max.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_window_shifts_non_retained_max",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_total_non_retained_shifts": max_total_non_retained_shifts,
        "snapshots_examined": snapshots_examined,
        "total_window_shifts": total_shifts,
        "total_non_retained_shifts": total_non_retained_shifts,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if total_non_retained_shifts > max_total_non_retained_shifts {
        return Err(format!(
            "vlist non-retained window-shift gate failed: total_non_retained_shifts={total_non_retained_shifts} exceeded max_total_non_retained_shifts={max_total_non_retained_shifts} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_vlist_window_shifts_have_prepaint_actions(
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_vlist_window_shifts_have_prepaint_actions_json(
        &bundle,
        bundle_path,
        out_dir,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_vlist_window_shifts_have_prepaint_actions_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let after_frame = warmup_frames.saturating_add(1);
    let mut offenders: u64 = 0;
    let mut failures: Vec<String> = Vec::new();
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);

            let debug = s.get("debug").unwrap_or(&serde_json::Value::Null);
            let vlist = debug
                .get("virtual_list_windows")
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if vlist.is_empty() {
                continue;
            }
            let actions = debug
                .get("prepaint_actions")
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);

            let shift_actions: Vec<&serde_json::Value> = actions
                .iter()
                .filter(|a| {
                    a.get("kind").and_then(|v| v.as_str()) == Some("virtual_list_window_shift")
                })
                .collect();

            for win in vlist {
                let source = win.get("source").and_then(|v| v.as_str());
                if source != Some("prepaint") {
                    continue;
                }
                let shift_kind = win
                    .get("window_shift_kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("none");
                if shift_kind == "none" {
                    continue;
                }

                let node = win.get("node").and_then(|v| v.as_u64());
                let element = win.get("element").and_then(|v| v.as_u64());
                let shift_reason = win.get("window_shift_reason").and_then(|v| v.as_str());

                let found = shift_actions.iter().any(|a| {
                    let a_node = a.get("node").and_then(|v| v.as_u64());
                    let a_element = a.get("element").and_then(|v| v.as_u64());
                    let a_kind = a
                        .get("virtual_list_window_shift_kind")
                        .and_then(|v| v.as_str());
                    let a_reason = a
                        .get("virtual_list_window_shift_reason")
                        .and_then(|v| v.as_str());

                    a_node == node
                        && a_element == element
                        && a_kind == Some(shift_kind)
                        && (shift_reason.is_none() || a_reason == shift_reason)
                });

                if !found {
                    offenders = offenders.saturating_add(1);
                    failures.push(format!(
                        "window={window_id} tick_id={tick_id} frame_id={frame_id} error=missing_vlist_window_shift_prepaint_action node={node:?} element={element:?} shift_kind={shift_kind} shift_reason={shift_reason:?}"
                    ));
                    if samples.len() < 64 {
                        samples.push(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "node": node,
                            "element": element,
                            "shift_kind": shift_kind,
                            "shift_reason": shift_reason,
                            "available_shift_actions": shift_actions.iter().take(8).map(|a| serde_json::json!({
                                "node": a.get("node").and_then(|v| v.as_u64()),
                                "element": a.get("element").and_then(|v| v.as_u64()),
                                "shift_kind": a.get("virtual_list_window_shift_kind").and_then(|v| v.as_str()),
                                "shift_reason": a.get("virtual_list_window_shift_reason").and_then(|v| v.as_str()),
                            })).collect::<Vec<_>>(),
                        }));
                    }
                }
            }
        }
    }

    let out_path = out_dir.join("check.vlist_window_shifts_have_prepaint_actions.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_window_shifts_have_prepaint_actions",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "offenders": offenders,
        "failures": failures,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if offenders > 0 {
        return Err(format!(
            "vlist window-shift prepaint-action gate failed: offenders={offenders} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_vlist_visible_range_refreshes_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    out_dir: &Path,
    max_total_refreshes: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut any_wheel = false;
    let mut total_refreshes: u64 = 0;
    let mut samples: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                continue;
            }
            let refreshes = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("virtual_list_visible_range_refreshes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if refreshes == 0 {
                continue;
            }
            total_refreshes = total_refreshes.saturating_add(refreshes);
            if samples.len() < 32 {
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                samples.push(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "refreshes": refreshes,
                }));
            }
        }
    }

    let out_path = out_dir.join("check.vlist_visible_range_refreshes_max.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "vlist_visible_range_refreshes_max",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "max_total_refreshes": max_total_refreshes,
        "any_wheel": any_wheel,
        "total_refreshes": total_refreshes,
        "samples": samples,
    });
    write_json_value(&out_path, &payload)?;

    if !any_wheel {
        return Err(format!(
            "vlist visible-range refresh gate requires wheel events, but none were observed (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    if max_total_refreshes > 0 && total_refreshes > max_total_refreshes {
        return Err(format!(
            "expected virtual list visible-range refreshes to stay under budget after wheel events, but total_refreshes={total_refreshes} exceeded max_total_refreshes={max_total_refreshes} (warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_drag_cache_root_paint_only(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut good_frames: u64 = 0;
    let mut bad_frames: Vec<String> = Vec::new();
    let mut missing_target_count: u64 = 0;
    let mut any_view_cache_active = false;
    let mut seen_good = false;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let view_cache_active = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if !view_cache_active {
                continue;
            }

            let Some(target_node_id) = semantics_node_id_for_test_id(s, test_id) else {
                missing_target_count = missing_target_count.saturating_add(1);
                continue;
            };

            let nodes = s
                .get("debug")
                .and_then(|v| v.get("semantics"))
                .and_then(|v| v.get("nodes"))
                .and_then(|v| v.as_array())
                .ok_or_else(|| "invalid bundle.json: missing debug.semantics.nodes".to_string())?;
            let mut parents: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
            for n in nodes {
                let Some(id) = n.get("id").and_then(|v| v.as_u64()) else {
                    continue;
                };
                if let Some(parent) = n.get("parent").and_then(|v| v.as_u64()) {
                    parents.insert(id, parent);
                }
            }

            let roots = s
                .get("debug")
                .and_then(|v| v.get("cache_roots"))
                .and_then(|v| v.as_array())
                .ok_or_else(|| "invalid bundle.json: missing debug.cache_roots".to_string())?;
            let mut cache_roots: std::collections::HashMap<u64, &serde_json::Value> =
                std::collections::HashMap::new();
            for r in roots {
                if let Some(root) = r.get("root").and_then(|v| v.as_u64()) {
                    cache_roots.insert(root, r);
                }
            }

            let mut current = target_node_id;
            let mut cache_root_node: Option<u64> = None;
            loop {
                if cache_roots.contains_key(&current) {
                    cache_root_node = Some(current);
                    break;
                }
                let Some(parent) = parents.get(&current).copied() else {
                    break;
                };
                current = parent;
            }
            let Some(cache_root_node) = cache_root_node else {
                return Err(format!(
                    "could not resolve a cache root ancestor for test_id={test_id} (node_id={target_node_id}) in bundle: {}",
                    bundle_path.display()
                ));
            };

            let root = cache_roots
                .get(&cache_root_node)
                .ok_or_else(|| "internal error: cache root missing".to_string())?;

            let reused = root
                .get("reused")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let contained_relayout_in_frame = root
                .get("contained_relayout_in_frame")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let dirty = s
                .get("debug")
                .and_then(|v| v.get("dirty_views"))
                .and_then(|v| v.as_array())
                .map_or(false, |dirty| {
                    dirty.iter().any(|d| {
                        d.get("root_node")
                            .and_then(|v| v.as_u64())
                            .is_some_and(|n| n == cache_root_node)
                    })
                });

            let ok = reused && !contained_relayout_in_frame && !dirty;
            if ok {
                good_frames = good_frames.saturating_add(1);
                seen_good = true;
                continue;
            }

            if seen_good {
                bad_frames.push(format!(
                    "window={window_id} frame_id={frame_id} cache_root={cache_root_node} reused={reused} contained_relayout_in_frame={contained_relayout_in_frame} dirty={dirty}"
                ));
            }
        }
    }

    if !bad_frames.is_empty() {
        let mut msg = String::new();
        msg.push_str("expected paint-only drag indicator updates (cache-root reuse, no contained relayout, no dirty view), but found violations after reuse began\n");
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!("test_id: {test_id}\n"));
        for line in bad_frames.into_iter().take(10) {
            msg.push_str("  ");
            msg.push_str(&line);
            msg.push('\n');
        }
        return Err(msg);
    }

    if good_frames == 0 {
        return Err(format!(
            "did not observe any cache-root-reuse paint-only frames for test_id={test_id} \
(any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}, missing_target_count={missing_target_count}) \
in bundle: {}",
            bundle_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_gc_sweep_liveness(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut offenders: Vec<String> = Vec::new();
    let mut offender_samples: Vec<serde_json::Value> = Vec::new();
    let mut examined_snapshots: u64 = 0;
    let mut removed_subtrees_total: u64 = 0;
    let mut removed_subtrees_offenders: u64 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let Some(removed) = s
                .get("debug")
                .and_then(|v| v.get("removed_subtrees"))
                .and_then(|v| v.as_array())
            else {
                continue;
            };

            for r in removed {
                removed_subtrees_total = removed_subtrees_total.saturating_add(1);
                let unreachable = r
                    .get("unreachable_from_liveness_roots")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let reachable_from_layer_roots = r
                    .get("reachable_from_layer_roots")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let reachable_from_view_cache_roots = r
                    .get("reachable_from_view_cache_roots")
                    .and_then(|v| v.as_bool());
                let root_layer_visible = r.get("root_layer_visible").and_then(|v| v.as_bool());

                if !unreachable
                    || reachable_from_layer_roots
                    || reachable_from_view_cache_roots == Some(true)
                    || root_layer_visible == Some(true)
                {
                    removed_subtrees_offenders = removed_subtrees_offenders.saturating_add(1);
                    let root = r.get("root").and_then(|v| v.as_u64()).unwrap_or(0);
                    let root_element_path = r
                        .get("root_element_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>");
                    let trigger_path = r
                        .get("trigger_element_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>");
                    let mut violations: Vec<&'static str> = Vec::new();
                    if !unreachable {
                        violations.push("reachable_from_liveness_roots");
                    }
                    if reachable_from_layer_roots {
                        violations.push("reachable_from_layer_roots");
                    }
                    if reachable_from_view_cache_roots == Some(true) {
                        violations.push("reachable_from_view_cache_roots");
                    }
                    if root_layer_visible == Some(true) {
                        violations.push("root_layer_visible");
                    }
                    offenders.push(format!(
                        "window={window_id} frame_id={frame_id} root={root} unreachable_from_liveness_roots={unreachable} reachable_from_layer_roots={reachable_from_layer_roots} reachable_from_view_cache_roots={reachable_from_view_cache_roots:?} root_layer_visible={root_layer_visible:?} root_element_path={root_element_path} trigger_element_path={trigger_path}"
                    ));

                    const MAX_SAMPLES: usize = 128;
                    if offender_samples.len() < MAX_SAMPLES {
                        offender_samples.push(serde_json::json!({
                            "window": window_id,
                            "frame_id": frame_id,
                            "tick_id": s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                            "root": r.get("root").and_then(|v| v.as_u64()).unwrap_or(0),
                            "root_root": r.get("root_root").and_then(|v| v.as_u64()),
                            "root_layer": r.get("root_layer").and_then(|v| v.as_u64()),
                            "root_layer_visible": root_layer_visible,
                            "reachable_from_layer_roots": reachable_from_layer_roots,
                            "reachable_from_view_cache_roots": reachable_from_view_cache_roots,
                            "unreachable_from_liveness_roots": unreachable,
                            "violations": violations,
                            "liveness_layer_roots_len": r.get("liveness_layer_roots_len").and_then(|v| v.as_u64()),
                            "view_cache_reuse_roots_len": r.get("view_cache_reuse_roots_len").and_then(|v| v.as_u64()),
                            "view_cache_reuse_root_nodes_len": r.get("view_cache_reuse_root_nodes_len").and_then(|v| v.as_u64()),
                            "root_element": r.get("root_element").and_then(|v| v.as_u64()),
                            "root_element_path": r.get("root_element_path").and_then(|v| v.as_str()),
                            "trigger_element": r.get("trigger_element").and_then(|v| v.as_u64()),
                            "trigger_element_path": r.get("trigger_element_path").and_then(|v| v.as_str()),
                            "trigger_element_in_view_cache_keep_alive": r.get("trigger_element_in_view_cache_keep_alive").and_then(|v| v.as_bool()),
                            "trigger_element_listed_under_reuse_root": r.get("trigger_element_listed_under_reuse_root").and_then(|v| v.as_u64()),
                            "root_root_parent_sever_parent": r.get("root_root_parent_sever_parent").and_then(|v| v.as_u64()),
                            "root_root_parent_sever_location": r.get("root_root_parent_sever_location").and_then(|v| v.as_str()),
                            "root_root_parent_sever_frame_id": r.get("root_root_parent_sever_frame_id").and_then(|v| v.as_u64()),
                        }));
                    }
                }
            }
        }
    }

    // Always write evidence so debugging doesn't require re-running the harness.
    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.gc_sweep_liveness.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "gc_sweep_liveness",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "removed_subtrees_total": removed_subtrees_total,
        "removed_subtrees_offenders": removed_subtrees_offenders,
        "offender_samples": offender_samples,
    });
    write_json_value(&evidence_path, &payload)?;

    if offenders.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("GC sweep liveness violation: removed_subtrees contains entries that appear live (reachable/visible)\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
    ));
    msg.push_str(&format!("evidence: {}\n", evidence_path.display()));
    for line in offenders.into_iter().take(10) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn check_bundle_for_view_cache_reuse_min(
    bundle_path: &Path,
    min_reuse_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_view_cache_reuse_min_json(
        &bundle,
        bundle_path,
        min_reuse_events,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_view_cache_reuse_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_reuse_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reuse_events: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut any_view_cache_active = false;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let view_cache_active = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if !view_cache_active {
                continue;
            }

            let roots = s
                .get("debug")
                .and_then(|v| v.get("cache_roots"))
                .and_then(|v| v.as_array());
            let Some(roots) = roots else {
                continue;
            };

            for r in roots {
                if r.get("reused").and_then(|v| v.as_bool()) == Some(true) {
                    reuse_events = reuse_events.saturating_add(1);
                    if reuse_events >= min_reuse_events {
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(format!(
        "expected at least {min_reuse_events} view-cache reuse events, got {reuse_events} \
 (any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) \
 in bundle: {}",
        bundle_path.display()
    ))
}

#[derive(Debug, Clone)]
struct ViewCacheReuseStableWindowReport {
    window: u64,
    examined_snapshots: u64,
    view_cache_active_snapshots: u64,
    non_reuse_cache_inactive_snapshots: u64,
    non_reuse_active_no_signal_snapshots: u64,
    reuse_snapshots: u64,
    reuse_streak_max: u64,
    reuse_streak_tail: u64,
    last_non_reuse: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy)]
struct ViewCacheReuseSignal {
    view_cache_active: bool,
    has_reuse_signal: bool,
    reused_roots: u64,
    paint_cache_replayed_ops: u64,
    cache_roots_present: bool,
}

impl ViewCacheReuseSignal {
    fn no_signal_reason(self) -> &'static str {
        if !self.view_cache_active {
            return "view_cache_inactive";
        }
        "active_no_signal"
    }
}

fn snapshot_view_cache_reuse_signal(snapshot: &serde_json::Value) -> ViewCacheReuseSignal {
    let stats = snapshot.get("debug").and_then(|v| v.get("stats"));
    let view_cache_active = stats
        .and_then(|v| v.get("view_cache_active"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let replayed_ops = stats
        .and_then(|v| v.get("paint_cache_replayed_ops"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let mut reused_roots: u64 = 0;
    let mut cache_roots_present = false;
    if let Some(roots) = snapshot
        .get("debug")
        .and_then(|v| v.get("cache_roots"))
        .and_then(|v| v.as_array())
    {
        cache_roots_present = true;
        for r in roots {
            if r.get("reused").and_then(|v| v.as_bool()) == Some(true) {
                reused_roots = reused_roots.saturating_add(1);
            }
        }
    }

    let has_signal = view_cache_active && (reused_roots > 0 || replayed_ops > 0);
    ViewCacheReuseSignal {
        view_cache_active,
        has_reuse_signal: has_signal,
        reused_roots,
        paint_cache_replayed_ops: replayed_ops,
        cache_roots_present,
    }
}

pub(super) fn check_bundle_for_view_cache_reuse_stable_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_tail_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reports: Vec<ViewCacheReuseStableWindowReport> = Vec::new();
    let mut failures: Vec<serde_json::Value> = Vec::new();

    let mut any_view_cache_active = false;
    let mut best_tail: u64 = 0;

    for w in windows {
        let window = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut examined_snapshots: u64 = 0;
        let mut view_cache_active_snapshots: u64 = 0;
        let mut non_reuse_cache_inactive_snapshots: u64 = 0;
        let mut non_reuse_active_no_signal_snapshots: u64 = 0;
        let mut reuse_snapshots: u64 = 0;
        let mut reuse_streak: u64 = 0;
        let mut reuse_streak_max: u64 = 0;
        let mut last_non_reuse: Option<serde_json::Value> = None;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let stats = s.get("debug").and_then(|v| v.get("stats"));
            let view_cache_active = stats
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if view_cache_active {
                view_cache_active_snapshots = view_cache_active_snapshots.saturating_add(1);
            }

            let signal = snapshot_view_cache_reuse_signal(s);
            if signal.has_reuse_signal {
                reuse_snapshots = reuse_snapshots.saturating_add(1);
                reuse_streak = reuse_streak.saturating_add(1);
                reuse_streak_max = reuse_streak_max.max(reuse_streak);
            } else {
                reuse_streak = 0;
                match signal.no_signal_reason() {
                    "view_cache_inactive" => {
                        non_reuse_cache_inactive_snapshots =
                            non_reuse_cache_inactive_snapshots.saturating_add(1);
                    }
                    _ => {
                        non_reuse_active_no_signal_snapshots =
                            non_reuse_active_no_signal_snapshots.saturating_add(1);
                    }
                }
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                last_non_reuse = Some(serde_json::json!({
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "reason": signal.no_signal_reason(),
                    "view_cache_active": signal.view_cache_active,
                    "cache_roots_present": signal.cache_roots_present,
                    "reused_roots": signal.reused_roots,
                    "paint_cache_replayed_ops": signal.paint_cache_replayed_ops,
                }));
            }
        }

        best_tail = best_tail.max(reuse_streak);

        reports.push(ViewCacheReuseStableWindowReport {
            window,
            examined_snapshots,
            view_cache_active_snapshots,
            non_reuse_cache_inactive_snapshots,
            non_reuse_active_no_signal_snapshots,
            reuse_snapshots,
            reuse_streak_max,
            reuse_streak_tail: reuse_streak,
            last_non_reuse: last_non_reuse.clone(),
        });

        if min_tail_frames > 0 && examined_snapshots < min_tail_frames {
            failures.push(serde_json::json!({
                "window": window,
                "reason": "insufficient_snapshots",
                "examined_snapshots": examined_snapshots,
            }));
        } else if min_tail_frames > 0 && reuse_streak < min_tail_frames {
            failures.push(serde_json::json!({
                "window": window,
                "reason": "reuse_tail_streak_too_small",
                "examined_snapshots": examined_snapshots,
                "view_cache_active_snapshots": view_cache_active_snapshots,
                "non_reuse_cache_inactive_snapshots": non_reuse_cache_inactive_snapshots,
                "non_reuse_active_no_signal_snapshots": non_reuse_active_no_signal_snapshots,
                "reuse_streak_tail": reuse_streak,
                "reuse_streak_max": reuse_streak_max,
                "reuse_snapshots": reuse_snapshots,
                "last_non_reuse": last_non_reuse,
            }));
        }
    }

    let out_path = out_dir.join("check.view_cache_reuse_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "view_cache_reuse_stable",
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_tail_frames": min_tail_frames,
        "any_view_cache_active": any_view_cache_active,
        "best_reuse_streak_tail": best_tail,
        "windows": reports.iter().map(|r| serde_json::json!({
            "window": r.window,
            "examined_snapshots": r.examined_snapshots,
            "view_cache_active_snapshots": r.view_cache_active_snapshots,
            "non_reuse_cache_inactive_snapshots": r.non_reuse_cache_inactive_snapshots,
            "non_reuse_active_no_signal_snapshots": r.non_reuse_active_no_signal_snapshots,
            "reuse_snapshots": r.reuse_snapshots,
            "reuse_streak_max": r.reuse_streak_max,
            "reuse_streak_tail": r.reuse_streak_tail,
            "last_non_reuse": r.last_non_reuse,
        })).collect::<Vec<_>>(),
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    if min_tail_frames == 0 {
        return Ok(());
    }
    if !any_view_cache_active {
        return Err(format!(
            "view-cache reuse stable gate requires view_cache_active snapshots, but none were observed (warmup_frames={warmup_frames})\n  hint: enable view-cache for the target demo if applicable (e.g. UI gallery: FRET_UI_GALLERY_VIEW_CACHE=1)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }
    if best_tail >= min_tail_frames {
        return Ok(());
    }

    Err(format!(
        "view-cache reuse stable gate failed (min_tail_frames={min_tail_frames}, best_tail={best_tail}, warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}

pub(super) fn check_bundle_for_overlay_synthesis_min(
    bundle_path: &Path,
    min_synthesized_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_overlay_synthesis_min_json(
        &bundle,
        bundle_path,
        min_synthesized_events,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_overlay_synthesis_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_synthesized_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut synthesized_events: u64 = 0;
    let mut suppression_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    let mut examined_snapshots: u64 = 0;
    let mut any_view_cache_active = false;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let view_cache_active = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;

            let Some(events) = s
                .get("debug")
                .and_then(|v| v.get("overlay_synthesis"))
                .and_then(|v| v.as_array())
            else {
                continue;
            };

            for e in events {
                let kind = e.get("kind").and_then(|v| v.as_str()).unwrap_or("unknown");
                let outcome = e
                    .get("outcome")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                if outcome == "synthesized" {
                    synthesized_events = synthesized_events.saturating_add(1);
                    if synthesized_events >= min_synthesized_events {
                        return Ok(());
                    }
                } else {
                    let key = format!("{kind}/{outcome}");
                    *suppression_counts.entry(key).or_insert(0) += 1;
                }
            }
        }
    }

    let mut suppressions: Vec<(String, u64)> = suppression_counts.into_iter().collect();
    suppressions.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    suppressions.truncate(12);
    let suppressions = if suppressions.is_empty() {
        String::new()
    } else {
        let mut msg = String::new();
        msg.push_str(" suppressions=[");
        for (idx, (k, c)) in suppressions.into_iter().enumerate() {
            if idx > 0 {
                msg.push_str(", ");
            }
            msg.push_str(&format!("{k}:{c}"));
        }
        msg.push(']');
        msg
    };

    Err(format!(
        "expected at least {min_synthesized_events} overlay synthesis events, got {synthesized_events} \
(any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}).{suppressions} \
bundle: {}",
        bundle_path.display()
    ))
}

pub(super) fn check_bundle_for_retained_vlist_reconcile_no_notify_min(
    bundle_path: &Path,
    min_reconcile_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
        &bundle,
        bundle_path,
        min_reconcile_events,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_reconcile_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reconcile_events: u64 = 0;
    let mut reconcile_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut notify_offenders: Vec<String> = Vec::new();

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let list_count = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let stats_count = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let count = list_count.max(stats_count);
            if count == 0 {
                continue;
            }

            reconcile_frames = reconcile_frames.saturating_add(1);
            reconcile_events = reconcile_events.saturating_add(count);

            let dirty_views = s
                .get("debug")
                .and_then(|v| v.get("dirty_views"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);

            for dv in dirty_views {
                let source = dv
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let detail = dv
                    .get("detail")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if source == "notify" || detail.contains("notify") {
                    let root_node = dv.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0);
                    notify_offenders.push(format!(
                        "frame_id={frame_id} dirty_view_root_node={root_node} source={source} detail={detail}"
                    ));
                    break;
                }
            }
        }
    }

    if !notify_offenders.is_empty() {
        let mut msg = String::new();
        msg.push_str(
            "retained virtual-list reconcile should not require notify-based dirty views\n",
        );
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!(
            "min_reconcile_events={min_reconcile_events} reconcile_events={reconcile_events} reconcile_frames={reconcile_frames} warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
        ));
        for line in notify_offenders.into_iter().take(10) {
            msg.push_str("  ");
            msg.push_str(&line);
            msg.push('\n');
        }
        return Err(msg);
    }

    if reconcile_events < min_reconcile_events {
        return Err(format!(
            "expected at least {min_reconcile_events} retained virtual-list reconcile events, got {reconcile_events} \
(reconcile_frames={reconcile_frames}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) \
bundle: {}",
            bundle_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_retained_vlist_attach_detach_max(
    bundle_path: &Path,
    max_delta: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_vlist_attach_detach_max_json(
        &bundle,
        bundle_path,
        max_delta,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_retained_vlist_attach_detach_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    max_delta: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reconcile_events: u64 = 0;
    let mut reconcile_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut offenders: Vec<String> = Vec::new();

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let list_count = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let stats_count = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let count = list_count.max(stats_count);
            if count == 0 {
                continue;
            }

            reconcile_frames = reconcile_frames.saturating_add(1);
            reconcile_events = reconcile_events.saturating_add(count);

            let records = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            let (attached, detached) = if records.is_empty() {
                let stats = s
                    .get("debug")
                    .and_then(|v| v.get("stats"))
                    .and_then(|v| v.as_object());
                let attached = stats
                    .and_then(|v| v.get("retained_virtual_list_attached_items"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let detached = stats
                    .and_then(|v| v.get("retained_virtual_list_detached_items"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                (attached, detached)
            } else {
                let attached = records
                    .iter()
                    .map(|r| {
                        r.get("attached_items")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                    })
                    .sum::<u64>();
                let detached = records
                    .iter()
                    .map(|r| {
                        r.get("detached_items")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                    })
                    .sum::<u64>();
                (attached, detached)
            };

            let delta = attached.saturating_add(detached);
            if delta > max_delta {
                offenders.push(format!(
                    "frame_id={frame_id} attached={attached} detached={detached} delta={delta} max={max_delta}"
                ));
            }
        }
    }

    if reconcile_events == 0 {
        return Err(format!(
            "expected at least 1 retained virtual-list reconcile event (required for attach/detach max check), got 0 \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
            bundle_path.display()
        ));
    }

    if offenders.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("retained virtual-list attach/detach delta exceeded the configured maximum\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "max_delta={max_delta} reconcile_events={reconcile_events} reconcile_frames={reconcile_frames} warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
    ));
    for line in offenders.into_iter().take(10) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(super) fn check_bundle_for_viewport_input_min(
    bundle_path: &Path,
    min_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_viewport_input_min_json(&bundle, bundle_path, min_events, warmup_frames)
}

pub(super) fn check_bundle_for_viewport_input_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut events: u64 = 0;
    let mut examined_snapshots: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let Some(arr) = s
                .get("debug")
                .and_then(|v| v.get("viewport_input"))
                .and_then(|v| v.as_array())
            else {
                continue;
            };

            events = events.saturating_add(arr.len() as u64);
            if events >= min_events {
                return Ok(());
            }
        }
    }

    Err(format!(
        "expected at least {min_events} viewport input events, got {events} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
        bundle_path.display()
    ))
}

pub(super) fn check_bundle_for_dock_drag_min(
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_dock_drag_min_json(&bundle, bundle_path, min_active_frames, warmup_frames)
}

pub(super) fn check_bundle_for_dock_drag_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut active_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let Some(dock_drag) = s
                .get("debug")
                .and_then(|v| v.get("docking_interaction"))
                .and_then(|v| v.get("dock_drag"))
            else {
                continue;
            };
            if dock_drag.is_object() {
                active_frames = active_frames.saturating_add(1);
                if active_frames >= min_active_frames {
                    return Ok(());
                }
            }
        }
    }

    Err(format!(
        "expected at least {min_active_frames} snapshots with an active dock drag, got {active_frames} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
        bundle_path.display()
    ))
}

pub(super) fn check_bundle_for_viewport_capture_min(
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_viewport_capture_min_json(
        &bundle,
        bundle_path,
        min_active_frames,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_viewport_capture_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut active_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let Some(viewport_capture) = s
                .get("debug")
                .and_then(|v| v.get("docking_interaction"))
                .and_then(|v| v.get("viewport_capture"))
            else {
                continue;
            };
            if viewport_capture.is_object() {
                active_frames = active_frames.saturating_add(1);
                if active_frames >= min_active_frames {
                    return Ok(());
                }
            }
        }
    }

    Err(format!(
        "expected at least {min_active_frames} snapshots with an active viewport capture, got {active_frames} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
        bundle_path.display()
    ))
}

pub(super) fn bundle_stats_from_json_with_options(
    bundle: &serde_json::Value,
    top: usize,
    sort: BundleStatsSort,
    opts: BundleStatsOptions,
) -> Result<BundleStatsReport, String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;

    let mut out = BundleStatsReport::default();
    out.sort = sort;
    out.warmup_frames = opts.warmup_frames;
    out.windows = windows.len().min(u32::MAX as usize) as u32;

    let mut rows: Vec<BundleStatsSnapshotRow> = Vec::new();
    let mut global_type_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    let mut model_source_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            out.snapshots = out.snapshots.saturating_add(1);
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < opts.warmup_frames {
                out.snapshots_skipped_warmup = out.snapshots_skipped_warmup.saturating_add(1);
                continue;
            }
            out.snapshots_considered = out.snapshots_considered.saturating_add(1);

            let changed_models = s
                .get("changed_models")
                .and_then(|v| v.as_array())
                .map(|v| v.len())
                .unwrap_or(0)
                .min(u32::MAX as usize) as u32;
            let changed_globals_arr = s
                .get("changed_globals")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let changed_globals = changed_globals_arr.len().min(u32::MAX as usize) as u32;
            let mut changed_global_types_sample: Vec<String> = Vec::new();
            for (idx, g) in changed_globals_arr.iter().enumerate() {
                let Some(ty) = g.as_str() else {
                    continue;
                };
                *global_type_counts.entry(ty.to_string()).or_insert(0) += 1;
                if idx < 6 {
                    changed_global_types_sample.push(ty.to_string());
                }
            }

            if let Some(arr) = s
                .get("changed_model_sources_top")
                .and_then(|v| v.as_array())
            {
                for item in arr {
                    let Some(type_name) = item.get("type_name").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(at) = item.get("changed_at").and_then(|v| v.as_object()) else {
                        continue;
                    };
                    let Some(file) = at.get("file").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let Some(line) = at.get("line").and_then(|v| v.as_u64()) else {
                        continue;
                    };
                    let Some(column) = at.get("column").and_then(|v| v.as_u64()) else {
                        continue;
                    };
                    let count = item.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                    let key = format!("{}@{}:{}:{}", type_name, file, line, column);
                    *model_source_counts.entry(key).or_insert(0) += count;
                }
            }

            if changed_models > 0 {
                out.snapshots_with_model_changes =
                    out.snapshots_with_model_changes.saturating_add(1);
            }
            if changed_globals > 0 {
                out.snapshots_with_global_changes =
                    out.snapshots_with_global_changes.saturating_add(1);
            }

            let stats = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.as_object());

            let layout_time_us = stats
                .and_then(|m| m.get("layout_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let prepaint_time_us = stats
                .and_then(|m| m.get("prepaint_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let paint_time_us = stats
                .and_then(|m| m.get("paint_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let total_time_us = layout_time_us
                .saturating_add(prepaint_time_us)
                .saturating_add(paint_time_us);
            let layout_nodes_performed = stats
                .and_then(|m| m.get("layout_nodes_performed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let paint_nodes_performed = stats
                .and_then(|m| m.get("paint_nodes_performed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let paint_cache_misses = stats
                .and_then(|m| m.get("paint_cache_misses"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let layout_engine_solves = stats
                .and_then(|m| m.get("layout_engine_solves"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let layout_engine_solve_time_us = stats
                .and_then(|m| m.get("layout_engine_solve_time_us"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let view_cache_contained_relayouts = stats
                .and_then(|m| m.get("view_cache_contained_relayouts"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let set_children_barrier_writes = stats
                .and_then(|m| m.get("set_children_barrier_writes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let barrier_relayouts_scheduled = stats
                .and_then(|m| m.get("barrier_relayouts_scheduled"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let barrier_relayouts_performed = stats
                .and_then(|m| m.get("barrier_relayouts_performed"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let virtual_list_visible_range_checks = stats
                .and_then(|m| m.get("virtual_list_visible_range_checks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let virtual_list_visible_range_refreshes = stats
                .and_then(|m| m.get("virtual_list_visible_range_refreshes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;

            let propagated_model_change_models = stats
                .and_then(|m| m.get("model_change_models"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let propagated_model_change_observation_edges = stats
                .and_then(|m| m.get("model_change_observation_edges"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let propagated_model_change_unobserved_models = stats
                .and_then(|m| m.get("model_change_unobserved_models"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let propagated_global_change_globals = stats
                .and_then(|m| m.get("global_change_globals"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let propagated_global_change_observation_edges = stats
                .and_then(|m| m.get("global_change_observation_edges"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let propagated_global_change_unobserved_globals = stats
                .and_then(|m| m.get("global_change_unobserved_globals"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;

            if propagated_model_change_models > 0 {
                out.snapshots_with_propagated_model_changes = out
                    .snapshots_with_propagated_model_changes
                    .saturating_add(1);
            }
            if propagated_global_change_globals > 0 {
                out.snapshots_with_propagated_global_changes = out
                    .snapshots_with_propagated_global_changes
                    .saturating_add(1);
            }

            let invalidation_walk_calls = stats
                .and_then(|m| m.get("invalidation_walk_calls"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes = stats
                .and_then(|m| m.get("invalidation_walk_nodes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let model_change_invalidation_roots = stats
                .and_then(|m| m.get("model_change_invalidation_roots"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let global_change_invalidation_roots = stats
                .and_then(|m| m.get("global_change_invalidation_roots"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_model_change = stats
                .and_then(|m| m.get("invalidation_walk_calls_model_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_model_change = stats
                .and_then(|m| m.get("invalidation_walk_nodes_model_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_global_change = stats
                .and_then(|m| m.get("invalidation_walk_calls_global_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let invalidation_walk_nodes_global_change = stats
                .and_then(|m| m.get("invalidation_walk_nodes_global_change"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let invalidation_walk_calls_hover = stats
                .and_then(|m| m.get("invalidation_walk_calls_hover"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_hover = stats
                .and_then(|m| m.get("invalidation_walk_nodes_hover"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_focus = stats
                .and_then(|m| m.get("invalidation_walk_calls_focus"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_focus = stats
                .and_then(|m| m.get("invalidation_walk_nodes_focus"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_calls_other = stats
                .and_then(|m| m.get("invalidation_walk_calls_other"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let invalidation_walk_nodes_other = stats
                .and_then(|m| m.get("invalidation_walk_nodes_other"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;

            let top_invalidation_walks = snapshot_top_invalidation_walks(s, 3);
            let hover_pressable_target_changes = stats
                .and_then(|m| m.get("hover_pressable_target_changes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hover_hover_region_target_changes = stats
                .and_then(|m| m.get("hover_hover_region_target_changes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hover_declarative_instance_changes = stats
                .and_then(|m| m.get("hover_declarative_instance_changes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let hover_declarative_hit_test_invalidations = stats
                .and_then(|m| m.get("hover_declarative_hit_test_invalidations"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let hover_declarative_layout_invalidations = stats
                .and_then(|m| m.get("hover_declarative_layout_invalidations"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let hover_declarative_paint_invalidations = stats
                .and_then(|m| m.get("hover_declarative_paint_invalidations"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64)
                as u32;
            let top_hover_declarative_invalidations =
                snapshot_top_hover_declarative_invalidations(s, 3);
            let (
                cache_roots,
                cache_roots_reused,
                cache_roots_contained_relayout,
                cache_replayed_ops,
                top_cache_roots,
                top_contained_relayout_cache_roots,
            ) = snapshot_cache_root_stats(s, 3);
            let top_layout_engine_solves = snapshot_layout_engine_solves(s, 3);
            let model_change_hotspots = snapshot_model_change_hotspots(s, 3);
            let model_change_unobserved = snapshot_model_change_unobserved(s, 3);
            let global_change_hotspots = snapshot_global_change_hotspots(s, 3);
            let global_change_unobserved = snapshot_global_change_unobserved(s, 3);

            out.sum_layout_time_us = out.sum_layout_time_us.saturating_add(layout_time_us);
            out.sum_prepaint_time_us = out.sum_prepaint_time_us.saturating_add(prepaint_time_us);
            out.sum_paint_time_us = out.sum_paint_time_us.saturating_add(paint_time_us);
            out.sum_total_time_us = out.sum_total_time_us.saturating_add(total_time_us);
            out.sum_cache_roots = out.sum_cache_roots.saturating_add(cache_roots as u64);
            out.sum_cache_roots_reused = out
                .sum_cache_roots_reused
                .saturating_add(cache_roots_reused as u64);
            out.sum_cache_replayed_ops = out
                .sum_cache_replayed_ops
                .saturating_add(cache_replayed_ops);
            out.sum_invalidation_walk_calls = out
                .sum_invalidation_walk_calls
                .saturating_add(invalidation_walk_calls as u64);
            out.sum_invalidation_walk_nodes = out
                .sum_invalidation_walk_nodes
                .saturating_add(invalidation_walk_nodes as u64);
            out.sum_model_change_invalidation_roots = out
                .sum_model_change_invalidation_roots
                .saturating_add(model_change_invalidation_roots as u64);
            out.sum_global_change_invalidation_roots = out
                .sum_global_change_invalidation_roots
                .saturating_add(global_change_invalidation_roots as u64);
            if hover_declarative_layout_invalidations > 0 {
                out.snapshots_with_hover_layout_invalidations = out
                    .snapshots_with_hover_layout_invalidations
                    .saturating_add(1);
            }
            out.sum_hover_layout_invalidations = out
                .sum_hover_layout_invalidations
                .saturating_add(hover_declarative_layout_invalidations as u64);

            out.max_invalidation_walk_calls =
                out.max_invalidation_walk_calls.max(invalidation_walk_calls);
            out.max_invalidation_walk_nodes =
                out.max_invalidation_walk_nodes.max(invalidation_walk_nodes);
            out.max_model_change_invalidation_roots = out
                .max_model_change_invalidation_roots
                .max(model_change_invalidation_roots);
            out.max_global_change_invalidation_roots = out
                .max_global_change_invalidation_roots
                .max(global_change_invalidation_roots);
            if hover_declarative_layout_invalidations > out.max_hover_layout_invalidations {
                out.worst_hover_layout = Some(BundleStatsWorstHoverLayout {
                    window: window_id,
                    tick_id: s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                    frame_id: s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
                    hover_declarative_layout_invalidations,
                    hotspots: snapshot_top_hover_declarative_invalidations(s, 8),
                });
            }
            out.max_hover_layout_invalidations = out
                .max_hover_layout_invalidations
                .max(hover_declarative_layout_invalidations);
            out.max_layout_time_us = out.max_layout_time_us.max(layout_time_us);
            out.max_prepaint_time_us = out.max_prepaint_time_us.max(prepaint_time_us);
            out.max_paint_time_us = out.max_paint_time_us.max(paint_time_us);
            out.max_total_time_us = out.max_total_time_us.max(total_time_us);

            rows.push(BundleStatsSnapshotRow {
                window: window_id,
                tick_id: s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                frame_id: s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
                timestamp_unix_ms: s.get("timestamp_unix_ms").and_then(|v| v.as_u64()),
                layout_time_us,
                prepaint_time_us,
                paint_time_us,
                total_time_us,
                layout_nodes_performed,
                paint_nodes_performed,
                paint_cache_misses,
                layout_engine_solves,
                layout_engine_solve_time_us,
                changed_models,
                changed_globals,
                changed_global_types_sample,
                propagated_model_change_models,
                propagated_model_change_observation_edges,
                propagated_model_change_unobserved_models,
                propagated_global_change_globals,
                propagated_global_change_observation_edges,
                propagated_global_change_unobserved_globals,
                invalidation_walk_calls,
                invalidation_walk_nodes,
                model_change_invalidation_roots,
                global_change_invalidation_roots,
                invalidation_walk_calls_model_change,
                invalidation_walk_nodes_model_change,
                invalidation_walk_calls_global_change,
                invalidation_walk_nodes_global_change,
                invalidation_walk_calls_hover,
                invalidation_walk_nodes_hover,
                invalidation_walk_calls_focus,
                invalidation_walk_nodes_focus,
                invalidation_walk_calls_other,
                invalidation_walk_nodes_other,
                top_invalidation_walks,
                hover_pressable_target_changes,
                hover_hover_region_target_changes,
                hover_declarative_instance_changes,
                hover_declarative_hit_test_invalidations,
                hover_declarative_layout_invalidations,
                hover_declarative_paint_invalidations,
                top_hover_declarative_invalidations,
                cache_roots,
                cache_roots_reused,
                cache_roots_contained_relayout,
                cache_replayed_ops,
                view_cache_contained_relayouts,
                set_children_barrier_writes,
                barrier_relayouts_scheduled,
                barrier_relayouts_performed,
                virtual_list_visible_range_checks,
                virtual_list_visible_range_refreshes,
                top_cache_roots,
                top_contained_relayout_cache_roots,
                top_layout_engine_solves,
                model_change_hotspots,
                model_change_unobserved,
                global_change_hotspots,
                global_change_unobserved,
            });
        }
    }

    match sort {
        BundleStatsSort::Invalidation => {
            rows.sort_by(|a, b| {
                b.invalidation_walk_nodes
                    .cmp(&a.invalidation_walk_nodes)
                    .then_with(|| b.invalidation_walk_calls.cmp(&a.invalidation_walk_calls))
                    .then_with(|| {
                        b.model_change_invalidation_roots
                            .cmp(&a.model_change_invalidation_roots)
                    })
                    .then_with(|| {
                        b.global_change_invalidation_roots
                            .cmp(&a.global_change_invalidation_roots)
                    })
                    .then_with(|| b.total_time_us.cmp(&a.total_time_us))
            });
        }
        BundleStatsSort::Time => {
            rows.sort_by(|a, b| {
                b.total_time_us
                    .cmp(&a.total_time_us)
                    .then_with(|| b.layout_time_us.cmp(&a.layout_time_us))
                    .then_with(|| b.paint_time_us.cmp(&a.paint_time_us))
                    .then_with(|| b.invalidation_walk_nodes.cmp(&a.invalidation_walk_nodes))
            });
        }
    }
    let mut hotspots: Vec<BundleStatsGlobalTypeHotspot> = global_type_counts
        .into_iter()
        .map(|(type_name, count)| BundleStatsGlobalTypeHotspot { type_name, count })
        .collect();
    hotspots.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.type_name.cmp(&b.type_name))
    });
    hotspots.truncate(top);
    out.global_type_hotspots = hotspots;

    let mut model_hotspots: Vec<BundleStatsModelSourceHotspot> = model_source_counts
        .into_iter()
        .map(|(source, count)| BundleStatsModelSourceHotspot { source, count })
        .collect();
    model_hotspots.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.source.cmp(&b.source)));
    model_hotspots.truncate(top);
    out.model_source_hotspots = model_hotspots;

    out.top = rows.into_iter().take(top).collect();
    Ok(out)
}

fn snapshot_top_invalidation_walks(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsInvalidationWalk> {
    let walks = snapshot
        .get("debug")
        .and_then(|v| v.get("invalidation_walks"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    if walks.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<BundleStatsInvalidationWalk> = walks
        .iter()
        .map(|w| BundleStatsInvalidationWalk {
            root_node: w.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0),
            root_element: w.get("root_element").and_then(|v| v.as_u64()),
            kind: w
                .get("kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            source: w
                .get("source")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            detail: w
                .get("detail")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            walked_nodes: w
                .get("walked_nodes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            truncated_at: w.get("truncated_at").and_then(|v| v.as_u64()),
            root_role: None,
            root_test_id: None,
        })
        .collect();

    out.sort_by(|a, b| b.walked_nodes.cmp(&a.walked_nodes));
    out.truncate(max);

    for walk in &mut out {
        let (role, test_id) = snapshot_lookup_semantics(snapshot, walk.root_node);
        walk.root_role = role;
        walk.root_test_id = test_id;
    }

    out
}

fn snapshot_cache_root_stats(
    snapshot: &serde_json::Value,
    max: usize,
) -> (
    u32,
    u32,
    u32,
    u64,
    Vec<BundleStatsCacheRoot>,
    Vec<BundleStatsCacheRoot>,
) {
    let roots = snapshot
        .get("debug")
        .and_then(|v| v.get("cache_roots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if roots.is_empty() {
        return (0, 0, 0, 0, Vec::new(), Vec::new());
    }

    let mut reused: u32 = 0;
    let mut contained_relayout: u32 = 0;
    let mut replayed_ops_sum: u64 = 0;

    let semantics_index = SemanticsIndex::from_snapshot(snapshot);

    let mut out: Vec<BundleStatsCacheRoot> = roots
        .iter()
        .map(|r| {
            let root_node = r.get("root").and_then(|v| v.as_u64()).unwrap_or(0);
            let paint_replayed_ops = r
                .get("paint_replayed_ops")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32;
            let reused_flag = r.get("reused").and_then(|v| v.as_bool()).unwrap_or(false);
            if reused_flag {
                reused = reused.saturating_add(1);
            }
            let contained_relayout_in_frame = r
                .get("contained_relayout_in_frame")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if contained_relayout_in_frame {
                contained_relayout = contained_relayout.saturating_add(1);
            }
            replayed_ops_sum = replayed_ops_sum.saturating_add(paint_replayed_ops as u64);

            let (role, test_id) = semantics_index.lookup_for_cache_root(root_node);
            BundleStatsCacheRoot {
                root_node,
                element: r.get("element").and_then(|v| v.as_u64()),
                element_path: r
                    .get("element_path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                reused: reused_flag,
                contained_layout: r
                    .get("contained_layout")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                contained_relayout_in_frame,
                paint_replayed_ops,
                reuse_reason: r
                    .get("reuse_reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                root_in_semantics: r.get("root_in_semantics").and_then(|v| v.as_bool()),
                root_role: role,
                root_test_id: test_id,
            }
        })
        .collect();

    out.sort_by(|a, b| b.paint_replayed_ops.cmp(&a.paint_replayed_ops));
    let top_cache_roots: Vec<BundleStatsCacheRoot> = out.iter().take(max).cloned().collect();
    let top_contained_relayout_cache_roots: Vec<BundleStatsCacheRoot> = out
        .iter()
        .filter(|r| r.contained_relayout_in_frame)
        .take(max)
        .cloned()
        .collect();

    (
        roots.len().min(u32::MAX as usize) as u32,
        reused,
        contained_relayout,
        replayed_ops_sum,
        top_cache_roots,
        top_contained_relayout_cache_roots,
    )
}

fn snapshot_top_hover_declarative_invalidations(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsHoverDeclarativeInvalidationHotspot> {
    let items = snapshot
        .get("debug")
        .and_then(|v| v.get("hover_declarative_invalidation_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    if items.is_empty() || max == 0 {
        return Vec::new();
    }

    let mut out: Vec<BundleStatsHoverDeclarativeInvalidationHotspot> = items
        .iter()
        .map(|h| BundleStatsHoverDeclarativeInvalidationHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            hit_test: h
                .get("hit_test")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            layout: h
                .get("layout")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            paint: h
                .get("paint")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            role: None,
            test_id: None,
        })
        .collect();

    out.sort_by(|a, b| {
        b.layout
            .cmp(&a.layout)
            .then_with(|| b.hit_test.cmp(&a.hit_test))
            .then_with(|| b.paint.cmp(&a.paint))
    });
    out.truncate(max);

    for item in &mut out {
        let (role, test_id) = snapshot_lookup_semantics(snapshot, item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

pub(super) fn check_report_for_hover_layout_invalidations(
    report: &BundleStatsReport,
    max_allowed: u32,
) -> Result<(), String> {
    if report.max_hover_layout_invalidations <= max_allowed {
        return Ok(());
    }

    let mut extra = String::new();
    if let Some(worst) = report.worst_hover_layout.as_ref() {
        extra.push_str(&format!(
            " worst(window={} tick={} frame={} hover_layout={})",
            worst.window,
            worst.tick_id,
            worst.frame_id,
            worst.hover_declarative_layout_invalidations
        ));
        if !worst.hotspots.is_empty() {
            let items: Vec<String> = worst
                .hotspots
                .iter()
                .take(3)
                .map(|h| {
                    let mut s = format!(
                        "layout={} hit={} paint={} node={}",
                        h.layout, h.hit_test, h.paint, h.node
                    );
                    if let Some(test_id) = h.test_id.as_deref()
                        && !test_id.is_empty()
                    {
                        s.push_str(&format!(" test_id={test_id}"));
                    }
                    if let Some(role) = h.role.as_deref()
                        && !role.is_empty()
                    {
                        s.push_str(&format!(" role={role}"));
                    }
                    s
                })
                .collect();
            extra.push_str(&format!(" hotspots=[{}]", items.join(" | ")));
        }
    }

    Err(format!(
        "hover-attributed declarative layout invalidations detected (max_per_frame={} allowed={max_allowed}).{}",
        report.max_hover_layout_invalidations, extra
    ))
}

fn snapshot_layout_engine_solves(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsLayoutEngineSolve> {
    let solves = snapshot
        .get("debug")
        .and_then(|v| v.get("layout_engine_solves"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if solves.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(snapshot);

    let mut out: Vec<BundleStatsLayoutEngineSolve> = solves
        .iter()
        .map(|s| {
            let top_measures = s
                .get("top_measures")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let mut top_measures: Vec<BundleStatsLayoutEngineMeasureHotspot> = top_measures
                .iter()
                .take(3)
                .map(|m| {
                    let children = m
                        .get("top_children")
                        .and_then(|v| v.as_array())
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);
                    let mut top_children: Vec<BundleStatsLayoutEngineMeasureChildHotspot> =
                        children
                            .iter()
                            .take(3)
                            .map(|c| BundleStatsLayoutEngineMeasureChildHotspot {
                                child: c.get("child").and_then(|v| v.as_u64()).unwrap_or(0),
                                measure_time_us: c
                                    .get("measure_time_us")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                                calls: c.get("calls").and_then(|v| v.as_u64()).unwrap_or(0),
                                element: c.get("element").and_then(|v| v.as_u64()),
                                element_kind: c
                                    .get("element_kind")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                role: None,
                                test_id: None,
                            })
                            .collect();

                    for item in &mut top_children {
                        let (role, test_id) =
                            semantics_index.lookup_for_node_or_ancestor_test_id(item.child);
                        item.role = role;
                        item.test_id = test_id;
                    }

                    BundleStatsLayoutEngineMeasureHotspot {
                        node: m.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
                        measure_time_us: m
                            .get("measure_time_us")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                        calls: m.get("calls").and_then(|v| v.as_u64()).unwrap_or(0),
                        cache_hits: m.get("cache_hits").and_then(|v| v.as_u64()).unwrap_or(0),
                        element: m.get("element").and_then(|v| v.as_u64()),
                        element_kind: m
                            .get("element_kind")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        top_children,
                        role: None,
                        test_id: None,
                    }
                })
                .collect();

            for item in &mut top_measures {
                let (role, test_id) =
                    semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
                item.role = role;
                item.test_id = test_id;
            }

            BundleStatsLayoutEngineSolve {
                root_node: s.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0),
                solve_time_us: s.get("solve_time_us").and_then(|v| v.as_u64()).unwrap_or(0),
                measure_calls: s.get("measure_calls").and_then(|v| v.as_u64()).unwrap_or(0),
                measure_cache_hits: s
                    .get("measure_cache_hits")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                measure_time_us: s
                    .get("measure_time_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                top_measures,
                root_role: None,
                root_test_id: None,
            }
        })
        .collect();

    out.sort_by(|a, b| b.solve_time_us.cmp(&a.solve_time_us));
    out.truncate(max);

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.root_node);
        item.root_role = role;
        item.root_test_id = test_id;
    }

    out
}

fn snapshot_model_change_hotspots(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsModelChangeHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("model_change_hotspots"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    hotspots
        .iter()
        .take(max)
        .map(|h| BundleStatsModelChangeHotspot {
            model: h.get("model").and_then(|v| v.as_u64()).unwrap_or(0),
            observation_edges: h
                .get("observation_edges")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            changed_at: h
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_model_change_unobserved(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsModelChangeUnobserved> {
    let unobserved = snapshot
        .get("debug")
        .and_then(|v| v.get("model_change_unobserved"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    unobserved
        .iter()
        .take(max)
        .map(|u| BundleStatsModelChangeUnobserved {
            model: u.get("model").and_then(|v| v.as_u64()).unwrap_or(0),
            created_type: u
                .get("created_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at: u
                .get("created_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
            changed_at: u
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_global_change_hotspots(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsGlobalChangeHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("global_change_hotspots"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    hotspots
        .iter()
        .take(max)
        .map(|h| BundleStatsGlobalChangeHotspot {
            type_name: h
                .get("type_name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            observation_edges: h
                .get("observation_edges")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            changed_at: h
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_global_change_unobserved(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsGlobalChangeUnobserved> {
    let unobserved = snapshot
        .get("debug")
        .and_then(|v| v.get("global_change_unobserved"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    unobserved
        .iter()
        .take(max)
        .map(|u| BundleStatsGlobalChangeUnobserved {
            type_name: u
                .get("type_name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            changed_at: u
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

fn snapshot_lookup_semantics(
    snapshot: &serde_json::Value,
    node_id: u64,
) -> (Option<String>, Option<String>) {
    let nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    for n in nodes {
        if n.get("id").and_then(|v| v.as_u64()) == Some(node_id) {
            let role = n
                .get("role")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let test_id = n
                .get("test_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            return (role, test_id);
        }
    }
    (None, None)
}

#[derive(Debug, Clone)]
struct SemanticsNodeLite {
    id: u64,
    parent: Option<u64>,
    role: Option<String>,
    test_id: Option<String>,
}

#[derive(Debug, Default)]
struct SemanticsIndex {
    by_id: std::collections::HashMap<u64, SemanticsNodeLite>,
    best_descendant_with_test_id: std::collections::HashMap<u64, (Option<String>, Option<String>)>,
}

impl SemanticsIndex {
    fn from_snapshot(snapshot: &serde_json::Value) -> Self {
        let nodes = snapshot
            .get("debug")
            .and_then(|v| v.get("semantics"))
            .and_then(|v| v.get("nodes"))
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let mut by_id: std::collections::HashMap<u64, SemanticsNodeLite> =
            std::collections::HashMap::new();
        by_id.reserve(nodes.len());

        for n in nodes {
            let Some(id) = n.get("id").and_then(|v| v.as_u64()) else {
                continue;
            };

            let parent = n.get("parent").and_then(|v| v.as_u64());
            let role = n
                .get("role")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let test_id = n
                .get("test_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            by_id.insert(
                id,
                SemanticsNodeLite {
                    id,
                    parent,
                    role,
                    test_id,
                },
            );
        }

        let mut best_descendant_with_test_id: std::collections::HashMap<
            u64,
            (Option<String>, Option<String>),
        > = std::collections::HashMap::new();

        for node in by_id.values() {
            let Some(test_id) = node.test_id.as_deref() else {
                continue;
            };
            if test_id.is_empty() {
                continue;
            }

            let mut cursor: Option<u64> = Some(node.id);
            let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
            while let Some(id) = cursor {
                if !seen.insert(id) {
                    break;
                }

                best_descendant_with_test_id
                    .entry(id)
                    .or_insert_with(|| (node.role.clone(), node.test_id.clone()));

                cursor = by_id.get(&id).and_then(|n| n.parent);
            }
        }

        Self {
            by_id,
            best_descendant_with_test_id,
        }
    }

    fn lookup_for_cache_root(&self, root_node: u64) -> (Option<String>, Option<String>) {
        if let Some(node) = self.by_id.get(&root_node) {
            return (node.role.clone(), node.test_id.clone());
        }

        if let Some((role, test_id)) = self.best_descendant_with_test_id.get(&root_node) {
            return (role.clone(), test_id.clone());
        }

        (None, None)
    }

    fn lookup_for_node_or_ancestor_test_id(
        &self,
        node_id: u64,
    ) -> (Option<String>, Option<String>) {
        const MAX_PARENT_HOPS: usize = 16;

        let mut role: Option<String> = None;
        let mut current: Option<u64> = Some(node_id);
        for _ in 0..MAX_PARENT_HOPS {
            let Some(id) = current else {
                break;
            };
            let Some(node) = self.by_id.get(&id) else {
                break;
            };
            if role.is_none() {
                role = node.role.clone();
            }
            if node.test_id.as_ref().is_some_and(|s| !s.is_empty()) {
                return (role, node.test_id.clone());
            }
            current = node.parent;
        }

        (role, None)
    }
}

#[derive(Debug, Clone)]
pub(super) struct ScriptResultSummary {
    pub(super) run_id: u64,
    pub(super) stage: Option<String>,
    pub(super) step_index: Option<u64>,
    pub(super) reason: Option<String>,
    pub(super) last_bundle_dir: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct PickResultSummary {
    pub(super) run_id: u64,
    pub(super) stage: Option<String>,
    pub(super) reason: Option<String>,
    pub(super) last_bundle_dir: Option<String>,
    pub(super) selector: Option<serde_json::Value>,
}

pub(super) fn run_script_and_wait(
    src: &Path,
    script_path: &Path,
    script_trigger_path: &Path,
    script_result_path: &Path,
    script_result_trigger_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<ScriptResultSummary, String> {
    let prev_run_id = read_script_result_run_id(script_result_path).unwrap_or(0);
    let mut target_run_id: Option<u64> = None;

    write_script(src, script_path)?;
    touch(script_trigger_path)?;

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if Instant::now() >= deadline {
            return Err(format!(
                "timeout waiting for script result (result: {}, trigger: {})",
                script_result_path.display(),
                script_result_trigger_path.display()
            ));
        }

        if let Some(result) = read_script_result(script_result_path) {
            let run_id = result.get("run_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if target_run_id.is_none() && run_id > prev_run_id {
                target_run_id = Some(run_id);
            }

            if Some(run_id) == target_run_id {
                let stage = result
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if matches!(stage.as_deref(), Some("passed") | Some("failed")) {
                    let step_index = result.get("step_index").and_then(|v| v.as_u64());
                    let reason = result
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let last_bundle_dir = result
                        .get("last_bundle_dir")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    return Ok(ScriptResultSummary {
                        run_id,
                        stage,
                        step_index,
                        reason,
                        last_bundle_dir,
                    });
                }
            }
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

pub(super) fn clear_script_result_files(
    script_result_path: &Path,
    script_result_trigger_path: &Path,
) {
    let _ = std::fs::remove_file(script_result_path);
    let _ = std::fs::remove_file(script_result_trigger_path);
}

pub(super) fn report_result_and_exit(result: &ScriptResultSummary) -> ! {
    match result.stage.as_deref() {
        Some("passed") => {
            println!("PASS (run_id={})", result.run_id);
            std::process::exit(0);
        }
        Some("failed") => {
            let reason = result.reason.as_deref().unwrap_or("unknown");
            let last_bundle_dir = result.last_bundle_dir.as_deref().unwrap_or("");
            if last_bundle_dir.is_empty() {
                if let Some(step) = result.step_index {
                    eprintln!(
                        "FAIL (run_id={}) step={} reason={reason}",
                        result.run_id, step
                    );
                } else {
                    eprintln!("FAIL (run_id={}) reason={reason}", result.run_id);
                }
            } else {
                if let Some(step) = result.step_index {
                    eprintln!(
                        "FAIL (run_id={}) step={} reason={reason} last_bundle_dir={last_bundle_dir}",
                        result.run_id, step
                    );
                } else {
                    eprintln!(
                        "FAIL (run_id={}) reason={reason} last_bundle_dir={last_bundle_dir}",
                        result.run_id
                    );
                }
            }
            std::process::exit(1);
        }
        _ => {
            eprintln!("unexpected script stage: {:?}", result);
            std::process::exit(1);
        }
    }
}

fn expected_failure_dump_suffixes(result: &ScriptResultSummary) -> Vec<String> {
    let Some(step_index) = result.step_index else {
        return Vec::new();
    };
    let Some(reason) = result.reason.as_deref() else {
        return Vec::new();
    };

    match reason {
        "wait_until_timeout" => vec![format!("script-step-{step_index:04}-wait_until-timeout")],
        "assert_failed" => vec![format!("script-step-{step_index:04}-assert-failed")],
        "no_semantics_snapshot" => vec![
            format!("script-step-{step_index:04}-wait_until-no-semantics"),
            format!("script-step-{step_index:04}-assert-no-semantics"),
        ],
        _ => Vec::new(),
    }
}

pub(super) fn wait_for_failure_dump_bundle(
    out_dir: &Path,
    result: &ScriptResultSummary,
    timeout_ms: u64,
    poll_ms: u64,
) -> Option<PathBuf> {
    let suffixes = expected_failure_dump_suffixes(result);
    if suffixes.is_empty() {
        return None;
    }

    let deadline = Instant::now() + Duration::from_millis(timeout_ms.min(5_000).max(250));
    while Instant::now() < deadline {
        for suffix in &suffixes {
            if let Some(dir) = find_latest_export_dir_with_suffix(out_dir, suffix)
                && dir.join("bundle.json").is_file()
            {
                return Some(dir);
            }
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
    None
}

fn find_latest_export_dir_with_suffix(out_dir: &Path, suffix: &str) -> Option<PathBuf> {
    let mut best: Option<(u64, PathBuf)> = None;
    let entries = std::fs::read_dir(out_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(suffix) {
            continue;
        }
        let Some((ts_str, _)) = name.split_once('-') else {
            continue;
        };
        let Ok(ts) = ts_str.parse::<u64>() else {
            continue;
        };
        match &best {
            Some((prev, _)) if *prev >= ts => {}
            _ => best = Some((ts, path)),
        }
    }
    best.map(|(_, p)| p)
}

pub(super) fn run_pick_and_wait(
    pick_trigger_path: &Path,
    pick_result_path: &Path,
    pick_result_trigger_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<PickResultSummary, String> {
    let prev_run_id = read_pick_result_run_id(pick_result_path).unwrap_or(0);
    let mut target_run_id: Option<u64> = None;

    touch(pick_trigger_path)?;

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if Instant::now() >= deadline {
            return Err(format!(
                "timeout waiting for pick result (result: {}, trigger: {})",
                pick_result_path.display(),
                pick_result_trigger_path.display()
            ));
        }

        if let Some(result) = read_pick_result(pick_result_path) {
            let run_id = result.get("run_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if target_run_id.is_none() && run_id > prev_run_id {
                target_run_id = Some(run_id);
            }

            if Some(run_id) == target_run_id {
                let stage = result
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if matches!(stage.as_deref(), Some("picked")) {
                    let reason = result
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let last_bundle_dir = result
                        .get("last_bundle_dir")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let selector = result
                        .get("selection")
                        .and_then(|v| v.get("selectors"))
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .cloned();

                    return Ok(PickResultSummary {
                        run_id,
                        stage,
                        reason,
                        last_bundle_dir,
                        selector,
                    });
                }
            }
        }

        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

pub(super) fn report_pick_result_and_exit(result: &PickResultSummary) -> ! {
    match result.stage.as_deref() {
        Some("picked") => {
            if let Some(sel) = result.selector.as_ref() {
                println!("{}", serde_json::to_string(sel).unwrap_or_default());
            } else {
                println!("PICKED (run_id={})", result.run_id);
            }
            std::process::exit(0);
        }
        Some("failed") => {
            let reason = result.reason.as_deref().unwrap_or("unknown");
            let last_bundle_dir = result.last_bundle_dir.as_deref().unwrap_or("");
            if last_bundle_dir.is_empty() {
                eprintln!("FAIL (run_id={}) reason={reason}", result.run_id);
            } else {
                eprintln!(
                    "FAIL (run_id={}) reason={reason} last_bundle_dir={last_bundle_dir}",
                    result.run_id
                );
            }
            std::process::exit(1);
        }
        _ => {
            eprintln!("unexpected pick stage: {:?}", result);
            std::process::exit(1);
        }
    }
}

pub(super) fn write_pick_script(selector: &serde_json::Value, dst: &Path) -> Result<(), String> {
    let script = serde_json::json!({
        "schema_version": 1,
        "steps": [
            { "type": "click", "target": selector },
            { "type": "wait_frames", "frames": 2 },
            { "type": "capture_bundle", "label": "after-picked-click" }
        ]
    });

    let bytes = serde_json::to_vec_pretty(&script).map_err(|e| e.to_string())?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(dst, bytes).map_err(|e| e.to_string())
}

pub(super) fn apply_pick_to_script(
    src: &Path,
    dst: &Path,
    json_pointer: &str,
    selector: serde_json::Value,
) -> Result<(), String> {
    let bytes = std::fs::read(src).map_err(|e| e.to_string())?;
    let mut script: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    json_pointer_set(&mut script, json_pointer, selector)?;

    let bytes = serde_json::to_vec_pretty(&script).map_err(|e| e.to_string())?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(dst, bytes).map_err(|e| e.to_string())
}

pub(super) fn json_pointer_set(
    root: &mut serde_json::Value,
    pointer: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    if pointer.is_empty() {
        *root = value;
        return Ok(());
    }
    if !pointer.starts_with('/') {
        return Err(format!(
            "invalid JSON pointer (must start with '/'): {pointer}"
        ));
    }

    let mut tokens: Vec<String> = pointer[1..]
        .split('/')
        .map(unescape_json_pointer_token)
        .collect();
    if tokens.is_empty() {
        *root = value;
        return Ok(());
    }

    let last = tokens
        .pop()
        .ok_or_else(|| "invalid JSON pointer".to_string())?;

    let mut cur: &mut serde_json::Value = root;
    for t in tokens {
        match cur {
            serde_json::Value::Object(map) => {
                let Some(next) = map.get_mut(&t) else {
                    return Err(format!("JSON pointer path does not exist: {pointer}"));
                };
                cur = next;
            }
            serde_json::Value::Array(arr) => {
                let idx = t
                    .parse::<usize>()
                    .map_err(|_| format!("JSON pointer expected array index, got: {t}"))?;
                let Some(next) = arr.get_mut(idx) else {
                    return Err(format!("JSON pointer array index out of bounds: {pointer}"));
                };
                cur = next;
            }
            _ => {
                return Err(format!(
                    "JSON pointer path does not resolve to a container: {pointer}"
                ));
            }
        }
    }

    match cur {
        serde_json::Value::Object(map) => {
            map.insert(last, value);
            Ok(())
        }
        serde_json::Value::Array(arr) => {
            if last == "-" {
                arr.push(value);
                return Ok(());
            }

            let idx = last
                .parse::<usize>()
                .map_err(|_| format!("JSON pointer expected array index, got: {last}"))?;
            if idx < arr.len() {
                arr[idx] = value;
                return Ok(());
            }
            if idx == arr.len() {
                arr.push(value);
                return Ok(());
            }
            Err(format!("JSON pointer array index out of bounds: {pointer}"))
        }
        _ => Err(format!(
            "JSON pointer path does not resolve to a container: {pointer}"
        )),
    }
}

pub(super) fn check_bundle_for_retained_vlist_keep_alive_reuse_min(
    bundle_path: &Path,
    min_keep_alive_reuse_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_vlist_keep_alive_reuse_min_json(
        &bundle,
        bundle_path,
        min_keep_alive_reuse_frames,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_retained_vlist_keep_alive_reuse_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_keep_alive_reuse_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut keep_alive_reuse_frames: u64 = 0;
    let mut offenders: Vec<String> = Vec::new();

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let reconciles = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            if reconciles.is_empty() {
                continue;
            }

            let any_keep_alive_reuse = reconciles.iter().any(|r| {
                r.get("reused_from_keep_alive_items")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    > 0
            });

            if any_keep_alive_reuse {
                keep_alive_reuse_frames = keep_alive_reuse_frames.saturating_add(1);
            } else {
                let kept_alive_sum = reconciles
                    .iter()
                    .map(|r| {
                        r.get("kept_alive_items")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                    })
                    .sum::<u64>();
                offenders.push(format!(
                    "frame_id={frame_id} reconciles={count} kept_alive_sum={kept_alive_sum}",
                    count = reconciles.len()
                ));
            }
        }
    }

    if keep_alive_reuse_frames < min_keep_alive_reuse_frames {
        let mut msg = String::new();
        msg.push_str("expected retained virtual-list to reuse keep-alive items\n");
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!(
            "min_keep_alive_reuse_frames={min_keep_alive_reuse_frames} keep_alive_reuse_frames={keep_alive_reuse_frames} warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
        ));
        for line in offenders.into_iter().take(10) {
            msg.push_str("  ");
            msg.push_str(&line);
            msg.push('\n');
        }
        return Err(msg);
    }

    Ok(())
}

fn unescape_json_pointer_token(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut it = raw.chars();
    while let Some(c) = it.next() {
        if c == '~' {
            match it.next() {
                Some('0') => out.push('~'),
                Some('1') => out.push('/'),
                Some(other) => {
                    out.push('~');
                    out.push(other);
                }
                None => out.push('~'),
            }
        } else {
            out.push(c);
        }
    }
    out
}
