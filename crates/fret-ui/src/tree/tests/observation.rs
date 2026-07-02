use super::*;
use crate::tree::view_boundary::BoundaryId;
use slotmap::KeyData;
use std::any::TypeId;

fn node(raw: u64) -> NodeId {
    NodeId::from(KeyData::from_ffi(raw))
}

fn model(raw: u64) -> ModelId {
    ModelId::from(KeyData::from_ffi(raw))
}

fn boundary(raw: u64) -> BoundaryId {
    BoundaryId::from(KeyData::from_ffi(raw))
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
    let subscriber = ObservationSubscriber::node(node);
    assert!(index.by_subscriber.contains_key(&subscriber));
    assert!(
        index
            .by_model
            .get(&model)
            .is_some_and(|nodes| nodes.contains_key(&subscriber))
    );

    let removed = index.record_with_stats(node, &[]);
    assert_eq!(removed.edges_added, 0);
    assert_eq!(removed.edges_removed, 1);
    assert_eq!(removed.edges_mask_changed, 0);

    assert!(!index.by_subscriber.contains_key(&subscriber));
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
fn model_observation_record_reports_subscriber_migration_as_edge_churn() {
    let node = node(1);
    let model = model(2);
    let first = ObservationSubscriber::boundary(boundary(3));
    let second = ObservationSubscriber::boundary(boundary(4));
    let mut index = ObservationIndex::default();

    index.record_node_for_subscriber_with_stats(node, first, &[(model, Invalidation::Paint)]);
    let migrated =
        index.record_node_for_subscriber_with_stats(node, second, &[(model, Invalidation::Paint)]);

    assert_eq!(migrated.edges_added, 1);
    assert_eq!(migrated.edges_removed, 1);
    assert_eq!(migrated.edges_mask_changed, 0);
    assert!(!index.by_subscriber.contains_key(&first));
    assert!(index.by_subscriber.contains_key(&second));
    assert!(index.by_model.get(&model).is_some_and(
        |subscribers| !subscribers.contains_key(&first) && subscribers.contains_key(&second)
    ));
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
    let subscriber = ObservationSubscriber::node(node);
    assert!(index.by_subscriber.contains_key(&subscriber));
    assert!(
        index
            .by_global
            .get(&global)
            .is_some_and(|nodes| nodes.contains_key(&subscriber))
    );

    let removed = index.record_with_stats(node, &[]);
    assert_eq!(removed.edges_added, 0);
    assert_eq!(removed.edges_removed, 1);
    assert_eq!(removed.edges_mask_changed, 0);

    assert!(!index.by_subscriber.contains_key(&subscriber));
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

#[test]
fn global_observation_record_reports_subscriber_migration_as_edge_churn() {
    let node = node(1);
    let global = TypeId::of::<usize>();
    let first = ObservationSubscriber::boundary(boundary(3));
    let second = ObservationSubscriber::boundary(boundary(4));
    let mut index = GlobalObservationIndex::default();

    index.record_node_for_subscriber_with_stats(node, first, &[(global, Invalidation::Paint)]);
    let migrated =
        index.record_node_for_subscriber_with_stats(node, second, &[(global, Invalidation::Paint)]);

    assert_eq!(migrated.edges_added, 1);
    assert_eq!(migrated.edges_removed, 1);
    assert_eq!(migrated.edges_mask_changed, 0);
    assert!(!index.by_subscriber.contains_key(&first));
    assert!(index.by_subscriber.contains_key(&second));
    assert!(index.by_global.get(&global).is_some_and(
        |subscribers| !subscribers.contains_key(&first) && subscribers.contains_key(&second)
    ));
}
