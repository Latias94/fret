use super::*;
use slotmap::KeyData;
use std::any::TypeId;

fn node(raw: u64) -> NodeId {
    NodeId::from(KeyData::from_ffi(raw))
}

fn model(raw: u64) -> ModelId {
    ModelId::from(KeyData::from_ffi(raw))
}

#[test]
fn empty_model_observation_record_removes_previous_node_entry() {
    let node = node(1);
    let model = model(2);
    let mut index = ObservationIndex::default();

    let added = index.record_with_stats(node, &[(model, Invalidation::Paint)]);
    assert_eq!(added.edges_added, 1);
    assert_eq!(added.edges_removed, 0);
    assert_eq!(added.edges_mask_changed, 0);
    assert!(index.by_node.contains_key(&node));
    assert!(
        index
            .by_model
            .get(&model)
            .is_some_and(|nodes| nodes.contains_key(&node))
    );

    let removed = index.record_with_stats(node, &[]);
    assert_eq!(removed.edges_added, 0);
    assert_eq!(removed.edges_removed, 1);
    assert_eq!(removed.edges_mask_changed, 0);

    assert!(!index.by_node.contains_key(&node));
    assert!(!index.by_model.contains_key(&model));
}

#[test]
fn model_observation_record_reports_mask_changes_without_edge_churn() {
    let node = node(1);
    let model = model(2);
    let mut index = ObservationIndex::default();

    index.record_with_stats(node, &[(model, Invalidation::Paint)]);
    let changed = index.record_with_stats(node, &[(model, Invalidation::Layout)]);

    assert_eq!(changed.edges_added, 0);
    assert_eq!(changed.edges_removed, 0);
    assert_eq!(changed.edges_mask_changed, 1);
}

#[test]
fn empty_global_observation_record_removes_previous_node_entry() {
    let node = node(1);
    let global = TypeId::of::<usize>();
    let mut index = GlobalObservationIndex::default();

    let added = index.record_with_stats(node, &[(global, Invalidation::Paint)]);
    assert_eq!(added.edges_added, 1);
    assert_eq!(added.edges_removed, 0);
    assert_eq!(added.edges_mask_changed, 0);
    assert!(index.by_node.contains_key(&node));
    assert!(
        index
            .by_global
            .get(&global)
            .is_some_and(|nodes| nodes.contains_key(&node))
    );

    let removed = index.record_with_stats(node, &[]);
    assert_eq!(removed.edges_added, 0);
    assert_eq!(removed.edges_removed, 1);
    assert_eq!(removed.edges_mask_changed, 0);

    assert!(!index.by_node.contains_key(&node));
    assert!(!index.by_global.contains_key(&global));
}

#[test]
fn global_observation_record_reports_mask_changes_without_edge_churn() {
    let node = node(1);
    let global = TypeId::of::<usize>();
    let mut index = GlobalObservationIndex::default();

    index.record_with_stats(node, &[(global, Invalidation::Paint)]);
    let changed = index.record_with_stats(node, &[(global, Invalidation::HitTestOnly)]);

    assert_eq!(changed.edges_added, 0);
    assert_eq!(changed.edges_removed, 0);
    assert_eq!(changed.edges_mask_changed, 1);
}
