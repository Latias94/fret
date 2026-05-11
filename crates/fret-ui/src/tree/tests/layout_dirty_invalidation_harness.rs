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
        nodes: Vec<TreeNodeSpec>,
        steps: Vec<TreeStep>,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct TreeNodeSpec {
    id: String,
    #[serde(default)]
    children: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum TreeStep {
    ClearAllInvalidations,
    SetLayoutDirty { node: String, dirty: bool },
    SetChildrenSuppressed { node: String, suppressed: bool },
    RemoveSubtree { node: String },
    SetSubtreeDirtyCount { node: String, count: u32 },
    SetLayoutInvalidationFlag { node: String, layout: bool },
    NoteLayoutInvalidationTransition { node: String, from: bool, to: bool },
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
        LayoutDirtyInvalidationScenario::Tree { root, nodes, steps } => {
            observe_tree_case(root, nodes, steps)
        }
    }
}

fn observe_tree_case(
    root: &str,
    nodes: &[TreeNodeSpec],
    steps: &[TreeStep],
) -> Result<ObservedTree, ScenarioObserveError> {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let mut ids = HashMap::new();
    for node in nodes {
        if ids
            .insert(node.id.clone(), ui.create_node(TestStack))
            .is_some()
        {
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
    for step in steps {
        apply_step(&mut ui, &mut services, &ids, nodes, step)?;
    }

    Ok(observe_metrics(&ui, &ids, nodes))
}

fn apply_step(
    ui: &mut UiTree<crate::test_host::TestHost>,
    services: &mut FakeUiServices,
    ids: &HashMap<String, fret_core::NodeId>,
    nodes: &[TreeNodeSpec],
    step: &TreeStep,
) -> Result<(), ScenarioObserveError> {
    match step {
        TreeStep::ClearAllInvalidations => {
            for node in nodes {
                let id = lookup_node(ids, &node.id)?;
                if ui.nodes.contains_key(id) {
                    ui.test_clear_node_invalidations(id);
                }
            }
        }
        TreeStep::SetLayoutDirty { node, dirty } => {
            ui.test_set_layout_invalidation(lookup_existing_node(ui, ids, node)?, *dirty);
        }
        TreeStep::SetChildrenSuppressed { node, suppressed } => {
            ui.set_layout_dirty_children_suppressed(
                lookup_existing_node(ui, ids, node)?,
                *suppressed,
            );
        }
        TreeStep::RemoveSubtree { node } => {
            ui.remove_subtree(services, lookup_existing_node(ui, ids, node)?);
        }
        TreeStep::SetSubtreeDirtyCount { node, count } => {
            let id = lookup_existing_node(ui, ids, node)?;
            ui.nodes[id].subtree_layout_dirty_count = *count;
        }
        TreeStep::SetLayoutInvalidationFlag { node, layout } => {
            let id = lookup_existing_node(ui, ids, node)?;
            ui.nodes[id].invalidation.layout = *layout;
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

fn observe_metrics(
    ui: &UiTree<crate::test_host::TestHost>,
    ids: &HashMap<String, fret_core::NodeId>,
    nodes: &[TreeNodeSpec],
) -> ObservedTree {
    let mut observed = ObservedTree::new(Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(1.0), Px(1.0)),
    ));

    for node in nodes {
        let id = ids[&node.id];
        let exists = ui.nodes.contains_key(id);
        observed.set_metric(format!("node.{}.exists", node.id), bool_metric(exists));
        if exists {
            let entry = &ui.nodes[id];
            observed.set_metric(
                format!("node.{}.subtree_layout_dirty_count", node.id),
                entry.subtree_layout_dirty_count as f32,
            );
            observed.set_metric(
                format!("node.{}.layout_dirty", node.id),
                bool_metric(entry.invalidation.layout),
            );
            observed.set_metric(
                format!("node.{}.layout_dirty_children_suppressed", node.id),
                bool_metric(entry.layout_dirty_children_suppressed),
            );
        }
    }

    observed.set_metric(
        "debug.layout_subtree_dirty_agg_rebuild_nodes",
        ui.debug_stats().layout_subtree_dirty_agg_rebuild_nodes as f32,
    );
    observed
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
