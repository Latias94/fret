use super::view_boundary::{BoundaryId, ViewBoundaryStore};
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ObservationMaskCounts {
    paint: u32,
    layout: u32,
    hit_test: u32,
}

impl ObservationMaskCounts {
    fn add_mask(&mut self, mask: ObservationMask) {
        if mask.paint {
            self.paint = self.paint.saturating_add(1);
        }
        if mask.layout {
            self.layout = self.layout.saturating_add(1);
        }
        if mask.hit_test {
            self.hit_test = self.hit_test.saturating_add(1);
        }
    }

    fn remove_mask(&mut self, mask: ObservationMask) {
        if mask.paint {
            self.paint = self.paint.saturating_sub(1);
        }
        if mask.layout {
            self.layout = self.layout.saturating_sub(1);
        }
        if mask.hit_test {
            self.hit_test = self.hit_test.saturating_sub(1);
        }
    }

    fn mask(self) -> ObservationMask {
        ObservationMask {
            paint: self.paint > 0,
            layout: self.layout > 0,
            hit_test: self.hit_test > 0,
        }
    }

    fn is_empty(self) -> bool {
        self.paint == 0 && self.layout == 0 && self.hit_test == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum ObservationSubscriber {
    Node(NodeId),
    Boundary(BoundaryId),
}

impl ObservationSubscriber {
    pub(super) fn node(node: NodeId) -> Self {
        Self::Node(node)
    }

    pub(super) fn boundary(boundary: BoundaryId) -> Self {
        Self::Boundary(boundary)
    }

    pub(super) fn live_node(self, boundaries: &ViewBoundaryStore) -> Option<NodeId> {
        match self {
            Self::Node(node) => Some(node),
            Self::Boundary(boundary) => boundaries.live_node_for_boundary(boundary),
        }
    }
}

#[derive(Debug, Clone)]
struct SubscriberObservationRecord<T> {
    subscriber: ObservationSubscriber,
    entries: Vec<(T, ObservationMask)>,
}

#[derive(Default)]
pub(super) struct ObservationIndex {
    by_node: HashMap<NodeId, SubscriberObservationRecord<ModelId>>,
    by_subscriber_counts: HashMap<ObservationSubscriber, HashMap<ModelId, ObservationMaskCounts>>,
    pub(super) by_subscriber: HashMap<ObservationSubscriber, Vec<(ModelId, ObservationMask)>>,
    pub(super) by_model: HashMap<ModelId, HashMap<ObservationSubscriber, ObservationMask>>,
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
    #[cfg(test)]
    pub(super) fn record_with_stats(
        &mut self,
        node: NodeId,
        observations: &[(ModelId, Invalidation)],
    ) -> ObservationIndexRecordStats {
        self.record_node_for_subscriber_with_stats(
            node,
            ObservationSubscriber::node(node),
            observations,
        )
    }

    pub(super) fn record_node_for_subscriber(
        &mut self,
        node: NodeId,
        subscriber: ObservationSubscriber,
        observations: &[(ModelId, Invalidation)],
    ) {
        let _ = self.record_node_for_subscriber_inner(node, subscriber, observations, false);
    }

    pub(super) fn record_node_for_subscriber_with_stats(
        &mut self,
        node: NodeId,
        subscriber: ObservationSubscriber,
        observations: &[(ModelId, Invalidation)],
    ) -> ObservationIndexRecordStats {
        self.record_node_for_subscriber_inner(node, subscriber, observations, true)
    }

    fn record_node_for_subscriber_inner(
        &mut self,
        node: NodeId,
        subscriber: ObservationSubscriber,
        observations: &[(ModelId, Invalidation)],
        collect_stats: bool,
    ) -> ObservationIndexRecordStats {
        let next_entries = model_observation_entries(observations);
        let prev = self.by_node.remove(&node);
        let prev_entries = prev
            .as_ref()
            .map(|record| record.entries.as_slice())
            .unwrap_or(&[]);

        if let Some(prev) = &prev {
            for (model, mask) in prev.entries.iter().copied() {
                self.apply_model_delta(prev.subscriber, model, mask, false);
            }
        }

        if !next_entries.is_empty() {
            for (model, mask) in next_entries.iter().copied() {
                self.apply_model_delta(subscriber, model, mask, true);
            }
            self.by_node.insert(
                node,
                SubscriberObservationRecord {
                    subscriber,
                    entries: next_entries.clone(),
                },
            );
        }

        if collect_stats {
            if prev
                .as_ref()
                .is_some_and(|record| record.subscriber != subscriber)
            {
                observation_subscriber_migration_stats(prev_entries, next_entries.as_slice())
            } else {
                observation_record_stats(prev_entries, next_entries.as_slice())
            }
        } else {
            ObservationIndexRecordStats::default()
        }
    }

    pub(super) fn remove_node(&mut self, node: NodeId) -> ObservationIndexRemoveStats {
        let Some(prev) = self.by_node.remove(&node) else {
            return ObservationIndexRemoveStats::default();
        };
        let edges = prev.entries.len().min(u32::MAX as usize) as u32;
        for (model, mask) in prev.entries.iter().copied() {
            self.apply_model_delta(prev.subscriber, model, mask, false);
        }
        ObservationIndexRemoveStats {
            removed: true,
            edges,
        }
    }

    pub(super) fn remove_boundary(&mut self, boundary: BoundaryId) -> ObservationIndexRemoveStats {
        let subscriber = ObservationSubscriber::boundary(boundary);
        let nodes: Vec<NodeId> = self
            .by_node
            .iter()
            .filter_map(|(&node, record)| (record.subscriber == subscriber).then_some(node))
            .collect();

        let mut stats = ObservationIndexRemoveStats::default();
        for node in nodes {
            let removed = self.remove_node(node);
            stats.removed |= removed.removed;
            stats.edges = stats.edges.saturating_add(removed.edges);
        }

        if !stats.removed
            && let Some(entries) = self.by_subscriber.get(&subscriber).cloned()
        {
            for (model, mask) in entries {
                self.apply_model_delta(subscriber, model, mask, false);
                stats.edges = stats.edges.saturating_add(1);
            }
            stats.removed = stats.edges > 0;
        }
        stats
    }

    fn apply_model_delta(
        &mut self,
        subscriber: ObservationSubscriber,
        model: ModelId,
        mask: ObservationMask,
        add: bool,
    ) {
        let counts = self
            .by_subscriber_counts
            .entry(subscriber)
            .or_default()
            .entry(model)
            .or_default();
        if add {
            counts.add_mask(mask);
        } else {
            counts.remove_mask(mask);
        }
        let next = counts.mask();
        let empty = counts.is_empty();

        if empty {
            if let Some(models) = self.by_subscriber_counts.get_mut(&subscriber) {
                models.remove(&model);
                if models.is_empty() {
                    self.by_subscriber_counts.remove(&subscriber);
                }
            }
            remove_model_from_subscriber(&mut self.by_subscriber, subscriber, &model);
            remove_subscriber_from_model(&mut self.by_model, model, subscriber);
        } else {
            upsert_subscriber_model(&mut self.by_subscriber, subscriber, model, next);
            self.by_model
                .entry(model)
                .or_default()
                .insert(subscriber, next);
        }
    }
}

#[derive(Default)]
pub(super) struct GlobalObservationIndex {
    by_node: HashMap<NodeId, SubscriberObservationRecord<TypeId>>,
    by_subscriber_counts: HashMap<ObservationSubscriber, HashMap<TypeId, ObservationMaskCounts>>,
    pub(super) by_subscriber: HashMap<ObservationSubscriber, Vec<(TypeId, ObservationMask)>>,
    pub(super) by_global: HashMap<TypeId, HashMap<ObservationSubscriber, ObservationMask>>,
}

impl GlobalObservationIndex {
    #[cfg(test)]
    pub(super) fn record_with_stats(
        &mut self,
        node: NodeId,
        observations: &[(TypeId, Invalidation)],
    ) -> ObservationIndexRecordStats {
        self.record_node_for_subscriber_with_stats(
            node,
            ObservationSubscriber::node(node),
            observations,
        )
    }

    pub(super) fn record_node_for_subscriber(
        &mut self,
        node: NodeId,
        subscriber: ObservationSubscriber,
        observations: &[(TypeId, Invalidation)],
    ) {
        let _ = self.record_node_for_subscriber_inner(node, subscriber, observations, false);
    }

    pub(super) fn record_node_for_subscriber_with_stats(
        &mut self,
        node: NodeId,
        subscriber: ObservationSubscriber,
        observations: &[(TypeId, Invalidation)],
    ) -> ObservationIndexRecordStats {
        self.record_node_for_subscriber_inner(node, subscriber, observations, true)
    }

    fn record_node_for_subscriber_inner(
        &mut self,
        node: NodeId,
        subscriber: ObservationSubscriber,
        observations: &[(TypeId, Invalidation)],
        collect_stats: bool,
    ) -> ObservationIndexRecordStats {
        let next_entries = global_observation_entries(observations);
        let prev = self.by_node.remove(&node);
        let prev_entries = prev
            .as_ref()
            .map(|record| record.entries.as_slice())
            .unwrap_or(&[]);

        if let Some(prev) = &prev {
            for (global, mask) in prev.entries.iter().copied() {
                self.apply_global_delta(prev.subscriber, global, mask, false);
            }
        }

        if !next_entries.is_empty() {
            for (global, mask) in next_entries.iter().copied() {
                self.apply_global_delta(subscriber, global, mask, true);
            }
            self.by_node.insert(
                node,
                SubscriberObservationRecord {
                    subscriber,
                    entries: next_entries.clone(),
                },
            );
        }

        if collect_stats {
            if prev
                .as_ref()
                .is_some_and(|record| record.subscriber != subscriber)
            {
                observation_subscriber_migration_stats(prev_entries, next_entries.as_slice())
            } else {
                observation_record_stats(prev_entries, next_entries.as_slice())
            }
        } else {
            ObservationIndexRecordStats::default()
        }
    }

    pub(super) fn remove_node(&mut self, node: NodeId) -> ObservationIndexRemoveStats {
        let Some(prev) = self.by_node.remove(&node) else {
            return ObservationIndexRemoveStats::default();
        };
        let edges = prev.entries.len().min(u32::MAX as usize) as u32;
        for (global, mask) in prev.entries.iter().copied() {
            self.apply_global_delta(prev.subscriber, global, mask, false);
        }
        ObservationIndexRemoveStats {
            removed: true,
            edges,
        }
    }

    pub(super) fn remove_boundary(&mut self, boundary: BoundaryId) -> ObservationIndexRemoveStats {
        let subscriber = ObservationSubscriber::boundary(boundary);
        let nodes: Vec<NodeId> = self
            .by_node
            .iter()
            .filter_map(|(&node, record)| (record.subscriber == subscriber).then_some(node))
            .collect();

        let mut stats = ObservationIndexRemoveStats::default();
        for node in nodes {
            let removed = self.remove_node(node);
            stats.removed |= removed.removed;
            stats.edges = stats.edges.saturating_add(removed.edges);
        }

        if !stats.removed
            && let Some(entries) = self.by_subscriber.get(&subscriber).cloned()
        {
            for (global, mask) in entries {
                self.apply_global_delta(subscriber, global, mask, false);
                stats.edges = stats.edges.saturating_add(1);
            }
            stats.removed = stats.edges > 0;
        }
        stats
    }

    fn apply_global_delta(
        &mut self,
        subscriber: ObservationSubscriber,
        global: TypeId,
        mask: ObservationMask,
        add: bool,
    ) {
        let counts = self
            .by_subscriber_counts
            .entry(subscriber)
            .or_default()
            .entry(global)
            .or_default();
        if add {
            counts.add_mask(mask);
        } else {
            counts.remove_mask(mask);
        }
        let next = counts.mask();
        let empty = counts.is_empty();

        if empty {
            if let Some(globals) = self.by_subscriber_counts.get_mut(&subscriber) {
                globals.remove(&global);
                if globals.is_empty() {
                    self.by_subscriber_counts.remove(&subscriber);
                }
            }
            remove_model_from_subscriber(&mut self.by_subscriber, subscriber, &global);
            remove_subscriber_from_model(&mut self.by_global, global, subscriber);
        } else {
            upsert_subscriber_model(&mut self.by_subscriber, subscriber, global, next);
            self.by_global
                .entry(global)
                .or_default()
                .insert(subscriber, next);
        }
    }
}

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn observation_subscriber_for_node(
        &mut self,
        node: NodeId,
    ) -> ObservationSubscriber {
        if self.view_cache_active()
            && let Some(root) = self.nearest_view_cache_root(node)
            && let Some(boundary) = self.ensure_view_boundary_state(root).map(|state| state.id)
        {
            return ObservationSubscriber::boundary(boundary);
        }
        ObservationSubscriber::node(node)
    }
}

fn model_observation_entries(
    observations: &[(ModelId, Invalidation)],
) -> Vec<(ModelId, ObservationMask)> {
    let mut entries: Vec<(ModelId, ObservationMask)> = Vec::with_capacity(observations.len());
    for &(model, inv) in observations {
        if let Some((_, mask)) = entries.iter_mut().find(|(m, _)| *m == model) {
            mask.add(inv);
        } else {
            let mut mask = ObservationMask::default();
            mask.add(inv);
            entries.push((model, mask));
        }
    }
    entries
}

fn global_observation_entries(
    observations: &[(TypeId, Invalidation)],
) -> Vec<(TypeId, ObservationMask)> {
    let mut entries: Vec<(TypeId, ObservationMask)> = Vec::with_capacity(observations.len());
    for &(global, inv) in observations {
        if let Some((_, mask)) = entries.iter_mut().find(|(g, _)| *g == global) {
            mask.add(inv);
        } else {
            let mut mask = ObservationMask::default();
            mask.add(inv);
            entries.push((global, mask));
        }
    }
    entries
}

fn upsert_subscriber_model<T: Copy + PartialEq>(
    by_subscriber: &mut HashMap<ObservationSubscriber, Vec<(T, ObservationMask)>>,
    subscriber: ObservationSubscriber,
    id: T,
    mask: ObservationMask,
) {
    let entries = by_subscriber.entry(subscriber).or_default();
    if let Some((_, existing)) = entries.iter_mut().find(|(existing, _)| *existing == id) {
        *existing = mask;
    } else {
        entries.push((id, mask));
    }
}

fn remove_model_from_subscriber<T: PartialEq>(
    by_subscriber: &mut HashMap<ObservationSubscriber, Vec<(T, ObservationMask)>>,
    subscriber: ObservationSubscriber,
    id: &T,
) {
    let remove_subscriber = if let Some(entries) = by_subscriber.get_mut(&subscriber) {
        entries.retain(|(entry_id, _)| entry_id != id);
        entries.is_empty()
    } else {
        false
    };
    if remove_subscriber {
        by_subscriber.remove(&subscriber);
    }
}

fn remove_subscriber_from_model<T: Eq + std::hash::Hash>(
    by_model: &mut HashMap<T, HashMap<ObservationSubscriber, ObservationMask>>,
    id: T,
    subscriber: ObservationSubscriber,
) {
    let remove_id = if let Some(subscribers) = by_model.get_mut(&id) {
        subscribers.remove(&subscriber);
        subscribers.is_empty()
    } else {
        false
    };
    if remove_id {
        by_model.remove(&id);
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

fn observation_subscriber_migration_stats<T>(
    prev: &[(T, ObservationMask)],
    next: &[(T, ObservationMask)],
) -> ObservationIndexRecordStats {
    ObservationIndexRecordStats {
        edges_added: next.len().min(u32::MAX as usize) as u32,
        edges_removed: prev.len().min(u32::MAX as usize) as u32,
        edges_mask_changed: 0,
    }
}
