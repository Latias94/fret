use fret_core::{AppWindowId, FrameId, NodeId, Px, Rect};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{Hash, Hasher},
    panic::Location,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use crate::SvgSource;
use crate::UiHost;
use crate::element::{
    AnyElement, ColumnProps, ContainerProps, ElementKind, FlexProps, GridProps, HoverRegionProps,
    ImageProps, LayoutStyle, PressableProps, PressableState, RowProps, ScrollProps, SpacerProps,
    SpinnerProps, StackProps, SvgIconProps, TextInputProps, TextProps, VirtualListOptions,
    VirtualListProps, VirtualListState,
};
use crate::widget::Invalidation;
use fret_runtime::{Effect, Model, ModelId};

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
    root_bounds: HashMap<GlobalElementId, Rect>,
    prev_bounds: HashMap<GlobalElementId, Rect>,
    cur_bounds: HashMap<GlobalElementId, Rect>,
    hovered_pressable: Option<GlobalElementId>,
    pressed_pressable: Option<GlobalElementId>,
    hovered_hover_region: Option<GlobalElementId>,
    continuous_frames: Arc<AtomicUsize>,
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

        std::mem::swap(&mut self.prev_bounds, &mut self.cur_bounds);
        self.cur_bounds.clear();
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

    pub(crate) fn wants_continuous_frames(&self) -> bool {
        self.continuous_frames.load(Ordering::Relaxed) > 0
    }

    pub(crate) fn begin_continuous_frames(&self) -> ContinuousFrames {
        self.continuous_frames.fetch_add(1, Ordering::Relaxed);
        ContinuousFrames {
            leases: self.continuous_frames.clone(),
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

    pub fn request_frame(&mut self) {
        self.app.request_redraw(self.window);
    }

    pub fn request_animation_frame(&mut self) {
        self.app
            .push_effect(Effect::RequestAnimationFrame(self.window));
    }

    pub fn begin_continuous_frames(&mut self) -> ContinuousFrames {
        let lease = self.window_state.begin_continuous_frames();
        self.request_animation_frame();
        lease
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
        self.stack_props(StackProps::default(), f)
    }

    #[track_caller]
    pub fn stack_props(
        &mut self,
        props: StackProps,
        f: impl FnOnce(&mut Self) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            let children = f(cx);
            AnyElement::new(id, ElementKind::Stack(props), children)
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
    pub fn text_props(&mut self, props: TextProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            AnyElement::new(id, ElementKind::Text(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn text_input(&mut self, props: TextInputProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            AnyElement::new(id, ElementKind::TextInput(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn image(&mut self, image: fret_core::ImageId) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            AnyElement::new(id, ElementKind::Image(ImageProps::new(image)), Vec::new())
        })
    }

    #[track_caller]
    pub fn image_props(&mut self, props: ImageProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            AnyElement::new(id, ElementKind::Image(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn svg_icon(&mut self, svg: SvgSource) -> AnyElement {
        self.svg_icon_props(SvgIconProps::new(svg))
    }

    #[track_caller]
    pub fn svg_icon_props(&mut self, props: SvgIconProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            AnyElement::new(id, ElementKind::SvgIcon(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn spinner(&mut self) -> AnyElement {
        self.spinner_props(SpinnerProps::default())
    }

    #[track_caller]
    pub fn spinner_props(&mut self, props: SpinnerProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            AnyElement::new(id, ElementKind::Spinner(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn hover_region(
        &mut self,
        props: HoverRegionProps,
        f: impl FnOnce(&mut Self, bool) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            let hovered = cx.window_state.hovered_hover_region == Some(id);
            let children = f(cx, hovered);
            AnyElement::new(id, ElementKind::HoverRegion(props), children)
        })
    }

    #[track_caller]
    pub fn scroll(
        &mut self,
        props: ScrollProps,
        f: impl FnOnce(&mut Self) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            let children = f(cx);
            AnyElement::new(id, ElementKind::Scroll(props), children)
        })
    }

    #[track_caller]
    pub fn virtual_list(
        &mut self,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        f: impl FnOnce(&mut Self, &[crate::virtual_list::VirtualItem]) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.virtual_list_with_layout(LayoutStyle::default(), len, options, scroll_handle, f)
    }

    #[track_caller]
    pub fn virtual_list_ex(
        &mut self,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        range_extractor: impl FnOnce(crate::virtual_list::VirtualRange) -> Vec<usize>,
        f: impl FnOnce(&mut Self, &[crate::virtual_list::VirtualItem]) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.virtual_list_with_layout_ex(
            LayoutStyle::default(),
            len,
            options,
            scroll_handle,
            range_extractor,
            f,
        )
    }

    #[track_caller]
    pub fn virtual_list_with_layout(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        f: impl FnOnce(&mut Self, &[crate::virtual_list::VirtualItem]) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.virtual_list_with_layout_ex(
            layout,
            len,
            options,
            scroll_handle,
            crate::virtual_list::default_range_extractor,
            f,
        )
    }

    #[track_caller]
    pub fn virtual_list_with_layout_ex(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        range_extractor: impl FnOnce(crate::virtual_list::VirtualRange) -> Vec<usize>,
        f: impl FnOnce(&mut Self, &[crate::virtual_list::VirtualItem]) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.virtual_list_with_layout_and_keys(
            layout,
            len,
            options,
            scroll_handle,
            |i| i as crate::ItemKey,
            range_extractor,
            f,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn virtual_list_with_layout_and_keys(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        mut item_key_at: impl FnMut(usize) -> crate::ItemKey,
        range_extractor: impl FnOnce(crate::virtual_list::VirtualRange) -> Vec<usize>,
        f: impl FnOnce(&mut Self, &[crate::virtual_list::VirtualItem]) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();

            let scroll_handle = scroll_handle.clone();
            scroll_handle.set_items_count(len);

            let range = cx.with_state(VirtualListState::default, |state| {
                let prev_cfg = (
                    state.metrics.estimate(),
                    state.metrics.gap(),
                    state.metrics.scroll_margin(),
                );
                let cfg = (
                    options.estimate_row_height,
                    options.gap,
                    options.scroll_margin,
                );

                state.metrics.ensure(
                    len,
                    options.estimate_row_height,
                    options.gap,
                    options.scroll_margin,
                );

                let needs_rebuild = state.items_revision != options.items_revision
                    || state.keys.len() != len
                    || prev_cfg != cfg;

                if needs_rebuild {
                    state.items_revision = options.items_revision;
                    state.keys.clear();
                    state.keys.reserve(len);

                    let mut heights = Vec::with_capacity(len);
                    let mut measured = Vec::with_capacity(len);

                    for i in 0..len {
                        let key = item_key_at(i);
                        state.keys.push(key);
                        if let Some(h) = state.size_cache.get(&key).copied() {
                            heights.push(h);
                            measured.push(true);
                        } else {
                            heights.push(options.estimate_row_height);
                            measured.push(false);
                        }
                    }

                    state.metrics.rebuild_from_heights(
                        heights,
                        measured,
                        options.estimate_row_height,
                        options.gap,
                        options.scroll_margin,
                    );
                }

                let viewport_h = Px(state.viewport_h.0.max(0.0));
                let offset_y = state
                    .metrics
                    .clamp_offset(scroll_handle.offset().y, viewport_h);

                state
                    .metrics
                    .visible_range(offset_y, viewport_h, options.overscan)
            });

            let mut indices = range
                .map(range_extractor)
                .unwrap_or_default()
                .into_iter()
                .filter(|&idx| idx < len)
                .collect::<Vec<_>>();
            indices.sort_unstable();
            indices.dedup();

            let visible_items = cx.with_state(VirtualListState::default, |state| {
                indices
                    .iter()
                    .map(|&idx| {
                        let key = state
                            .keys
                            .get(idx)
                            .copied()
                            .unwrap_or(idx as crate::ItemKey);
                        state.metrics.virtual_item(idx, key)
                    })
                    .collect::<Vec<_>>()
            });

            let children = f(cx, &visible_items);
            AnyElement::new(
                id,
                ElementKind::VirtualList(VirtualListProps {
                    layout,
                    len,
                    items_revision: options.items_revision,
                    estimate_row_height: options.estimate_row_height,
                    overscan: options.overscan,
                    scroll_margin: options.scroll_margin,
                    gap: options.gap,
                    scroll_handle,
                    visible_items,
                }),
                children,
            )
        })
    }

    #[track_caller]
    pub fn flex(
        &mut self,
        props: FlexProps,
        f: impl FnOnce(&mut Self) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            let children = f(cx);
            AnyElement::new(id, ElementKind::Flex(props), children)
        })
    }

    #[track_caller]
    pub fn grid(
        &mut self,
        props: GridProps,
        f: impl FnOnce(&mut Self) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            let children = f(cx);
            AnyElement::new(id, ElementKind::Grid(props), children)
        })
    }

    /// Virtualized list helper that enforces stable element identity by entering a keyed scope
    /// for each visible row.
    ///
    /// Prefer this over index-identity list rendering for any dynamic collection that can reorder,
    /// so element-local state (caret/selection/scroll) does not “stick to positions”.
    #[track_caller]
    pub fn virtual_list_keyed(
        &mut self,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        key_at: impl FnMut(usize) -> crate::ItemKey,
        row: impl FnMut(&mut Self, usize) -> AnyElement,
    ) -> AnyElement {
        self.virtual_list_keyed_with_layout(
            LayoutStyle::default(),
            len,
            options,
            scroll_handle,
            key_at,
            row,
        )
    }

    #[track_caller]
    pub fn virtual_list_keyed_ex(
        &mut self,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        key_at: impl FnMut(usize) -> crate::ItemKey,
        range_extractor: impl FnOnce(crate::virtual_list::VirtualRange) -> Vec<usize>,
        row: impl FnMut(&mut Self, usize) -> AnyElement,
    ) -> AnyElement {
        self.virtual_list_keyed_with_layout_ex(
            LayoutStyle::default(),
            len,
            options,
            scroll_handle,
            key_at,
            range_extractor,
            row,
        )
    }

    #[track_caller]
    pub fn virtual_list_keyed_with_layout(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        key_at: impl FnMut(usize) -> crate::ItemKey,
        row: impl FnMut(&mut Self, usize) -> AnyElement,
    ) -> AnyElement {
        self.virtual_list_keyed_with_layout_ex(
            layout,
            len,
            options,
            scroll_handle,
            key_at,
            crate::virtual_list::default_range_extractor,
            row,
        )
    }

    #[track_caller]
    #[allow(clippy::too_many_arguments)]
    pub fn virtual_list_keyed_with_layout_ex(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        key_at: impl FnMut(usize) -> crate::ItemKey,
        range_extractor: impl FnOnce(crate::virtual_list::VirtualRange) -> Vec<usize>,
        mut row: impl FnMut(&mut Self, usize) -> AnyElement,
    ) -> AnyElement {
        self.virtual_list_with_layout_and_keys(
            layout,
            len,
            options,
            scroll_handle,
            key_at,
            range_extractor,
            |cx, items| {
                items
                    .iter()
                    .copied()
                    .map(|item| cx.keyed(item.key, |cx| row(cx, item.index)))
                    .collect()
            },
        )
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

pub(crate) fn update_hovered_hover_region<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    next: Option<GlobalElementId>,
) -> (Option<NodeId>, Option<NodeId>) {
    with_window_state(app, window, |st| {
        let prev = st.hovered_hover_region;
        if prev == next {
            return (None, None);
        }
        let prev_node = prev.and_then(|id| st.node_entry(id).map(|e| e.node));
        let next_node = next.and_then(|id| st.node_entry(id).map(|e| e.node));
        st.hovered_hover_region = next;
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

pub(crate) fn is_pressed_pressable<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> bool {
    with_window_state(app, window, |st| st.pressed_pressable == Some(element))
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

pub fn root_bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<Rect> {
    app.with_global_mut(ElementRuntime::new, |runtime, _app| {
        let state = runtime.for_window_mut(window);
        let root = state.node_entry(element).map(|e| e.root)?;
        state.root_bounds(root)
    })
}

/// Returns the last frame's bounds for a declarative element, if available.
///
/// This is a cross-frame geometry query intended for component-layer policies (e.g. anchored
/// overlays) that need a stable anchor rect. The value is updated during layout.
pub fn bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<Rect> {
    with_window_state(app, window, |st| st.last_bounds(element))
}

pub(crate) fn record_bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
    bounds: Rect,
) {
    with_window_state(app, window, |st| st.record_bounds(element, bounds));
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
