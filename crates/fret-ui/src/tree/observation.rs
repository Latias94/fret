use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct ObservationMask {
    pub(super) paint: bool,
    pub(super) layout: bool,
    pub(super) hit_test: bool,
}

impl ObservationMask {
    pub(super) fn add(&mut self, inv: Invalidation) {
        match inv {
            Invalidation::Paint => self.paint = true,
            Invalidation::Layout => {
                self.layout = true;
                self.paint = true;
            }
            Invalidation::HitTest => {
                self.hit_test = true;
                self.layout = true;
                self.paint = true;
            }
            Invalidation::HitTestOnly => {
                self.hit_test = true;
                self.paint = true;
            }
        }
    }

    pub(super) fn union(self, other: Self) -> Self {
        Self {
            paint: self.paint || other.paint,
            layout: self.layout || other.layout,
            hit_test: self.hit_test || other.hit_test,
        }
    }

    pub(super) fn is_empty(self) -> bool {
        !(self.paint || self.layout || self.hit_test)
    }
}

#[derive(Default)]
pub(super) struct ObservationIndex {
    pub(super) by_node: HashMap<NodeId, Vec<(ModelId, ObservationMask)>>,
    pub(super) by_model: HashMap<ModelId, HashMap<NodeId, ObservationMask>>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(in crate::tree) struct ObservationIndexRecordStats {
    pub(super) edges_added: u32,
    pub(super) edges_removed: u32,
    pub(super) edges_mask_changed: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(in crate::tree) struct ObservationIndexRemoveStats {
    pub(super) removed: bool,
    pub(super) edges: u32,
}

impl ObservationIndex {
    pub(super) fn record(&mut self, node: NodeId, observations: &[(ModelId, Invalidation)]) {
        let _ = self.record_inner(node, observations, false);
    }

    pub(super) fn record_with_stats(
        &mut self,
        node: NodeId,
        observations: &[(ModelId, Invalidation)],
    ) -> ObservationIndexRecordStats {
        self.record_inner(node, observations, true)
    }

    fn record_inner(
        &mut self,
        node: NodeId,
        observations: &[(ModelId, Invalidation)],
        collect_stats: bool,
    ) -> ObservationIndexRecordStats {
        if observations.is_empty() {
            let removed = self.remove_node(node);
            if collect_stats {
                return ObservationIndexRecordStats {
                    edges_removed: removed.edges,
                    ..Default::default()
                };
            }
            return ObservationIndexRecordStats::default();
        }

        let entry = self.by_node.entry(node).or_default();

        let prev_entries = collect_stats.then(|| entry.clone());
        let mut prev_models = SmallCopyList::<ModelId, 8>::default();
        for (model, _) in entry.iter() {
            prev_models.push(*model);
        }

        entry.clear();
        entry.reserve(observations.len());
        for &(model, inv) in observations {
            if let Some((_, mask)) = entry.iter_mut().find(|(m, _)| *m == model) {
                mask.add(inv);
            } else {
                let mut mask = ObservationMask::default();
                mask.add(inv);
                entry.push((model, mask));
            }
        }

        for model in prev_models.as_slice() {
            if entry.iter().any(|(m, _)| *m == *model) {
                continue;
            }
            if let Some(nodes) = self.by_model.get_mut(model) {
                nodes.remove(&node);
                if nodes.is_empty() {
                    self.by_model.remove(model);
                }
            }
        }

        for (model, mask) in entry.iter().copied() {
            self.by_model.entry(model).or_default().insert(node, mask);
        }

        prev_entries
            .as_ref()
            .map(|prev_entries| observation_record_stats(prev_entries, entry))
            .unwrap_or_default()
    }

    pub(super) fn remove_node(&mut self, node: NodeId) -> ObservationIndexRemoveStats {
        let Some(prev) = self.by_node.remove(&node) else {
            return ObservationIndexRemoveStats::default();
        };
        let edges = prev.len().min(u32::MAX as usize) as u32;
        for (model, _) in &prev {
            if let Some(nodes) = self.by_model.get_mut(model) {
                nodes.remove(&node);
                if nodes.is_empty() {
                    self.by_model.remove(model);
                }
            }
        }
        ObservationIndexRemoveStats {
            removed: true,
            edges,
        }
    }
}

#[derive(Default)]
pub(super) struct GlobalObservationIndex {
    pub(super) by_node: HashMap<NodeId, Vec<(TypeId, ObservationMask)>>,
    pub(super) by_global: HashMap<TypeId, HashMap<NodeId, ObservationMask>>,
}

impl GlobalObservationIndex {
    pub(super) fn record(&mut self, node: NodeId, observations: &[(TypeId, Invalidation)]) {
        let _ = self.record_inner(node, observations, false);
    }

    pub(super) fn record_with_stats(
        &mut self,
        node: NodeId,
        observations: &[(TypeId, Invalidation)],
    ) -> ObservationIndexRecordStats {
        self.record_inner(node, observations, true)
    }

    fn record_inner(
        &mut self,
        node: NodeId,
        observations: &[(TypeId, Invalidation)],
        collect_stats: bool,
    ) -> ObservationIndexRecordStats {
        if observations.is_empty() {
            let removed = self.remove_node(node);
            if collect_stats {
                return ObservationIndexRecordStats {
                    edges_removed: removed.edges,
                    ..Default::default()
                };
            }
            return ObservationIndexRecordStats::default();
        }

        let entry = self.by_node.entry(node).or_default();

        let prev_entries = collect_stats.then(|| entry.clone());
        let mut prev_globals = SmallCopyList::<TypeId, 8>::default();
        for (global, _) in entry.iter() {
            prev_globals.push(*global);
        }

        entry.clear();
        entry.reserve(observations.len());
        for &(global, inv) in observations {
            if let Some((_, mask)) = entry.iter_mut().find(|(g, _)| *g == global) {
                mask.add(inv);
            } else {
                let mut mask = ObservationMask::default();
                mask.add(inv);
                entry.push((global, mask));
            }
        }

        for global in prev_globals.as_slice() {
            if entry.iter().any(|(g, _)| *g == *global) {
                continue;
            }
            if let Some(nodes) = self.by_global.get_mut(global) {
                nodes.remove(&node);
                if nodes.is_empty() {
                    self.by_global.remove(global);
                }
            }
        }

        for (global, mask) in entry.iter().copied() {
            self.by_global.entry(global).or_default().insert(node, mask);
        }

        prev_entries
            .as_ref()
            .map(|prev_entries| observation_record_stats(prev_entries, entry))
            .unwrap_or_default()
    }

    pub(super) fn remove_node(&mut self, node: NodeId) -> ObservationIndexRemoveStats {
        let Some(prev) = self.by_node.remove(&node) else {
            return ObservationIndexRemoveStats::default();
        };
        let edges = prev.len().min(u32::MAX as usize) as u32;
        for (global, _) in &prev {
            if let Some(nodes) = self.by_global.get_mut(global) {
                nodes.remove(&node);
                if nodes.is_empty() {
                    self.by_global.remove(global);
                }
            }
        }
        ObservationIndexRemoveStats {
            removed: true,
            edges,
        }
    }
}

fn observation_record_stats<T: PartialEq>(
    prev: &[(T, ObservationMask)],
    next: &[(T, ObservationMask)],
) -> ObservationIndexRecordStats {
    let mut edges_added = 0u32;
    let mut edges_removed = 0u32;
    let mut edges_mask_changed = 0u32;

    for (id, mask) in next {
        match prev.iter().find(|(prev_id, _)| prev_id == id) {
            Some((_, prev_mask)) if prev_mask != mask => {
                edges_mask_changed = edges_mask_changed.saturating_add(1);
            }
            Some(_) => {}
            None => edges_added = edges_added.saturating_add(1),
        }
    }

    for (id, _) in prev {
        if !next.iter().any(|(next_id, _)| next_id == id) {
            edges_removed = edges_removed.saturating_add(1);
        }
    }

    ObservationIndexRecordStats {
        edges_added,
        edges_removed,
        edges_mask_changed,
    }
}
