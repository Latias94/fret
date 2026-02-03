use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use fret_core::{AppWindowId, NodeId, Rect};
use fret_runtime::{FrameId, ModelId, TimerToken};
#[cfg(feature = "diagnostics")]
use slotmap::Key as _;
#[cfg(feature = "diagnostics")]
use std::sync::Arc as StdArc;

use crate::widget::Invalidation;

use super::GlobalElementId;

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone)]
pub struct WindowElementDiagnosticsSnapshot {
    pub focused_element: Option<GlobalElementId>,
    pub focused_element_node: Option<NodeId>,
    pub focused_element_bounds: Option<Rect>,
    pub focused_element_visual_bounds: Option<Rect>,
    pub active_text_selection: Option<(GlobalElementId, GlobalElementId)>,
    pub hovered_pressable: Option<GlobalElementId>,
    pub hovered_pressable_node: Option<NodeId>,
    pub hovered_pressable_bounds: Option<Rect>,
    pub hovered_pressable_visual_bounds: Option<Rect>,
    pub pressed_pressable: Option<GlobalElementId>,
    pub pressed_pressable_node: Option<NodeId>,
    pub pressed_pressable_bounds: Option<Rect>,
    pub pressed_pressable_visual_bounds: Option<Rect>,
    pub hovered_hover_region: Option<GlobalElementId>,
    pub wants_continuous_frames: bool,
    pub observed_models: Vec<(GlobalElementId, Vec<(u64, Invalidation)>)>,
    pub observed_globals: Vec<(GlobalElementId, Vec<(String, Invalidation)>)>,
    pub view_cache_reuse_roots: Vec<GlobalElementId>,
    pub view_cache_reuse_root_element_counts: Vec<(GlobalElementId, u32)>,
    pub view_cache_reuse_root_element_samples: Vec<ViewCacheReuseRootElementsSample>,
    pub node_entry_root_overwrites: Vec<NodeEntryRootOverwrite>,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone)]
pub struct ViewCacheReuseRootElementsSample {
    pub root: GlobalElementId,
    pub node: Option<NodeId>,
    pub elements_len: u32,
    pub elements_head: Vec<GlobalElementId>,
    pub elements_tail: Vec<GlobalElementId>,
}

#[derive(Default)]
pub struct ElementRuntime {
    windows: HashMap<AppWindowId, WindowElementState>,
    gc_lag_frames: u64,
}

impl ElementRuntime {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            gc_lag_frames: 2,
        }
    }

    pub fn gc_lag_frames(&self) -> u64 {
        self.gc_lag_frames
    }

    pub fn set_gc_lag_frames(&mut self, frames: u64) {
        self.gc_lag_frames = frames;
    }

    pub fn for_window_mut(&mut self, window: AppWindowId) -> &mut WindowElementState {
        self.windows.entry(window).or_default()
    }

    pub(crate) fn for_window(&self, window: AppWindowId) -> Option<&WindowElementState> {
        self.windows.get(&window)
    }

    pub fn prepare_window_for_frame(&mut self, window: AppWindowId, frame_id: FrameId) {
        let lag = self.gc_lag_frames;
        self.for_window_mut(window).prepare_for_frame(frame_id, lag);
    }

    #[cfg(feature = "diagnostics")]
    pub fn diagnostics_snapshot(
        &self,
        window: AppWindowId,
    ) -> Option<WindowElementDiagnosticsSnapshot> {
        let state = self.windows.get(&window)?;
        Some(state.diagnostics_snapshot())
    }

    #[cfg(feature = "diagnostics")]
    pub fn element_for_node(&self, window: AppWindowId, node: NodeId) -> Option<GlobalElementId> {
        let state = self.windows.get(&window)?;
        state.element_for_node(node)
    }

    #[cfg(feature = "diagnostics")]
    pub fn node_for_element(
        &self,
        window: AppWindowId,
        element: GlobalElementId,
    ) -> Option<NodeId> {
        let state = self.windows.get(&window)?;
        state.node_entry(element).map(|e| e.node)
    }

    #[cfg(feature = "diagnostics")]
    pub fn debug_path_for_element(
        &self,
        window: AppWindowId,
        element: GlobalElementId,
    ) -> Option<String> {
        let state = self.windows.get(&window)?;
        state.debug_path_for_element(element)
    }
}

#[derive(Default)]
pub struct WindowElementState {
    pub(super) rendered_state: HashMap<(GlobalElementId, TypeId), Box<dyn Any>>,
    pub(super) next_state: HashMap<(GlobalElementId, TypeId), Box<dyn Any>>,
    pub(super) lag_state: Vec<HashMap<(GlobalElementId, TypeId), Box<dyn Any>>>,
    pub(super) view_cache_state_keys_rendered:
        HashMap<GlobalElementId, Vec<(GlobalElementId, TypeId)>>,
    pub(super) view_cache_state_keys_next: HashMap<GlobalElementId, Vec<(GlobalElementId, TypeId)>>,
    view_cache_keys_rendered: HashMap<GlobalElementId, u64>,
    view_cache_keys_next: HashMap<GlobalElementId, u64>,
    view_cache_key_mismatch_roots: HashSet<GlobalElementId>,
    pub(super) view_cache_elements_rendered: HashMap<GlobalElementId, Vec<GlobalElementId>>,
    pub(super) view_cache_elements_next: HashMap<GlobalElementId, Vec<GlobalElementId>>,
    pub(super) view_cache_reuse_roots: HashSet<GlobalElementId>,
    view_cache_last_reused_frame: HashMap<GlobalElementId, FrameId>,
    view_cache_transitioned_reuse_roots: HashSet<GlobalElementId>,
    view_cache_stack: Vec<GlobalElementId>,
    raf_notify_roots: HashSet<GlobalElementId>,
    pub(super) pending_retained_virtual_list_reconciles: HashSet<GlobalElementId>,
    prepared_frame: FrameId,
    #[cfg(any(test, feature = "diagnostics"))]
    strict_ownership: bool,
    pub(super) prev_unkeyed_fingerprints: HashMap<u64, Vec<u64>>,
    pub(super) cur_unkeyed_fingerprints: HashMap<u64, Vec<u64>>,
    pub(super) observed_models_rendered: HashMap<GlobalElementId, Vec<(ModelId, Invalidation)>>,
    pub(super) observed_models_next: HashMap<GlobalElementId, Vec<(ModelId, Invalidation)>>,
    pub(super) observed_globals_rendered: HashMap<GlobalElementId, Vec<(TypeId, Invalidation)>>,
    pub(super) observed_globals_next: HashMap<GlobalElementId, Vec<(TypeId, Invalidation)>>,
    pub(super) timer_targets: HashMap<TimerToken, GlobalElementId>,
    scratch_view_cache_keep_alive_elements: HashSet<GlobalElementId>,
    scratch_view_cache_keep_alive_visited_roots: HashSet<GlobalElementId>,
    scratch_view_cache_keep_alive_stack: Vec<GlobalElementId>,
    nodes: HashMap<GlobalElementId, NodeEntry>,
    root_bounds: HashMap<GlobalElementId, Rect>,
    prev_bounds: HashMap<GlobalElementId, Rect>,
    cur_bounds: HashMap<GlobalElementId, Rect>,
    prev_visual_bounds: HashMap<GlobalElementId, Rect>,
    cur_visual_bounds: HashMap<GlobalElementId, Rect>,
    pub(super) focused_element: Option<GlobalElementId>,
    pub(super) active_text_selection: Option<ActiveTextSelection>,
    pub(super) hovered_pressable: Option<GlobalElementId>,
    pub(super) pressed_pressable: Option<GlobalElementId>,
    pub(super) hovered_hover_region: Option<GlobalElementId>,
    continuous_frames: Arc<AtomicUsize>,
    #[cfg(feature = "diagnostics")]
    debug_identity: DebugIdentityRegistry,
    #[cfg(feature = "diagnostics")]
    debug_node_entry_root_overwrites: Vec<NodeEntryRootOverwrite>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActiveTextSelection {
    pub root: GlobalElementId,
    pub element: GlobalElementId,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NodeEntry {
    pub node: NodeId,
    pub last_seen_frame: FrameId,
    pub root: GlobalElementId,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct NodeEntryRootOverwrite {
    pub frame_id: FrameId,
    pub element: GlobalElementId,
    pub old_root: GlobalElementId,
    pub new_root: GlobalElementId,
    pub old_node: NodeId,
    pub new_node: NodeId,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

impl WindowElementState {
    pub(crate) fn take_scratch_view_cache_keep_alive_elements(
        &mut self,
    ) -> HashSet<GlobalElementId> {
        std::mem::take(&mut self.scratch_view_cache_keep_alive_elements)
    }

    pub(crate) fn restore_scratch_view_cache_keep_alive_elements(
        &mut self,
        scratch: HashSet<GlobalElementId>,
    ) {
        self.scratch_view_cache_keep_alive_elements = scratch;
    }

    pub(crate) fn take_scratch_view_cache_keep_alive_visited_roots(
        &mut self,
    ) -> HashSet<GlobalElementId> {
        std::mem::take(&mut self.scratch_view_cache_keep_alive_visited_roots)
    }

    pub(crate) fn restore_scratch_view_cache_keep_alive_visited_roots(
        &mut self,
        scratch: HashSet<GlobalElementId>,
    ) {
        self.scratch_view_cache_keep_alive_visited_roots = scratch;
    }

    pub(crate) fn take_scratch_view_cache_keep_alive_stack(&mut self) -> Vec<GlobalElementId> {
        std::mem::take(&mut self.scratch_view_cache_keep_alive_stack)
    }

    pub(crate) fn restore_scratch_view_cache_keep_alive_stack(
        &mut self,
        scratch: Vec<GlobalElementId>,
    ) {
        self.scratch_view_cache_keep_alive_stack = scratch;
    }

    #[cfg(any(test, feature = "diagnostics"))]
    #[allow(dead_code)]
    pub(crate) fn set_strict_ownership(&mut self, strict: bool) {
        self.strict_ownership = strict;
    }

    fn prepare_for_frame(&mut self, frame_id: FrameId, lag_frames: u64) {
        if self.prepared_frame == frame_id {
            return;
        }
        self.prepared_frame = frame_id;

        self.advance_element_state_buffers(lag_frames);

        self.raf_notify_roots.clear();
        self.view_cache_key_mismatch_roots.clear();

        std::mem::swap(
            &mut self.view_cache_keys_rendered,
            &mut self.view_cache_keys_next,
        );
        self.view_cache_keys_next.clear();

        std::mem::swap(
            &mut self.view_cache_state_keys_rendered,
            &mut self.view_cache_state_keys_next,
        );
        self.view_cache_state_keys_next.clear();

        std::mem::swap(
            &mut self.view_cache_elements_rendered,
            &mut self.view_cache_elements_next,
        );
        self.view_cache_elements_next.clear();

        self.view_cache_reuse_roots.clear();
        self.view_cache_transitioned_reuse_roots.clear();
        self.view_cache_stack.clear();

        std::mem::swap(
            &mut self.prev_unkeyed_fingerprints,
            &mut self.cur_unkeyed_fingerprints,
        );
        self.cur_unkeyed_fingerprints.clear();

        std::mem::swap(
            &mut self.observed_models_rendered,
            &mut self.observed_models_next,
        );
        self.observed_models_next.clear();
        std::mem::swap(
            &mut self.observed_globals_rendered,
            &mut self.observed_globals_next,
        );
        self.observed_globals_next.clear();

        // Keep cross-frame geometry queries stable even when layout/paint skips subtrees due to
        // caching:
        // - `prev_*` stores the last committed snapshot (used by cross-frame queries).
        // - `cur_*` stores only the current frame's recorded deltas.
        //
        // Committing deltas at frame boundaries avoids cloning full maps on cache-hit frames.
        if !self.cur_bounds.is_empty() {
            self.prev_bounds.reserve(self.cur_bounds.len());
            self.prev_bounds.extend(self.cur_bounds.drain());
        }
        self.cur_bounds.clear();

        if !self.cur_visual_bounds.is_empty() {
            self.prev_visual_bounds
                .reserve(self.cur_visual_bounds.len());
            self.prev_visual_bounds
                .extend(self.cur_visual_bounds.drain());
        }
        self.cur_visual_bounds.clear();

        self.focused_element = None;

        #[cfg(feature = "diagnostics")]
        {
            let cutoff = frame_id.0.saturating_sub(lag_frames);
            self.debug_identity
                .entries
                .retain(|_, v| v.last_seen_frame.0 >= cutoff);
            self.debug_node_entry_root_overwrites
                .retain(|r| r.frame_id.0 >= cutoff);
        }
    }

    fn advance_element_state_buffers(&mut self, lag_frames: u64) {
        if lag_frames == 0 {
            self.lag_state.clear();
        } else {
            self.lag_state
                .push(std::mem::take(&mut self.rendered_state));
            let max = lag_frames as usize;
            if self.lag_state.len() > max {
                let drain = self.lag_state.len() - max;
                self.lag_state.drain(0..drain);
            }
        }

        self.rendered_state = std::mem::take(&mut self.next_state);
        self.next_state.clear();
    }

    pub(super) fn state_any_ref(&self, key: &(GlobalElementId, TypeId)) -> Option<&dyn Any> {
        if let Some(v) = self.next_state.get(key) {
            return Some(&**v);
        }
        if let Some(v) = self.rendered_state.get(key) {
            return Some(&**v);
        }
        for map in self.lag_state.iter().rev() {
            if let Some(v) = map.get(key) {
                return Some(&**v);
            }
        }
        None
    }

    pub(crate) fn has_state<S: Any>(&self, element: GlobalElementId) -> bool {
        let key = (element, TypeId::of::<S>());
        self.state_any_ref(&key).is_some()
    }

    pub(super) fn take_state_box(
        &mut self,
        key: &(GlobalElementId, TypeId),
    ) -> Option<Box<dyn Any>> {
        if let Some(v) = self.next_state.remove(key) {
            return Some(v);
        }
        if let Some(v) = self.rendered_state.remove(key) {
            return Some(v);
        }
        for map in self.lag_state.iter_mut().rev() {
            if let Some(v) = map.remove(key) {
                return Some(v);
            }
        }
        None
    }

    pub(super) fn insert_state_box(&mut self, key: (GlobalElementId, TypeId), value: Box<dyn Any>) {
        self.next_state.insert(key, value);
    }

    pub(crate) fn with_state_mut<S: Any, R>(
        &mut self,
        element: GlobalElementId,
        init: impl FnOnce() -> S,
        f: impl FnOnce(&mut S) -> R,
    ) -> R {
        let key = (element, TypeId::of::<S>());
        self.record_state_key_access(key);
        let mut value = self
            .take_state_box(&key)
            .unwrap_or_else(|| Box::new(init()));
        let out = {
            let state = value
                .downcast_mut::<S>()
                .expect("element state type mismatch");
            f(state)
        };
        self.insert_state_box(key, value);
        out
    }

    pub(crate) fn try_with_state_mut<S: Any, R>(
        &mut self,
        element: GlobalElementId,
        f: impl FnOnce(&mut S) -> R,
    ) -> Option<R> {
        let key = (element, TypeId::of::<S>());
        self.record_state_key_access(key);
        let mut value = self.take_state_box(&key)?;
        let out = {
            let state = value
                .downcast_mut::<S>()
                .expect("element state type mismatch");
            f(state)
        };
        self.insert_state_box(key, value);
        Some(out)
    }

    pub(crate) fn mark_retained_virtual_list_needs_reconcile(&mut self, element: GlobalElementId) {
        self.pending_retained_virtual_list_reconciles
            .insert(element);
    }

    pub(crate) fn take_retained_virtual_list_reconciles(&mut self) -> Vec<GlobalElementId> {
        self.pending_retained_virtual_list_reconciles
            .drain()
            .collect()
    }

    pub(super) fn record_state_key_access(&mut self, key: (GlobalElementId, TypeId)) {
        if self.view_cache_stack.is_empty() {
            return;
        };
        // Nested view-cache correctness: when entering a child view-cache scope, parent cache
        // roots still need to keep the child's state alive if the parent reuses without
        // re-rendering that subtree.
        for &root in &self.view_cache_stack {
            self.view_cache_state_keys_next
                .entry(root)
                .or_default()
                .push(key);
        }
    }

    pub(super) fn begin_view_cache_scope(&mut self, root: GlobalElementId) {
        self.view_cache_stack.push(root);
        self.view_cache_state_keys_next.remove(&root);
        self.view_cache_elements_next.remove(&root);
    }

    pub(super) fn end_view_cache_scope(&mut self, root: GlobalElementId) {
        let popped = self.view_cache_stack.pop();
        debug_assert_eq!(popped, Some(root));
        if let Some(keys) = self.view_cache_state_keys_next.get_mut(&root) {
            let mut seen: HashSet<(GlobalElementId, TypeId)> = HashSet::with_capacity(keys.len());
            keys.retain(|&key| seen.insert(key));
        }
        if let Some(elements) = self.view_cache_elements_next.get_mut(&root) {
            let mut seen: HashSet<GlobalElementId> = HashSet::with_capacity(elements.len());
            elements.retain(|&id| seen.insert(id));
        }
    }

    pub(crate) fn touch_observed_models_for_element_if_recorded(
        &mut self,
        element: GlobalElementId,
    ) {
        if self.observed_models_next.contains_key(&element) {
            return;
        }
        let Some(list) = self.observed_models_rendered.get(&element) else {
            return;
        };
        self.observed_models_next.insert(element, list.clone());
    }

    pub(crate) fn touch_observed_globals_for_element_if_recorded(
        &mut self,
        element: GlobalElementId,
    ) {
        if self.observed_globals_next.contains_key(&element) {
            return;
        }
        let Some(list) = self.observed_globals_rendered.get(&element) else {
            return;
        };
        self.observed_globals_next.insert(element, list.clone());
    }

    pub(crate) fn mark_view_cache_reuse_root(&mut self, root: GlobalElementId) {
        self.view_cache_reuse_roots.insert(root);
    }

    /// Returns `true` if this cache root was *not* reused in the immediately-previous frame.
    ///
    /// This is used to refresh liveness/recording when a view-cache root transitions into reuse,
    /// avoiding GC sweeping stale-but-live subtrees on the first cache-hit frame.
    pub(crate) fn record_view_cache_reuse_frame(
        &mut self,
        root: GlobalElementId,
        frame_id: FrameId,
    ) -> bool {
        let transitioned = self
            .view_cache_last_reused_frame
            .get(&root)
            .is_none_or(|last| last.0.saturating_add(1) < frame_id.0);
        self.view_cache_last_reused_frame.insert(root, frame_id);
        if transitioned {
            self.view_cache_transitioned_reuse_roots.insert(root);
        }
        transitioned
    }

    pub(crate) fn view_cache_transitioned_reuse_roots(
        &self,
    ) -> impl Iterator<Item = GlobalElementId> + '_ {
        self.view_cache_transitioned_reuse_roots.iter().copied()
    }

    pub(crate) fn should_reuse_view_cache_root(&self, root: GlobalElementId) -> bool {
        self.view_cache_reuse_roots.contains(&root)
    }

    pub(crate) fn view_cache_reuse_roots(&self) -> impl Iterator<Item = GlobalElementId> + '_ {
        self.view_cache_reuse_roots.iter().copied()
    }

    pub(crate) fn current_view_cache_root(&self) -> Option<GlobalElementId> {
        self.view_cache_stack.last().copied()
    }

    pub(crate) fn request_notify_for_animation_frame(&mut self, root: GlobalElementId) {
        self.raf_notify_roots.insert(root);
    }

    pub(crate) fn take_notify_for_animation_frame(&mut self) -> Vec<GlobalElementId> {
        if self.raf_notify_roots.is_empty() {
            return Vec::new();
        }
        let out: Vec<GlobalElementId> = self.raf_notify_roots.iter().copied().collect();
        self.raf_notify_roots.clear();
        out
    }

    pub(crate) fn view_cache_key_matches_and_touch(
        &mut self,
        root: GlobalElementId,
        key: u64,
    ) -> bool {
        if self.view_cache_keys_next.contains_key(&root) {
            return self.view_cache_keys_next.get(&root).copied() == Some(key);
        }
        let Some(prev) = self.view_cache_keys_rendered.get(&root).copied() else {
            return false;
        };
        if prev != key {
            return false;
        }
        self.view_cache_keys_next.insert(root, key);
        true
    }

    pub(crate) fn set_view_cache_key(&mut self, root: GlobalElementId, key: u64) {
        self.view_cache_keys_next.insert(root, key);
    }

    pub(crate) fn record_view_cache_key_mismatch(&mut self, root: GlobalElementId) {
        self.view_cache_key_mismatch_roots.insert(root);
    }

    pub(crate) fn view_cache_key_mismatch(&self, root: GlobalElementId) -> bool {
        self.view_cache_key_mismatch_roots.contains(&root)
    }

    pub(super) fn touch_state_key(&mut self, key: (GlobalElementId, TypeId)) {
        let Some(value) = self.take_state_box(&key) else {
            return;
        };
        self.insert_state_box(key, value);
    }

    pub(crate) fn touch_view_cache_state_keys_if_recorded(&mut self, root: GlobalElementId) {
        let Some(keys) = self.view_cache_state_keys_rendered.get(&root).cloned() else {
            return;
        };
        for &key in &keys {
            self.touch_state_key(key);
        }
        self.view_cache_state_keys_next.insert(root, keys);
    }

    pub(crate) fn record_view_cache_subtree_elements(
        &mut self,
        root: GlobalElementId,
        elements: Vec<GlobalElementId>,
    ) {
        self.view_cache_elements_next.insert(root, elements);
    }

    pub(crate) fn forget_view_cache_subtree_elements(&mut self, root: GlobalElementId) {
        self.view_cache_elements_rendered.remove(&root);
        self.view_cache_elements_next.remove(&root);
    }

    pub(crate) fn touch_view_cache_subtree_elements_if_recorded(
        &mut self,
        root: GlobalElementId,
        frame_id: FrameId,
        root_id: GlobalElementId,
    ) -> bool {
        if self.view_cache_elements_next.contains_key(&root) {
            return true;
        }

        let Some(elements) = self.view_cache_elements_rendered.get(&root).cloned() else {
            return false;
        };

        self.view_cache_elements_next.insert(root, elements.clone());

        let mut missing_node_entries: u32 = 0;
        for element in elements {
            let Some(entry) = self.nodes.get_mut(&element) else {
                missing_node_entries = missing_node_entries.saturating_add(1);
                continue;
            };
            entry.last_seen_frame = frame_id;
            // Touching a retained subtree must not reassign cross-root ownership (ADR 0191).
            // If the element is already owned by a different root, keep the original owner and
            // rely on diagnostics to flag the mismatch rather than "repairing" it implicitly.
            if entry.root == root_id {
                // Fast path: expected owner.
            }

            #[cfg(feature = "diagnostics")]
            self.touch_debug_identity_for_element(frame_id, element);
        }

        // If the recorded list references elements that no longer have node entries, treat the
        // list as potentially incomplete/stale so the caller can fall back to a retained-subtree
        // walk and re-record the membership deterministically.
        missing_node_entries == 0
    }

    pub(crate) fn view_cache_elements_for_root(
        &self,
        root: GlobalElementId,
    ) -> Option<&[GlobalElementId]> {
        if let Some(v) = self.view_cache_elements_next.get(&root) {
            return Some(v.as_slice());
        }
        self.view_cache_elements_rendered
            .get(&root)
            .map(|v| v.as_slice())
    }

    pub(crate) fn active_text_selection(&self) -> Option<ActiveTextSelection> {
        self.active_text_selection
    }

    pub(crate) fn set_active_text_selection(&mut self, selection: Option<ActiveTextSelection>) {
        self.active_text_selection = selection;
    }

    pub(crate) fn node_entry(&self, id: GlobalElementId) -> Option<NodeEntry> {
        self.nodes.get(&id).copied()
    }

    pub(crate) fn for_each_observed_model_for_invalidation(
        &self,
        frame_id: FrameId,
        mut f: impl FnMut(GlobalElementId, &Vec<(ModelId, Invalidation)>),
    ) {
        if self.prepared_frame != frame_id {
            for (&element, observations) in &self.observed_models_next {
                f(element, observations);
            }
            return;
        }

        for (&element, observations) in &self.observed_models_rendered {
            if self.observed_models_next.contains_key(&element) {
                continue;
            }
            f(element, observations);
        }
        for (&element, observations) in &self.observed_models_next {
            f(element, observations);
        }
    }

    pub(crate) fn for_each_observed_global_for_invalidation(
        &self,
        frame_id: FrameId,
        mut f: impl FnMut(GlobalElementId, &Vec<(TypeId, Invalidation)>),
    ) {
        if self.prepared_frame != frame_id {
            for (&element, observations) in &self.observed_globals_next {
                f(element, observations);
            }
            return;
        }

        for (&element, observations) in &self.observed_globals_rendered {
            if self.observed_globals_next.contains_key(&element) {
                continue;
            }
            f(element, observations);
        }
        for (&element, observations) in &self.observed_globals_next {
            f(element, observations);
        }
    }

    pub(crate) fn element_for_node(&self, node: NodeId) -> Option<GlobalElementId> {
        self.nodes
            .iter()
            .find_map(|(&element, entry)| (entry.node == node).then_some(element))
    }

    #[track_caller]
    pub(crate) fn set_node_entry(&mut self, id: GlobalElementId, entry: NodeEntry) {
        #[cfg(feature = "diagnostics")]
        if let Some(prev) = self.nodes.get(&id) {
            if prev.root != entry.root {
                let location = std::panic::Location::caller();
                self.debug_node_entry_root_overwrites
                    .push(NodeEntryRootOverwrite {
                        frame_id: entry.last_seen_frame,
                        element: id,
                        old_root: prev.root,
                        new_root: entry.root,
                        old_node: prev.node,
                        new_node: entry.node,
                        file: location.file(),
                        line: location.line(),
                        column: location.column(),
                    });
                const MAX_RECORDS: usize = 256;
                if self.debug_node_entry_root_overwrites.len() > MAX_RECORDS {
                    let drain = self.debug_node_entry_root_overwrites.len() - MAX_RECORDS;
                    self.debug_node_entry_root_overwrites.drain(0..drain);
                }
            }
        }
        #[cfg(any(test, feature = "diagnostics"))]
        if self.strict_ownership {
            if let Some(prev) = self.nodes.get(&id) {
                assert_eq!(
                    prev.root, entry.root,
                    "ownership root overwrite detected for element {id:?}: old_root={:?} new_root={:?} (cross-root reparenting must be explicit; see ADR 0191)",
                    prev.root, entry.root
                );
            }
        }
        self.nodes.insert(id, entry);
    }

    pub(crate) fn retain_nodes(&mut self, f: impl FnMut(&GlobalElementId, &mut NodeEntry) -> bool) {
        self.nodes.retain(f);
        if let Some(selection) = self.active_text_selection
            && !self.nodes.contains_key(&selection.element)
        {
            self.active_text_selection = None;
        }
    }

    pub(crate) fn set_root_bounds(&mut self, root: GlobalElementId, bounds: Rect) {
        self.root_bounds.insert(root, bounds);
    }

    pub(crate) fn root_bounds(&self, root: GlobalElementId) -> Option<Rect> {
        self.root_bounds.get(&root).copied()
    }

    pub(crate) fn record_bounds(&mut self, element: GlobalElementId, bounds: Rect) {
        self.cur_bounds.insert(element, bounds);
    }

    pub(crate) fn element_nodes(&self) -> Vec<(GlobalElementId, NodeId)> {
        self.nodes
            .iter()
            .map(|(&element, entry)| (element, entry.node))
            .collect()
    }

    pub(crate) fn last_bounds(&self, element: GlobalElementId) -> Option<Rect> {
        self.prev_bounds.get(&element).copied()
    }

    pub(crate) fn current_bounds(&self, element: GlobalElementId) -> Option<Rect> {
        self.cur_bounds
            .get(&element)
            .copied()
            .or_else(|| self.prev_bounds.get(&element).copied())
    }

    pub(crate) fn record_visual_bounds(&mut self, element: GlobalElementId, bounds: Rect) {
        self.cur_visual_bounds.insert(element, bounds);
    }

    pub(crate) fn last_visual_bounds(&self, element: GlobalElementId) -> Option<Rect> {
        self.prev_visual_bounds.get(&element).copied()
    }

    pub(crate) fn current_visual_bounds(&self, element: GlobalElementId) -> Option<Rect> {
        self.cur_visual_bounds
            .get(&element)
            .copied()
            .or_else(|| self.prev_visual_bounds.get(&element).copied())
    }

    pub(crate) fn wants_continuous_frames(&self) -> bool {
        self.continuous_frames.load(Ordering::Relaxed) > 0
    }

    pub(crate) fn begin_continuous_frames(&self) -> ContinuousFrames {
        self.continuous_frames.fetch_add(1, Ordering::Relaxed);
        ContinuousFrames {
            leases: self.continuous_frames.clone(),
        }
    }

    #[cfg(feature = "diagnostics")]
    fn diagnostics_snapshot(&self) -> WindowElementDiagnosticsSnapshot {
        let bounds_for = |element: Option<GlobalElementId>| {
            element.and_then(|id| {
                self.prev_bounds
                    .get(&id)
                    .copied()
                    .or_else(|| self.cur_bounds.get(&id).copied())
            })
        };
        let visual_bounds_for = |element: Option<GlobalElementId>| {
            element.and_then(|id| {
                self.prev_visual_bounds
                    .get(&id)
                    .copied()
                    .or_else(|| self.cur_visual_bounds.get(&id).copied())
            })
        };
        let node_for = |element: Option<GlobalElementId>| {
            element.and_then(|id| self.node_entry(id).map(|e| e.node))
        };

        let mut view_cache_reuse_roots: Vec<GlobalElementId> =
            self.view_cache_reuse_roots.iter().copied().collect();
        view_cache_reuse_roots.sort_by_key(|id| id.0);

        let view_cache_reuse_root_element_counts: Vec<(GlobalElementId, u32)> =
            view_cache_reuse_roots
                .iter()
                .map(|root| {
                    let count = self
                        .view_cache_elements_rendered
                        .get(root)
                        .map(|v| v.len())
                        .unwrap_or(0);
                    (*root, count.min(u32::MAX as usize) as u32)
                })
                .collect();

        const ELEMENT_SAMPLE: usize = 16;
        let view_cache_reuse_root_element_samples: Vec<ViewCacheReuseRootElementsSample> =
            view_cache_reuse_roots
                .iter()
                .map(|&root| {
                    let elements = self.view_cache_elements_for_root(root).unwrap_or(&[]);
                    let elements_len = elements.len().min(u32::MAX as usize) as u32;
                    let elements_head: Vec<GlobalElementId> =
                        elements.iter().take(ELEMENT_SAMPLE).copied().collect();
                    let elements_tail: Vec<GlobalElementId> = if elements.len() > ELEMENT_SAMPLE {
                        elements
                            .iter()
                            .skip(elements.len().saturating_sub(ELEMENT_SAMPLE))
                            .copied()
                            .collect()
                    } else {
                        Vec::new()
                    };

                    ViewCacheReuseRootElementsSample {
                        root,
                        node: self.node_entry(root).map(|e| e.node),
                        elements_len,
                        elements_head,
                        elements_tail,
                    }
                })
                .collect();

        WindowElementDiagnosticsSnapshot {
            focused_element: self.focused_element,
            focused_element_node: node_for(self.focused_element),
            focused_element_bounds: bounds_for(self.focused_element),
            focused_element_visual_bounds: visual_bounds_for(self.focused_element),
            active_text_selection: self
                .active_text_selection
                .map(|sel| (sel.root, sel.element)),
            hovered_pressable: self.hovered_pressable,
            hovered_pressable_node: node_for(self.hovered_pressable),
            hovered_pressable_bounds: bounds_for(self.hovered_pressable),
            hovered_pressable_visual_bounds: visual_bounds_for(self.hovered_pressable),
            pressed_pressable: self.pressed_pressable,
            pressed_pressable_node: node_for(self.pressed_pressable),
            pressed_pressable_bounds: bounds_for(self.pressed_pressable),
            pressed_pressable_visual_bounds: visual_bounds_for(self.pressed_pressable),
            hovered_hover_region: self.hovered_hover_region,
            wants_continuous_frames: self.wants_continuous_frames(),
            observed_models: self
                .observed_models_next
                .iter()
                .map(|(element, list)| {
                    (
                        *element,
                        list.iter()
                            .map(|(model, inv)| (model.data().as_ffi(), *inv))
                            .collect(),
                    )
                })
                .collect(),
            observed_globals: self
                .observed_globals_next
                .iter()
                .map(|(element, list)| {
                    (
                        *element,
                        list.iter()
                            .map(|(ty, inv)| (format!("{ty:?}"), *inv))
                            .collect(),
                    )
                })
                .collect(),
            view_cache_reuse_roots,
            view_cache_reuse_root_element_counts,
            view_cache_reuse_root_element_samples,
            node_entry_root_overwrites: self.debug_node_entry_root_overwrites.clone(),
        }
    }

    #[cfg(feature = "diagnostics")]
    pub(crate) fn record_debug_root(
        &mut self,
        frame_id: FrameId,
        root: GlobalElementId,
        name: &str,
    ) {
        self.debug_identity.entries.insert(
            root,
            DebugIdentityEntry {
                parent: None,
                segment: DebugIdentitySegment::Root {
                    name: StdArc::<str>::from(name),
                },
                last_seen_frame: frame_id,
            },
        );
    }

    #[cfg(feature = "diagnostics")]
    pub(crate) fn record_debug_child(
        &mut self,
        frame_id: FrameId,
        parent: GlobalElementId,
        child: GlobalElementId,
        file: &'static str,
        line: u32,
        column: u32,
        key_hash: Option<u64>,
        name: Option<&str>,
        slot: u64,
    ) {
        self.debug_identity.entries.insert(
            child,
            DebugIdentityEntry {
                parent: Some(parent),
                segment: DebugIdentitySegment::Child {
                    file,
                    line,
                    column,
                    key_hash,
                    name: name.map(StdArc::<str>::from),
                    slot,
                },
                last_seen_frame: frame_id,
            },
        );
    }

    #[cfg(feature = "diagnostics")]
    pub(crate) fn touch_debug_identity_for_element(
        &mut self,
        frame_id: FrameId,
        element: GlobalElementId,
    ) {
        if let Some(entry) = self.debug_identity.entries.get_mut(&element) {
            entry.last_seen_frame = frame_id;
        }
    }

    #[cfg(feature = "diagnostics")]
    pub(crate) fn debug_path_for_element(&self, element: GlobalElementId) -> Option<String> {
        let mut segments: Vec<String> = Vec::new();
        let mut cur = element;
        let mut guard = 0usize;
        while guard < 256 {
            guard += 1;
            let entry = self.debug_identity.entries.get(&cur)?;
            segments.push(entry.segment.format());
            if let Some(parent) = entry.parent {
                cur = parent;
                continue;
            }
            break;
        }
        segments.reverse();
        Some(format!("{} ({:#x})", segments.join("."), element.0))
    }
}

#[must_use]
pub struct ContinuousFrames {
    leases: Arc<AtomicUsize>,
}

impl Drop for ContinuousFrames {
    fn drop(&mut self) {
        let _ = self
            .leases
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_sub(1));
    }
}

#[cfg(feature = "diagnostics")]
#[derive(Default)]
struct DebugIdentityRegistry {
    entries: HashMap<GlobalElementId, DebugIdentityEntry>,
}

#[cfg(feature = "diagnostics")]
struct DebugIdentityEntry {
    parent: Option<GlobalElementId>,
    segment: DebugIdentitySegment,
    last_seen_frame: FrameId,
}

#[cfg(feature = "diagnostics")]
enum DebugIdentitySegment {
    Root {
        name: StdArc<str>,
    },
    Child {
        file: &'static str,
        line: u32,
        column: u32,
        key_hash: Option<u64>,
        name: Option<StdArc<str>>,
        slot: u64,
    },
}

#[cfg(feature = "diagnostics")]
impl DebugIdentitySegment {
    fn format(&self) -> String {
        match self {
            DebugIdentitySegment::Root { name } => format!("root[{name}]"),
            DebugIdentitySegment::Child {
                file,
                line,
                column,
                key_hash,
                name,
                slot,
            } => {
                if let Some(name) = name.as_deref() {
                    format!("{file}:{line}:{column}[name={name}]")
                } else if let Some(k) = key_hash {
                    format!("{file}:{line}:{column}[key={k:#x}]")
                } else {
                    format!("{file}:{line}:{column}[slot={slot}]")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "ownership root overwrite detected")]
    fn strict_ownership_panics_on_root_overwrite() {
        let mut state = WindowElementState::default();
        state.set_strict_ownership(true);

        let element = GlobalElementId(123);
        state.set_node_entry(
            element,
            NodeEntry {
                node: NodeId::default(),
                last_seen_frame: FrameId(1),
                root: GlobalElementId(1),
            },
        );
        state.set_node_entry(
            element,
            NodeEntry {
                node: NodeId::default(),
                last_seen_frame: FrameId(1),
                root: GlobalElementId(2),
            },
        );
    }
}
