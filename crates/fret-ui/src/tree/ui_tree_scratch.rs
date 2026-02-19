use super::*;

impl<H: UiHost> UiTree<H> {
    pub(crate) fn queue_layout_bounds_for_element(
        &mut self,
        element: GlobalElementId,
        bounds: Rect,
    ) {
        self.scratch_bounds_records.push((element, bounds));
    }

    pub(in crate::tree) fn flush_layout_bounds_records_if_needed(&mut self, app: &mut H) {
        let Some(window) = self.window else {
            self.scratch_bounds_records.clear();
            return;
        };
        if self.scratch_bounds_records.is_empty() {
            return;
        }

        let mut records = std::mem::take(&mut self.scratch_bounds_records);
        crate::elements::with_window_state(app, window, |st| {
            for (element, bounds) in records.drain(..) {
                if st
                    .current_bounds(element)
                    .is_some_and(|existing| existing == bounds)
                {
                    continue;
                }
                st.record_bounds(element, bounds);
            }
        });
        self.scratch_bounds_records = records;
    }

    pub(crate) fn take_scratch_pending_invalidations(&mut self) -> HashMap<NodeId, u8> {
        std::mem::take(&mut self.scratch_pending_invalidations)
    }

    pub(crate) fn restore_scratch_pending_invalidations(&mut self, scratch: HashMap<NodeId, u8>) {
        self.scratch_pending_invalidations = scratch;
    }

    pub(crate) fn take_scratch_gc_reachable_from_layers(&mut self) -> HashSet<NodeId> {
        self.frame_arena.gc_reachable_from_layers_cap_on_take =
            self.frame_arena.gc_reachable_from_layers.capacity();
        std::mem::take(&mut self.frame_arena.gc_reachable_from_layers)
    }

    pub(crate) fn restore_scratch_gc_reachable_from_layers(&mut self, scratch: HashSet<NodeId>) {
        if scratch.capacity() > self.frame_arena.gc_reachable_from_layers_cap_on_take {
            self.debug_stats.frame_arena_grow_events =
                self.debug_stats.frame_arena_grow_events.saturating_add(1);
        }
        self.frame_arena.gc_reachable_from_layers = scratch;
    }

    pub(crate) fn take_scratch_gc_reachable_from_view_cache_roots(&mut self) -> HashSet<NodeId> {
        self.frame_arena
            .gc_reachable_from_view_cache_roots_cap_on_take = self
            .frame_arena
            .gc_reachable_from_view_cache_roots
            .capacity();
        std::mem::take(&mut self.frame_arena.gc_reachable_from_view_cache_roots)
    }

    pub(crate) fn restore_scratch_gc_reachable_from_view_cache_roots(
        &mut self,
        scratch: HashSet<NodeId>,
    ) {
        if scratch.capacity()
            > self
                .frame_arena
                .gc_reachable_from_view_cache_roots_cap_on_take
        {
            self.debug_stats.frame_arena_grow_events =
                self.debug_stats.frame_arena_grow_events.saturating_add(1);
        }
        self.frame_arena.gc_reachable_from_view_cache_roots = scratch;
    }

    pub(crate) fn take_scratch_gc_stack(&mut self) -> Vec<NodeId> {
        self.frame_arena.gc_stack_cap_on_take = self.frame_arena.gc_stack.capacity();
        std::mem::take(&mut self.frame_arena.gc_stack)
    }

    pub(crate) fn restore_scratch_gc_stack(&mut self, scratch: Vec<NodeId>) {
        if scratch.capacity() > self.frame_arena.gc_stack_cap_on_take {
            self.debug_stats.frame_arena_grow_events =
                self.debug_stats.frame_arena_grow_events.saturating_add(1);
        }
        self.frame_arena.gc_stack = scratch;
    }

    pub(crate) fn take_scratch_semantics_visited(&mut self) -> HashSet<NodeId> {
        self.frame_arena.semantics_visited_cap_on_take =
            self.frame_arena.semantics_visited.capacity();
        std::mem::take(&mut self.frame_arena.semantics_visited)
    }

    pub(crate) fn restore_scratch_semantics_visited(&mut self, scratch: HashSet<NodeId>) {
        if scratch.capacity() > self.frame_arena.semantics_visited_cap_on_take {
            self.debug_stats.frame_arena_grow_events =
                self.debug_stats.frame_arena_grow_events.saturating_add(1);
        }
        self.frame_arena.semantics_visited = scratch;
    }

    pub(crate) fn take_scratch_semantics_stack(&mut self) -> Vec<(NodeId, Transform2D)> {
        self.frame_arena.semantics_stack_cap_on_take = self.frame_arena.semantics_stack.capacity();
        std::mem::take(&mut self.frame_arena.semantics_stack)
    }

    pub(crate) fn restore_scratch_semantics_stack(&mut self, scratch: Vec<(NodeId, Transform2D)>) {
        if scratch.capacity() > self.frame_arena.semantics_stack_cap_on_take {
            self.debug_stats.frame_arena_grow_events =
                self.debug_stats.frame_arena_grow_events.saturating_add(1);
        }
        self.frame_arena.semantics_stack = scratch;
    }

    pub(crate) fn take_scratch_node_stack(&mut self) -> Vec<NodeId> {
        std::mem::take(&mut self.scratch_node_stack)
    }

    pub(crate) fn restore_scratch_node_stack(&mut self, scratch: Vec<NodeId>) {
        self.scratch_node_stack = scratch;
    }

    pub(crate) fn flush_deferred_cleanup(&mut self, services: &mut dyn UiServices) {
        for mut widget in self.deferred_cleanup.drain(..) {
            widget.cleanup_resources(services);
        }
    }
}
