use super::{
    BundleStatsGlobalChangeHotspot, BundleStatsGlobalChangeUnobserved,
    BundleStatsLayoutEngineMeasureChildHotspot, BundleStatsLayoutEngineMeasureHotspot,
    BundleStatsLayoutEngineSolve, BundleStatsLayoutHotspot, BundleStatsModelChangeHotspot,
    BundleStatsModelChangeUnobserved, BundleStatsPaintTextPrepareHotspot,
    BundleStatsPaintWidgetHotspot, BundleStatsWidgetMeasureHotspot,
};

pub(super) fn snapshot_paint_widget_hotspots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsPaintWidgetHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("paint_widget_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsPaintWidgetHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsPaintWidgetHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            widget_type: h
                .get("widget_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            paint_time_us: h.get("paint_time_us").and_then(|v| v.as_u64()).unwrap_or(0),
            inclusive_time_us: h
                .get("inclusive_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            inclusive_scene_ops_delta: h
                .get("inclusive_scene_ops_delta")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            exclusive_scene_ops_delta: h
                .get("exclusive_scene_ops_delta")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            role: None,
            test_id: None,
        })
        .collect();

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

pub(super) fn snapshot_layout_hotspots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsLayoutHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("layout_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsLayoutHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsLayoutHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            element_path: h
                .get("element_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            widget_type: h
                .get("widget_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            layout_time_us: h
                .get("layout_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            inclusive_time_us: h
                .get("inclusive_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            role: None,
            test_id: None,
        })
        .collect();

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

pub(super) fn snapshot_widget_measure_hotspots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsWidgetMeasureHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("widget_measure_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsWidgetMeasureHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsWidgetMeasureHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            element_path: h
                .get("element_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            widget_type: h
                .get("widget_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            measure_time_us: h
                .get("measure_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            inclusive_time_us: h
                .get("inclusive_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            role: None,
            test_id: None,
        })
        .collect();

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

pub(super) fn snapshot_paint_text_prepare_hotspots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsPaintTextPrepareHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("paint_text_prepare_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsPaintTextPrepareHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsPaintTextPrepareHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            prepare_time_us: h
                .get("prepare_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            text_len: h
                .get("text_len")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            max_width: h
                .get("max_width")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32),
            wrap: h
                .get("wrap")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            overflow: h
                .get("overflow")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            scale_factor: h
                .get("scale_factor")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32),
            reasons_mask: h
                .get("reasons_mask")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u16::MAX as u64) as u16,
            role: None,
            test_id: None,
        })
        .collect();

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

pub(super) fn format_text_prepare_reasons(mask: u16) -> String {
    let mut out = String::new();
    let mut push = |name: &str| {
        if !out.is_empty() {
            out.push('|');
        }
        out.push_str(name);
    };
    if mask & (1 << 0) != 0 {
        push("blob");
    }
    if mask & (1 << 1) != 0 {
        push("scale");
    }
    if mask & (1 << 2) != 0 {
        push("text");
    }
    if mask & (1 << 3) != 0 {
        push("rich");
    }
    if mask & (1 << 4) != 0 {
        push("style");
    }
    if mask & (1 << 5) != 0 {
        push("wrap");
    }
    if mask & (1 << 6) != 0 {
        push("overflow");
    }
    if mask & (1 << 7) != 0 {
        push("width");
    }
    if mask & (1 << 8) != 0 {
        push("font");
    }
    if out.is_empty() {
        out.push('0');
    }
    out
}

pub(super) fn snapshot_layout_engine_solves(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
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

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

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
                root_element: s.get("root_element").and_then(|v| v.as_u64()),
                root_element_kind: s
                    .get("root_element_kind")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                root_element_path: s
                    .get("root_element_path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
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

pub(super) fn snapshot_model_change_hotspots(
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

pub(super) fn snapshot_model_change_unobserved(
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

pub(super) fn snapshot_global_change_hotspots(
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

pub(super) fn snapshot_global_change_unobserved(
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

pub(super) fn snapshot_lookup_semantics(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    node_id: u64,
) -> (Option<String>, Option<String>) {
    let nodes = semantics.nodes(snapshot).unwrap_or(&[]);

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
pub(super) struct SemanticsIndex {
    by_id: std::collections::HashMap<u64, SemanticsNodeLite>,
    best_descendant_with_test_id: std::collections::HashMap<u64, (Option<String>, Option<String>)>,
}

impl SemanticsIndex {
    pub(super) fn from_snapshot(
        semantics: &crate::json_bundle::SemanticsResolver<'_>,
        snapshot: &serde_json::Value,
    ) -> Self {
        let nodes = semantics.nodes(snapshot).unwrap_or(&[]);

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

    pub(super) fn lookup_for_cache_root(&self, root_node: u64) -> (Option<String>, Option<String>) {
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

// NOTE: Gate checks (retained-vlist keep-alive budget, notify hotspot counters, etc.) intentionally
// stay in `crates/fret-diag/src/stats.rs` (or dedicated `*_gates.rs` modules). This file is scoped
// to snapshot-derived helpers used by bundle stats/hotspots reporting.
