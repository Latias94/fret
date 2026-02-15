use super::*;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(super) struct PendingInvalidation {
    pub(super) inv: Invalidation,
    pub(super) source: UiDebugInvalidationSource,
    pub(super) detail: UiDebugInvalidationDetail,
}

impl<H: UiHost> UiTree<H> {
    fn invalidation_rank(inv: Invalidation) -> u8 {
        match inv {
            Invalidation::Paint => 1,
            Invalidation::HitTestOnly => 2,
            Invalidation::Layout => 3,
            Invalidation::HitTest => 4,
        }
    }

    fn stronger_invalidation(a: Invalidation, b: Invalidation) -> Invalidation {
        if Self::invalidation_rank(a) >= Self::invalidation_rank(b) {
            a
        } else {
            b
        }
    }

    fn invalidation_source_rank(source: UiDebugInvalidationSource) -> u8 {
        match source {
            UiDebugInvalidationSource::ModelChange => 6,
            UiDebugInvalidationSource::GlobalChange => 5,
            UiDebugInvalidationSource::Hover => 4,
            UiDebugInvalidationSource::Focus => 3,
            UiDebugInvalidationSource::Notify => 2,
            UiDebugInvalidationSource::Other => 1,
        }
    }

    fn stronger_invalidation_source(
        a: UiDebugInvalidationSource,
        b: UiDebugInvalidationSource,
    ) -> UiDebugInvalidationSource {
        if Self::invalidation_source_rank(a) >= Self::invalidation_source_rank(b) {
            a
        } else {
            b
        }
    }

    pub(super) fn pending_invalidation_merge(
        pending: &mut HashMap<NodeId, PendingInvalidation>,
        node: NodeId,
        inv: Invalidation,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        pending
            .entry(node)
            .and_modify(|cur| {
                cur.inv = Self::stronger_invalidation(cur.inv, inv);
                cur.source = Self::stronger_invalidation_source(cur.source, source);
                cur.detail = if cur.source == UiDebugInvalidationSource::Other {
                    match (cur.detail, detail) {
                        (UiDebugInvalidationDetail::Unknown, d) => d,
                        (d, UiDebugInvalidationDetail::Unknown) => d,
                        (d, _) => d,
                    }
                } else {
                    UiDebugInvalidationDetail::from_source(cur.source)
                };
            })
            .or_insert(PendingInvalidation {
                inv,
                source,
                detail,
            });
    }

    fn node_depth_for_invalidation_order(&self, node: NodeId) -> u32 {
        let mut depth: u32 = 0;
        let mut current: Option<NodeId> = Some(node);
        while let Some(id) = current {
            let Some(n) = self.nodes.get(id) else {
                break;
            };
            depth = depth.saturating_add(1);
            current = n.parent;
        }
        depth
    }

    pub(super) fn apply_pending_invalidations(
        &mut self,
        pending: HashMap<NodeId, PendingInvalidation>,
        visited: &mut impl InvalidationVisited,
    ) {
        if pending.is_empty() {
            return;
        }

        let mut entries: Vec<(NodeId, PendingInvalidation)> = pending.into_iter().collect();
        entries.sort_by_key(|(node, _)| {
            std::cmp::Reverse(self.node_depth_for_invalidation_order(*node))
        });
        for (node, pending) in entries {
            self.mark_invalidation_dedup_with_detail(
                node,
                pending.inv,
                visited,
                pending.source,
                pending.detail,
            );
        }
    }
}
