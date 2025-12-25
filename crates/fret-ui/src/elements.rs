use fret_core::{AppWindowId, FrameId, NodeId, Px, Rect};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{Hash, Hasher},
    panic::Location,
};

use crate::UiHost;
use crate::element::{
    AnyElement, ColumnProps, ContainerProps, ElementKind, PressableProps, PressableState, RowProps,
    SpacerProps, StackProps, TextProps, VirtualListProps, VirtualListState,
};
use crate::widget::Invalidation;
use fret_runtime::{Model, ModelId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalElementId(pub u64);

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
}

#[derive(Default)]
pub struct WindowElementState {
    state: HashMap<(GlobalElementId, TypeId), StateEntry>,
    prepared_frame: FrameId,
    prev_unkeyed_fingerprints: HashMap<u64, Vec<u64>>,
    cur_unkeyed_fingerprints: HashMap<u64, Vec<u64>>,
    observed_models: HashMap<GlobalElementId, Vec<(ModelId, Invalidation)>>,
    nodes: HashMap<GlobalElementId, NodeEntry>,
    hovered_pressable: Option<GlobalElementId>,
    pressed_pressable: Option<GlobalElementId>,
}

#[derive(Debug)]
struct StateEntry {
    value: Box<dyn Any>,
    last_seen_frame: FrameId,
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
    }

    pub(crate) fn node_entry(&self, id: GlobalElementId) -> Option<NodeEntry> {
        self.nodes.get(&id).copied()
    }

    pub(crate) fn set_node_entry(&mut self, id: GlobalElementId, entry: NodeEntry) {
        self.nodes.insert(id, entry);
    }

    pub(crate) fn retain_nodes(&mut self, f: impl FnMut(&GlobalElementId, &mut NodeEntry) -> bool) {
        self.nodes.retain(f);
    }
}

pub struct ElementCx<'a, H: UiHost> {
    pub app: &'a mut H,
    pub window: AppWindowId,
    pub frame_id: FrameId,
    pub bounds: Rect,
    window_state: &'a mut WindowElementState,
    stack: Vec<GlobalElementId>,
    child_counters: Vec<u32>,
}

impl<'a, H: UiHost> ElementCx<'a, H> {
    pub fn new(
        app: &'a mut H,
        runtime: &'a mut ElementRuntime,
        window: AppWindowId,
        bounds: Rect,
        root: GlobalElementId,
    ) -> Self {
        let frame_id = app.frame_id();
        runtime.prepare_window_for_frame(window, frame_id);

        let window_state = runtime.for_window_mut(window);

        Self {
            app,
            window,
            frame_id,
            bounds,
            window_state,
            stack: vec![root],
            child_counters: vec![0],
        }
    }

    pub fn new_for_root_name(
        app: &'a mut H,
        runtime: &'a mut ElementRuntime,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
    ) -> Self {
        Self::new(app, runtime, window, bounds, global_root(window, root_name))
    }

    pub fn root_id(&self) -> GlobalElementId {
        *self.stack.last().expect("root exists")
    }

    #[track_caller]
    pub fn scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let caller = callsite_hash(Location::caller());
        self.enter(caller, None, f)
    }

    #[track_caller]
    pub fn keyed<K: Hash, R>(&mut self, key: K, f: impl FnOnce(&mut Self) -> R) -> R {
        let caller = callsite_hash(Location::caller());
        let key_hash = stable_hash(&key);
        self.enter(caller, Some(key_hash), f)
    }

    pub fn with_state<S: Any, R>(
        &mut self,
        init: impl FnOnce() -> S,
        f: impl FnOnce(&mut S) -> R,
    ) -> R {
        let id = self.root_id();
        let key = (id, TypeId::of::<S>());

        let entry = self
            .window_state
            .state
            .entry(key)
            .or_insert_with(|| StateEntry {
                value: Box::new(init()),
                last_seen_frame: self.frame_id,
            });
        entry.last_seen_frame = self.frame_id;

        let state = entry
            .value
            .downcast_mut::<S>()
            .expect("element state type mismatch");
        f(state)
    }

    pub fn observe_model<T>(&mut self, model: Model<T>, invalidation: Invalidation) {
        self.observe_model_id(model.id(), invalidation);
    }

    pub fn observe_model_id(&mut self, model: ModelId, invalidation: Invalidation) {
        let id = self.root_id();
        let list = self.window_state.observed_models.entry(id).or_default();
        if list
            .iter()
            .any(|(m, inv)| *m == model && *inv == invalidation)
        {
            return;
        }
        list.push((model, invalidation));
    }

    #[track_caller]
    pub fn for_each_keyed<T, K: Hash>(
        &mut self,
        items: &[T],
        mut key: impl FnMut(&T) -> K,
        mut f: impl FnMut(&mut Self, usize, &T),
    ) {
        self.scope(|cx| {
            for (index, item) in items.iter().enumerate() {
                let k = key(item);
                cx.keyed(k, |cx| f(cx, index, item));
            }
        });
    }

    #[track_caller]
    pub fn for_each_unkeyed<T: Hash>(
        &mut self,
        items: &[T],
        mut f: impl FnMut(&mut Self, usize, &T),
    ) {
        let list_id = callsite_hash(Location::caller());
        let fingerprints: Vec<u64> = items.iter().map(stable_hash).collect();
        self.window_state
            .cur_unkeyed_fingerprints
            .insert(list_id, fingerprints.clone());

        if let Some(prev) = self.window_state.prev_unkeyed_fingerprints.get(&list_id)
            && prev != &fingerprints
            && items.len() > 1
            && cfg!(debug_assertions)
        {
            tracing::warn!(
                list_id = format_args!("{list_id:#x}"),
                "unkeyed element list order changed; add explicit keys to preserve state"
            );
        }

        self.scope(|cx| {
            for (index, item) in items.iter().enumerate() {
                let index_key = index as u64;
                cx.enter(list_id, Some(index_key), |cx| f(cx, index, item));
            }
        });
    }

    fn enter<R>(
        &mut self,
        callsite: u64,
        key_hash: Option<u64>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let parent = self.root_id();
        let child_index = self.child_counters.last_mut().expect("counter exists");
        let slot = *child_index as u64;
        *child_index = child_index.saturating_add(1);

        let child_salt = key_hash.unwrap_or(slot);
        let id = derive_child_id(parent, callsite, child_salt);

        self.stack.push(id);
        self.child_counters.push(0);
        let out = f(self);
        self.child_counters.pop();
        self.stack.pop();
        out
    }

    #[track_caller]
    pub fn container(
        &mut self,
        props: ContainerProps,
        f: impl FnOnce(&mut Self) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            let children = f(cx);
            AnyElement::new(id, ElementKind::Container(props), children)
        })
    }

    #[track_caller]
    pub fn pressable(
        &mut self,
        props: PressableProps,
        f: impl FnOnce(&mut Self, PressableState) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            let hovered = cx.window_state.hovered_pressable == Some(id);
            let pressed = cx.window_state.pressed_pressable == Some(id);
            let children = f(cx, PressableState { hovered, pressed });
            AnyElement::new(id, ElementKind::Pressable(props), children)
        })
    }

    #[track_caller]
    pub fn stack(&mut self, f: impl FnOnce(&mut Self) -> Vec<AnyElement>) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            let children = f(cx);
            AnyElement::new(id, ElementKind::Stack(StackProps::default()), children)
        })
    }

    #[track_caller]
    pub fn column(
        &mut self,
        props: ColumnProps,
        f: impl FnOnce(&mut Self) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            let children = f(cx);
            AnyElement::new(id, ElementKind::Column(props), children)
        })
    }

    #[track_caller]
    pub fn row(
        &mut self,
        props: RowProps,
        f: impl FnOnce(&mut Self) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            let children = f(cx);
            AnyElement::new(id, ElementKind::Row(props), children)
        })
    }

    #[track_caller]
    pub fn spacer(&mut self, props: SpacerProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            AnyElement::new(id, ElementKind::Spacer(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn text(&mut self, text: impl Into<std::sync::Arc<str>>) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            AnyElement::new(id, ElementKind::Text(TextProps::new(text)), Vec::new())
        })
    }

    #[track_caller]
    pub fn virtual_list(
        &mut self,
        len: usize,
        row_height: Px,
        overscan: usize,
        f: impl FnOnce(&mut Self, std::ops::Range<usize>) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();

            let (offset_y, viewport_h) = cx.with_state(VirtualListState::default, |state| {
                (
                    Px(state.offset_y.0.max(0.0)),
                    Px(state.viewport_h.0.max(0.0)),
                )
            });

            let content_h = Px(row_height.0.max(0.0) * len as f32);
            let max_offset = Px((content_h.0 - viewport_h.0).max(0.0));
            let offset_y = Px(offset_y.0.min(max_offset.0));

            let (mut start, mut end) = if viewport_h.0 <= 0.0 || row_height.0 <= 0.0 || len == 0 {
                (0usize, 0usize)
            } else {
                let start = (offset_y.0 / row_height.0).floor() as isize;
                let end = ((offset_y.0 + viewport_h.0) / row_height.0).ceil() as isize;
                let start = start.max(0) as usize;
                let end = end.max(start as isize) as usize;
                (start, end.min(len))
            };

            let o = overscan.min(len);
            start = start.saturating_sub(o);
            end = (end + o).min(len);

            let children = f(cx, start..end);
            AnyElement::new(
                id,
                ElementKind::VirtualList(VirtualListProps {
                    len,
                    row_height,
                    overscan,
                    visible_start: start,
                    visible_end: end,
                }),
                children,
            )
        })
    }

    /// Virtualized list helper that enforces stable element identity by entering a keyed scope
    /// for each visible row.
    ///
    /// Prefer this over index-identity list rendering for any dynamic collection that can reorder,
    /// so element-local state (caret/selection/scroll) does not “stick to positions”.
    #[track_caller]
    pub fn virtual_list_keyed<K: Hash>(
        &mut self,
        len: usize,
        row_height: Px,
        overscan: usize,
        mut key_at: impl FnMut(usize) -> K,
        mut row: impl FnMut(&mut Self, usize) -> AnyElement,
    ) -> AnyElement {
        self.virtual_list(len, row_height, overscan, |cx, range| {
            range
                .map(|i| {
                    let key = key_at(i);
                    cx.keyed(key, |cx| row(cx, i))
                })
                .collect()
        })
    }
}

pub fn with_element_state<H: UiHost, S: Any, R>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
    init: impl FnOnce() -> S,
    f: impl FnOnce(&mut S) -> R,
) -> R {
    let frame_id = app.frame_id();
    app.with_global_mut(ElementRuntime::new, |runtime, _app| {
        runtime.prepare_window_for_frame(window, frame_id);
        let window_state = runtime.for_window_mut(window);

        let key = (element, TypeId::of::<S>());
        let entry = window_state.state.entry(key).or_insert_with(|| StateEntry {
            value: Box::new(init()),
            last_seen_frame: frame_id,
        });
        entry.last_seen_frame = frame_id;

        let state = entry
            .value
            .downcast_mut::<S>()
            .expect("element state type mismatch");
        f(state)
    })
}

pub(crate) fn observed_models_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Vec<(ModelId, Invalidation)> {
    let frame_id = app.frame_id();
    app.with_global_mut(ElementRuntime::new, |runtime, _app| {
        runtime.prepare_window_for_frame(window, frame_id);
        runtime
            .for_window_mut(window)
            .observed_models
            .get(&element)
            .cloned()
            .unwrap_or_default()
    })
}

pub(crate) fn with_window_state<H: UiHost, R>(
    app: &mut H,
    window: AppWindowId,
    f: impl FnOnce(&mut WindowElementState) -> R,
) -> R {
    let frame_id = app.frame_id();
    app.with_global_mut(ElementRuntime::new, |runtime, _app| {
        runtime.prepare_window_for_frame(window, frame_id);
        let window_state = runtime.for_window_mut(window);
        f(window_state)
    })
}

pub(crate) fn update_hovered_pressable<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    next: Option<GlobalElementId>,
) -> (Option<NodeId>, Option<NodeId>) {
    with_window_state(app, window, |st| {
        let prev = st.hovered_pressable;
        if prev == next {
            return (None, None);
        }
        let prev_node = prev.and_then(|id| st.node_entry(id).map(|e| e.node));
        let next_node = next.and_then(|id| st.node_entry(id).map(|e| e.node));
        st.hovered_pressable = next;
        (prev_node, next_node)
    })
}

pub(crate) fn set_pressed_pressable<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pressed: Option<GlobalElementId>,
) -> Option<NodeId> {
    with_window_state(app, window, |st| {
        let prev = st.pressed_pressable;
        if prev == pressed {
            return None;
        }
        let prev_node = prev.and_then(|id| st.node_entry(id).map(|e| e.node));
        st.pressed_pressable = pressed;
        prev_node
    })
}

pub(crate) fn is_hovered_pressable<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> bool {
    with_window_state(app, window, |st| st.hovered_pressable == Some(element))
}

pub fn take_element_state<H: UiHost, S: Any>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<S> {
    app.with_global_mut(ElementRuntime::new, |runtime, app| {
        runtime.prepare_window_for_frame(window, app.frame_id());
        let window_state = runtime.for_window_mut(window);
        window_state
            .state
            .remove(&(element, TypeId::of::<S>()))
            .and_then(|e| e.value.downcast::<S>().ok())
            .map(|b| *b)
    })
}

pub fn global_root(window: AppWindowId, name: &str) -> GlobalElementId {
    let mut h = Fnv1a64::default();
    window.hash(&mut h);
    h.write(name.as_bytes());
    GlobalElementId(h.finish())
}

pub fn with_element_cx<H: UiHost, R>(
    app: &mut H,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> R,
) -> R {
    app.with_global_mut(ElementRuntime::new, |runtime, app| {
        let mut cx = ElementCx::new_for_root_name(app, runtime, window, bounds, root_name);
        f(&mut cx)
    })
}

fn derive_child_id(parent: GlobalElementId, callsite: u64, child_salt: u64) -> GlobalElementId {
    let mut h = Fnv1a64::default();
    h.write_u64(parent.0);
    h.write_u64(callsite);
    h.write_u64(child_salt);
    GlobalElementId(h.finish())
}

fn stable_hash<T: Hash>(value: &T) -> u64 {
    let mut h = Fnv1a64::default();
    value.hash(&mut h);
    h.finish()
}

fn callsite_hash(loc: &Location<'_>) -> u64 {
    let mut h = Fnv1a64::default();
    h.write(loc.file().as_bytes());
    h.write_u32(loc.line());
    h.write_u32(loc.column());
    h.finish()
}

#[derive(Default)]
struct Fnv1a64(u64);

impl Hasher for Fnv1a64 {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 {
            0xcbf29ce484222325
        } else {
            self.0
        };
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.0 = hash;
    }
}
