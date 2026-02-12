use super::*;

#[derive(Default)]
pub(super) struct FrameArenaScratch {
    pub(super) gc_reachable_from_layers: HashSet<NodeId>,
    pub(super) gc_reachable_from_view_cache_roots: HashSet<NodeId>,
    pub(super) gc_stack: Vec<NodeId>,
    pub(super) semantics_visited: HashSet<NodeId>,
    pub(super) semantics_stack: Vec<(NodeId, Transform2D)>,

    pub(super) gc_reachable_from_layers_cap_on_take: usize,
    pub(super) gc_reachable_from_view_cache_roots_cap_on_take: usize,
    pub(super) gc_stack_cap_on_take: usize,
    pub(super) semantics_visited_cap_on_take: usize,
    pub(super) semantics_stack_cap_on_take: usize,
}

impl FrameArenaScratch {
    pub(super) fn capacity_estimate_bytes(&self) -> u64 {
        let mut bytes: u128 = 0;
        bytes = bytes.saturating_add(
            (self.gc_stack.capacity() as u128)
                .saturating_mul(std::mem::size_of::<NodeId>() as u128),
        );
        bytes = bytes.saturating_add(
            (self.semantics_stack.capacity() as u128)
                .saturating_mul(std::mem::size_of::<(NodeId, Transform2D)>() as u128),
        );
        // HashSet capacity is the number of elements it can hold without reallocating. We treat
        // it as `capacity * size_of::<NodeId>` as a lower bound.
        bytes = bytes.saturating_add(
            (self.gc_reachable_from_layers.capacity() as u128)
                .saturating_mul(std::mem::size_of::<NodeId>() as u128),
        );
        bytes = bytes.saturating_add(
            (self.gc_reachable_from_view_cache_roots.capacity() as u128)
                .saturating_mul(std::mem::size_of::<NodeId>() as u128),
        );
        bytes = bytes.saturating_add(
            (self.semantics_visited.capacity() as u128)
                .saturating_mul(std::mem::size_of::<NodeId>() as u128),
        );
        bytes.min(u64::MAX as u128) as u64
    }
}
