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

    index.record(node, &[(model, Invalidation::Paint)]);
    assert!(index.by_node.contains_key(&node));
    assert!(
        index
            .by_model
            .get(&model)
            .is_some_and(|nodes| nodes.contains_key(&node))
    );

    index.record(node, &[]);

    assert!(!index.by_node.contains_key(&node));
    assert!(!index.by_model.contains_key(&model));
}

#[test]
fn empty_global_observation_record_removes_previous_node_entry() {
    let node = node(1);
    let global = TypeId::of::<usize>();
    let mut index = GlobalObservationIndex::default();

    index.record(node, &[(global, Invalidation::Paint)]);
    assert!(index.by_node.contains_key(&node));
    assert!(
        index
            .by_global
            .get(&global)
            .is_some_and(|nodes| nodes.contains_key(&node))
    );

    index.record(node, &[]);

    assert!(!index.by_node.contains_key(&node));
    assert!(!index.by_global.contains_key(&global));
}
