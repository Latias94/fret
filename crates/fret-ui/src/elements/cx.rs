use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::Hash;
use std::panic::Location;
use std::sync::Arc;

use smallvec::SmallVec;

use fret_core::input::PointerType;
use fret_core::window::{ColorScheme, ContrastPreference, ForcedColorsMode};
use fret_core::{AppWindowId, Color, Edges, EffectChain, EffectMode, NodeId, Px, Rect};
use fret_runtime::{Effect, FrameId, Model, ModelId, ModelUpdateError};

use crate::action::OnHoverChange;
use crate::action::{
    CommandActionHooks, CommandAvailabilityActionHooks, DismissibleActionHooks, KeyActionHooks,
    OnActivate, OnCommand, OnCommandAvailability, OnDismissRequest, OnDismissiblePointerMove,
    OnKeyDown, OnPinchGesture, OnPointerCancel, OnPointerDown, OnPointerMove, OnPointerUp,
    OnPressablePointerDown, OnPressablePointerMove, OnPressablePointerUp, OnRovingActiveChange,
    OnRovingNavigate, OnRovingTypeahead, OnTimer, OnWheel, PointerActionHooks,
    PressableActionHooks, PressableHoverActionHooks, PressablePointerUpResult, RovingActionHooks,
    TimerActionHooks,
};
use crate::canvas::{CanvasPaintHooks, CanvasPainter, OnCanvasPaint};
use crate::element::{
    AnyElement, CanvasProps, ColumnProps, ContainerProps, EffectLayerProps, ElementKind, FlexProps,
    FocusTraversalGateProps, GridProps, HitTestGateProps, HoverRegionProps, ImageProps,
    InteractivityGateProps, LayoutQueryRegionProps, LayoutStyle, OpacityProps, PointerRegionProps,
    PressableProps, PressableState, ResizablePanelGroupProps, RowProps, ScrollProps,
    ScrollbarProps, SelectableTextProps, SpacerProps, SpinnerProps, StackProps, StyledTextProps,
    SvgIconProps, TextAreaProps, TextInputProps, TextProps, ViewportSurfaceProps,
    VirtualListOptions, VirtualListProps, VirtualListState, VisualTransformProps,
};
use crate::overlay_placement::{AnchoredPanelLayoutTrace, Side};
use crate::widget::Invalidation;
use crate::{SvgSource, Theme, UiHost};
use fret_core::window::WindowMetricsService;

use super::hash::{callsite_hash, derive_child_id, stable_hash};
use super::runtime::{EnvironmentQueryKey, LayoutQueryRegionMarker};
use super::{ContinuousFrames, ElementRuntime, GlobalElementId, WindowElementState, global_root};

pub struct ElementContext<'a, H: UiHost> {
    pub app: &'a mut H,
    pub window: AppWindowId,
    pub frame_id: FrameId,
    pub bounds: Rect,
    window_state: &'a mut WindowElementState,
    stack: Vec<GlobalElementId>,
    callsite_counters: Vec<CallsiteCounters>,
    view_cache_should_reuse: Option<&'a mut dyn FnMut(NodeId) -> bool>,
}

type CallsiteCounters = SmallVec<[(u64, u32); 16]>;

fn bump_callsite_counter(counters: &mut CallsiteCounters, callsite: u64) -> u64 {
    for (seen, next) in counters.iter_mut() {
        if *seen != callsite {
            continue;
        }
        let slot = (*next) as u64;
        *next = next.saturating_add(1);
        return slot;
    }
    counters.push((callsite, 1));
    0
}

impl<'a, H: UiHost> ElementContext<'a, H> {
    pub(crate) fn collect_children(
        &mut self,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Vec<AnyElement> {
        let iter = children.into_iter();
        let (min, _) = iter.size_hint();
        let mut out = self.window_state.take_scratch_element_children_vec(min);
        out.extend(iter);
        out
    }

    pub(crate) fn retained_virtual_list_row_any_element(
        &mut self,
        key: crate::ItemKey,
        index: usize,
        row: &crate::windowed_surface_host::RetainedVirtualListRowFn<H>,
    ) -> AnyElement {
        self.keyed(key, |cx| (row)(cx, index))
    }

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
            callsite_counters: vec![CallsiteCounters::new()],
            view_cache_should_reuse: None,
        }
    }

    pub(crate) fn set_view_cache_should_reuse(&mut self, f: &'a mut dyn FnMut(NodeId) -> bool) {
        self.view_cache_should_reuse = Some(f);
    }

    pub fn new_for_root_name(
        app: &'a mut H,
        runtime: &'a mut ElementRuntime,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
    ) -> Self {
        let root = global_root(window, root_name);
        #[cfg(feature = "diagnostics")]
        {
            let cx = Self::new(app, runtime, window, bounds, root);
            cx.window_state
                .record_debug_root(cx.frame_id, root, root_name);
            cx
        }
        #[cfg(not(feature = "diagnostics"))]
        {
            Self::new(app, runtime, window, bounds, root)
        }
    }

    pub(crate) fn new_for_existing_window_state(
        app: &'a mut H,
        window: AppWindowId,
        bounds: Rect,
        root: GlobalElementId,
        window_state: &'a mut WindowElementState,
    ) -> Self {
        let frame_id = app.frame_id();
        Self {
            app,
            window,
            frame_id,
            bounds,
            window_state,
            stack: vec![root],
            callsite_counters: vec![CallsiteCounters::new()],
            view_cache_should_reuse: None,
        }
    }

    pub fn root_id(&self) -> GlobalElementId {
        *self.stack.last().expect("root exists")
    }

    fn new_any_element(
        &mut self,
        id: GlobalElementId,
        kind: ElementKind,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        AnyElement::new(id, kind, children)
    }

    /// Returns the nearest ancestor state value of type `S` in the current element scope stack.
    ///
    /// This is a lightweight building block for component-layer "provider" patterns (e.g. Radix
    /// `DirectionProvider`) without requiring a dedicated runtime context mechanism.
    pub fn inherited_state<S: Any>(&self) -> Option<&S> {
        self.inherited_state_where(|_state: &S| true)
    }

    /// Like `inherited_state`, but allows skipping "inactive" states while continuing to search.
    ///
    /// This is useful when a state entry remains allocated but temporarily holds no active value
    /// (e.g. an `Option<T>` that is `None` outside of a scope).
    pub fn inherited_state_where<S: Any>(&self, predicate: impl Fn(&S) -> bool) -> Option<&S> {
        let ty = TypeId::of::<S>();
        for &id in self.stack.iter().rev() {
            let key = (id, ty);
            let Some(any) = self.window_state.state_any_ref(&key) else {
                continue;
            };
            let Some(state) = any.downcast_ref::<S>() else {
                continue;
            };
            if predicate(state) {
                return Some(state);
            }
        }
        None
    }

    /// Returns the last known `NodeId` for a declarative element, if available.
    ///
    /// This is safe to call during element rendering: it reads from the `ElementCx`'s already
    /// borrowed window state, avoiding re-entrant `UiHost::with_global_mut` leases.
    pub fn node_for_element(&self, element: GlobalElementId) -> Option<NodeId> {
        self.window_state.node_entry(element).map(|e| e.node)
    }

    pub fn focused_element(&self) -> Option<GlobalElementId> {
        self.window_state.focused_element
    }

    pub fn is_focused_element(&self, element: GlobalElementId) -> bool {
        self.window_state.focused_element == Some(element)
    }

    pub fn has_active_text_selection(&self) -> bool {
        self.window_state.active_text_selection().is_some()
    }

    pub fn has_active_text_selection_in_root(&self, root: GlobalElementId) -> bool {
        self.window_state
            .active_text_selection()
            .is_some_and(|selection| selection.root == root)
    }

    pub(crate) fn sync_focused_element_from_focused_node(&mut self, focused: Option<NodeId>) {
        self.window_state.focused_element =
            focused.and_then(|node| self.window_state.element_for_node(node));
    }

    /// Returns the last frame's bounds for a declarative element, if available.
    ///
    /// This is safe to call during element rendering: it reads from the `ElementCx`'s already
    /// borrowed window state, avoiding re-entrant `UiHost::with_global_mut` leases.
    pub fn last_bounds_for_element(&self, element: GlobalElementId) -> Option<Rect> {
        self.window_state.last_bounds(element)
    }

    /// Returns the last frame's **visual** bounds (post-`render_transform` AABB) for a declarative
    /// element, if available.
    ///
    /// This is intended for anchored overlay policies that must track render transforms (ADR 0083)
    /// without mixing layout transforms into the layout solver.
    pub fn last_visual_bounds_for_element(&self, element: GlobalElementId) -> Option<Rect> {
        self.window_state.last_visual_bounds(element)
    }

    /// Returns the last recorded root bounds for the element's root, if available.
    ///
    /// This is safe to call during element rendering for the same reason as
    /// `last_bounds_for_element`.
    pub fn root_bounds_for_element(&self, element: GlobalElementId) -> Option<Rect> {
        let root = self.window_state.node_entry(element).map(|e| e.root)?;
        self.window_state.root_bounds(root)
    }

    /// Consume a transient event flag for the current element.
    ///
    /// This returns `true` at most once per recorded event (clear-on-read).
    pub fn take_transient(&mut self, key: u64) -> bool {
        let element = self.root_id();
        self.take_transient_for(element, key)
    }

    /// Consume a transient event flag for the specified element.
    ///
    /// This returns `true` at most once per recorded event (clear-on-read).
    pub fn take_transient_for(&mut self, element: GlobalElementId, key: u64) -> bool {
        self.window_state.take_transient_event(element, key)
    }

    pub fn with_root_name<R>(&mut self, root_name: &str, f: impl FnOnce(&mut Self) -> R) -> R {
        let root = global_root(self.window, root_name);

        let prev_stack = std::mem::take(&mut self.stack);
        let prev_counters = std::mem::take(&mut self.callsite_counters);

        self.stack = vec![root];
        self.callsite_counters = vec![CallsiteCounters::new()];

        let out = f(self);

        self.stack = prev_stack;
        self.callsite_counters = prev_counters;

        out
    }

    pub fn with_callsite_counters_snapshot<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let prev_stack = self.stack.clone();
        let prev_counters = self.callsite_counters.clone();

        let out = f(self);

        self.stack = prev_stack;
        self.callsite_counters = prev_counters;

        out
    }

    /// Request a window redraw (one-shot).
    ///
    /// Use this after mutating state/models to schedule a repaint.
    ///
    /// Notes:
    /// - This does not guarantee frame-driven progression. If you need to advance without input
    ///   events (animations, progressive rendering), prefer `request_animation_frame()` or
    ///   `begin_continuous_frames()`.
    /// - This is not a timer: callers that need continuous progression must keep requesting frames.
    pub fn request_frame(&mut self) {
        self.app.request_redraw(self.window);
    }

    /// Mark the nearest cache root as needing a paint notification on the next mount.
    ///
    /// This is a lightweight way to force paint-cache roots to rerun paint (e.g. while animating
    /// opacity/transform) without necessarily requesting a new animation frame from the runner.
    pub fn notify_for_animation_frame(&mut self) {
        // Drive invalidation from the current element, letting propagation and cache-root
        // truncation pick the appropriate boundary (e.g. nearest view-cache root when enabled).
        self.window_state
            .request_notify_for_animation_frame(self.root_id());
    }

    /// Request the next animation frame for this window.
    ///
    /// Use this for frame-driven updates that must advance without input events.
    ///
    /// This is a one-shot request. Prefer `begin_continuous_frames()` when driving animations from
    /// declarative UI code.
    pub fn request_animation_frame(&mut self) {
        self.notify_for_animation_frame();
        self.app
            .push_effect(Effect::RequestAnimationFrame(self.window));
    }

    /// Begin a "continuous frames" lease for this window.
    ///
    /// This is the preferred way for declarative UI to drive animations: while the returned
    /// lease is held, the mount pass will continue requesting animation frames.
    pub fn begin_continuous_frames(&mut self) -> ContinuousFrames {
        let lease = self.window_state.begin_continuous_frames();
        self.request_animation_frame();
        lease
    }

    #[track_caller]
    pub fn scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let loc = Location::caller();
        let callsite = callsite_hash(loc);
        self.enter_with_callsite(loc, callsite, None, None, f)
    }

    #[track_caller]
    pub fn keyed<K: Hash, R>(&mut self, key: K, f: impl FnOnce(&mut Self) -> R) -> R {
        let loc = Location::caller();
        let caller = callsite_hash(loc);
        let key_hash = stable_hash(&key);
        self.enter_with_callsite(loc, caller, Some(key_hash), None, f)
    }

    #[track_caller]
    pub fn named<R>(&mut self, name: &str, f: impl FnOnce(&mut Self) -> R) -> R {
        let loc = Location::caller();
        let caller = callsite_hash(loc);
        let key_hash = stable_hash(&name);
        self.enter_with_callsite(loc, caller, Some(key_hash), Some(name), f)
    }

    pub fn with_state<S: Any, R>(
        &mut self,
        init: impl FnOnce() -> S,
        f: impl FnOnce(&mut S) -> R,
    ) -> R {
        let id = self.root_id();
        self.with_state_for(id, init, f)
    }

    pub fn with_state_for<S: Any, R>(
        &mut self,
        element: GlobalElementId,
        init: impl FnOnce() -> S,
        f: impl FnOnce(&mut S) -> R,
    ) -> R {
        let key = (element, TypeId::of::<S>());
        self.window_state.record_state_key_access(key);
        let mut value = self
            .window_state
            .take_state_box(&key)
            .unwrap_or_else(|| Box::new(init()));

        let out = {
            let state = value
                .downcast_mut::<S>()
                .expect("element state type mismatch");
            f(state)
        };

        self.window_state.insert_state_box(key, value);
        out
    }

    pub fn observe_model<T>(&mut self, model: &Model<T>, invalidation: Invalidation) {
        self.observe_model_id(model.id(), invalidation);
    }

    pub fn read_model<T: Any, R>(
        &mut self,
        model: &Model<T>,
        invalidation: Invalidation,
        f: impl FnOnce(&mut H, &T) -> R,
    ) -> Result<R, ModelUpdateError> {
        self.observe_model(model, invalidation);
        model.read(&mut *self.app, f)
    }

    pub fn read_model_ref<T: Any, R>(
        &mut self,
        model: &Model<T>,
        invalidation: Invalidation,
        f: impl FnOnce(&T) -> R,
    ) -> Result<R, ModelUpdateError> {
        self.observe_model(model, invalidation);
        self.app.models().read(model, f)
    }

    pub fn get_model_copied<T: Any + Copy>(
        &mut self,
        model: &Model<T>,
        invalidation: Invalidation,
    ) -> Option<T> {
        self.read_model_ref(model, invalidation, |v| *v).ok()
    }

    pub fn get_model_cloned<T: Any + Clone>(
        &mut self,
        model: &Model<T>,
        invalidation: Invalidation,
    ) -> Option<T> {
        self.read_model_ref(model, invalidation, Clone::clone).ok()
    }

    pub fn observe_model_id(&mut self, model: ModelId, invalidation: Invalidation) {
        let id = self
            .window_state
            .current_view_cache_root()
            .unwrap_or_else(|| self.root_id());
        let list = self
            .window_state
            .observed_models_next
            .entry(id)
            .or_default();
        if list
            .iter()
            .any(|(m, inv)| *m == model && *inv == invalidation)
        {
            return;
        }
        list.push((model, invalidation));
    }

    pub fn observe_global<T: Any>(&mut self, invalidation: Invalidation) {
        self.observe_global_id(TypeId::of::<T>(), invalidation);
    }

    pub fn observe_global_id(&mut self, global: TypeId, invalidation: Invalidation) {
        let id = self
            .window_state
            .current_view_cache_root()
            .unwrap_or_else(|| self.root_id());
        let list = self
            .window_state
            .observed_globals_next
            .entry(id)
            .or_default();
        if list
            .iter()
            .any(|(g, inv)| *g == global && *inv == invalidation)
        {
            return;
        }
        list.push((global, invalidation));
    }

    fn observe_environment_query(&mut self, key: EnvironmentQueryKey, invalidation: Invalidation) {
        let id = self
            .window_state
            .current_view_cache_root()
            .unwrap_or_else(|| self.root_id());
        let list = self
            .window_state
            .observed_environment_next
            .entry(id)
            .or_default();
        if list
            .iter()
            .any(|(k, inv)| *k == key && *inv == invalidation)
        {
            return;
        }
        list.push((key, invalidation));
    }

    pub fn observe_layout_query_region(
        &mut self,
        region: GlobalElementId,
        invalidation: Invalidation,
    ) {
        let id = self
            .window_state
            .current_view_cache_root()
            .unwrap_or_else(|| self.root_id());
        let list = self
            .window_state
            .observed_layout_queries_next
            .entry(id)
            .or_default();
        if list
            .iter()
            .any(|(r, inv)| *r == region && *inv == invalidation)
        {
            return;
        }
        list.push((region, invalidation));
    }

    pub fn layout_query_bounds(
        &mut self,
        region: GlobalElementId,
        invalidation: Invalidation,
    ) -> Option<Rect> {
        self.observe_layout_query_region(region, invalidation);
        self.last_bounds_for_element(region)
    }

    pub fn environment_viewport_bounds(&mut self, invalidation: Invalidation) -> Rect {
        self.observe_environment_query(EnvironmentQueryKey::ViewportSize, invalidation);
        self.window_state.committed_viewport_bounds()
    }

    pub fn environment_viewport_width(&mut self, invalidation: Invalidation) -> Px {
        self.environment_viewport_bounds(invalidation).size.width
    }

    pub fn environment_viewport_height(&mut self, invalidation: Invalidation) -> Px {
        self.environment_viewport_bounds(invalidation).size.height
    }

    pub fn environment_scale_factor(&mut self, invalidation: Invalidation) -> f32 {
        self.observe_environment_query(EnvironmentQueryKey::ScaleFactor, invalidation);
        self.window_state.committed_scale_factor()
    }

    pub fn environment_color_scheme(&mut self, invalidation: Invalidation) -> Option<ColorScheme> {
        self.observe_environment_query(EnvironmentQueryKey::ColorScheme, invalidation);
        self.window_state.committed_color_scheme()
    }

    pub fn environment_prefers_reduced_motion(
        &mut self,
        invalidation: Invalidation,
    ) -> Option<bool> {
        self.observe_environment_query(EnvironmentQueryKey::PrefersReducedMotion, invalidation);
        self.window_state.committed_prefers_reduced_motion()
    }

    pub fn environment_text_scale_factor(&mut self, invalidation: Invalidation) -> Option<f32> {
        self.observe_environment_query(EnvironmentQueryKey::TextScaleFactor, invalidation);
        self.window_state.committed_text_scale_factor()
    }

    pub fn environment_prefers_reduced_transparency(
        &mut self,
        invalidation: Invalidation,
    ) -> Option<bool> {
        self.observe_environment_query(
            EnvironmentQueryKey::PrefersReducedTransparency,
            invalidation,
        );
        self.window_state.committed_prefers_reduced_transparency()
    }

    pub fn environment_accent_color(&mut self, invalidation: Invalidation) -> Option<Color> {
        self.observe_environment_query(EnvironmentQueryKey::AccentColor, invalidation);
        self.window_state.committed_accent_color()
    }

    pub fn environment_prefers_contrast(
        &mut self,
        invalidation: Invalidation,
    ) -> Option<ContrastPreference> {
        self.observe_environment_query(EnvironmentQueryKey::PrefersContrast, invalidation);
        self.window_state.committed_contrast_preference()
    }

    pub fn environment_forced_colors_mode(
        &mut self,
        invalidation: Invalidation,
    ) -> Option<ForcedColorsMode> {
        self.observe_environment_query(EnvironmentQueryKey::ForcedColorsMode, invalidation);
        self.window_state.committed_forced_colors_mode()
    }

    pub fn environment_safe_area_insets(&mut self, invalidation: Invalidation) -> Option<Edges> {
        self.observe_environment_query(EnvironmentQueryKey::SafeAreaInsets, invalidation);
        self.window_state.committed_safe_area_insets()
    }

    pub fn environment_occlusion_insets(&mut self, invalidation: Invalidation) -> Option<Edges> {
        self.observe_environment_query(EnvironmentQueryKey::OcclusionInsets, invalidation);
        self.window_state.committed_occlusion_insets()
    }

    pub fn environment_primary_pointer_type(&mut self, invalidation: Invalidation) -> PointerType {
        self.observe_environment_query(EnvironmentQueryKey::PrimaryPointerType, invalidation);
        self.window_state.committed_primary_pointer_type()
    }

    pub fn environment_primary_pointer_can_hover(
        &mut self,
        invalidation: Invalidation,
        default_when_unknown: bool,
    ) -> bool {
        match self.environment_primary_pointer_type(invalidation) {
            PointerType::Touch => false,
            PointerType::Unknown => default_when_unknown,
            PointerType::Mouse | PointerType::Pen => true,
        }
    }

    pub fn environment_primary_pointer_is_coarse(
        &mut self,
        invalidation: Invalidation,
        default_when_unknown: bool,
    ) -> bool {
        match self.environment_primary_pointer_type(invalidation) {
            PointerType::Touch => true,
            PointerType::Unknown => default_when_unknown,
            PointerType::Mouse | PointerType::Pen => false,
        }
    }

    pub fn diagnostics_record_overlay_placement_anchored_panel(
        &mut self,
        overlay_root_name: Option<&str>,
        anchor_element: Option<GlobalElementId>,
        content_element: Option<GlobalElementId>,
        trace: AnchoredPanelLayoutTrace,
    ) {
        #[cfg(feature = "diagnostics")]
        self.window_state.record_overlay_placement_anchored_panel(
            self.frame_id,
            overlay_root_name.map(Arc::<str>::from),
            anchor_element,
            content_element,
            trace,
        );
        #[cfg(not(feature = "diagnostics"))]
        let _ = (overlay_root_name, anchor_element, content_element, trace);
    }

    pub fn diagnostics_record_overlay_placement_placed_rect(
        &mut self,
        overlay_root_name: Option<&str>,
        anchor_element: Option<GlobalElementId>,
        content_element: Option<GlobalElementId>,
        outer: Rect,
        anchor: Rect,
        placed: Rect,
        side: Option<Side>,
    ) {
        #[cfg(feature = "diagnostics")]
        self.window_state.record_overlay_placement_placed_rect(
            self.frame_id,
            overlay_root_name.map(Arc::<str>::from),
            anchor_element,
            content_element,
            outer,
            anchor,
            placed,
            side,
        );
        #[cfg(not(feature = "diagnostics"))]
        let _ = (
            overlay_root_name,
            anchor_element,
            content_element,
            outer,
            anchor,
            placed,
            side,
        );
    }

    pub fn theme(&mut self) -> &Theme {
        self.observe_global::<Theme>(Invalidation::Layout);
        Theme::global(&*self.app)
    }

    #[track_caller]
    pub fn for_each_keyed<T, K: Hash>(
        &mut self,
        items: &[T],
        mut key: impl FnMut(&T) -> K,
        mut f: impl FnMut(&mut Self, usize, &T),
    ) {
        let loc = Location::caller();
        self.scope(|cx| {
            let mut first_dup: Option<(u64, usize, usize)> = None;
            let mut seen: HashMap<u64, usize> = HashMap::new();
            for (index, item) in items.iter().enumerate() {
                let k = key(item);
                let key_hash = stable_hash(&k);
                if first_dup.is_none()
                    && let Some(prev) = seen.insert(key_hash, index)
                {
                    first_dup = Some((key_hash, prev, index));
                }
                cx.keyed(k, |cx| f(cx, index, item));
            }

            if let Some((key_hash, a, b)) = first_dup
                && cfg!(debug_assertions)
                && items.len() > 1
            {
                let element_path: Option<String> = {
                    #[cfg(feature = "diagnostics")]
                    {
                        cx.window_state.debug_path_for_element(cx.root_id())
                    }
                    #[cfg(not(feature = "diagnostics"))]
                    {
                        None
                    }
                };

                tracing::warn!(
                    file = loc.file(),
                    line = loc.line(),
                    column = loc.column(),
                    key_hash = format_args!("{key_hash:#x}"),
                    first_index = a,
                    second_index = b,
                    element_path = element_path.as_deref().unwrap_or("<unknown>"),
                    "duplicate keyed list item key hash; element state may collide"
                );
            }
        });
    }

    #[track_caller]
    pub fn for_each_unkeyed<T: Hash>(
        &mut self,
        items: &[T],
        mut f: impl FnMut(&mut Self, usize, &T),
    ) {
        let loc = Location::caller();
        let list_id = callsite_hash(loc);
        let fingerprints: Vec<u64> = items.iter().map(stable_hash).collect();
        self.window_state
            .cur_unkeyed_fingerprints
            .insert(list_id, fingerprints.clone());

        if let Some(prev) = self.window_state.prev_unkeyed_fingerprints.get(&list_id)
            && prev != &fingerprints
            && items.len() > 1
            && cfg!(debug_assertions)
        {
            let element_path: Option<String> = {
                #[cfg(feature = "diagnostics")]
                {
                    self.window_state.debug_path_for_element(self.root_id())
                }
                #[cfg(not(feature = "diagnostics"))]
                {
                    None
                }
            };
            tracing::warn!(
                list_id = format_args!("{list_id:#x}"),
                file = loc.file(),
                line = loc.line(),
                column = loc.column(),
                element_path = element_path.as_deref().unwrap_or("<unknown>"),
                "unkeyed element list order changed; add explicit keys to preserve state"
            );
        }

        self.scope(|cx| {
            for (index, item) in items.iter().enumerate() {
                let index_key = index as u64;
                cx.enter_with_callsite(loc, list_id, Some(index_key), None, |cx| {
                    f(cx, index, item)
                });
            }
        });
    }

    fn enter_with_callsite<R>(
        &mut self,
        _loc: &'static Location<'static>,
        callsite: u64,
        key_hash: Option<u64>,
        _debug_name: Option<&str>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let parent = self.root_id();
        let counters = self
            .callsite_counters
            .last_mut()
            .expect("callsite counters exist");
        let slot = bump_callsite_counter(counters, callsite);

        let child_salt = key_hash.unwrap_or(slot);
        let id = derive_child_id(parent, callsite, child_salt);

        #[cfg(feature = "diagnostics")]
        self.window_state.record_debug_child(
            self.frame_id,
            parent,
            id,
            _loc.file(),
            _loc.line(),
            _loc.column(),
            key_hash,
            _debug_name,
            slot,
        );

        self.stack.push(id);
        self.callsite_counters.push(CallsiteCounters::new());
        let out = f(self);
        self.callsite_counters.pop();
        self.stack.pop();
        out
    }

    #[track_caller]
    pub fn container<I>(
        &mut self,
        props: ContainerProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Container(props), children)
        })
    }

    /// Creates a `Semantics` layout wrapper around the subtree.
    ///
    /// `Semantics` is intentionally input- and paint-transparent, but it **does** participate in
    /// layout via `SemanticsProps.layout`. Use it when you need a semantics node boundary (tree
    /// structure) or wrapper-only semantics features (e.g. a focusable semantics node).
    ///
    /// If you only need to stamp `test_id` / `label` / `role` / `value` for diagnostics or UI
    /// automation, prefer attaching `SemanticsDecoration` to an existing element via
    /// `AnyElement::attach_semantics(...)` to avoid introducing a layout node.
    #[track_caller]
    pub fn semantics<I>(
        &mut self,
        props: crate::element::SemanticsProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Semantics(props), children)
        })
    }

    #[track_caller]
    pub fn semantic_flex<I>(
        &mut self,
        props: crate::element::SemanticFlexProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::SemanticFlex(props), children)
        })
    }

    #[track_caller]
    pub fn focus_scope<I>(
        &mut self,
        props: crate::element::FocusScopeProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::FocusScope(props), children)
        })
    }

    #[track_caller]
    pub fn view_cache<I>(
        &mut self,
        props: crate::element::ViewCacheProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let should_reuse = cx
                .window_state
                .node_entry(id)
                .map(|e| e.node)
                .and_then(|node| cx.view_cache_should_reuse.as_mut().map(|f| f(node)))
                .unwrap_or(false);

            let theme_revision = Theme::global(&*cx.app).revision();
            let scale_factor = cx
                .app
                .global::<WindowMetricsService>()
                .and_then(|svc| svc.scale_factor(cx.window))
                .unwrap_or(1.0);

            // View-cache keys must incorporate any "external" dependencies that can affect the
            // cached subtree without triggering a normal invalidation walk (e.g. environment
            // queries, layout queries).
            //
            // When reusing, depend on the previously-recorded deps for this cache root. When
            // re-rendering, the key is refreshed from `*_next` after the closure runs.
            let scale_bits = scale_factor.to_bits();
            let deps_key_rendered = if should_reuse {
                (
                    cx.window_state.environment_deps_fingerprint_rendered(id),
                    cx.window_state.layout_query_deps_fingerprint_rendered(id),
                )
            } else {
                (0, 0)
            };
            let key = stable_hash(&(
                theme_revision,
                scale_bits,
                props.cache_key,
                deps_key_rendered.0,
                deps_key_rendered.1,
            ));

            let key_matches = if should_reuse {
                let matches = cx.window_state.view_cache_key_matches_and_touch(id, key);
                if !matches {
                    cx.window_state.record_view_cache_key_mismatch(id);
                }
                matches
            } else {
                false
            };

            let reuse = should_reuse && key_matches;

            let children: Vec<AnyElement> = if reuse {
                cx.window_state.mark_view_cache_reuse_root(id);
                cx.window_state.touch_view_cache_state_keys_if_recorded(id);
                cx.window_state
                    .touch_observed_models_for_element_if_recorded(id);
                cx.window_state
                    .touch_observed_globals_for_element_if_recorded(id);
                cx.window_state
                    .touch_observed_environment_for_element_if_recorded(id);
                cx.window_state
                    .touch_observed_layout_queries_for_element_if_recorded(id);
                Vec::new()
            } else {
                cx.window_state.begin_view_cache_scope(id);
                let built = f(cx);
                let children = cx.collect_children(built);
                cx.window_state.end_view_cache_scope(id);

                let deps_key_next = (
                    cx.window_state.environment_deps_fingerprint_next(id),
                    cx.window_state.layout_query_deps_fingerprint_next(id),
                );
                let key = stable_hash(&(
                    theme_revision,
                    scale_bits,
                    props.cache_key,
                    deps_key_next.0,
                    deps_key_next.1,
                ));
                cx.window_state.set_view_cache_key(id, key);
                children
            };
            cx.new_any_element(id, ElementKind::ViewCache(props), children)
        })
    }

    #[track_caller]
    pub fn focus_scope_with_id<I>(
        &mut self,
        props: crate::element::FocusScopeProps,
        f: impl FnOnce(&mut Self, GlobalElementId) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx, id);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::FocusScope(props), children)
        })
    }

    #[track_caller]
    pub fn layout_query_region_with_id<I>(
        &mut self,
        props: LayoutQueryRegionProps,
        f: impl FnOnce(&mut Self, GlobalElementId) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let name = props.name.clone();
            cx.with_state_for(id, LayoutQueryRegionMarker::default, |marker| {
                marker.name = name.clone();
            });
            let built = f(cx, id);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::LayoutQueryRegion(props), children)
        })
    }

    #[track_caller]
    pub fn layout_query_region<I>(
        &mut self,
        props: LayoutQueryRegionProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.layout_query_region_with_id(props, |cx, _id| f(cx))
    }

    /// Creates a `Semantics` layout wrapper around the subtree and passes its element id.
    ///
    /// See [`Self::semantics`] for guidance on when to use `Semantics` vs `attach_semantics`
    /// (`SemanticsDecoration`).
    #[track_caller]
    pub fn semantics_with_id<I>(
        &mut self,
        props: crate::element::SemanticsProps,
        f: impl FnOnce(&mut Self, GlobalElementId) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx, id);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Semantics(props), children)
        })
    }

    #[track_caller]
    pub fn opacity<I>(&mut self, opacity: f32, f: impl FnOnce(&mut Self) -> I) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        let props = OpacityProps {
            opacity: opacity.clamp(0.0, 1.0),
            ..Default::default()
        };
        self.opacity_props(props, f)
    }

    #[track_caller]
    pub fn opacity_props<I>(
        &mut self,
        props: OpacityProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Opacity(props), children)
        })
    }

    #[track_caller]
    pub fn effect_layer<I>(
        &mut self,
        mode: EffectMode,
        chain: EffectChain,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.effect_layer_props(
            EffectLayerProps {
                mode,
                chain,
                ..Default::default()
            },
            f,
        )
    }

    #[track_caller]
    pub fn effect_layer_props<I>(
        &mut self,
        props: EffectLayerProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::EffectLayer(props), children)
        })
    }

    #[track_caller]
    pub fn visual_transform<I>(
        &mut self,
        transform: fret_core::Transform2D,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.visual_transform_props(
            VisualTransformProps {
                layout: LayoutStyle::default(),
                transform,
            },
            f,
        )
    }

    #[track_caller]
    pub fn render_transform<I>(
        &mut self,
        transform: fret_core::Transform2D,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.render_transform_props(
            crate::element::RenderTransformProps {
                layout: LayoutStyle::default(),
                transform,
            },
            f,
        )
    }

    #[track_caller]
    pub fn fractional_render_transform<I>(
        &mut self,
        translate_x_fraction: f32,
        translate_y_fraction: f32,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.fractional_render_transform_props(
            crate::element::FractionalRenderTransformProps {
                layout: LayoutStyle::default(),
                translate_x_fraction,
                translate_y_fraction,
            },
            f,
        )
    }

    #[track_caller]
    pub fn visual_transform_props<I>(
        &mut self,
        props: VisualTransformProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::VisualTransform(props), children)
        })
    }

    #[track_caller]
    pub fn render_transform_props<I>(
        &mut self,
        props: crate::element::RenderTransformProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::RenderTransform(props), children)
        })
    }

    #[track_caller]
    pub fn fractional_render_transform_props<I>(
        &mut self,
        props: crate::element::FractionalRenderTransformProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::FractionalRenderTransform(props), children)
        })
    }

    #[track_caller]
    pub fn anchored_props<I>(
        &mut self,
        props: crate::element::AnchoredProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Anchored(props), children)
        })
    }

    #[track_caller]
    pub fn interactivity_gate<I>(
        &mut self,
        present: bool,
        interactive: bool,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.interactivity_gate_props(
            InteractivityGateProps {
                present,
                interactive,
                ..Default::default()
            },
            f,
        )
    }

    #[track_caller]
    pub fn interactivity_gate_props<I>(
        &mut self,
        props: InteractivityGateProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::InteractivityGate(props), children)
        })
    }

    #[track_caller]
    pub fn hit_test_gate<I>(&mut self, hit_test: bool, f: impl FnOnce(&mut Self) -> I) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.hit_test_gate_props(
            HitTestGateProps {
                hit_test,
                ..Default::default()
            },
            f,
        )
    }

    #[track_caller]
    pub fn hit_test_gate_props<I>(
        &mut self,
        props: HitTestGateProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::HitTestGate(props), children)
        })
    }

    #[track_caller]
    pub fn focus_traversal_gate<I>(
        &mut self,
        traverse: bool,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.focus_traversal_gate_props(
            FocusTraversalGateProps {
                traverse,
                ..Default::default()
            },
            f,
        )
    }

    #[track_caller]
    pub fn focus_traversal_gate_props<I>(
        &mut self,
        props: FocusTraversalGateProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::FocusTraversalGate(props), children)
        })
    }

    #[track_caller]
    pub fn pressable<I>(
        &mut self,
        props: PressableProps,
        f: impl FnOnce(&mut Self, PressableState) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let hovered = cx.window_state.hovered_pressable == Some(id);
            let hovered_raw = cx.window_state.hovered_pressable_raw == Some(id);
            let hovered_raw_below_barrier =
                cx.window_state.hovered_pressable_raw_below_barrier == Some(id);
            let pressed = cx.window_state.pressed_pressable == Some(id);
            let focused = cx.window_state.focused_element == Some(id);
            cx.pressable_clear_on_activate();
            cx.pressable_clear_on_hover_change();
            let built = f(
                cx,
                PressableState {
                    hovered,
                    hovered_raw,
                    hovered_raw_below_barrier,
                    pressed,
                    focused,
                },
            );
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Pressable(props), children)
        })
    }

    #[track_caller]
    pub fn pressable_with_id<I>(
        &mut self,
        props: PressableProps,
        f: impl FnOnce(&mut Self, PressableState, GlobalElementId) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let hovered = cx.window_state.hovered_pressable == Some(id);
            let hovered_raw = cx.window_state.hovered_pressable_raw == Some(id);
            let hovered_raw_below_barrier =
                cx.window_state.hovered_pressable_raw_below_barrier == Some(id);
            let pressed = cx.window_state.pressed_pressable == Some(id);
            let focused = cx.window_state.focused_element == Some(id);
            cx.pressable_clear_on_activate();
            cx.pressable_clear_on_hover_change();
            let built = f(
                cx,
                PressableState {
                    hovered,
                    hovered_raw,
                    hovered_raw_below_barrier,
                    pressed,
                    focused,
                },
                id,
            );
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Pressable(props), children)
        })
    }

    #[track_caller]
    pub fn pressable_with_id_props<I>(
        &mut self,
        f: impl FnOnce(&mut Self, PressableState, GlobalElementId) -> (PressableProps, I),
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let hovered = cx.window_state.hovered_pressable == Some(id);
            let hovered_raw = cx.window_state.hovered_pressable_raw == Some(id);
            let hovered_raw_below_barrier =
                cx.window_state.hovered_pressable_raw_below_barrier == Some(id);
            let pressed = cx.window_state.pressed_pressable == Some(id);
            let focused = cx.window_state.focused_element == Some(id);
            cx.pressable_clear_on_activate();
            cx.pressable_clear_on_pointer_down();
            cx.pressable_clear_on_hover_change();
            let (props, children) = f(
                cx,
                PressableState {
                    hovered,
                    hovered_raw,
                    hovered_raw_below_barrier,
                    pressed,
                    focused,
                },
                id,
            );
            let children = cx.collect_children(children);
            cx.new_any_element(id, ElementKind::Pressable(props), children)
        })
    }

    /// Register a component-owned activation handler for the current pressable element.
    ///
    /// This is a policy hook mechanism (ADR 0074): components decide what activation does (model
    /// writes, overlay requests, command dispatch), while the runtime remains mechanism-only.
    pub fn pressable_on_activate(&mut self, handler: OnActivate) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_activate = Some(handler);
        });
    }

    pub fn pressable_on_activate_for(&mut self, element: GlobalElementId, handler: OnActivate) {
        self.with_state_for(element, PressableActionHooks::default, |hooks| {
            hooks.on_activate = Some(handler);
        });
    }

    pub fn pressable_add_on_activate(&mut self, handler: OnActivate) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_activate = match hooks.on_activate.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, reason| {
                        prev(host, cx, reason);
                        next(host, cx, reason);
                    }))
                }
            };
        });
    }

    pub fn pressable_add_on_activate_for(&mut self, element: GlobalElementId, handler: OnActivate) {
        self.with_state_for(element, PressableActionHooks::default, |hooks| {
            hooks.on_activate = match hooks.on_activate.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, reason| {
                        prev(host, cx, reason);
                        next(host, cx, reason);
                    }))
                }
            };
        });
    }

    pub fn pressable_clear_on_activate(&mut self) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_activate = None;
        });
    }

    /// Register a component-owned pointer down handler for the current pressable element.
    ///
    /// This is a policy hook mechanism (ADR 0074): components can opt into Radix-style "select on
    /// mouse down" semantics without changing the default click-like activation behavior.
    pub fn pressable_on_pointer_down(&mut self, handler: OnPressablePointerDown) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_pointer_down = Some(handler);
        });
    }

    pub fn pressable_on_pointer_move(&mut self, handler: OnPressablePointerMove) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_pointer_move = Some(handler);
        });
    }

    pub fn pressable_on_pointer_up(&mut self, handler: OnPressablePointerUp) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_pointer_up = Some(handler);
        });
    }

    pub fn pressable_on_pointer_down_for(
        &mut self,
        element: GlobalElementId,
        handler: OnPressablePointerDown,
    ) {
        self.with_state_for(element, PressableActionHooks::default, |hooks| {
            hooks.on_pointer_down = Some(handler);
        });
    }

    pub fn pressable_on_pointer_move_for(
        &mut self,
        element: GlobalElementId,
        handler: OnPressablePointerMove,
    ) {
        self.with_state_for(element, PressableActionHooks::default, |hooks| {
            hooks.on_pointer_move = Some(handler);
        });
    }

    pub fn pressable_on_pointer_up_for(
        &mut self,
        element: GlobalElementId,
        handler: OnPressablePointerUp,
    ) {
        self.with_state_for(element, PressableActionHooks::default, |hooks| {
            hooks.on_pointer_up = Some(handler);
        });
    }

    pub fn pressable_add_on_pointer_down(&mut self, handler: OnPressablePointerDown) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_pointer_down = match hooks.on_pointer_down.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, down| {
                        let prev_result = prev(host, cx, down);
                        let next_result = next(host, cx, down);
                        use crate::action::PressablePointerDownResult as R;
                        match (prev_result, next_result) {
                            (R::SkipDefaultAndStopPropagation, _)
                            | (_, R::SkipDefaultAndStopPropagation) => {
                                R::SkipDefaultAndStopPropagation
                            }
                            (R::SkipDefault, _) | (_, R::SkipDefault) => R::SkipDefault,
                            _ => R::Continue,
                        }
                    }))
                }
            };
        });
    }

    pub fn pressable_add_on_pointer_move(&mut self, handler: OnPressablePointerMove) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_pointer_move = match hooks.on_pointer_move.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, mv| {
                        let prev_handled = prev(host, cx, mv);
                        let next_handled = next(host, cx, mv);
                        prev_handled || next_handled
                    }))
                }
            };
        });
    }

    pub fn pressable_add_on_pointer_up(&mut self, handler: OnPressablePointerUp) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_pointer_up = match hooks.on_pointer_up.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, up| {
                        let prev_result = prev(host, cx, up);
                        let next_result = next(host, cx, up);
                        match (prev_result, next_result) {
                            (PressablePointerUpResult::SkipActivate, _)
                            | (_, PressablePointerUpResult::SkipActivate) => {
                                PressablePointerUpResult::SkipActivate
                            }
                            _ => PressablePointerUpResult::Continue,
                        }
                    }))
                }
            };
        });
    }

    pub fn pressable_add_on_pointer_down_for(
        &mut self,
        element: GlobalElementId,
        handler: OnPressablePointerDown,
    ) {
        self.with_state_for(element, PressableActionHooks::default, |hooks| {
            hooks.on_pointer_down = match hooks.on_pointer_down.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, down| {
                        let prev_result = prev(host, cx, down);
                        let next_result = next(host, cx, down);
                        use crate::action::PressablePointerDownResult as R;
                        match (prev_result, next_result) {
                            (R::SkipDefaultAndStopPropagation, _)
                            | (_, R::SkipDefaultAndStopPropagation) => {
                                R::SkipDefaultAndStopPropagation
                            }
                            (R::SkipDefault, _) | (_, R::SkipDefault) => R::SkipDefault,
                            _ => R::Continue,
                        }
                    }))
                }
            };
        });
    }

    pub fn pressable_add_on_pointer_move_for(
        &mut self,
        element: GlobalElementId,
        handler: OnPressablePointerMove,
    ) {
        self.with_state_for(element, PressableActionHooks::default, |hooks| {
            hooks.on_pointer_move = match hooks.on_pointer_move.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, mv| {
                        let prev_handled = prev(host, cx, mv);
                        let next_handled = next(host, cx, mv);
                        prev_handled || next_handled
                    }))
                }
            };
        });
    }

    pub fn pressable_add_on_pointer_up_for(
        &mut self,
        element: GlobalElementId,
        handler: OnPressablePointerUp,
    ) {
        self.with_state_for(element, PressableActionHooks::default, |hooks| {
            hooks.on_pointer_up = match hooks.on_pointer_up.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, up| {
                        let prev_result = prev(host, cx, up);
                        let next_result = next(host, cx, up);
                        match (prev_result, next_result) {
                            (PressablePointerUpResult::SkipActivate, _)
                            | (_, PressablePointerUpResult::SkipActivate) => {
                                PressablePointerUpResult::SkipActivate
                            }
                            _ => PressablePointerUpResult::Continue,
                        }
                    }))
                }
            };
        });
    }

    pub fn pressable_clear_on_pointer_down(&mut self) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_pointer_down = None;
        });
    }

    pub fn pressable_clear_on_pointer_move(&mut self) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_pointer_move = None;
        });
    }

    pub fn pressable_clear_on_pointer_up(&mut self) {
        self.with_state(PressableActionHooks::default, |hooks| {
            hooks.on_pointer_up = None;
        });
    }

    /// Register a component-owned hover change handler for the current pressable element.
    ///
    /// This is a mechanism-only hook: the runtime tracks hover deterministically and invokes
    /// component code on hover transitions, without baking hover policy into `fret-ui`.
    pub fn pressable_on_hover_change(&mut self, handler: OnHoverChange) {
        self.with_state(PressableHoverActionHooks::default, |hooks| {
            hooks.on_hover_change = Some(handler);
        });
    }

    pub fn pressable_add_on_hover_change(&mut self, handler: OnHoverChange) {
        self.with_state(PressableHoverActionHooks::default, |hooks| {
            hooks.on_hover_change = match hooks.on_hover_change.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, hovered| {
                        prev(host, cx, hovered);
                        next(host, cx, hovered);
                    }))
                }
            };
        });
    }

    pub fn pressable_clear_on_hover_change(&mut self) {
        self.with_state(PressableHoverActionHooks::default, |hooks| {
            hooks.on_hover_change = None;
        });
    }

    #[track_caller]
    pub fn pointer_region<I>(
        &mut self,
        props: PointerRegionProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.pointer_region_clear_on_pointer_down();
            cx.pointer_region_clear_on_pointer_move();
            cx.pointer_region_clear_on_pointer_up();
            cx.pointer_region_clear_on_wheel();
            cx.pointer_region_clear_on_pinch_gesture();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::PointerRegion(props), children)
        })
    }

    #[track_caller]
    pub fn text_input_region<I>(
        &mut self,
        props: crate::element::TextInputRegionProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.text_input_region_clear_on_text_input();
            cx.text_input_region_clear_on_ime();
            cx.text_input_region_clear_on_clipboard_text();
            cx.text_input_region_clear_on_clipboard_unavailable();
            cx.text_input_region_clear_on_set_selection();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::TextInputRegion(props), children)
        })
    }

    pub fn internal_drag_region<I>(
        &mut self,
        props: crate::element::InternalDragRegionProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.internal_drag_region_clear_on_internal_drag();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::InternalDragRegion(props), children)
        })
    }

    pub fn internal_drag_region_on_internal_drag(
        &mut self,
        handler: crate::action::OnInternalDrag,
    ) {
        self.with_state(crate::action::InternalDragActionHooks::default, |hooks| {
            hooks.on_internal_drag = Some(handler);
        });
    }

    pub fn internal_drag_region_clear_on_internal_drag(&mut self) {
        self.with_state(crate::action::InternalDragActionHooks::default, |hooks| {
            hooks.on_internal_drag = None;
        });
    }

    /// Register a component-owned pointer down handler for the current pointer region element.
    ///
    /// This is a mechanism-only hook point: components decide what a pointer down does (open a
    /// context menu, start a drag, request focus, etc.), while the runtime remains policy-free.
    pub fn pointer_region_on_pointer_down(&mut self, handler: OnPointerDown) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pointer_down = Some(handler);
        });
    }

    pub fn pointer_region_add_on_pointer_down(&mut self, handler: OnPointerDown) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pointer_down = match hooks.on_pointer_down.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, down| {
                        prev(host, cx, down) || next(host, cx, down)
                    }))
                }
            };
        });
    }

    pub fn pointer_region_clear_on_pointer_down(&mut self) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pointer_down = None;
        });
    }

    /// Register a component-owned pointer move handler for the current pointer region element.
    ///
    /// This hook is invoked when the pointer region receives `PointerEvent::Move` events via
    /// normal hit-testing or pointer capture.
    pub fn pointer_region_on_pointer_move(&mut self, handler: OnPointerMove) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pointer_move = Some(handler);
        });
    }

    pub fn pointer_region_add_on_pointer_move(&mut self, handler: OnPointerMove) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pointer_move = match hooks.on_pointer_move.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, mv| {
                        prev(host, cx, mv) || next(host, cx, mv)
                    }))
                }
            };
        });
    }

    pub fn pointer_region_clear_on_pointer_move(&mut self) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pointer_move = None;
        });
    }

    /// Register a component-owned pointer up handler for the current pointer region element.
    ///
    /// This hook is invoked when the pointer region receives `PointerEvent::Up` events via
    /// normal hit-testing or pointer capture.
    pub fn pointer_region_on_pointer_up(&mut self, handler: OnPointerUp) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pointer_up = Some(handler);
        });
    }

    /// Register a component-owned pointer cancel handler for the current pointer region element.
    ///
    /// This hook is invoked when the runtime receives `Event::PointerCancel` for a pointer stream
    /// that was previously interacting with this region (typically via pointer capture).
    pub fn pointer_region_on_pointer_cancel(&mut self, handler: OnPointerCancel) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pointer_cancel = Some(handler);
        });
    }

    pub fn pointer_region_on_wheel(&mut self, handler: OnWheel) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_wheel = Some(handler);
        });
    }

    pub fn pointer_region_on_pinch_gesture(&mut self, handler: OnPinchGesture) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pinch_gesture = Some(handler);
        });
    }

    pub fn pointer_region_add_on_pointer_up(&mut self, handler: OnPointerUp) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pointer_up = match hooks.on_pointer_up.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, up| {
                        prev(host, cx, up) || next(host, cx, up)
                    }))
                }
            };
        });
    }

    pub fn pointer_region_add_on_wheel(&mut self, handler: OnWheel) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_wheel = match hooks.on_wheel.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, wheel| {
                        prev(host, cx, wheel) || next(host, cx, wheel)
                    }))
                }
            };
        });
    }

    pub fn pointer_region_add_on_pinch_gesture(&mut self, handler: OnPinchGesture) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pinch_gesture = match hooks.on_pinch_gesture.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, pinch| {
                        prev(host, cx, pinch) || next(host, cx, pinch)
                    }))
                }
            };
        });
    }

    pub fn pointer_region_clear_on_pointer_up(&mut self) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pointer_up = None;
        });
    }

    pub fn pointer_region_clear_on_wheel(&mut self) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_wheel = None;
        });
    }

    pub fn pointer_region_clear_on_pinch_gesture(&mut self) {
        self.with_state(PointerActionHooks::default, |hooks| {
            hooks.on_pinch_gesture = None;
        });
    }

    pub fn text_input_region_on_text_input(
        &mut self,
        handler: crate::action::OnTextInputRegionTextInput,
    ) {
        self.with_state(
            crate::action::TextInputRegionActionHooks::default,
            |hooks| {
                hooks.on_text_input = Some(handler);
            },
        );
    }

    pub fn text_input_region_clear_on_text_input(&mut self) {
        self.with_state(
            crate::action::TextInputRegionActionHooks::default,
            |hooks| {
                hooks.on_text_input = None;
            },
        );
    }

    pub fn text_input_region_on_ime(&mut self, handler: crate::action::OnTextInputRegionIme) {
        self.with_state(
            crate::action::TextInputRegionActionHooks::default,
            |hooks| {
                hooks.on_ime = Some(handler);
            },
        );
    }

    pub fn text_input_region_clear_on_ime(&mut self) {
        self.with_state(
            crate::action::TextInputRegionActionHooks::default,
            |hooks| {
                hooks.on_ime = None;
            },
        );
    }

    pub fn text_input_region_on_clipboard_text(
        &mut self,
        handler: crate::action::OnTextInputRegionClipboardText,
    ) {
        self.with_state(
            crate::action::TextInputRegionActionHooks::default,
            |hooks| {
                hooks.on_clipboard_text = Some(handler);
            },
        );
    }

    pub fn text_input_region_clear_on_clipboard_text(&mut self) {
        self.with_state(
            crate::action::TextInputRegionActionHooks::default,
            |hooks| {
                hooks.on_clipboard_text = None;
            },
        );
    }

    pub fn text_input_region_on_clipboard_unavailable(
        &mut self,
        handler: crate::action::OnTextInputRegionClipboardUnavailable,
    ) {
        self.with_state(
            crate::action::TextInputRegionActionHooks::default,
            |hooks| {
                hooks.on_clipboard_unavailable = Some(handler);
            },
        );
    }

    pub fn text_input_region_clear_on_clipboard_unavailable(&mut self) {
        self.with_state(
            crate::action::TextInputRegionActionHooks::default,
            |hooks| {
                hooks.on_clipboard_unavailable = None;
            },
        );
    }

    pub fn text_input_region_on_set_selection(
        &mut self,
        handler: crate::action::OnTextInputRegionSetSelection,
    ) {
        self.with_state(
            crate::action::TextInputRegionActionHooks::default,
            |hooks| {
                hooks.on_set_selection = Some(handler);
            },
        );
    }

    pub fn text_input_region_clear_on_set_selection(&mut self) {
        self.with_state(
            crate::action::TextInputRegionActionHooks::default,
            |hooks| {
                hooks.on_set_selection = None;
            },
        );
    }

    pub fn key_on_key_down_for(&mut self, element: GlobalElementId, handler: OnKeyDown) {
        self.with_state_for(element, KeyActionHooks::default, |hooks| {
            hooks.on_key_down = Some(handler);
        });
    }

    pub fn key_add_on_key_down_for(&mut self, element: GlobalElementId, handler: OnKeyDown) {
        self.with_state_for(element, KeyActionHooks::default, |hooks| {
            hooks.on_key_down = match hooks.on_key_down.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, down| {
                        prev(host, cx, down) || next(host, cx, down)
                    }))
                }
            };
        });
    }

    pub fn key_prepend_on_key_down_for(&mut self, element: GlobalElementId, handler: OnKeyDown) {
        self.with_state_for(element, KeyActionHooks::default, |hooks| {
            hooks.on_key_down = match hooks.on_key_down.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, down| {
                        next(host, cx, down) || prev(host, cx, down)
                    }))
                }
            };
        });
    }

    pub fn key_clear_on_key_down_for(&mut self, element: GlobalElementId) {
        self.with_state_for(element, KeyActionHooks::default, |hooks| {
            hooks.on_key_down = None;
        });
    }

    pub fn command_on_command_for(&mut self, element: GlobalElementId, handler: OnCommand) {
        self.with_state_for(element, CommandActionHooks::default, |hooks| {
            hooks.on_command = Some(handler);
        });
    }

    pub fn command_add_on_command_for(&mut self, element: GlobalElementId, handler: OnCommand) {
        self.with_state_for(element, CommandActionHooks::default, |hooks| {
            hooks.on_command = match hooks.on_command.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, command| {
                        prev(host, cx, command.clone()) || next(host, cx, command)
                    }))
                }
            };
        });
    }

    pub fn command_prepend_on_command_for(&mut self, element: GlobalElementId, handler: OnCommand) {
        self.with_state_for(element, CommandActionHooks::default, |hooks| {
            hooks.on_command = match hooks.on_command.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, command| {
                        next(host, cx, command.clone()) || prev(host, cx, command)
                    }))
                }
            };
        });
    }

    pub fn command_clear_on_command_for(&mut self, element: GlobalElementId) {
        self.with_state_for(element, CommandActionHooks::default, |hooks| {
            hooks.on_command = None;
        });
    }

    pub fn command_on_command_availability_for(
        &mut self,
        element: GlobalElementId,
        handler: OnCommandAvailability,
    ) {
        self.with_state_for(element, CommandAvailabilityActionHooks::default, |hooks| {
            hooks.on_command_availability = Some(handler);
        });
    }

    pub fn command_add_on_command_availability_for(
        &mut self,
        element: GlobalElementId,
        handler: OnCommandAvailability,
    ) {
        self.with_state_for(element, CommandAvailabilityActionHooks::default, |hooks| {
            hooks.on_command_availability = match hooks.on_command_availability.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, command| {
                        let availability = prev(host, cx.clone(), command.clone());
                        if availability != crate::widget::CommandAvailability::NotHandled {
                            return availability;
                        }
                        next(host, cx, command)
                    }))
                }
            };
        });
    }

    pub fn command_prepend_on_command_availability_for(
        &mut self,
        element: GlobalElementId,
        handler: OnCommandAvailability,
    ) {
        self.with_state_for(element, CommandAvailabilityActionHooks::default, |hooks| {
            hooks.on_command_availability = match hooks.on_command_availability.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, command| {
                        let availability = next(host, cx.clone(), command.clone());
                        if availability != crate::widget::CommandAvailability::NotHandled {
                            return availability;
                        }
                        prev(host, cx, command)
                    }))
                }
            };
        });
    }

    pub fn command_clear_on_command_availability_for(&mut self, element: GlobalElementId) {
        self.with_state_for(element, CommandAvailabilityActionHooks::default, |hooks| {
            hooks.on_command_availability = None;
        });
    }

    pub fn timer_on_timer_for(&mut self, element: GlobalElementId, handler: OnTimer) {
        self.with_state_for(element, TimerActionHooks::default, |hooks| {
            hooks.on_timer = Some(handler);
        });
    }

    pub fn timer_add_on_timer_for(&mut self, element: GlobalElementId, handler: OnTimer) {
        self.with_state_for(element, TimerActionHooks::default, |hooks| {
            hooks.on_timer = match hooks.on_timer.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, token| {
                        prev(host, cx, token) || next(host, cx, token)
                    }))
                }
            };
        });
    }

    pub fn timer_clear_on_timer_for(&mut self, element: GlobalElementId) {
        self.with_state_for(element, TimerActionHooks::default, |hooks| {
            hooks.on_timer = None;
        });
    }

    /// Register a component-owned dismiss handler for the current dismissible root element.
    ///
    /// This is intended for overlay policy code that composes
    /// `render_dismissible_root_with_hooks(...)` and
    /// wants full control over dismissal semantics (ADR 0074).
    pub fn dismissible_on_dismiss_request(&mut self, handler: OnDismissRequest) {
        self.with_state(DismissibleActionHooks::default, |hooks| {
            hooks.on_dismiss_request = Some(handler);
        });
    }

    /// Register a component-owned pointer-move observer for the current dismissible root element.
    ///
    /// This is used for overlay policies that need global pointer movement (e.g. submenu
    /// safe-hover corridors) without making the overlay hit-testable outside its content.
    pub fn dismissible_on_pointer_move(&mut self, handler: OnDismissiblePointerMove) {
        self.with_state(DismissibleActionHooks::default, |hooks| {
            hooks.on_pointer_move = Some(handler);
        });
    }

    pub fn dismissible_add_on_dismiss_request(&mut self, handler: OnDismissRequest) {
        self.with_state(DismissibleActionHooks::default, |hooks| {
            hooks.on_dismiss_request = match hooks.on_dismiss_request.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, req| {
                        prev(host, cx, req);
                        next(host, cx, req);
                    }))
                }
            };
        });
    }

    pub fn dismissible_add_on_pointer_move(&mut self, handler: OnDismissiblePointerMove) {
        self.with_state(DismissibleActionHooks::default, |hooks| {
            hooks.on_pointer_move = match hooks.on_pointer_move.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, mv| {
                        prev(host, cx, mv) || next(host, cx, mv)
                    }))
                }
            };
        });
    }

    pub fn dismissible_clear_on_dismiss_request(&mut self) {
        self.with_state(DismissibleActionHooks::default, |hooks| {
            hooks.on_dismiss_request = None;
        });
    }

    pub fn dismissible_clear_on_pointer_move(&mut self) {
        self.with_state(DismissibleActionHooks::default, |hooks| {
            hooks.on_pointer_move = None;
        });
    }

    /// Register a component-owned roving active-change handler for the current roving element.
    ///
    /// This hook is invoked when the roving container changes focus among its children due to
    /// keyboard navigation (arrow keys, Home/End, or typeahead).
    ///
    /// Components can implement “automatic activation” (e.g. Tabs) by updating selection models
    /// here, keeping selection policy out of the runtime (ADR 0074).
    pub fn roving_on_active_change(&mut self, handler: OnRovingActiveChange) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_active_change = Some(handler);
        });
    }

    pub fn roving_add_on_active_change(&mut self, handler: OnRovingActiveChange) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_active_change = match hooks.on_active_change.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, idx| {
                        prev(host, cx, idx);
                        next(host, cx, idx);
                    }))
                }
            };
        });
    }

    pub fn roving_clear_on_active_change(&mut self) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_active_change = None;
        });
    }

    /// Register a component-owned roving typeahead handler for the current roving element.
    ///
    /// When set, the runtime forwards alphanumeric key presses to this handler so components can
    /// decide how typeahead should work (buffering, prefix matching, wrapping, etc.).
    pub fn roving_on_typeahead(&mut self, handler: OnRovingTypeahead) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_typeahead = Some(handler);
        });
    }

    pub fn roving_add_on_typeahead(&mut self, handler: OnRovingTypeahead) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_typeahead = match hooks.on_typeahead.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, it| {
                        prev(host, cx, it.clone()).or_else(|| next(host, cx, it))
                    }))
                }
            };
        });
    }

    pub fn roving_clear_on_typeahead(&mut self) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_typeahead = None;
        });
    }

    pub fn roving_on_key_down(&mut self, handler: OnKeyDown) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_key_down = Some(handler);
        });
    }

    pub fn roving_add_on_key_down(&mut self, handler: OnKeyDown) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_key_down = match hooks.on_key_down.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, down| {
                        prev(host, cx, down) || next(host, cx, down)
                    }))
                }
            };
        });
    }

    pub fn roving_clear_on_key_down(&mut self) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_key_down = None;
        });
    }

    /// Register a component-owned roving navigation handler for the current roving element.
    ///
    /// This is invoked for key down events that bubble through the roving container so component
    /// code can decide which child should become focused (arrow keys, Home/End, etc.).
    pub fn roving_on_navigate(&mut self, handler: OnRovingNavigate) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_navigate = Some(handler);
        });
    }

    pub fn roving_add_on_navigate(&mut self, handler: OnRovingNavigate) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_navigate = match hooks.on_navigate.clone() {
                None => Some(handler),
                Some(prev) => {
                    let next = handler.clone();
                    Some(Arc::new(move |host, cx, it| {
                        match prev(host, cx, it.clone()) {
                            crate::action::RovingNavigateResult::NotHandled => next(host, cx, it),
                            other => other,
                        }
                    }))
                }
            };
        });
    }

    pub fn roving_clear_on_navigate(&mut self) {
        self.with_state(RovingActionHooks::default, |hooks| {
            hooks.on_navigate = None;
        });
    }

    #[track_caller]
    pub fn stack<I>(&mut self, f: impl FnOnce(&mut Self) -> I) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.stack_props(StackProps::default(), f)
    }

    #[track_caller]
    pub fn stack_props<I>(
        &mut self,
        props: StackProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Stack(props), children)
        })
    }

    #[track_caller]
    pub fn column<I>(&mut self, props: ColumnProps, f: impl FnOnce(&mut Self) -> I) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Column(props), children)
        })
    }

    #[track_caller]
    pub fn row<I>(&mut self, props: RowProps, f: impl FnOnce(&mut Self) -> I) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Row(props), children)
        })
    }

    #[track_caller]
    pub fn spacer(&mut self, props: SpacerProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::Spacer(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn text(&mut self, text: impl Into<std::sync::Arc<str>>) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::Text(TextProps::new(text)), Vec::new())
        })
    }

    #[track_caller]
    pub fn text_props(&mut self, props: TextProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::Text(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn styled_text(&mut self, rich: fret_core::AttributedText) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(
                id,
                ElementKind::StyledText(StyledTextProps::new(rich)),
                Vec::new(),
            )
        })
    }

    #[track_caller]
    pub fn styled_text_props(&mut self, props: StyledTextProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::StyledText(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn selectable_text(&mut self, rich: fret_core::AttributedText) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(
                id,
                ElementKind::SelectableText(SelectableTextProps::new(rich)),
                Vec::new(),
            )
        })
    }

    #[track_caller]
    pub fn selectable_text_props(&mut self, props: SelectableTextProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::SelectableText(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn text_input(&mut self, props: TextInputProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::TextInput(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn text_input_with_id_props(
        &mut self,
        f: impl FnOnce(&mut Self, GlobalElementId) -> TextInputProps,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            let props = f(cx, id);
            cx.new_any_element(id, ElementKind::TextInput(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn text_area(&mut self, props: TextAreaProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::TextArea(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn resizable_panel_group<I>(
        &mut self,
        props: ResizablePanelGroupProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::ResizablePanelGroup(props), children)
        })
    }

    #[track_caller]
    pub fn image(&mut self, image: fret_core::ImageId) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::Image(ImageProps::new(image)), Vec::new())
        })
    }

    #[track_caller]
    pub fn image_props(&mut self, props: ImageProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::Image(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn canvas(
        &mut self,
        props: CanvasProps,
        paint: impl for<'p> Fn(&mut CanvasPainter<'p>) + 'static,
    ) -> AnyElement {
        let on_paint: OnCanvasPaint = Arc::new(paint);
        self.scope(|cx| {
            let id = cx.root_id();
            cx.with_state_for(id, CanvasPaintHooks::default, |hooks| {
                hooks.on_paint = Some(on_paint.clone());
            });
            cx.new_any_element(id, ElementKind::Canvas(props), Vec::new())
        })
    }

    #[cfg(feature = "unstable-retained-bridge")]
    #[track_caller]
    pub fn retained_subtree(
        &mut self,
        props: crate::retained_bridge::RetainedSubtreeProps,
    ) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::RetainedSubtree(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn viewport_surface(&mut self, target: fret_core::RenderTargetId) -> AnyElement {
        self.viewport_surface_props(ViewportSurfaceProps::new(target))
    }

    #[track_caller]
    pub fn viewport_surface_mapped(
        &mut self,
        target: fret_core::RenderTargetId,
        target_px_size: (u32, u32),
        fit: fret_core::ViewportFit,
    ) -> AnyElement {
        self.viewport_surface_props(ViewportSurfaceProps {
            target_px_size,
            fit,
            ..ViewportSurfaceProps::new(target)
        })
    }

    #[track_caller]
    pub fn viewport_surface_props(&mut self, props: ViewportSurfaceProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::ViewportSurface(props), Vec::new())
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
            cx.new_any_element(id, ElementKind::SvgIcon(props), Vec::new())
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
            cx.new_any_element(id, ElementKind::Spinner(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn hover_region<I>(
        &mut self,
        props: HoverRegionProps,
        f: impl FnOnce(&mut Self, bool) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let hovered = cx.window_state.hovered_hover_region == Some(id);
            let built = f(cx, hovered);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::HoverRegion(props), children)
        })
    }

    #[track_caller]
    pub fn wheel_region<I>(
        &mut self,
        props: crate::element::WheelRegionProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::WheelRegion(props), children)
        })
    }

    #[track_caller]
    pub fn scroll<I>(&mut self, props: ScrollProps, f: impl FnOnce(&mut Self) -> I) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Scroll(props), children)
        })
    }

    #[track_caller]
    pub fn scrollbar(&mut self, props: ScrollbarProps) -> AnyElement {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.new_any_element(id, ElementKind::Scrollbar(props), Vec::new())
        })
    }

    #[track_caller]
    pub fn virtual_list<F, I>(
        &mut self,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        f: F,
    ) -> AnyElement
    where
        F: FnOnce(&mut Self, &[crate::virtual_list::VirtualItem]) -> I,
        I: IntoIterator<Item = AnyElement>,
    {
        self.virtual_list_with_layout(LayoutStyle::default(), len, options, scroll_handle, f)
    }

    #[track_caller]
    pub fn virtual_list_ex<F, I>(
        &mut self,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        range_extractor: impl FnOnce(crate::virtual_list::VirtualRange) -> Vec<usize>,
        f: F,
    ) -> AnyElement
    where
        F: FnOnce(&mut Self, &[crate::virtual_list::VirtualItem]) -> I,
        I: IntoIterator<Item = AnyElement>,
    {
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
    pub fn virtual_list_with_layout<F, I>(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        f: F,
    ) -> AnyElement
    where
        F: FnOnce(&mut Self, &[crate::virtual_list::VirtualItem]) -> I,
        I: IntoIterator<Item = AnyElement>,
    {
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
    pub fn virtual_list_with_layout_ex<F, I>(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        range_extractor: impl FnOnce(crate::virtual_list::VirtualRange) -> Vec<usize>,
        f: F,
    ) -> AnyElement
    where
        F: FnOnce(&mut Self, &[crate::virtual_list::VirtualItem]) -> I,
        I: IntoIterator<Item = AnyElement>,
    {
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
    fn virtual_list_with_layout_and_keys<F, I>(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        mut item_key_at: impl FnMut(usize) -> crate::ItemKey,
        range_extractor: impl FnOnce(crate::virtual_list::VirtualRange) -> Vec<usize>,
        f: F,
    ) -> AnyElement
    where
        F: FnOnce(&mut Self, &[crate::virtual_list::VirtualItem]) -> I,
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();

            let scroll_handle = scroll_handle.clone();
            scroll_handle.set_items_count(len);

            let key_cache = match options.measure_mode {
                crate::element::VirtualListMeasureMode::Measured => {
                    crate::element::VirtualListKeyCacheMode::AllKeys
                }
                crate::element::VirtualListMeasureMode::Fixed => options.key_cache,
                crate::element::VirtualListMeasureMode::Known => options.key_cache,
            };

            let range = cx.with_state(VirtualListState::default, |state| {
                let axis = options.axis;
                let (viewport, offset) = match axis {
                    fret_core::Axis::Vertical => (state.viewport_h, state.offset_y),
                    fret_core::Axis::Horizontal => (state.viewport_w, state.offset_x),
                };

                let prev_anchor = if viewport.0 > 0.0 && len > 0 {
                    state.metrics.visible_range(offset, viewport, 0).map(|r| {
                        let idx = r.start_index;
                        let key = if idx >= len {
                            idx as crate::ItemKey
                        } else {
                            match key_cache {
                                crate::element::VirtualListKeyCacheMode::AllKeys => state
                                    .keys
                                    .get(idx)
                                    .copied()
                                    .unwrap_or_else(|| item_key_at(idx)),
                                crate::element::VirtualListKeyCacheMode::VisibleOnly => {
                                    item_key_at(idx)
                                }
                            }
                        };
                        let start = state.metrics.offset_for_index(idx);
                        let offset_in_viewport = Px((offset.0 - start.0).max(0.0));
                        (key, offset_in_viewport)
                    })
                } else {
                    None
                };

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

                state.metrics.ensure_with_mode(
                    options.measure_mode,
                    len,
                    options.estimate_row_height,
                    options.gap,
                    options.scroll_margin,
                );

                let needs_rebuild = state.items_revision != options.items_revision
                    || state.items_len != len
                    || state.key_cache != key_cache
                    || prev_cfg != cfg;

                if needs_rebuild {
                    state.items_revision = options.items_revision;
                    state.items_len = len;
                    state.key_cache = key_cache;

                    match key_cache {
                        crate::element::VirtualListKeyCacheMode::AllKeys => {
                            state.keys.clear();
                            state.keys.reserve(len);

                            for i in 0..len {
                                let key = item_key_at(i);
                                state.keys.push(key);
                            }
                        }
                        crate::element::VirtualListKeyCacheMode::VisibleOnly => {
                            state.keys.clear();
                        }
                    }

                    state.metrics.sync_keys(&state.keys, options.items_revision);

                    if options.measure_mode == crate::element::VirtualListMeasureMode::Known
                        && let Some(height_at) = options.known_row_height_at.as_ref()
                    {
                        let heights = (0..len).map(|i| height_at(i)).collect::<Vec<_>>();
                        state.metrics.rebuild_from_known_heights(
                            heights,
                            options.estimate_row_height,
                            options.gap,
                            options.scroll_margin,
                        );
                    }

                    if key_cache == crate::element::VirtualListKeyCacheMode::AllKeys {
                        let has_deferred_scroll = scroll_handle.deferred_scroll_to_item().is_some();
                        if !has_deferred_scroll
                            && let Some((key, offset_in_viewport)) = prev_anchor
                            && let Some(index) = state.keys.iter().position(|&k| k == key)
                        {
                            let start = state.metrics.offset_for_index(index);
                            let desired = Px(start.0 + offset_in_viewport.0);
                            let prev = scroll_handle.offset();
                            let clamped = state.metrics.clamp_offset(desired, viewport);
                            match axis {
                                fret_core::Axis::Vertical => {
                                    scroll_handle
                                        .set_offset(fret_core::Point::new(prev.x, clamped));
                                    state.offset_y = clamped;
                                }
                                fret_core::Axis::Horizontal => {
                                    scroll_handle
                                        .set_offset(fret_core::Point::new(clamped, prev.y));
                                    state.offset_x = clamped;
                                }
                            }
                        }
                    }
                }

                let viewport = Px(viewport.0.max(0.0));
                let offset = state.metrics.clamp_offset(offset, viewport);

                state.deferred_scroll_offset_hint = None;

                let mut range = state.render_window_range.filter(|r| {
                    r.count == len && r.overscan == options.overscan && r.start_index <= r.end_index
                });
                if range.is_none() {
                    range = state.window_range.filter(|r| {
                        r.count == len
                            && r.overscan == options.overscan
                            && r.start_index <= r.end_index
                    });
                }

                // When a scroll handle offset changes out-of-band (wheel, inertial scroll, or a
                // component-driven `set_offset`), the handle's current offset may lead the
                // element-local `state.offset_*` which is only updated during layout.
                //
                // If we are rerendering this frame, compute the visible range against the latest
                // scroll handle offset so "window jump" frames can rebuild the correct visible
                // items without requiring a follow-up rerender.
                let mut preview_offset = offset;
                if state.has_final_viewport && viewport.0 > 0.0 && len > 0 {
                    let handle_offset = scroll_handle.offset();
                    let handle_axis = match axis {
                        fret_core::Axis::Vertical => handle_offset.y,
                        fret_core::Axis::Horizontal => handle_offset.x,
                    };
                    let handle_axis = state.metrics.clamp_offset(handle_axis, viewport);
                    if (handle_axis.0 - offset.0).abs() > 0.01 {
                        preview_offset = handle_axis;
                    }
                }

                // Preview deferred scroll-to-item requests during render so we compute the correct
                // visible range without consuming the request. The final layout pass will apply
                // the scroll offset and clear the request.
                if state.has_final_viewport
                    && viewport.0 > 0.0
                    && len > 0
                    && let Some((index, strategy)) = scroll_handle.deferred_scroll_to_item()
                {
                    let desired = state
                        .metrics
                        .scroll_offset_for_item(index, viewport, offset, strategy);
                    let desired = state.metrics.clamp_offset(desired, viewport);
                    preview_offset = desired;
                    state.deferred_scroll_offset_hint = Some(desired);
                    range = state
                        .metrics
                        .visible_range(desired, viewport, options.overscan);
                }

                if state.has_final_viewport && viewport.0 > 0.0 && len > 0 {
                    let visible = state.metrics.visible_range(preview_offset, viewport, 0);
                    if let (Some(prev), Some(visible)) = (range, visible) {
                        let prev_visible_len = prev
                            .end_index
                            .saturating_sub(prev.start_index)
                            .saturating_add(1);
                        let visible_len = visible
                            .end_index
                            .saturating_sub(visible.start_index)
                            .saturating_add(1);

                        let win_start = prev.start_index.saturating_sub(prev.overscan);
                        let win_end =
                            (prev.end_index + prev.overscan).min(prev.count.saturating_sub(1));
                        let out_of_window =
                            visible.start_index < win_start || visible.end_index > win_end;

                        // If the viewport grows (e.g. after intrinsic probes settle), the stored
                        // render-derived window may under-estimate the visible span while still
                        // appearing "within overscan". Force a one-shot recompute so we don't get
                        // stuck in a too-small window forever under view-cache reuse.
                        if visible_len > prev_visible_len || out_of_window {
                            range = state.metrics.visible_range(
                                preview_offset,
                                viewport,
                                options.overscan,
                            );
                        }
                    } else if range.is_none() {
                        range =
                            state
                                .metrics
                                .visible_range(preview_offset, viewport, options.overscan);
                    }
                } else if range.is_none() {
                    range = state
                        .metrics
                        .visible_range(offset, viewport, options.overscan);
                }

                state.render_window_range = range;
                range
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
                        let key = if idx >= len {
                            idx as crate::ItemKey
                        } else {
                            match key_cache {
                                crate::element::VirtualListKeyCacheMode::AllKeys => state
                                    .keys
                                    .get(idx)
                                    .copied()
                                    .unwrap_or_else(|| item_key_at(idx)),
                                crate::element::VirtualListKeyCacheMode::VisibleOnly => {
                                    item_key_at(idx)
                                }
                            }
                        };
                        state.metrics.virtual_item(idx, key)
                    })
                    .collect::<Vec<_>>()
            });

            let built = f(cx, &visible_items);
            let children = cx.collect_children(built);
            cx.new_any_element(
                id,
                ElementKind::VirtualList(VirtualListProps {
                    layout,
                    axis: options.axis,
                    len,
                    items_revision: options.items_revision,
                    estimate_row_height: options.estimate_row_height,
                    measure_mode: options.measure_mode,
                    key_cache,
                    overscan: options.overscan,
                    keep_alive: options.keep_alive,
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
    pub fn flex<I>(&mut self, props: FlexProps, f: impl FnOnce(&mut Self) -> I) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Flex(props), children)
        })
    }

    #[track_caller]
    pub fn roving_flex<I>(
        &mut self,
        props: crate::element::RovingFlexProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            cx.roving_clear_on_active_change();
            cx.roving_clear_on_typeahead();
            cx.roving_clear_on_navigate();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::RovingFlex(props), children)
        })
    }

    #[track_caller]
    pub fn grid<I>(&mut self, props: GridProps, f: impl FnOnce(&mut Self) -> I) -> AnyElement
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.scope(|cx| {
            let id = cx.root_id();
            let built = f(cx);
            let children = cx.collect_children(built);
            cx.new_any_element(id, ElementKind::Grid(props), children)
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

    /// Retained-host VirtualList helper (ADR 0192).
    ///
    /// This is an opt-in surface that stores `'static` row callbacks in element-local state so
    /// the runtime can attach/detach row subtrees when a cache root reuses without rerendering.
    #[track_caller]
    pub fn virtual_list_keyed_retained(
        &mut self,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        key_at: crate::windowed_surface_host::RetainedVirtualListKeyAtFn,
        row: crate::windowed_surface_host::RetainedVirtualListRowFn<H>,
    ) -> AnyElement
    where
        H: 'static,
    {
        self.virtual_list_keyed_retained_with_layout(
            LayoutStyle::default(),
            len,
            options,
            scroll_handle,
            key_at,
            row,
        )
    }

    #[track_caller]
    pub fn virtual_list_keyed_retained_fn(
        &mut self,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        key_at: impl Fn(usize) -> crate::ItemKey + 'static,
        row: impl for<'b> Fn(&mut ElementContext<'b, H>, usize) -> AnyElement + 'static,
    ) -> AnyElement
    where
        H: 'static,
    {
        self.virtual_list_keyed_retained_with_layout_fn(
            LayoutStyle::default(),
            len,
            options,
            scroll_handle,
            key_at,
            row,
        )
    }

    #[track_caller]
    pub fn virtual_list_keyed_retained_with_layout(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        key_at: crate::windowed_surface_host::RetainedVirtualListKeyAtFn,
        row: crate::windowed_surface_host::RetainedVirtualListRowFn<H>,
    ) -> AnyElement
    where
        H: 'static,
    {
        self.virtual_list_keyed_retained_with_layout_ex(
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
    pub fn virtual_list_keyed_retained_with_layout_fn(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        key_at: impl Fn(usize) -> crate::ItemKey + 'static,
        row: impl for<'b> Fn(&mut ElementContext<'b, H>, usize) -> AnyElement + 'static,
    ) -> AnyElement
    where
        H: 'static,
    {
        let key_at: crate::windowed_surface_host::RetainedVirtualListKeyAtFn = Arc::new(key_at);
        let row: crate::windowed_surface_host::RetainedVirtualListRowFn<H> = Arc::new(row);
        self.virtual_list_keyed_retained_with_layout(
            layout,
            len,
            options,
            scroll_handle,
            key_at,
            row,
        )
    }

    #[track_caller]
    #[allow(clippy::too_many_arguments)]
    pub fn virtual_list_keyed_retained_with_layout_ex(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        key_at: crate::windowed_surface_host::RetainedVirtualListKeyAtFn,
        range_extractor: crate::windowed_surface_host::RetainedVirtualListRangeExtractor,
        row: crate::windowed_surface_host::RetainedVirtualListRowFn<H>,
    ) -> AnyElement
    where
        H: 'static,
    {
        let key_at_for_keys = Arc::clone(&key_at);
        self.virtual_list_with_layout_and_keys(
            layout,
            len,
            options,
            scroll_handle,
            move |i| (key_at_for_keys)(i),
            range_extractor,
            move |cx, items| {
                cx.with_state(
                    crate::windowed_surface_host::RetainedVirtualListHostMarker::default,
                    |_| {},
                );
                // Keep the retained keep-alive bucket's element-local state alive across frames
                // (including view-cache hits) so window shifts can actually reuse previously
                // mounted item subtrees. The actual budget is controlled by `VirtualListProps`.
                cx.with_state(
                    crate::windowed_surface_host::RetainedVirtualListKeepAliveState::default,
                    |_| {},
                );
                cx.with_state(
                    || crate::windowed_surface_host::RetainedVirtualListHostCallbacks::<H> {
                        key_at: Arc::clone(&key_at),
                        row: Arc::clone(&row),
                        range_extractor,
                    },
                    |st| {
                        st.key_at = Arc::clone(&key_at);
                        st.row = Arc::clone(&row);
                        st.range_extractor = range_extractor;
                    },
                );

                items
                    .iter()
                    .copied()
                    .map(|item| {
                        cx.retained_virtual_list_row_any_element(item.key, item.index, &row)
                    })
                    .collect::<Vec<_>>()
            },
        )
    }

    #[track_caller]
    #[allow(clippy::too_many_arguments)]
    pub fn virtual_list_keyed_retained_with_layout_ex_fn(
        &mut self,
        layout: LayoutStyle,
        len: usize,
        options: VirtualListOptions,
        scroll_handle: &crate::scroll::VirtualListScrollHandle,
        key_at: impl Fn(usize) -> crate::ItemKey + 'static,
        range_extractor: crate::windowed_surface_host::RetainedVirtualListRangeExtractor,
        row: impl for<'b> Fn(&mut ElementContext<'b, H>, usize) -> AnyElement + 'static,
    ) -> AnyElement
    where
        H: 'static,
    {
        let key_at: crate::windowed_surface_host::RetainedVirtualListKeyAtFn = Arc::new(key_at);
        let row: crate::windowed_surface_host::RetainedVirtualListRowFn<H> = Arc::new(row);
        self.virtual_list_keyed_retained_with_layout_ex(
            layout,
            len,
            options,
            scroll_handle,
            key_at,
            range_extractor,
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
        let loc = Location::caller();
        self.virtual_list_with_layout_and_keys(
            layout,
            len,
            options,
            scroll_handle,
            key_at,
            range_extractor,
            |cx, items| {
                if cfg!(debug_assertions) && items.len() > 1 {
                    let mut first_dup: Option<(crate::ItemKey, usize, usize)> = None;
                    let mut seen: HashMap<crate::ItemKey, usize> = HashMap::new();
                    for (pos, item) in items.iter().enumerate() {
                        if first_dup.is_none()
                            && let Some(prev) = seen.insert(item.key, pos)
                        {
                            first_dup = Some((item.key, prev, pos));
                        }
                    }

                    if let Some((key, a, b)) = first_dup {
                        let element_path: Option<String> = {
                            #[cfg(feature = "diagnostics")]
                            {
                                cx.window_state.debug_path_for_element(cx.root_id())
                            }
                            #[cfg(not(feature = "diagnostics"))]
                            {
                                None
                            }
                        };

                        tracing::warn!(
                            file = loc.file(),
                            line = loc.line(),
                            column = loc.column(),
                            key = format_args!("{key:#x}"),
                            first_visible_pos = a,
                            second_visible_pos = b,
                            element_path = element_path.as_deref().unwrap_or("<unknown>"),
                            "duplicate virtual_list item key; element identity may collide"
                        );
                    }
                }

                items
                    .iter()
                    .copied()
                    .map(|item| cx.keyed(item.key, |cx| row(cx, item.index)))
                    .collect::<Vec<_>>()
            },
        )
    }
}
