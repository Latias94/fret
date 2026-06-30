use std::collections::HashMap;

use super::*;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use serde::Deserialize;

const LAYOUT_DIRTY_INVALIDATION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tree/tests/fixtures/layout_dirty_invalidation_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum LayoutDirtyInvalidationScenario {
    Tree {
        root: String,
        #[serde(default)]
        view_cache_enabled: bool,
        nodes: Vec<TreeNodeSpec>,
        steps: Vec<TreeStep>,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct TreeNodeSpec {
    id: String,
    #[serde(default)]
    element_id: Option<u64>,
    #[serde(default)]
    children: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum TreeStep {
    AdvanceFrame,
    BeginDebugFrame,
    CaptureMetrics {
        label: String,
    },
    ClearAllInvalidations,
    Invalidate {
        node: String,
        invalidation: InvalidationKind,
    },
    LayoutAll {
        width: f32,
        height: f32,
    },
    RemoveSubtree {
        node: String,
    },
    SetBoundsAll {
        width: f32,
        height: f32,
    },
    SetChildren {
        node: String,
        children: Vec<String>,
    },
    SetChildrenSuppressed {
        node: String,
        suppressed: bool,
    },
    SetLayoutDirty {
        node: String,
        dirty: bool,
    },
    SetLayoutInvalidationFlag {
        node: String,
        layout: bool,
    },
    SetSubtreeDirtyCount {
        node: String,
        count: u32,
    },
    SetViewCacheFlags {
        node: String,
        enabled: bool,
        layout_contained_when_bounds_known: bool,
        layout_definite: bool,
    },
    NoteLayoutInvalidationTransition {
        node: String,
        from: bool,
        to: bool,
    },
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InvalidationKind {
    HitTest,
    HitTestOnly,
    Layout,
    Paint,
}

impl From<InvalidationKind> for Invalidation {
    fn from(kind: InvalidationKind) -> Self {
        match kind {
            InvalidationKind::HitTest => Invalidation::HitTest,
            InvalidationKind::HitTestOnly => Invalidation::HitTestOnly,
            InvalidationKind::Layout => Invalidation::Layout,
            InvalidationKind::Paint => Invalidation::Paint,
        }
    }
}

#[test]
fn mechanism_harness_layout_dirty_invalidation_matches_oracles() {
    let suite: MechanismSuite<LayoutDirtyInvalidationScenario> =
        MechanismSuite::from_json_str(LAYOUT_DIRTY_INVALIDATION)
            .expect("layout dirty invalidation fixture suite");

    let mut observer: fn(
        &MechanismCase<LayoutDirtyInvalidationScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<LayoutDirtyInvalidationScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        LayoutDirtyInvalidationScenario::Tree {
            root,
            view_cache_enabled,
            nodes,
            steps,
        } => observe_tree_case(root, *view_cache_enabled, nodes, steps),
    }
}

fn observe_tree_case(
    root: &str,
    view_cache_enabled: bool,
    nodes: &[TreeNodeSpec],
    steps: &[TreeStep],
) -> Result<ObservedTree, ScenarioObserveError> {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);
    ui.set_view_cache_enabled(view_cache_enabled);

    let mut ids = HashMap::new();
    for node in nodes {
        let ui_node = if let Some(element_id) = node.element_id {
            ui.create_node_for_element(crate::elements::GlobalElementId(element_id), TestStack)
        } else {
            ui.create_node(TestStack)
        };
        if ids.insert(node.id.clone(), ui_node).is_some() {
            return Err(ScenarioObserveError::new(format!(
                "duplicate node id {:?}",
                node.id
            )));
        }
    }

    let root_id = lookup_node(&ids, root)?;
    ui.set_root(root_id);
    for node in nodes {
        let parent = lookup_node(&ids, &node.id)?;
        let children = node
            .children
            .iter()
            .map(|child| lookup_node(&ids, child))
            .collect::<Result<Vec<_>, _>>()?;
        ui.set_children(parent, children);
    }

    let mut services = FakeUiServices;
    let mut observed = ObservedTree::new(Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(1.0), Px(1.0)),
    ));
    for step in steps {
        apply_step(
            &mut app,
            &mut ui,
            &mut services,
            &ids,
            nodes,
            step,
            &mut observed,
        )?;
    }

    append_metrics(&ui, &ids, nodes, &mut observed, None);
    Ok(observed)
}

fn apply_step(
    app: &mut crate::test_host::TestHost,
    ui: &mut UiTree<crate::test_host::TestHost>,
    services: &mut FakeUiServices,
    ids: &HashMap<String, fret_core::NodeId>,
    nodes: &[TreeNodeSpec],
    step: &TreeStep,
    observed: &mut ObservedTree,
) -> Result<(), ScenarioObserveError> {
    match step {
        TreeStep::AdvanceFrame => {
            app.advance_frame();
        }
        TreeStep::BeginDebugFrame => {
            ui.begin_debug_frame_if_needed(app.frame_id());
        }
        TreeStep::CaptureMetrics { label } => {
            append_metrics(ui, ids, nodes, observed, Some(label));
        }
        TreeStep::ClearAllInvalidations => {
            for node in nodes {
                let id = lookup_node(ids, &node.id)?;
                if ui.nodes.contains_key(id) {
                    ui.test_clear_node_invalidations(id);
                }
            }
        }
        TreeStep::Invalidate { node, invalidation } => {
            ui.invalidate(lookup_existing_node(ui, ids, node)?, (*invalidation).into());
        }
        TreeStep::LayoutAll { width, height } => {
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(*width), Px(*height)),
            );
            ui.layout_all(app, services, bounds, 1.0);
        }
        TreeStep::RemoveSubtree { node } => {
            ui.remove_subtree(services, lookup_existing_node(ui, ids, node)?);
        }
        TreeStep::SetBoundsAll { width, height } => {
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(*width), Px(*height)),
            );
            for node in nodes {
                let id = lookup_node(ids, &node.id)?;
                if ui.nodes.contains_key(id) {
                    ui.nodes[id].bounds = bounds;
                    ui.nodes[id].measured_size = bounds.size;
                }
            }
        }
        TreeStep::SetChildren { node, children } => {
            let parent = lookup_existing_node(ui, ids, node)?;
            let child_ids = children
                .iter()
                .map(|child| lookup_existing_node(ui, ids, child))
                .collect::<Result<Vec<_>, _>>()?;
            ui.set_children(parent, child_ids);
        }
        TreeStep::SetChildrenSuppressed { node, suppressed } => {
            ui.set_layout_dirty_children_suppressed(
                lookup_existing_node(ui, ids, node)?,
                *suppressed,
            );
        }
        TreeStep::SetLayoutDirty { node, dirty } => {
            ui.test_set_layout_invalidation(lookup_existing_node(ui, ids, node)?, *dirty);
        }
        TreeStep::SetLayoutInvalidationFlag { node, layout } => {
            let id = lookup_existing_node(ui, ids, node)?;
            ui.nodes[id].invalidation.layout = *layout;
        }
        TreeStep::SetSubtreeDirtyCount { node, count } => {
            let id = lookup_existing_node(ui, ids, node)?;
            ui.nodes[id].subtree_layout_dirty_count = *count;
        }
        TreeStep::SetViewCacheFlags {
            node,
            enabled,
            layout_contained_when_bounds_known,
            layout_definite,
        } => {
            ui.set_node_view_cache_flags(
                lookup_existing_node(ui, ids, node)?,
                *enabled,
                *layout_contained_when_bounds_known,
                *layout_definite,
            );
        }
        TreeStep::NoteLayoutInvalidationTransition { node, from, to } => {
            ui.note_layout_invalidation_transition_for_subtree_aggregation(
                lookup_existing_node(ui, ids, node)?,
                *from,
                *to,
            );
        }
    }
    Ok(())
}

fn append_metrics(
    ui: &UiTree<crate::test_host::TestHost>,
    ids: &HashMap<String, fret_core::NodeId>,
    nodes: &[TreeNodeSpec],
    observed: &mut ObservedTree,
    prefix: Option<&str>,
) {
    for node in nodes {
        let id = ids[&node.id];
        let exists = ui.nodes.contains_key(id);
        set_metric(
            observed,
            prefix,
            format!("node.{}.exists", node.id),
            bool_metric(exists),
        );
        if exists {
            let entry = &ui.nodes[id];
            set_metric(
                observed,
                prefix,
                format!("node.{}.subtree_layout_dirty_count", node.id),
                entry.subtree_layout_dirty_count as f32,
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.subtree_layout_dirty", node.id),
                bool_metric(ui.node_subtree_layout_dirty(id)),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.layout_dirty", node.id),
                bool_metric(entry.invalidation.layout),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.paint_dirty", node.id),
                bool_metric(entry.invalidation.paint),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.hit_test_dirty", node.id),
                bool_metric(entry.invalidation.hit_test),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.node_needs_layout", node.id),
                bool_metric(ui.node_needs_layout(id)),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.layout_dirty_children_suppressed", node.id),
                bool_metric(entry.layout_dirty_children_suppressed),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.view_cache_enabled", node.id),
                bool_metric(entry.view_cache.enabled),
            );
            set_metric(
                observed,
                prefix,
                format!(
                    "node.{}.view_cache_layout_contained_when_bounds_known",
                    node.id
                ),
                bool_metric(entry.view_cache.layout_contained_when_bounds_known()),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.view_cache_layout_definite", node.id),
                bool_metric(entry.view_cache.layout_definite),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.view_cache_needs_rerender", node.id),
                bool_metric(entry.view_cache_needs_rerender),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.should_reuse_view_cache", node.id),
                bool_metric(ui.should_reuse_view_cache_node(id)),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.dirty_cache_root", node.id),
                bool_metric(ui.boundary_layout_dirty(id)),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.dirty_boundary", node.id),
                bool_metric(ui.boundary_layout_dirty(id)),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.contained_relayout_root", node.id),
                bool_metric(ui.debug_view_cache_contained_relayout_roots().contains(&id)),
            );
            set_metric(
                observed,
                prefix,
                format!("node.{}.dirty_view", node.id),
                bool_metric(
                    ui.debug_dirty_views()
                        .iter()
                        .any(|dirty| dirty.view.0 == id),
                ),
            );
            set_metric(
                observed,
                prefix,
                format!(
                    "node.{}.subtree_layout_dirty_covered_by_contained_view_cache_roots",
                    node.id
                ),
                bool_metric(ui.node_subtree_layout_dirty_covered_by_contained_view_cache_roots(id)),
            );
        }
    }

    set_metric(
        observed,
        prefix,
        "debug.layout_subtree_dirty_agg_rebuild_nodes",
        ui.debug_stats().layout_subtree_dirty_agg_rebuild_nodes as f32,
    );
    set_metric(
        observed,
        prefix,
        "debug.view_cache_contained_relayouts",
        ui.debug_stats().view_cache_contained_relayouts as f32,
    );
    set_metric(
        observed,
        prefix,
        "debug.dirty_cache_roots_count",
        ui.test_dirty_view_frontier_len() as f32,
    );
    set_metric(
        observed,
        prefix,
        "debug.dirty_boundaries_count",
        ui.test_dirty_view_frontier_len() as f32,
    );
    set_metric(
        observed,
        prefix,
        "debug.contained_relayout_roots_count",
        ui.debug_view_cache_contained_relayout_roots().len() as f32,
    );
    set_metric(
        observed,
        prefix,
        "debug.dirty_views_count",
        ui.debug_dirty_views().len() as f32,
    );
    set_metric(
        observed,
        prefix,
        "debug.layout_fast_path_or_skipped",
        bool_metric(
            ui.debug_stats().layout_fast_path_taken || ui.debug_stats().layout_skipped_engine_frame,
        ),
    );
    set_metric(
        observed,
        prefix,
        "debug.layout_request_build_roots_count",
        ui.debug_layout_request_build_roots().len() as f32,
    );
    append_layout_request_build_root_metrics(ui, ids, observed, prefix);
}

fn append_layout_request_build_root_metrics(
    ui: &UiTree<crate::test_host::TestHost>,
    ids: &HashMap<String, fret_core::NodeId>,
    observed: &mut ObservedTree,
    prefix: Option<&str>,
) {
    let labels = ids
        .iter()
        .map(|(label, id)| (*id, label.as_str()))
        .collect::<HashMap<_, _>>();

    for record in ui.debug_layout_request_build_roots() {
        let Some(root_label) = labels.get(&record.root).copied() else {
            continue;
        };
        set_metric(
            observed,
            prefix,
            format!("node.{root_label}.layout_request_descendant_layout_dirty_count"),
            record.descendant_layout_dirty_count as f32,
        );
        for dirty in &record.dirty_descendants {
            let Some(detail) = dirty.detail.and_then(|detail| detail.as_str()) else {
                continue;
            };
            set_metric_add(
                observed,
                prefix,
                format!("node.{root_label}.layout_request_dirty_descendant_detail.{detail}_count"),
                1.0,
            );
            if let Some(source_label) = dirty
                .source_root
                .and_then(|source| labels.get(&source).copied())
            {
                set_metric_add(
                    observed,
                    prefix,
                    format!(
                        "node.{root_label}.layout_request_dirty_descendant_detail.{detail}_from.{source_label}_count"
                    ),
                    1.0,
                );
            }
            if let Some(node_label) = labels.get(&dirty.node).copied() {
                set_metric_add(
                    observed,
                    prefix,
                    format!(
                        "node.{root_label}.layout_request_dirty_descendant_detail.{detail}_node.{node_label}_count"
                    ),
                    1.0,
                );
            }
        }
    }
}

fn set_metric(
    observed: &mut ObservedTree,
    prefix: Option<&str>,
    id: impl Into<String>,
    value: f32,
) {
    observed.set_metric(metric_id(prefix, id.into()), value);
}

fn set_metric_add(
    observed: &mut ObservedTree,
    prefix: Option<&str>,
    id: impl Into<String>,
    value: f32,
) {
    let id = metric_id(prefix, id.into());
    let next = observed.metric_value(&id).unwrap_or(0.0) + value;
    observed.set_metric(id, next);
}

fn metric_id(prefix: Option<&str>, id: String) -> String {
    if let Some(prefix) = prefix {
        format!("capture.{prefix}.{id}")
    } else {
        id
    }
}

fn lookup_node(
    ids: &HashMap<String, fret_core::NodeId>,
    node: &str,
) -> Result<fret_core::NodeId, ScenarioObserveError> {
    ids.get(node)
        .copied()
        .ok_or_else(|| ScenarioObserveError::new(format!("unknown node id {node:?}")))
}

fn lookup_existing_node(
    ui: &UiTree<crate::test_host::TestHost>,
    ids: &HashMap<String, fret_core::NodeId>,
    node: &str,
) -> Result<fret_core::NodeId, ScenarioObserveError> {
    let id = lookup_node(ids, node)?;
    if ui.nodes.contains_key(id) {
        Ok(id)
    } else {
        Err(ScenarioObserveError::new(format!(
            "node id {node:?} no longer exists"
        )))
    }
}

fn bool_metric(value: bool) -> f32 {
    if value { 1.0 } else { 0.0 }
}
