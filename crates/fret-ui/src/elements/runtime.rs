use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use fret_core::{AppWindowId, NodeId, Rect};
use fret_runtime::{FrameId, ModelId, TimerToken};
#[cfg(feature = "diagnostics")]
use slotmap::Key as _;

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
}

#[derive(Default)]
pub struct WindowElementState {
    pub(super) state: HashMap<(GlobalElementId, TypeId), StateEntry>,
    prepared_frame: FrameId,
    pub(super) prev_unkeyed_fingerprints: HashMap<u64, Vec<u64>>,
    pub(super) cur_unkeyed_fingerprints: HashMap<u64, Vec<u64>>,
    pub(super) observed_models: HashMap<GlobalElementId, Vec<(ModelId, Invalidation)>>,
    pub(super) observed_globals: HashMap<GlobalElementId, Vec<(TypeId, Invalidation)>>,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActiveTextSelection {
    pub root: GlobalElementId,
    pub element: GlobalElementId,
}

#[derive(Debug)]
pub(super) struct StateEntry {
    pub(super) value: Box<dyn Any>,
    pub(super) last_seen_frame: FrameId,
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

        let cutoff = frame_id.0.saturating_sub(lag_frames);
        self.state.retain(|_, e| e.last_seen_frame.0 >= cutoff);

        std::mem::swap(
            &mut self.prev_unkeyed_fingerprints,
            &mut self.cur_unkeyed_fingerprints,
        );
        self.cur_unkeyed_fingerprints.clear();
        self.observed_models.clear();
        self.observed_globals.clear();

        std::mem::swap(&mut self.prev_bounds, &mut self.cur_bounds);
        self.cur_bounds.clear();

        std::mem::swap(&mut self.prev_visual_bounds, &mut self.cur_visual_bounds);
        self.cur_visual_bounds.clear();

        self.focused_element = None;
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
                .observed_models
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
                .observed_globals
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
