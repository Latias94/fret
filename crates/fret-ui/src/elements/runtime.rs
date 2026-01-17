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
    pub active_text_selection: Option<(GlobalElementId, GlobalElementId)>,
    pub hovered_pressable: Option<GlobalElementId>,
    pub pressed_pressable: Option<GlobalElementId>,
    pub hovered_hover_region: Option<GlobalElementId>,
    pub wants_continuous_frames: bool,
    pub observed_models: Vec<(GlobalElementId, Vec<(u64, Invalidation)>)>,
    pub observed_globals: Vec<(GlobalElementId, Vec<(String, Invalidation)>)>,
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
    pub(super) view_cache_reuse_roots: HashSet<GlobalElementId>,
    view_cache_stack: Vec<GlobalElementId>,
    prepared_frame: FrameId,
    pub(super) prev_unkeyed_fingerprints: HashMap<u64, Vec<u64>>,
    pub(super) cur_unkeyed_fingerprints: HashMap<u64, Vec<u64>>,
    pub(super) observed_models_rendered: HashMap<GlobalElementId, Vec<(ModelId, Invalidation)>>,
    pub(super) observed_models_next: HashMap<GlobalElementId, Vec<(ModelId, Invalidation)>>,
    pub(super) observed_globals_rendered: HashMap<GlobalElementId, Vec<(TypeId, Invalidation)>>,
    pub(super) observed_globals_next: HashMap<GlobalElementId, Vec<(TypeId, Invalidation)>>,
    pub(super) timer_targets: HashMap<TimerToken, GlobalElementId>,
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

impl WindowElementState {
    fn prepare_for_frame(&mut self, frame_id: FrameId, lag_frames: u64) {
        if self.prepared_frame == frame_id {
            return;
        }
        self.prepared_frame = frame_id;

        self.advance_element_state_buffers(lag_frames);

        std::mem::swap(
            &mut self.view_cache_state_keys_rendered,
            &mut self.view_cache_state_keys_next,
        );
        self.view_cache_state_keys_next.clear();
        self.view_cache_reuse_roots.clear();
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

        std::mem::swap(&mut self.prev_bounds, &mut self.cur_bounds);
        self.cur_bounds.clear();

        std::mem::swap(&mut self.prev_visual_bounds, &mut self.cur_visual_bounds);
        self.cur_visual_bounds.clear();

        self.focused_element = None;

        #[cfg(feature = "diagnostics")]
        {
            let cutoff = frame_id.0.saturating_sub(lag_frames);
            self.debug_identity
                .entries
                .retain(|_, v| v.last_seen_frame.0 >= cutoff);
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

    pub(super) fn record_state_key_access(&mut self, key: (GlobalElementId, TypeId)) {
        let Some(root) = self.view_cache_stack.last().copied() else {
            return;
        };
        self.view_cache_state_keys_next
            .entry(root)
            .or_default()
            .push(key);
    }

    pub(super) fn begin_view_cache_scope(&mut self, root: GlobalElementId) {
        self.view_cache_stack.push(root);
        self.view_cache_state_keys_next.remove(&root);
    }

    pub(super) fn end_view_cache_scope(&mut self, root: GlobalElementId) {
        let popped = self.view_cache_stack.pop();
        debug_assert_eq!(popped, Some(root));
        if let Some(keys) = self.view_cache_state_keys_next.get_mut(&root) {
            let mut seen: HashSet<(GlobalElementId, TypeId)> = HashSet::with_capacity(keys.len());
            keys.retain(|&key| seen.insert(key));
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

    pub(crate) fn should_reuse_view_cache_root(&self, root: GlobalElementId) -> bool {
        self.view_cache_reuse_roots.contains(&root)
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

    pub(crate) fn active_text_selection(&self) -> Option<ActiveTextSelection> {
        self.active_text_selection
    }

    pub(crate) fn set_active_text_selection(&mut self, selection: Option<ActiveTextSelection>) {
        self.active_text_selection = selection;
    }

    pub(crate) fn node_entry(&self, id: GlobalElementId) -> Option<NodeEntry> {
        self.nodes.get(&id).copied()
    }

    pub(crate) fn element_for_node(&self, node: NodeId) -> Option<GlobalElementId> {
        self.nodes
            .iter()
            .find_map(|(&element, entry)| (entry.node == node).then_some(element))
    }

    pub(crate) fn set_node_entry(&mut self, id: GlobalElementId, entry: NodeEntry) {
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

    pub(crate) fn last_bounds(&self, element: GlobalElementId) -> Option<Rect> {
        self.prev_bounds.get(&element).copied()
    }

    pub(crate) fn record_visual_bounds(&mut self, element: GlobalElementId, bounds: Rect) {
        self.cur_visual_bounds.insert(element, bounds);
    }

    pub(crate) fn last_visual_bounds(&self, element: GlobalElementId) -> Option<Rect> {
        self.prev_visual_bounds.get(&element).copied()
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
        WindowElementDiagnosticsSnapshot {
            focused_element: self.focused_element,
            active_text_selection: self
                .active_text_selection
                .map(|sel| (sel.root, sel.element)),
            hovered_pressable: self.hovered_pressable,
            pressed_pressable: self.pressed_pressable,
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
