use std::sync::Arc;
use std::time::Duration;

use fret_core::{
    Edges, KeyCode, LayoutDirection, MouseButton, Point, Px, SemanticsOrientation, SemanticsRole,
};
use fret_icons::ids;
use fret_runtime::{Effect, Model, ModelHost, ModelStore, TimerToken};
use fret_ui::action::{ActionCx, ActivateReason, KeyDownCx, OnKeyDown, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, ElementKind, FlexProps, HoverRegionProps, LayoutStyle,
    MainAlign, PointerRegionProps, RenderTransformProps, SemanticsDecoration, VisualTransformProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::prefers_reduced_motion;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::transition as decl_transition;
use fret_ui_kit::headless::carousel as headless_carousel;
use fret_ui_kit::headless::embla as headless_embla;
use fret_ui_kit::headless::snap_points as headless_snap_points;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, LengthRefinement, MetricRef, Radius, Space};

use crate::{Button, ButtonSize, ButtonVariant};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CarouselApiSnapshot {
    /// Zero-based selected slide index.
    pub selected_index: usize,
    /// Total snap count (when measurable).
    pub snap_count: usize,
    pub can_scroll_prev: bool,
    pub can_scroll_next: bool,
    /// True while the carousel is actively settling toward a target snap.
    ///
    /// Note: this coalesces both the v1 recipe settle driver (`settling`) and the Embla-engine
    /// driver (`embla_settling`) into a single observable flag.
    pub settling: bool,
    /// True while the legacy v1 deterministic settle driver is active.
    ///
    /// This driver predates the Embla-style headless engine; it remains available as a fallback
    /// (and for bisecting regressions), but it is not the preferred behavior and may be removed in
    /// a future refactor.
    ///
    /// This is intended for docs/diagnostics only.
    pub recipe_settling: bool,
    /// True while the Embla engine settle driver is active.
    ///
    /// This is intended for docs/diagnostics only.
    pub embla_settling: bool,
    /// True when the current offset is within an epsilon of the selected snap.
    ///
    /// This is intended for docs/diagnostics (e.g. asserting relative settle speed) and should not
    /// be treated as a hard contract for application logic.
    pub at_selected_snap: bool,
    /// Current rendered offset in the main axis (px, positive).
    ///
    /// This is intended for docs/diagnostics only.
    pub offset_px: f32,
    /// Current selected snap offset in the main axis (px, positive).
    ///
    /// This is intended for docs/diagnostics only.
    pub selected_snap_px: f32,
    /// True when the Embla-style headless engine is enabled via options.
    ///
    /// Note: this does not account for environment reduced-motion suppression; use `settling` and
    /// `embla_engine_present` to determine whether the engine is actually driving motion.
    pub embla_engine_enabled: bool,
    /// Embla-style `duration` option (integrator parameter, default `25`).
    pub embla_duration: f32,
    /// True when the recipe currently holds an Embla engine instance.
    pub embla_engine_present: bool,
    /// Current effective scroll duration in the Embla engine (when present).
    pub embla_scroll_duration: f32,
    /// Monotonically increasing counter that increments when the selected index changes.
    ///
    /// This is an MVP event surface intended to support shadcn-style `api.on("select", ...)`
    /// outcomes without storing closures inside models.
    pub select_generation: u64,
    /// Monotonically increasing counter that increments when the carousel re-initializes due to
    /// geometry changes (snaps/limits/view size).
    ///
    /// This is an MVP event surface intended to support shadcn-style `api.on("reInit", ...)`
    /// outcomes without storing closures inside models.
    pub reinit_generation: u64,
}

#[derive(Debug, Clone)]
pub struct CarouselApi {
    index: Model<usize>,
    offset: Model<Px>,
    runtime: Model<CarouselRuntime>,
    extent: Model<Px>,
    options: Model<CarouselOptions>,
    snaps: Model<Arc<[Px]>>,
    commands: Model<CarouselCommandQueue>,
    slides_in_view_snapshot: Option<Model<CarouselSlidesInViewSnapshot>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CarouselEventCursor {
    pub select_generation: u64,
    pub reinit_generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselEvent {
    Select { selected_index: usize },
    ReInit,
}

impl CarouselApi {
    pub fn scroll_prev(&self, host: &mut impl ModelHost) {
        let _ = host.update_model(&self.commands, |q, _| {
            q.pending.push(CarouselCommand::ScrollPrev);
        });
    }

    pub fn scroll_next(&self, host: &mut impl ModelHost) {
        let _ = host.update_model(&self.commands, |q, _| {
            q.pending.push(CarouselCommand::ScrollNext);
        });
    }

    pub fn scroll_to(&self, host: &mut impl ModelHost, index: usize) {
        let _ = host.update_model(&self.commands, |q, _| {
            q.pending.push(CarouselCommand::ScrollTo { index });
        });
    }

    pub fn scroll_prev_store(&self, store: &mut ModelStore) {
        let _ = store.update(&self.commands, |q| {
            q.pending.push(CarouselCommand::ScrollPrev);
        });
    }

    pub fn scroll_next_store(&self, store: &mut ModelStore) {
        let _ = store.update(&self.commands, |q| {
            q.pending.push(CarouselCommand::ScrollNext);
        });
    }

    pub fn scroll_to_store(&self, store: &mut ModelStore, index: usize) {
        let _ = store.update(&self.commands, |q| {
            q.pending.push(CarouselCommand::ScrollTo { index });
        });
    }

    pub fn snapshot(&self, host: &mut impl ModelHost) -> CarouselApiSnapshot {
        let selected_index = host.read(&self.index, |_host, v| *v).ok().unwrap_or(0);
        let offset = host
            .read(&self.offset, |_host, v| *v)
            .ok()
            .unwrap_or(Px(0.0));
        let view_size = host.read(&self.extent, |_host, v| v.0).ok().unwrap_or(0.0);
        let snap_count_raw = host.read(&self.snaps, |_host, v| v.len()).ok().unwrap_or(0);
        let extent_ready = view_size > 0.0 && snap_count_raw > 0;
        let loop_enabled = host
            .read(&self.options, |_host, v| v.loop_enabled)
            .ok()
            .unwrap_or(false);
        let embla_engine_enabled = host
            .read(&self.options, |_host, v| v.embla_engine)
            .ok()
            .unwrap_or(false);
        let embla_duration = host
            .read(&self.options, |_host, v| v.embla_duration)
            .ok()
            .unwrap_or(0.0);

        let runtime = host
            .read(&self.runtime, |_host, v| *v)
            .ok()
            .unwrap_or_default();

        let snap_count = if extent_ready { snap_count_raw } else { 0 };
        let can_scroll_prev =
            extent_ready && snap_count_raw > 1 && (loop_enabled || selected_index > 0);
        let can_scroll_next = extent_ready
            && snap_count_raw > 1
            && (loop_enabled || selected_index + 1 < snap_count_raw);
        let selected_snap = host
            .read(&self.snaps, |_host, v| v.get(selected_index).copied())
            .ok()
            .flatten();
        let at_selected_snap =
            extent_ready && selected_snap.is_some_and(|snap| (snap.0 - offset.0).abs() <= 0.5);
        let selected_snap_px = if extent_ready {
            selected_snap.unwrap_or(Px(0.0)).0
        } else {
            0.0
        };

        CarouselApiSnapshot {
            selected_index,
            snap_count,
            can_scroll_prev,
            can_scroll_next,
            settling: runtime.settling || runtime.embla_settling,
            recipe_settling: runtime.settling,
            embla_settling: runtime.embla_settling,
            at_selected_snap,
            offset_px: offset.0,
            selected_snap_px,
            embla_engine_enabled,
            embla_duration,
            embla_engine_present: false,
            embla_scroll_duration: 0.0,
            select_generation: runtime.api_select_generation,
            reinit_generation: runtime.api_reinit_generation,
        }
    }

    pub fn selected_scroll_snap(&self, host: &mut impl ModelHost) -> usize {
        self.snapshot(host).selected_index
    }

    pub fn scroll_snap_list(&self, host: &mut impl ModelHost) -> Arc<[Px]> {
        host.read(&self.snaps, |_host, v| v.clone())
            .ok()
            .unwrap_or_else(|| Arc::from(Vec::<Px>::new()))
    }

    pub fn slides_in_view(&self, host: &mut impl ModelHost) -> Arc<[usize]> {
        let Some(model) = self.slides_in_view_snapshot.as_ref() else {
            return Arc::from(Vec::<usize>::new());
        };
        host.read(model, |_host, v| v.slides_in_view.clone())
            .ok()
            .unwrap_or_else(|| Arc::from(Vec::<usize>::new()))
    }

    pub fn events_since(
        &self,
        host: &mut impl ModelHost,
        cursor: &mut CarouselEventCursor,
    ) -> Vec<CarouselEvent> {
        let snap = self.snapshot(host);

        let mut out = Vec::new();
        if snap.reinit_generation != cursor.reinit_generation {
            cursor.reinit_generation = snap.reinit_generation;
            out.push(CarouselEvent::ReInit);
        }
        if snap.select_generation != cursor.select_generation {
            cursor.select_generation = snap.select_generation;
            out.push(CarouselEvent::Select {
                selected_index: snap.selected_index,
            });
        }
        out
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CarouselSlidesInViewSnapshot {
    pub slides_in_view: Arc<[usize]>,
    pub slides_enter_view: Arc<[usize]>,
    pub slides_left_view: Arc<[usize]>,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarouselAutoplayConfig {
    pub delay: Duration,
    pub stop_on_interaction: bool,
    pub pause_on_hover: bool,
    pub reset_on_hover_leave: bool,
}

impl Default for CarouselAutoplayConfig {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(2000),
            stop_on_interaction: true,
            pause_on_hover: true,
            reset_on_hover_leave: true,
        }
    }
}

impl CarouselAutoplayConfig {
    pub fn new(delay: Duration) -> Self {
        Self {
            delay,
            ..Default::default()
        }
    }

    pub fn stop_on_interaction(mut self, stop_on_interaction: bool) -> Self {
        self.stop_on_interaction = stop_on_interaction;
        self
    }

    pub fn pause_on_hover(mut self, pause_on_hover: bool) -> Self {
        self.pause_on_hover = pause_on_hover;
        self
    }

    pub fn reset_on_hover_leave(mut self, reset_on_hover_leave: bool) -> Self {
        self.reset_on_hover_leave = reset_on_hover_leave;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Default)]
enum CarouselControls {
    #[default]
    BuiltIn,
    Parts {
        previous: CarouselPrevious,
        next: CarouselNext,
    },
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselAlign {
    Start,
    #[default]
    Center,
    End,
}

impl CarouselAlign {
    fn to_headless(self) -> headless_carousel::CarouselSnapAlign {
        match self {
            CarouselAlign::Start => headless_carousel::CarouselSnapAlign::Start,
            CarouselAlign::Center => headless_carousel::CarouselSnapAlign::Center,
            CarouselAlign::End => headless_carousel::CarouselSnapAlign::End,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselSlidesToScroll {
    Fixed(usize),
    Auto,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CarouselOptionsPatch {
    pub align: Option<CarouselAlign>,
    pub slides_to_scroll: Option<CarouselSlidesToScroll>,
    pub contain_scroll: Option<CarouselContainScroll>,
    pub direction: Option<LayoutDirection>,
    pub draggable: Option<bool>,
    pub loop_enabled: Option<bool>,
    pub skip_snaps: Option<bool>,
    pub drag_free: Option<bool>,
    pub duration: Option<Duration>,
    pub embla_engine: Option<bool>,
    pub embla_duration: Option<f32>,
    pub ignore_reduced_motion: Option<bool>,
    pub in_view_threshold: Option<f32>,
    pub in_view_margin_px: Option<Px>,
    pub watch_focus: Option<bool>,
    pub pixel_tolerance_px: Option<f32>,
}

impl CarouselOptionsPatch {
    fn apply(self, base: CarouselOptions) -> CarouselOptions {
        CarouselOptions {
            align: self.align.unwrap_or(base.align),
            slides_to_scroll: self.slides_to_scroll.unwrap_or(base.slides_to_scroll),
            contain_scroll: self.contain_scroll.unwrap_or(base.contain_scroll),
            direction: self.direction.unwrap_or(base.direction),
            start_snap: base.start_snap,
            draggable: self.draggable.unwrap_or(base.draggable),
            loop_enabled: self.loop_enabled.unwrap_or(base.loop_enabled),
            skip_snaps: self.skip_snaps.unwrap_or(base.skip_snaps),
            drag_free: self.drag_free.unwrap_or(base.drag_free),
            duration: self.duration.unwrap_or(base.duration),
            embla_engine: self.embla_engine.unwrap_or(base.embla_engine),
            embla_duration: self.embla_duration.unwrap_or(base.embla_duration),
            ignore_reduced_motion: self
                .ignore_reduced_motion
                .unwrap_or(base.ignore_reduced_motion),
            in_view_threshold: self.in_view_threshold.unwrap_or(base.in_view_threshold),
            in_view_margin_px: self.in_view_margin_px.unwrap_or(base.in_view_margin_px),
            watch_focus: self.watch_focus.unwrap_or(base.watch_focus),
            pixel_tolerance_px: self.pixel_tolerance_px.unwrap_or(base.pixel_tolerance_px),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CarouselBreakpoint {
    pub min_width_px: Px,
    pub patch: CarouselOptionsPatch,
}

fn resolve_breakpoint_options(
    base: CarouselOptions,
    view_width: Px,
    breakpoints: &[CarouselBreakpoint],
) -> CarouselOptions {
    if view_width.0 <= 0.0 || breakpoints.is_empty() {
        return base;
    }

    let mut best = None;
    for bp in breakpoints {
        if bp.min_width_px.0 <= view_width.0 {
            best = Some(*bp);
        }
    }
    match best {
        Some(bp) => bp.patch.apply(base),
        None => base,
    }
}

impl Default for CarouselSlidesToScroll {
    fn default() -> Self {
        Self::Fixed(1)
    }
}

impl CarouselSlidesToScroll {
    fn to_headless(self) -> headless_carousel::CarouselSlidesToScrollOption {
        match self {
            CarouselSlidesToScroll::Fixed(n) => {
                headless_carousel::CarouselSlidesToScrollOption::Fixed(n.max(1))
            }
            CarouselSlidesToScroll::Auto => headless_carousel::CarouselSlidesToScrollOption::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselContainScroll {
    None,
    KeepSnaps,
    #[default]
    TrimSnaps,
}

impl CarouselContainScroll {
    fn to_headless(self) -> headless_carousel::CarouselContainScrollOption {
        match self {
            CarouselContainScroll::None => headless_carousel::CarouselContainScrollOption::None,
            CarouselContainScroll::KeepSnaps => {
                headless_carousel::CarouselContainScrollOption::KeepSnaps
            }
            CarouselContainScroll::TrimSnaps => {
                headless_carousel::CarouselContainScrollOption::TrimSnaps
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CarouselOptions {
    pub align: CarouselAlign,
    pub slides_to_scroll: CarouselSlidesToScroll,
    pub contain_scroll: CarouselContainScroll,
    /// Layout direction (Embla `direction`).
    pub direction: LayoutDirection,
    /// Initial selected snap index (Embla `startSnap`).
    pub start_snap: usize,
    /// Whether pointer dragging is enabled (Embla `draggable`).
    pub draggable: bool,
    /// Wrap prev/next selection (note: this is *not* a seamless loop engine yet).
    pub loop_enabled: bool,
    /// Embla-style skipSnaps (best-effort without momentum physics).
    pub skip_snaps: bool,
    /// Drag-free release (settle to the projected offset instead of the nearest snap).
    pub drag_free: bool,
    /// Settle animation duration for the deterministic (non-physics) driver (v1 behavior).
    pub duration: Duration,
    /// Enable the Embla-style headless engine (v2 parity).
    ///
    /// When disabled, Carousel falls back to the legacy v1 deterministic settle driver. This is
    /// kept primarily for bisecting and as an escape hatch; it is not a long-term contract.
    pub embla_engine: bool,
    /// Embla-style `duration` (integrator parameter, default `25`).
    ///
    /// This is *not* a wall-clock duration in milliseconds. See:
    /// - `docs/workstreams/carousel-embla-parity-v2/contracts.md`
    pub embla_duration: f32,
    /// When true, ignore window `prefers-reduced-motion` and keep motion enabled.
    ///
    /// This is intended for demos/diagnostics only; production UIs should respect reduced motion.
    pub ignore_reduced_motion: bool,
    /// Minimum visible fraction (0..=1) required to count a slide as "in view" for slidesInView.
    pub in_view_threshold: f32,
    /// Viewport margin (px) applied on both ends for slidesInView intersection tests.
    pub in_view_margin_px: Px,
    /// Embla-style `focus` watcher (default `true`): when focus moves into a slide via keyboard
    /// traversal, scroll the focused slide into view.
    pub watch_focus: bool,
    pub pixel_tolerance_px: f32,
}

impl Default for CarouselOptions {
    fn default() -> Self {
        Self {
            // Match Embla defaults unless examples override via `opts`.
            align: CarouselAlign::Center,
            slides_to_scroll: CarouselSlidesToScroll::Fixed(1),
            contain_scroll: CarouselContainScroll::TrimSnaps,
            direction: LayoutDirection::Ltr,
            start_snap: 0,
            draggable: true,
            loop_enabled: false,
            skip_snaps: false,
            drag_free: false,
            duration: Duration::from_millis(25),
            // v2 parity default: use the Embla-style headless engine unless explicitly disabled.
            embla_engine: true,
            embla_duration: 25.0,
            ignore_reduced_motion: false,
            in_view_threshold: 0.0,
            in_view_margin_px: Px(0.0),
            watch_focus: true,
            pixel_tolerance_px: 2.0,
        }
    }
}

impl CarouselOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn align(mut self, align: CarouselAlign) -> Self {
        self.align = align;
        self
    }

    pub fn slides_to_scroll(mut self, slides_to_scroll: CarouselSlidesToScroll) -> Self {
        self.slides_to_scroll = slides_to_scroll;
        self
    }

    pub fn contain_scroll(mut self, contain_scroll: CarouselContainScroll) -> Self {
        self.contain_scroll = contain_scroll;
        self
    }

    pub fn direction(mut self, direction: LayoutDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn start_snap(mut self, start_snap: usize) -> Self {
        self.start_snap = start_snap;
        self
    }

    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    pub fn loop_enabled(mut self, loop_enabled: bool) -> Self {
        self.loop_enabled = loop_enabled;
        self
    }

    pub fn skip_snaps(mut self, skip_snaps: bool) -> Self {
        self.skip_snaps = skip_snaps;
        self
    }

    pub fn drag_free(mut self, drag_free: bool) -> Self {
        self.drag_free = drag_free;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn embla_engine(mut self, embla_engine: bool) -> Self {
        self.embla_engine = embla_engine;
        self
    }

    pub fn embla_duration(mut self, duration: f32) -> Self {
        self.embla_duration = duration;
        self
    }

    pub fn ignore_reduced_motion(mut self, ignore: bool) -> Self {
        self.ignore_reduced_motion = ignore;
        self
    }

    pub fn in_view_threshold(mut self, threshold: f32) -> Self {
        self.in_view_threshold = threshold;
        self
    }

    pub fn in_view_margin_px(mut self, margin: Px) -> Self {
        self.in_view_margin_px = margin;
        self
    }

    pub fn watch_focus(mut self, enabled: bool) -> Self {
        self.watch_focus = enabled;
        self
    }

    pub fn pixel_tolerance_px(mut self, pixel_tolerance_px: f32) -> Self {
        self.pixel_tolerance_px = pixel_tolerance_px;
        self
    }
}

#[derive(Debug)]
pub struct Carousel {
    items: Vec<CarouselItem>,
    layout: LayoutRefinement,
    viewport_layout: LayoutRefinement,
    track_layout: LayoutRefinement,
    item_layout: LayoutRefinement,
    orientation: CarouselOrientation,
    track_start_neg_margin: Space,
    item_padding_start: Space,
    item_basis_main_px: Option<Px>,
    breakpoints: Vec<CarouselBreakpoint>,
    options: CarouselOptions,
    drag_config: headless_carousel::CarouselDragConfig,
    api_snapshot: Option<Model<CarouselApiSnapshot>>,
    api_handle: Option<Model<Option<CarouselApi>>>,
    slides_in_view_snapshot: Option<Model<CarouselSlidesInViewSnapshot>>,
    autoplay: Option<CarouselAutoplayConfig>,
    controls: CarouselControls,
    test_id: Option<Arc<str>>,
}

#[derive(Debug, Clone, Copy)]
struct CarouselRuntime {
    drag: headless_carousel::CarouselDragState,
    settling: bool,
    embla_settling: bool,
    prevent_click: bool,
    settle_from: Px,
    settle_to: Px,
    settle_generation: u64,
    selection_initialized: bool,
    autoplay_token: Option<TimerToken>,
    autoplay_paused: bool,
    autoplay_hovered: bool,
    api_select_generation: u64,
    api_reinit_generation: u64,
    api_last_reinit_emit_frame: Option<u64>,
    api_reinit_pending: bool,
    api_last_selected_index: usize,
    focus_tab_generation: u64,
    focus_last_handled_tab_generation: u64,
    focus_last_focused_element: Option<fret_ui::elements::GlobalElementId>,
}

impl Default for CarouselRuntime {
    fn default() -> Self {
        Self {
            drag: headless_carousel::CarouselDragState::default(),
            settling: false,
            embla_settling: false,
            prevent_click: false,
            settle_from: Px(0.0),
            settle_to: Px(0.0),
            settle_generation: 0,
            selection_initialized: false,
            autoplay_token: None,
            autoplay_paused: false,
            autoplay_hovered: false,
            api_select_generation: 0,
            api_reinit_generation: 0,
            api_last_reinit_emit_frame: None,
            api_reinit_pending: false,
            api_last_selected_index: 0,
            focus_tab_generation: 0,
            focus_last_handled_tab_generation: 0,
            focus_last_focused_element: None,
        }
    }
}

#[derive(Default)]
struct CarouselState {
    index: Option<Model<usize>>,
    offset: Option<Model<Px>>,
    runtime: Option<Model<CarouselRuntime>>,
    extent: Option<Model<Px>>,
    viewport_width: Option<Model<Px>>,
    options: Option<Model<CarouselOptions>>,
    snaps: Option<Model<Arc<[Px]>>>,
    slides: Option<Model<Arc<[headless_carousel::CarouselSlide1D]>>>,
    max_offset: Option<Model<Px>>,
    embla_engine: Option<Model<Option<headless_embla::engine::Engine>>>,
    api_commands: Option<Model<CarouselCommandQueue>>,
    slides_in_view_tracker: Option<Model<headless_embla::slides_in_view::SlidesInViewTracker>>,
    slide_content_ids: Option<Model<Arc<[fret_ui::elements::GlobalElementId]>>>,
}

#[derive(Debug, Clone, Default)]
struct CarouselCommandQueue {
    pending: Vec<CarouselCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CarouselCommand {
    ScrollPrev,
    ScrollNext,
    ScrollTo { index: usize },
}

fn carousel_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    options_base: CarouselOptions,
) -> (
    Model<usize>,
    Model<Px>,
    Model<CarouselRuntime>,
    Model<Px>,
    Model<Px>,
    Model<CarouselOptions>,
    Model<Arc<[Px]>>,
    Model<Arc<[headless_carousel::CarouselSlide1D]>>,
    Model<Px>,
    Model<Option<headless_embla::engine::Engine>>,
) {
    let needs_init = cx.with_state(CarouselState::default, |st| {
        st.index.is_none()
            || st.offset.is_none()
            || st.runtime.is_none()
            || st.extent.is_none()
            || st.viewport_width.is_none()
            || st.options.is_none()
            || st.snaps.is_none()
            || st.slides.is_none()
            || st.max_offset.is_none()
            || st.embla_engine.is_none()
    });

    if needs_init {
        let index = cx.app.models_mut().insert(options_base.start_snap);
        let offset = cx.app.models_mut().insert(Px(0.0));
        let runtime = cx.app.models_mut().insert(CarouselRuntime::default());
        let _ = cx.app.models_mut().update(&runtime, |st| {
            st.api_last_selected_index = options_base.start_snap
        });
        let extent = cx.app.models_mut().insert(Px(0.0));
        let viewport_width = cx.app.models_mut().insert(Px(0.0));
        let options = cx.app.models_mut().insert(options_base);
        let snaps: Arc<[Px]> = Arc::from(Vec::<Px>::new());
        let snaps = cx.app.models_mut().insert(snaps);
        let slides: Arc<[headless_carousel::CarouselSlide1D]> =
            Arc::from(Vec::<headless_carousel::CarouselSlide1D>::new());
        let slides = cx.app.models_mut().insert(slides);
        let max_offset = cx.app.models_mut().insert(Px(0.0));
        let embla_engine: Option<headless_embla::engine::Engine> = None;
        let embla_engine = cx.app.models_mut().insert(embla_engine);
        cx.with_state(CarouselState::default, |st| {
            st.index = Some(index.clone());
            st.offset = Some(offset.clone());
            st.runtime = Some(runtime.clone());
            st.extent = Some(extent.clone());
            st.viewport_width = Some(viewport_width.clone());
            st.options = Some(options.clone());
            st.snaps = Some(snaps.clone());
            st.slides = Some(slides.clone());
            st.max_offset = Some(max_offset.clone());
            st.embla_engine = Some(embla_engine.clone());
        });
        return (
            index,
            offset,
            runtime,
            extent,
            viewport_width,
            options,
            snaps,
            slides,
            max_offset,
            embla_engine,
        );
    }

    let (
        index,
        offset,
        runtime,
        extent,
        viewport_width,
        options,
        snaps,
        slides,
        max_offset,
        embla_engine,
    ) = cx.with_state(CarouselState::default, |st| {
        (
            st.index.clone().expect("index"),
            st.offset.clone().expect("offset"),
            st.runtime.clone().expect("runtime"),
            st.extent.clone().expect("extent"),
            st.viewport_width.clone().expect("viewport_width"),
            st.options.clone().expect("options"),
            st.snaps.clone().expect("snaps"),
            st.slides.clone().expect("slides"),
            st.max_offset.clone().expect("max_offset"),
            st.embla_engine.clone().expect("embla_engine"),
        )
    });
    (
        index,
        offset,
        runtime,
        extent,
        viewport_width,
        options,
        snaps,
        slides,
        max_offset,
        embla_engine,
    )
}

fn carousel_api_commands_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<CarouselCommandQueue> {
    let existing = cx.with_state(CarouselState::default, |st| st.api_commands.clone());
    if let Some(existing) = existing {
        return existing;
    }

    let model = cx.app.models_mut().insert(CarouselCommandQueue::default());
    cx.with_state(CarouselState::default, |st| {
        st.api_commands = Some(model.clone());
    });
    model
}

fn carousel_slides_in_view_tracker_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<headless_embla::slides_in_view::SlidesInViewTracker> {
    let existing = cx.with_state(CarouselState::default, |st| {
        st.slides_in_view_tracker.clone()
    });
    if let Some(existing) = existing {
        return existing;
    }

    let tracker = cx
        .app
        .models_mut()
        .insert(headless_embla::slides_in_view::SlidesInViewTracker::default());
    cx.with_state(CarouselState::default, |st| {
        st.slides_in_view_tracker = Some(tracker.clone());
    });
    tracker
}

fn carousel_slide_content_ids_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Arc<[fret_ui::elements::GlobalElementId]>> {
    let existing = cx.with_state(CarouselState::default, |st| st.slide_content_ids.clone());
    if let Some(existing) = existing {
        return existing;
    }

    let ids: Arc<[fret_ui::elements::GlobalElementId]> =
        Arc::from(Vec::<fret_ui::elements::GlobalElementId>::new());
    let model = cx.app.models_mut().insert(ids);
    cx.with_state(CarouselState::default, |st| {
        st.slide_content_ids = Some(model.clone());
    });
    model
}

#[derive(Default)]
struct CarouselPartsModelsState {
    api_handle: Option<Model<Option<CarouselApi>>>,
    api_snapshot: Option<Model<CarouselApiSnapshot>>,
}

fn carousel_parts_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<Option<CarouselApi>>, Model<CarouselApiSnapshot>) {
    let (handle, snapshot) = cx.with_state(CarouselPartsModelsState::default, |st| {
        (st.api_handle.clone(), st.api_snapshot.clone())
    });
    if let (Some(handle), Some(snapshot)) = (handle, snapshot) {
        return (handle, snapshot);
    }

    let handle = cx.app.models_mut().insert(None::<CarouselApi>);
    let snapshot = cx.app.models_mut().insert(CarouselApiSnapshot::default());
    cx.with_state(CarouselPartsModelsState::default, |st| {
        st.api_handle = Some(handle.clone());
        st.api_snapshot = Some(snapshot.clone());
    });
    (handle, snapshot)
}

impl Default for Carousel {
    fn default() -> Self {
        Self::new(Vec::<CarouselItem>::new())
    }
}

impl Carousel {
    pub fn new<I>(items: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<CarouselItem>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            layout: LayoutRefinement::default(),
            viewport_layout: LayoutRefinement::default(),
            track_layout: LayoutRefinement::default(),
            item_layout: LayoutRefinement::default(),
            orientation: CarouselOrientation::Horizontal,
            track_start_neg_margin: Space::N4,
            item_padding_start: Space::N4,
            item_basis_main_px: None,
            breakpoints: Vec::new(),
            options: CarouselOptions::default(),
            drag_config: headless_carousel::CarouselDragConfig::default(),
            api_snapshot: None,
            api_handle: None,
            slides_in_view_snapshot: None,
            autoplay: None,
            controls: CarouselControls::BuiltIn,
            test_id: None,
        }
    }

    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<CarouselItem>,
    {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_viewport_layout(mut self, layout: LayoutRefinement) -> Self {
        self.viewport_layout = self.viewport_layout.merge(layout);
        self
    }

    pub fn refine_track_layout(mut self, layout: LayoutRefinement) -> Self {
        self.track_layout = self.track_layout.merge(layout);
        self
    }

    pub fn refine_item_layout(mut self, layout: LayoutRefinement) -> Self {
        self.item_layout = self.item_layout.merge(layout);
        self
    }

    pub fn orientation(mut self, orientation: CarouselOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Embla-style options that influence snap model generation (alignment, containment, etc).
    pub fn opts(mut self, opts: CarouselOptions) -> Self {
        self.options = opts;
        self
    }

    pub fn drag_config(mut self, config: headless_carousel::CarouselDragConfig) -> Self {
        self.drag_config = config;
        self
    }

    pub fn drag_threshold_px(mut self, threshold: Px) -> Self {
        self.drag_config.drag_threshold_px = threshold.0;
        self
    }

    pub fn snap_threshold_fraction(mut self, threshold: f32) -> Self {
        self.drag_config.snap_threshold_fraction = threshold;
        self
    }

    pub fn touch_prevent_scroll(mut self, enabled: bool) -> Self {
        self.drag_config.touch_prevent_scroll = enabled;
        self
    }

    pub fn touch_scroll_lock_threshold_px(mut self, threshold: Px) -> Self {
        self.drag_config.touch_scroll_lock_threshold_px = threshold.0;
        self
    }

    /// Exposes a small, deterministic API surface as a snapshot model (policy-only).
    ///
    /// This exists to support shadcn-style "API" examples (slide counters, etc.) without exposing
    /// Embla's full imperative API surface.
    pub fn api_snapshot_model(mut self, model: Model<CarouselApiSnapshot>) -> Self {
        self.api_snapshot = Some(model);
        self
    }

    /// Exposes a Rust-native handle that can enqueue scroll commands and observe events.
    ///
    /// This aligns with the shadcn `setApi` example ergonomics without storing closures inside
    /// models.
    pub fn api_handle_model(mut self, model: Model<Option<CarouselApi>>) -> Self {
        self.api_handle = Some(model);
        self
    }

    pub fn slides_in_view_snapshot_model(
        mut self,
        model: Model<CarouselSlidesInViewSnapshot>,
    ) -> Self {
        self.slides_in_view_snapshot = Some(model);
        self
    }

    /// Adds an Embla-style autoplay policy surface (shadcn `carousel-plugin` outcome).
    pub fn autoplay(mut self, config: CarouselAutoplayConfig) -> Self {
        self.autoplay = Some(config);
        self
    }

    pub fn track_start_neg_margin(mut self, margin: Space) -> Self {
        self.track_start_neg_margin = margin;
        self
    }

    pub fn item_padding_start(mut self, padding: Space) -> Self {
        self.item_padding_start = padding;
        self
    }

    pub fn item_basis_main_px(mut self, basis: Px) -> Self {
        self.item_basis_main_px = Some(basis);
        self
    }

    /// Breakpoints for Embla-style options overrides, evaluated using the measured carousel viewport
    /// width.
    ///
    /// This is intentionally "Rust-native": callers provide explicit width thresholds instead of
    /// CSS media queries.
    pub fn breakpoints(
        mut self,
        breakpoints: impl IntoIterator<Item = CarouselBreakpoint>,
    ) -> Self {
        self.breakpoints = breakpoints.into_iter().collect::<Vec<_>>();
        self.breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    pub fn breakpoint(mut self, min_width_px: Px, patch: CarouselOptionsPatch) -> Self {
        self.breakpoints.push(CarouselBreakpoint {
            min_width_px,
            patch,
        });
        self.breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    /// Enable/disable the built-in previous/next controls.
    ///
    /// Default: `true` (built-in controls enabled).
    pub fn controls(mut self, enabled: bool) -> Self {
        self.controls = if enabled {
            CarouselControls::BuiltIn
        } else {
            CarouselControls::None
        };
        self
    }

    fn controls_parts(mut self, previous: CarouselPrevious, next: CarouselNext) -> Self {
        self.controls = CarouselControls::Parts { previous, next };
        self
    }

    /// Part-based authoring surface aligned with shadcn/ui v4 exports.
    ///
    /// This is a thin adapter over [`Carousel::into_element`] that accepts shadcn-style parts
    /// (`CarouselContent`, `CarouselItem`, `CarouselPrevious`, `CarouselNext`).
    #[track_caller]
    pub fn into_element_parts<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> CarouselContent,
        previous: CarouselPrevious,
        next: CarouselNext,
    ) -> AnyElement {
        let (default_api_handle, default_api_snapshot) = carousel_parts_models(cx);
        let api_handle = self.api_handle.clone().unwrap_or(default_api_handle);
        let api_snapshot = self.api_snapshot.clone().unwrap_or(default_api_snapshot);
        let content = content(cx);
        let track_start_neg_margin = content.track_start_neg_margin;

        let mut this = self.items(content.items);
        if let Some(margin) = track_start_neg_margin {
            this = this.track_start_neg_margin(margin);
        }

        this.refine_viewport_layout(content.viewport_layout)
            .refine_track_layout(content.track_layout)
            .refine_item_layout(content.item_layout)
            .api_handle_model(api_handle)
            .api_snapshot_model(api_snapshot)
            .controls_parts(previous, next)
            .into_element(cx)
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).snapshot();
            let orientation = self.orientation;
            let options_base = self.options;
            let breakpoints = self.breakpoints;
            let autoplay_cfg = self.autoplay;
            let autoplay_stop_on_interaction =
                autoplay_cfg.is_some_and(|cfg| cfg.stop_on_interaction);
            let root_test_id = self.test_id.unwrap_or_else(|| Arc::from("carousel"));
            let slides_in_view_snapshot_model = self.slides_in_view_snapshot;
            let api_handle_model = self.api_handle;
            let api_snapshot_model = self.api_snapshot;
            let controls = self.controls;

            let (
                index_model,
                offset_model,
                runtime_model,
                extent_model,
                viewport_width_model,
                options_model,
                snaps_model,
                slides_model,
                max_offset_model,
                embla_engine_model,
            ) = carousel_models(cx, options_base);

            let options_prev = cx
                .watch_model(&options_model)
                .copied()
                .unwrap_or(options_base);
            let view_width_for_breakpoints = cx
                .watch_model(&viewport_width_model)
                .copied()
                .unwrap_or(Px(0.0));
            let options =
                resolve_breakpoint_options(options_base, view_width_for_breakpoints, &breakpoints);
            let _ = cx
                .app
                .models_mut()
                .update(&options_model, |v| *v = options);

            let api_commands_model = api_handle_model
                .as_ref()
                .map(|_| carousel_api_commands_model(cx));
            if let (Some(api_handle_model), Some(api_commands_model)) =
                (api_handle_model.as_ref(), api_commands_model.as_ref())
            {
                let api = CarouselApi {
                    index: index_model.clone(),
                    offset: offset_model.clone(),
                    runtime: runtime_model.clone(),
                    extent: extent_model.clone(),
                    options: options_model.clone(),
                    snaps: snaps_model.clone(),
                    commands: api_commands_model.clone(),
                    slides_in_view_snapshot: slides_in_view_snapshot_model.clone(),
                };
                let _ = cx.app.models_mut().update(api_handle_model, |v| {
                    if v.is_none() {
                        *v = Some(api);
                    }
                });
            }

            cx.with_state(CarouselContextProviderState::default, |st| {
                st.current = match (api_handle_model.clone(), api_snapshot_model.clone()) {
                    (Some(api_handle), Some(api_snapshot)) => Some(CarouselContext {
                        api_handle,
                        api_snapshot,
                        orientation,
                        options,
                        root_test_id: root_test_id.clone(),
                    }),
                    _ => None,
                };
            });

            let root_layout = decl_style::layout_style(
                &theme,
                // Upstream shadcn places the prev/next controls outside the viewport (`-left-12` /
                // `-right-12`). Keep the root overflow-visible so hit-testing can reach those
                // controls even when their bounds extend outside the carousel panel.
                LayoutRefinement::default()
                    .relative()
                    .overflow_visible()
                    .merge(self.layout),
            );

            let viewport_layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default()
                    .w_full()
                    .overflow_hidden()
                    .merge(self.viewport_layout),
            );

            let track_layout = match orientation {
                CarouselOrientation::Horizontal => LayoutRefinement::default()
                    .w_full()
                    .ml_neg(self.track_start_neg_margin)
                    .merge(self.track_layout),
                CarouselOrientation::Vertical => LayoutRefinement::default()
                    .w_full()
                    .mt_neg(self.track_start_neg_margin)
                    .merge(self.track_layout),
            };
            let track_layout = decl_style::layout_style(&theme, track_layout);

            let item_pad_default_space = self.item_padding_start;

            let (track_direction, button_axis) = match orientation {
                CarouselOrientation::Horizontal => {
                    (fret_core::Axis::Horizontal, fret_core::Axis::Vertical)
                }
                CarouselOrientation::Vertical => {
                    (fret_core::Axis::Vertical, fret_core::Axis::Horizontal)
                }
            };
            let layout_direction = options.direction;

            let items_len = self.items.len();
            let item_basis = self.item_basis_main_px;
            let item_layout_patch = self.item_layout;
            let drag_config = self.drag_config;
            let items = self.items;
            let theme_for_items = theme.clone();
            let root_test_id_for_items = root_test_id.clone();

            let index_now = cx.watch_model(&index_model).copied().unwrap_or(0);
            let offset_now = cx.watch_model(&offset_model).copied().unwrap_or(Px(0.0));
            let runtime_snapshot = cx.watch_model(&runtime_model).copied().unwrap_or_default();

            let reduced_motion =
                prefers_reduced_motion(cx, Invalidation::Paint, false) && !options.ignore_reduced_motion;

            let embla_engine_enabled = match std::env::var("FRET_DEBUG_CAROUSEL_EMBLA_ENGINE").ok() {
                Some(v) if v == "0" || v.eq_ignore_ascii_case("false") => false,
                Some(v) if v == "1" || v.eq_ignore_ascii_case("true") => true,
                _ => options.embla_engine,
            };
            let embla_engine_enabled = embla_engine_enabled && !reduced_motion;

            let mut applied_api_command = false;
            if let Some(api_commands_model) = api_commands_model.as_ref() {
                let pointer_down = runtime_snapshot.drag.armed || runtime_snapshot.drag.dragging;
                let commands = cx
                    .app
                    .models_mut()
                    .update(api_commands_model, |q| {
                        if pointer_down {
                            return Vec::new();
                        }
                        std::mem::take(&mut q.pending)
                    })
                    .unwrap_or_default();

                let cmd = commands.last().copied();
                if let Some(cmd) = cmd {
                    let snaps: Arc<[Px]> = cx
                        .app
                        .models_mut()
                        .read(&snaps_model, |v| v.clone())
                        .ok()
                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
                    let snaps_len = snaps.len();
                    if snaps_len > 1 {
                        if autoplay_stop_on_interaction {
                            let token = cx
                                .app
                                .models_mut()
                                .read(&runtime_model, |st| st.autoplay_token)
                                .ok()
                                .flatten();
                            if let Some(token) = token {
                                cx.app.push_effect(Effect::CancelTimer { token });
                            }
                            let _ = cx.app.models_mut().update(&runtime_model, |st| {
                                st.autoplay_token = None;
                                st.autoplay_paused = true;
                            });
                        }

                        let index: usize = cx
                            .app
                            .models_mut()
                            .read(&index_model, |v| *v)
                            .ok()
                            .unwrap_or(0);
                        let view_size_for_loop = if embla_engine_enabled {
                            cx.app
                                .models_mut()
                                .read(&extent_model, |v| v.0.max(0.0))
                                .ok()
                                .unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        let loop_enabled_effective = if options.loop_enabled && embla_engine_enabled {
                            cx.app
                                .models_mut()
                                .read(&slides_model, |slides| {
                                    if slides.len() != items_len {
                                        return true;
                                    }
                                    let slides = slides
                                        .iter()
                                        .map(|s| headless_embla::slide_looper::Slide1D {
                                            start: s.start.0,
                                            size: s.size.0,
                                        })
                                        .collect::<Vec<_>>();
                                    headless_embla::slide_looper::can_loop(
                                        &slides,
                                        view_size_for_loop,
                                    )
                                })
                                .ok()
                                .unwrap_or(false)
                        } else {
                            options.loop_enabled
                        };
                        let target_index = match cmd {
                            CarouselCommand::ScrollPrev => {
                                if loop_enabled_effective {
                                    headless_snap_points::step_index_wrapped(snaps_len, index, -1)
                                } else {
                                    headless_snap_points::step_index_clamped(snaps_len, index, -1)
                                }
                            }
                            CarouselCommand::ScrollNext => {
                                if loop_enabled_effective {
                                    headless_snap_points::step_index_wrapped(snaps_len, index, 1)
                                } else {
                                    headless_snap_points::step_index_clamped(snaps_len, index, 1)
                                }
                            }
                            CarouselCommand::ScrollTo { index } => Some(if loop_enabled_effective {
                                index % snaps_len.max(1)
                            } else {
                                index.min(snaps_len.saturating_sub(1))
                            }),
                        };

                            if let Some(target_index) = target_index {
                                if embla_engine_enabled {
                                let view_size = view_size_for_loop;
                                let max_offset = cx
                                    .app
                                    .models_mut()
                                    .read(&max_offset_model, |v| *v)
                                    .ok()
                                    .unwrap_or(Px(0.0));
                                let content_size =
                                    (max_offset.0.max(0.0) + view_size.max(0.0)).max(0.0);
                                let mut scroll_snaps =
                                    snaps.iter().map(|px| -px.0).collect::<Vec<_>>();
                                if scroll_snaps.is_empty() {
                                    scroll_snaps.push(0.0);
                                }

                                let mut engine = headless_embla::engine::Engine::new(
                                    scroll_snaps,
                                    content_size,
                                    headless_embla::engine::EngineConfig {
                                        loop_enabled: loop_enabled_effective,
                                        drag_free: options.drag_free,
                                        skip_snaps: options.skip_snaps,
                                        duration: options.embla_duration.max(0.0),
                                        base_friction: 0.68,
                                        view_size,
                                        start_snap: index,
                                    },
                                );
                                let cur = cx
                                    .app
                                    .models_mut()
                                    .read(&offset_model, |v| *v)
                                    .ok()
                                    .unwrap_or(Px(0.0));
                                let loc = -cur.0;
                                engine.scroll_body.set_location(loc);
                                engine.scroll_body.set_target(loc);
                                engine.scroll_target.set_target_vector(loc);
                                let select = engine.scroll_to_index(target_index, 0);

                                let _ = cx.app.models_mut().update(&embla_engine_model, |v| {
                                    *v = Some(engine);
                                });
                                if let Some(select) = select {
                                    let _ = cx.app.models_mut().update(&index_model, |v| {
                                        *v = select.target_snap;
                                    });
                                }
                                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                                    st.drag = headless_carousel::CarouselDragState::default();
                                    st.settling = false;
                                    st.embla_settling = true;
                                    st.selection_initialized = true;
                                });
                                cx.request_frame();
                            } else {
                                let cur = cx
                                    .app
                                    .models_mut()
                                    .read(&offset_model, |v| *v)
                                    .ok()
                                    .unwrap_or(Px(0.0));
                                let target = snaps[target_index];
                                let _ = cx.app.models_mut().update(&embla_engine_model, |v| {
                                    *v = None;
                                });
                                let _ =
                                    cx.app.models_mut().update(&index_model, |v| *v = target_index);
                                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                                    st.drag = headless_carousel::CarouselDragState::default();
                                    st.settling = true;
                                    st.embla_settling = false;
                                    st.settle_from = cur;
                                    st.settle_to = target;
                                    st.settle_generation = st.settle_generation.saturating_add(1);
                                    st.selection_initialized = true;
                                });
                                cx.request_frame();
                            }

                            applied_api_command = true;
                        }
                    }
                }
            }

            let index_now = if applied_api_command {
                cx.watch_model(&index_model).copied().unwrap_or(0)
            } else {
                index_now
            };
            let mut offset_now = if applied_api_command {
                cx.watch_model(&offset_model).copied().unwrap_or(Px(0.0))
            } else {
                offset_now
            };
            let runtime_snapshot = if applied_api_command {
                cx.watch_model(&runtime_model).copied().unwrap_or_default()
            } else {
                runtime_snapshot
            };

            if runtime_snapshot.embla_settling {
                if reduced_motion {
                    let _ = cx.app.models_mut().update(&embla_engine_model, |v| *v = None);
                    let _ = cx.app.models_mut().update(&runtime_model, |st| {
                        st.embla_settling = false;
                    });
                } else {
                let _frames = cx.begin_continuous_frames();

                let max_offset = cx
                    .watch_model(&max_offset_model)
                    .copied()
                    .unwrap_or(Px(0.0));
                let view_size = cx.watch_model(&extent_model).copied().unwrap_or(Px(0.0));
                let mut next_offset = None;
                let mut settled = false;
                let pointer_down = runtime_snapshot.drag.armed || runtime_snapshot.drag.dragging;

                let _ = cx.app.models_mut().update(&embla_engine_model, |v| {
                    let Some(engine) = v.as_mut() else {
                        return;
                    };

                    engine.tick(pointer_down);
                    settled = engine.scroll_body.settled();
                    let loc = engine.scroll_body.location();
                    let max = if engine.loop_enabled() {
                        (max_offset.0 + view_size.0).max(0.0)
                    } else {
                        max_offset.0.max(0.0)
                    };
                    let mapped = Px((-loc).clamp(0.0, max));
                    next_offset = Some(mapped);
                });

                if let Some(next) = next_offset {
                    offset_now = next;
                    let _ = cx.app.models_mut().update(&offset_model, |v| *v = next);
                }

                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                    if settled {
                        st.embla_settling = false;
                    }
                });
                if !settled {
                    cx.request_frame();
                }
                }
            }

            if runtime_snapshot.settling {
                if reduced_motion {
                    let _ = cx.app.models_mut().update(&offset_model, |v| {
                        *v = runtime_snapshot.settle_to;
                    });
                    offset_now = runtime_snapshot.settle_to;
                    let _ = cx.app.models_mut().update(&runtime_model, |st| {
                        st.settling = false;
                    });
                } else {
                let duration = options.duration;
                let settle_generation = runtime_snapshot.settle_generation;
                let motion = cx.keyed(("carousel-settle", settle_generation), |cx| {
                    decl_transition::drive_transition_with_durations_and_easing_duration_with_mount_behavior(
                        cx,
                        true,
                        duration,
                        duration,
                        crate::overlay_motion::shadcn_ease,
                        true,
                    )
                });
                let next = Px(runtime_snapshot.settle_from.0
                    + (runtime_snapshot.settle_to.0 - runtime_snapshot.settle_from.0)
                        * motion.progress);
                offset_now = next;
                let _ = cx.app.models_mut().update(&offset_model, |v| *v = next);
                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                    if !motion.animating {
                        st.settling = false;
                    }
                });
                }
            }

            let axis_offset = offset_now;
            let transform = match orientation {
                CarouselOrientation::Horizontal => {
                    fret_core::Transform2D::translation(Point::new(Px(-axis_offset.0), Px(0.0)))
                }
                CarouselOrientation::Vertical => {
                    fret_core::Transform2D::translation(Point::new(Px(0.0), Px(-axis_offset.0)))
                }
            };

            let offset_for_down = offset_model.clone();
            let runtime_for_down = runtime_model.clone();
            let embla_engine_for_down = embla_engine_model.clone();
            let snaps_for_down = snaps_model.clone();
            let slides_for_down = slides_model.clone();
            let max_offset_for_down = max_offset_model.clone();
            let extent_for_down = extent_model.clone();
            let index_for_down = index_model.clone();
            let autoplay_stop_for_down = autoplay_stop_on_interaction;
            let loop_requested_for_down = options.loop_enabled;
            let embla_engine_enabled_for_down = embla_engine_enabled;
            let embla_duration_for_down = options.embla_duration;
            let skip_snaps_for_down = options.skip_snaps;
            let drag_free_for_down = options.drag_free;
            let on_down: fret_ui::action::OnPointerDown = Arc::new(move |host, _cx, down| {
                if down.button != MouseButton::Left {
                    return false;
                }
                if down.hit_is_text_input {
                    return false;
                }

                let stop_click_should_be_prevented = (|| {
                    if !drag_free_for_down {
                        return false;
                    }
                    if down.pointer_type != fret_core::PointerType::Mouse {
                        return false;
                    }
                    let settling = host
                        .models_mut()
                        .read(&runtime_for_down, |st| st.embla_settling)
                        .ok()
                        .unwrap_or(false);
                    if !settling {
                        return false;
                    }
                    host.models_mut()
                        .read(&embla_engine_for_down, |v| {
                            v.as_ref().is_some_and(|engine| {
                                (engine.scroll_body.target() - engine.scroll_body.location()).abs()
                                    >= 2.0
                            })
                        })
                        .ok()
                        .unwrap_or(false)
                })();

                if autoplay_stop_for_down {
                    let token = host
                        .models_mut()
                        .read(&runtime_for_down, |st| st.autoplay_token)
                        .ok()
                        .flatten();
                    if let Some(token) = token {
                        host.push_effect(Effect::CancelTimer { token });
                    }
                    let _ = host.models_mut().update(&runtime_for_down, |st| {
                        st.autoplay_token = None;
                        st.autoplay_paused = true;
                    });
                }

                let start_offset = host
                    .models_mut()
                    .read(&offset_for_down, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));
                let _ = host.models_mut().update(&runtime_for_down, |st| {
                    headless_carousel::on_pointer_down(
                        &mut st.drag,
                        true,
                        down.position,
                        start_offset,
                    );
                    st.settling = false;
                    st.embla_settling = false;
                    st.prevent_click = stop_click_should_be_prevented;
                });

                let snaps: Arc<[Px]> = host
                    .models_mut()
                    .read(&snaps_for_down, |v| v.clone())
                    .ok()
                    .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
                let max_offset = host
                    .models_mut()
                    .read(&max_offset_for_down, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));
                let view_size = host
                    .models_mut()
                    .read(&extent_for_down, |v| v.0.max(0.0))
                    .ok()
                    .unwrap_or(0.0);
                let index: usize = host
                    .models_mut()
                    .read(&index_for_down, |v| *v)
                    .ok()
                    .unwrap_or(0);

                let can_use_embla_engine = embla_engine_enabled_for_down && snaps.len() > 1;
                let loop_enabled = if can_use_embla_engine && loop_requested_for_down {
                    host.models_mut()
                        .read(&slides_for_down, |slides| {
                            if slides.len() != items_len {
                                return true;
                            }
                            let slides = slides
                                .iter()
                                .map(|s| headless_embla::slide_looper::Slide1D {
                                    start: s.start.0,
                                    size: s.size.0,
                                })
                                .collect::<Vec<_>>();
                            headless_embla::slide_looper::can_loop(&slides, view_size)
                        })
                        .ok()
                        .unwrap_or(loop_requested_for_down)
                } else {
                    false
                };

                let _ = host.models_mut().update(&embla_engine_for_down, |v| {
                    if !can_use_embla_engine || view_size <= 0.0 {
                        *v = None;
                        return;
                    }

                    let content_size = (max_offset.0.max(0.0) + view_size.max(0.0)).max(0.0);
                    let mut scroll_snaps = snaps.iter().map(|px| -px.0).collect::<Vec<_>>();
                    if scroll_snaps.is_empty() {
                        scroll_snaps.push(0.0);
                    }

                    let mut engine = headless_embla::engine::Engine::new(
                        scroll_snaps,
                        content_size,
                        headless_embla::engine::EngineConfig {
                            loop_enabled,
                            drag_free: drag_free_for_down,
                            skip_snaps: skip_snaps_for_down,
                            duration: embla_duration_for_down.max(0.0),
                            base_friction: 0.68,
                            view_size,
                            start_snap: index,
                        },
                    );

                    let loc = -start_offset.0;
                    engine.scroll_body.set_location(loc);
                    engine.scroll_body.set_target(loc);
                    engine.scroll_target.set_target_vector(loc);
                    *v = Some(engine);
                });
                if stop_click_should_be_prevented {
                    host.capture_pointer();
                    true
                } else {
                    false
                }
            });

            let runtime_for_move = runtime_model.clone();
            let offset_for_move = offset_model.clone();
            let max_offset_for_move = max_offset_model.clone();
            let embla_engine_for_move = embla_engine_model.clone();
            let dnd_service_for_move = fret_ui_kit::dnd::dnd_service_model(cx);
            let direction_for_move = layout_direction;
            let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, _cx, mv| {
                let runtime = host
                    .models_mut()
                    .read(&runtime_for_move, |st| *st)
                    .ok()
                    .unwrap_or_default();
                if !runtime.drag.armed && !runtime.drag.dragging {
                    return false;
                }

                if fret_ui_kit::dnd::pointer_is_tracking_any_sensor(
                    host.models_mut(),
                    &dnd_service_for_move,
                    _cx.window,
                    mv.pointer_id,
                ) {
                    let _ = host.models_mut().update(&runtime_for_move, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                    });
                    return false;
                }

                if !mv.buttons.left {
                    let _ = host.models_mut().update(&runtime_for_move, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                    });
                    return false;
                }

                let max_offset = host
                    .models_mut()
                    .read(&max_offset_for_move, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));

                let cur_offset = host
                    .models_mut()
                    .read(&offset_for_move, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));

                let max_offset = if max_offset.0 > 0.0 {
                    max_offset
                } else {
                    // Fall back to the legacy extent-based max offset when snap geometry has not
                    // been measured yet (common in single-frame tests and before the first render
                    // loop settles).
                    let bounds = host.bounds();
                    let extent = match item_basis {
                        Some(px) => px,
                        None => match track_direction {
                            fret_core::Axis::Horizontal => bounds.size.width,
                            fret_core::Axis::Vertical => bounds.size.height,
                        },
                    };
                    Px((extent.0 * (items_len.saturating_sub(1) as f32)).max(0.0))
                };

                if max_offset.0 <= 0.0 {
                    let _ = host.models_mut().update(&runtime_for_move, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                    });
                    return false;
                }

                let mut steal_capture = false;
                let mut consumed = false;
                let mut desired_offset_unclamped = None;

                let _ = host.models_mut().update(&runtime_for_move, |st| {
                    let out = headless_carousel::on_pointer_move(
                        drag_config,
                        &mut st.drag,
                        track_direction,
                        direction_for_move,
                        mv.position,
                        mv.buttons.left,
                        match mv.pointer_type {
                            fret_core::PointerType::Touch => {
                                headless_carousel::CarouselDragInputKind::Touch
                            }
                            _ => headless_carousel::CarouselDragInputKind::Mouse,
                        },
                        max_offset,
                    );
                    steal_capture = out.steal_capture;
                    consumed = out.consumed;

                    if st.drag.dragging {
                        let delta = match track_direction {
                            fret_core::Axis::Horizontal => mv.position.x.0 - st.drag.start.x.0,
                            fret_core::Axis::Vertical => mv.position.y.0 - st.drag.start.y.0,
                        };
                        let sign = if track_direction == fret_core::Axis::Horizontal
                            && direction_for_move == LayoutDirection::Rtl
                        {
                            -1.0
                        } else {
                            1.0
                        };
                        desired_offset_unclamped = Some(Px(st.drag.start_offset.0 - delta * sign));
                    }
                });

                if steal_capture {
                    host.capture_pointer();
                }

                if let Some(desired) = desired_offset_unclamped {
                    let mut next_offset = None;
                    let _ = host.models_mut().update(&embla_engine_for_move, |v| {
                        let Some(engine) = v.as_mut() else {
                            next_offset = Some(Px(desired.0.clamp(0.0, max_offset.0)));
                            return;
                        };

                        let location = -cur_offset.0;
                        let target = -desired.0;
                        engine.scroll_body.set_location(location);
                        engine.scroll_body.set_target(target);
                        engine.constrain_bounds(true);
                        engine.normalize_loop_entities();

                        let constrained = engine.scroll_body.target();
                        next_offset = Some(Px(-constrained));

                        engine.scroll_body.set_location(constrained);
                        engine.scroll_body.set_target(constrained);
                        engine.scroll_target.set_target_vector(constrained);
                    });

                    if let Some(next) = next_offset {
                        let _ = host.models_mut().update(&offset_for_move, |v| *v = next);
                        host.request_redraw(_cx.window);
                    }
                }

                consumed
            });

            let runtime_for_up = runtime_model.clone();
            let offset_for_up = offset_model.clone();
            let index_for_up = index_model.clone();
            let snaps_for_up = snaps_model.clone();
            let max_offset_for_up = max_offset_model.clone();
            let embla_engine_for_up = embla_engine_model.clone();
            let embla_engine_enabled_for_up = embla_engine_enabled;
            let embla_duration_for_up = options.embla_duration;
            let skip_snaps_for_up = options.skip_snaps;
            let drag_free_for_up = options.drag_free;
            let loop_requested_for_up = options.loop_enabled;
            let slides_for_up = slides_model.clone();
            let direction_for_up = layout_direction;
            let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, cx, up| {
                let runtime = host
                    .models_mut()
                    .read(&runtime_for_up, |st| *st)
                    .ok()
                    .unwrap_or_default();
                if !runtime.drag.dragging {
                    if runtime.prevent_click {
                        host.release_pointer_capture();
                        let _ = host.models_mut().update(&runtime_for_up, |st| {
                            st.drag = headless_carousel::CarouselDragState::default();
                            st.prevent_click = false;
                        });
                        return true;
                    }
                    let _ = host.models_mut().update(&runtime_for_up, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                        st.prevent_click = false;
                    });
                    return false;
                }

                host.release_pointer_capture();

                let offset = host
                    .models_mut()
                    .read(&offset_for_up, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));
                let max_offset = host
                    .models_mut()
                    .read(&max_offset_for_up, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));

                let mut drag = runtime.drag;
                let snaps: Arc<[Px]> = host
                    .models_mut()
                    .read(&snaps_for_up, |v| v.clone())
                    .ok()
                    .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));

                let can_use_embla_engine =
                    embla_engine_enabled_for_up && snaps.len() > 1 && up.velocity_window.is_some();

                if can_use_embla_engine {
                    let bounds = host.bounds();
                    let view_size = match track_direction {
                        fret_core::Axis::Horizontal => bounds.size.width.0.max(0.0),
                        fret_core::Axis::Vertical => bounds.size.height.0.max(0.0),
                    };

                    let content_size = if max_offset.0 > 0.0 {
                        max_offset.0 + view_size
                    } else {
                        let extent = match item_basis {
                            Some(px) => px,
                            None => match track_direction {
                                fret_core::Axis::Horizontal => bounds.size.width,
                                fret_core::Axis::Vertical => bounds.size.height,
                            },
                        };
                        (extent.0 * (items_len as f32)).max(0.0)
                    };

                    let mut scroll_snaps = snaps.iter().map(|px| -px.0).collect::<Vec<_>>();
                    if scroll_snaps.is_empty() {
                        scroll_snaps.push(0.0);
                    }

                    let index: usize = host
                        .models_mut()
                        .read(&index_for_up, |v| *v)
                        .ok()
                        .unwrap_or(0);

                    let loop_enabled = if loop_requested_for_up {
                        host.models_mut()
                            .read(&slides_for_up, |slides| {
                                if slides.len() != items_len {
                                    return true;
                                }
                                let slides = slides
                                    .iter()
                                    .map(|s| headless_embla::slide_looper::Slide1D {
                                        start: s.start.0,
                                        size: s.size.0,
                                    })
                                    .collect::<Vec<_>>();
                                headless_embla::slide_looper::can_loop(&slides, view_size)
                            })
                            .ok()
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    let mut engine = headless_embla::engine::Engine::new(
                        scroll_snaps,
                        content_size,
                        headless_embla::engine::EngineConfig {
                            loop_enabled,
                            drag_free: drag_free_for_up,
                            skip_snaps: skip_snaps_for_up,
                            duration: embla_duration_for_up.max(0.0),
                            base_friction: 0.68,
                            view_size,
                            start_snap: index,
                        },
                    );

                    // Sync engine state to the current drag location (Embla updates target while
                    // dragging; our v1 path directly updates the offset model).
                    let engine_location = -offset.0;
                    engine.scroll_body.set_location(engine_location);
                    engine.scroll_body.set_target(engine_location);
                    engine.scroll_target.set_target_vector(engine_location);

                    let pointer_kind = match up.pointer_type {
                        fret_core::PointerType::Touch => headless_embla::drag_release::PointerKind::Touch,
                        _ => headless_embla::drag_release::PointerKind::Mouse,
                    };
                    let velocity = up
                        .velocity_window
                        .map(|v| match track_direction {
                            fret_core::Axis::Horizontal => v.x.0,
                            fret_core::Axis::Vertical => v.y.0,
                        })
                        .unwrap_or(0.0);
                    // Embla uses px/ms (`event.timeStamp` is ms). Fret provides px/s.
                    let velocity = velocity / 1000.0;

                    let sign = if track_direction == fret_core::Axis::Horizontal
                        && direction_for_up == LayoutDirection::Rtl
                    {
                        -1.0
                    } else {
                        1.0
                    };
                    let (_release, select) =
                        engine.on_drag_release(pointer_kind, velocity, move |v| v * sign);

                    let _ = host.models_mut().update(&embla_engine_for_up, |v| {
                        *v = Some(engine);
                    });

                    if let Some(select) = select {
                        let _ = host
                            .models_mut()
                            .update(&index_for_up, |v| *v = select.target_snap);
                    }

                    let _ = host.models_mut().update(&runtime_for_up, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                        st.settling = false;
                        st.embla_settling = true;
                        st.prevent_click = false;
                        st.selection_initialized = true;
                    });
                    host.push_effect(Effect::RequestAnimationFrame(cx.window));
                    host.request_redraw(cx.window);
                    return true;
                }
                let release = if snaps.len() > 1 {
                    headless_carousel::on_pointer_up_with_snaps_options(
                        drag_config,
                        &mut drag,
                        track_direction,
                        direction_for_up,
                        up.position,
                        &snaps,
                        max_offset,
                        loop_requested_for_up,
                        skip_snaps_for_up,
                        drag_free_for_up,
                    )
                    .expect("release output when dragging")
                } else {
                    let bounds = host.bounds();
                    let extent = match item_basis {
                        Some(px) => px,
                        None => match track_direction {
                            fret_core::Axis::Horizontal => bounds.size.width,
                            fret_core::Axis::Vertical => bounds.size.height,
                        },
                    };
                    headless_carousel::on_pointer_up(
                        drag_config,
                        &mut drag,
                        track_direction,
                        direction_for_up,
                        up.position,
                        extent,
                        items_len,
                    )
                    .expect("release output when dragging")
                };

                let _ = host
                    .models_mut()
                    .update(&index_for_up, |v| *v = release.next_index);
                let _ = host.models_mut().update(&runtime_for_up, |st| {
                    st.drag = headless_carousel::CarouselDragState::default();
                    st.settling = true;
                    st.embla_settling = false;
                    st.prevent_click = false;
                    st.settle_from = offset;
                    st.settle_to = release.target_offset;
                    st.settle_generation = st.settle_generation.saturating_add(1);
                });
                host.request_redraw(cx.window);
                true
            });

            let runtime_for_cancel = runtime_model.clone();
            let embla_engine_for_cancel = embla_engine_model.clone();
            let on_cancel: fret_ui::action::OnPointerCancel = Arc::new(move |host, cx, _cancel| {
                let runtime = host
                    .models_mut()
                    .read(&runtime_for_cancel, |st| *st)
                    .ok()
                    .unwrap_or_default();
                if !runtime.drag.dragging && !runtime.drag.armed {
                    return false;
                }

                host.release_pointer_capture();
                let _ = host.models_mut().update(&runtime_for_cancel, |st| {
                    st.drag = headless_carousel::CarouselDragState::default();
                    st.settling = false;
                    st.embla_settling = false;
                    st.prevent_click = false;
                });
                let _ = host.models_mut().update(&embla_engine_for_cancel, |v| {
                    *v = None;
                });
                host.request_redraw(cx.window);
                true
            });

            let offset_for_prev = offset_model.clone();
            let runtime_for_prev = runtime_model.clone();
            let index_for_prev = index_model.clone();
            let snaps_for_prev = snaps_model.clone();
            let embla_engine_for_prev = embla_engine_model.clone();
            let max_offset_for_prev = max_offset_model.clone();
            let extent_for_prev = extent_model.clone();
            let autoplay_stop_for_prev = autoplay_stop_on_interaction;
            let loop_requested_for_prev = options.loop_enabled;
            let slides_for_prev = slides_model.clone();
            let embla_engine_enabled_for_prev = embla_engine_enabled;
            let embla_duration_for_prev = options.embla_duration;
            let skip_snaps_for_prev = options.skip_snaps;
            let drag_free_for_prev = options.drag_free;
            let on_prev: fret_ui::action::OnActivate = Arc::new(
                move |host: &mut dyn UiActionHost, cx: ActionCx, _reason: ActivateReason| {
                    if autoplay_stop_for_prev {
                        let token = host
                            .models_mut()
                            .read(&runtime_for_prev, |st| st.autoplay_token)
                            .ok()
                            .flatten();
                        if let Some(token) = token {
                            host.push_effect(Effect::CancelTimer { token });
                        }
                        let _ = host.models_mut().update(&runtime_for_prev, |st| {
                            st.autoplay_token = None;
                            st.autoplay_paused = true;
                        });
                    }

                    let snaps: Arc<[Px]> = host
                        .models_mut()
                        .read(&snaps_for_prev, |v| v.clone())
                        .ok()
                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
                    let index: usize = host
                        .models_mut()
                        .read(&index_for_prev, |v| *v)
                        .ok()
                        .unwrap_or(0);
                    if snaps.len() <= 1 {
                        return;
                    }

                    if embla_engine_enabled_for_prev {
                        let view_size = host
                            .models_mut()
                            .read(&extent_for_prev, |v| v.0.max(0.0))
                            .ok()
                            .unwrap_or(0.0);
                        let max_offset = host
                            .models_mut()
                            .read(&max_offset_for_prev, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let content_size = (max_offset.0.max(0.0) + view_size.max(0.0)).max(0.0);
                        let mut scroll_snaps = snaps.iter().map(|px| -px.0).collect::<Vec<_>>();
                        if scroll_snaps.is_empty() {
                            scroll_snaps.push(0.0);
                        }

                        let loop_enabled = if loop_requested_for_prev {
                            host.models_mut()
                                .read(&slides_for_prev, |slides| {
                                    if slides.len() != items_len {
                                        return true;
                                    }
                                    let slides = slides
                                        .iter()
                                        .map(|s| headless_embla::slide_looper::Slide1D {
                                            start: s.start.0,
                                            size: s.size.0,
                                        })
                                        .collect::<Vec<_>>();
                                    headless_embla::slide_looper::can_loop(&slides, view_size)
                                })
                                .ok()
                                .unwrap_or(false)
                        } else {
                            false
                        };

                        let mut engine = headless_embla::engine::Engine::new(
                            scroll_snaps,
                            content_size,
                            headless_embla::engine::EngineConfig {
                                loop_enabled,
                                drag_free: drag_free_for_prev,
                                skip_snaps: skip_snaps_for_prev,
                                duration: embla_duration_for_prev.max(0.0),
                                base_friction: 0.68,
                                view_size,
                                start_snap: index,
                            },
                        );
                        let cur = host
                            .models_mut()
                            .read(&offset_for_prev, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let loc = -cur.0;
                        engine.scroll_body.set_location(loc);
                        engine.scroll_body.set_target(loc);
                        engine.scroll_target.set_target_vector(loc);
                        let select = engine.scroll_to_prev();

                        let _ = host.models_mut().update(&embla_engine_for_prev, |v| {
                            *v = Some(engine);
                        });
                        if let Some(select) = select {
                            let _ = host
                                .models_mut()
                                .update(&index_for_prev, |v| *v = select.target_snap);
                        }
                        let _ = host.models_mut().update(&runtime_for_prev, |st| {
                            st.drag = headless_carousel::CarouselDragState::default();
                            st.settling = false;
                            st.embla_settling = true;
                            st.selection_initialized = true;
                        });
                        host.push_effect(Effect::RequestAnimationFrame(cx.window));
                        host.request_redraw(cx.window);
                        return;
                    }

                    let target_index = if loop_requested_for_prev {
                        headless_snap_points::step_index_wrapped(snaps.len(), index, -1)
                    } else {
                        headless_snap_points::step_index_clamped(snaps.len(), index, -1)
                    };
                    let Some(target_index) = target_index else {
                        return;
                    };
                    if target_index == index {
                        return;
                    }
                    let target = snaps[target_index];
                    let cur = host
                        .models_mut()
                        .read(&offset_for_prev, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));
                    let _ = host
                        .models_mut()
                        .update(&index_for_prev, |v| *v = target_index);
                    let _ = host.models_mut().update(&runtime_for_prev, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                        st.settling = true;
                        st.embla_settling = false;
                        st.settle_from = cur;
                        st.settle_to = target;
                        st.settle_generation = st.settle_generation.saturating_add(1);
                    });
                    host.request_redraw(cx.window);
                },
            );

            let offset_for_next = offset_model.clone();
            let runtime_for_next = runtime_model.clone();
            let index_for_next = index_model.clone();
            let snaps_for_next = snaps_model.clone();
            let embla_engine_for_next = embla_engine_model.clone();
            let max_offset_for_next = max_offset_model.clone();
            let extent_for_next = extent_model.clone();
            let autoplay_stop_for_next = autoplay_stop_on_interaction;
            let loop_requested_for_next = options.loop_enabled;
            let slides_for_next = slides_model.clone();
            let embla_engine_enabled_for_next = embla_engine_enabled;
            let embla_duration_for_next = options.embla_duration;
            let skip_snaps_for_next = options.skip_snaps;
            let drag_free_for_next = options.drag_free;
            let on_next: fret_ui::action::OnActivate = Arc::new(
                move |host: &mut dyn UiActionHost, cx: ActionCx, _reason: ActivateReason| {
                    if autoplay_stop_for_next {
                        let token = host
                            .models_mut()
                            .read(&runtime_for_next, |st| st.autoplay_token)
                            .ok()
                            .flatten();
                        if let Some(token) = token {
                            host.push_effect(Effect::CancelTimer { token });
                        }
                        let _ = host.models_mut().update(&runtime_for_next, |st| {
                            st.autoplay_token = None;
                            st.autoplay_paused = true;
                        });
                    }

                    let snaps: Arc<[Px]> = host
                        .models_mut()
                        .read(&snaps_for_next, |v| v.clone())
                        .ok()
                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
                    let index: usize = host
                        .models_mut()
                        .read(&index_for_next, |v| *v)
                        .ok()
                        .unwrap_or(0);
                    if snaps.len() <= 1 {
                        return;
                    }

                    if embla_engine_enabled_for_next {
                        let view_size = host
                            .models_mut()
                            .read(&extent_for_next, |v| v.0.max(0.0))
                            .ok()
                            .unwrap_or(0.0);
                        let max_offset = host
                            .models_mut()
                            .read(&max_offset_for_next, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let content_size = (max_offset.0.max(0.0) + view_size.max(0.0)).max(0.0);
                        let mut scroll_snaps = snaps.iter().map(|px| -px.0).collect::<Vec<_>>();
                        if scroll_snaps.is_empty() {
                            scroll_snaps.push(0.0);
                        }

                        let loop_enabled = if loop_requested_for_next {
                            host.models_mut()
                                .read(&slides_for_next, |slides| {
                                    if slides.len() != items_len {
                                        return true;
                                    }
                                    let slides = slides
                                        .iter()
                                        .map(|s| headless_embla::slide_looper::Slide1D {
                                            start: s.start.0,
                                            size: s.size.0,
                                        })
                                        .collect::<Vec<_>>();
                                    headless_embla::slide_looper::can_loop(&slides, view_size)
                                })
                                .ok()
                                .unwrap_or(false)
                        } else {
                            false
                        };

                        let mut engine = headless_embla::engine::Engine::new(
                            scroll_snaps,
                            content_size,
                            headless_embla::engine::EngineConfig {
                                loop_enabled,
                                drag_free: drag_free_for_next,
                                skip_snaps: skip_snaps_for_next,
                                duration: embla_duration_for_next.max(0.0),
                                base_friction: 0.68,
                                view_size,
                                start_snap: index,
                            },
                        );
                        let cur = host
                            .models_mut()
                            .read(&offset_for_next, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let loc = -cur.0;
                        engine.scroll_body.set_location(loc);
                        engine.scroll_body.set_target(loc);
                        engine.scroll_target.set_target_vector(loc);
                        let select = engine.scroll_to_next();

                        let _ = host.models_mut().update(&embla_engine_for_next, |v| {
                            *v = Some(engine);
                        });
                        if let Some(select) = select {
                            let _ = host
                                .models_mut()
                                .update(&index_for_next, |v| *v = select.target_snap);
                        }
                        let _ = host.models_mut().update(&runtime_for_next, |st| {
                            st.drag = headless_carousel::CarouselDragState::default();
                            st.settling = false;
                            st.embla_settling = true;
                            st.selection_initialized = true;
                        });
                        host.push_effect(Effect::RequestAnimationFrame(cx.window));
                        host.request_redraw(cx.window);
                        return;
                    }

                    let target_index = if loop_requested_for_next {
                        headless_snap_points::step_index_wrapped(snaps.len(), index, 1)
                    } else {
                        headless_snap_points::step_index_clamped(snaps.len(), index, 1)
                    };
                    let Some(target_index) = target_index else {
                        return;
                    };
                    if target_index == index {
                        return;
                    }
                    let target = snaps[target_index];
                    let cur = host
                        .models_mut()
                        .read(&offset_for_next, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));
                    let _ = host
                        .models_mut()
                        .update(&index_for_next, |v| *v = target_index);
                    let _ = host.models_mut().update(&runtime_for_next, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                        st.settling = true;
                        st.embla_settling = false;
                        st.settle_from = cur;
                        st.settle_to = target;
                        st.settle_generation = st.settle_generation.saturating_add(1);
                    });
                    host.request_redraw(cx.window);
                },
            );

            let index_for_key = index_model.clone();
            let offset_for_key = offset_model.clone();
            let runtime_for_key = runtime_model.clone();
            let snaps_for_key = snaps_model.clone();
            let embla_engine_for_key = embla_engine_model.clone();
            let max_offset_for_key = max_offset_model.clone();
            let extent_for_key = extent_model.clone();
            let autoplay_stop_for_key = autoplay_stop_on_interaction;
            let loop_requested_for_key = options.loop_enabled;
            let slides_for_key = slides_model.clone();
            let embla_engine_enabled_for_key = embla_engine_enabled;
            let embla_duration_for_key = options.embla_duration;
            let skip_snaps_for_key = options.skip_snaps;
            let drag_free_for_key = options.drag_free;
            let watch_focus_for_key = options.watch_focus;
            let direction_for_key = layout_direction;
            let on_key_down: OnKeyDown = Arc::new(
                move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                      cx: ActionCx,
                      down: KeyDownCx| {
                    if watch_focus_for_key && down.key == KeyCode::Tab {
                        let _ = host.models_mut().update(&runtime_for_key, |st| {
                            st.focus_tab_generation = st.focus_tab_generation.saturating_add(1);
                        });
                        host.request_redraw(cx.window);
                        return false;
                    }

                    let snaps: Arc<[Px]> = host
                        .models_mut()
                        .read(&snaps_for_key, |v| v.clone())
                        .ok()
                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
                    if snaps.len() <= 1 {
                        return false;
                    }

                    // shadcn/ui v4 Carousel uses left/right keys even when `orientation="vertical"`
                    // (it rotates the controls instead of switching the key mapping).
                    let (prev_key, next_key) = match direction_for_key {
                        LayoutDirection::Rtl => (KeyCode::ArrowRight, KeyCode::ArrowLeft),
                        LayoutDirection::Ltr => (KeyCode::ArrowLeft, KeyCode::ArrowRight),
                    };

                    if down.key != prev_key && down.key != next_key {
                        return false;
                    }

                    if autoplay_stop_for_key {
                        let token = host
                            .models_mut()
                            .read(&runtime_for_key, |st| st.autoplay_token)
                            .ok()
                            .flatten();
                        if let Some(token) = token {
                            host.push_effect(Effect::CancelTimer { token });
                        }
                        let _ = host.models_mut().update(&runtime_for_key, |st| {
                            st.autoplay_token = None;
                            st.autoplay_paused = true;
                        });
                    }

                    let index: usize = host
                        .models_mut()
                        .read(&index_for_key, |v| *v)
                        .ok()
                        .unwrap_or(0);

                    let view_size_for_loop = host
                        .models_mut()
                        .read(&extent_for_key, |v| v.0.max(0.0))
                        .ok()
                        .unwrap_or(0.0);
                    let loop_enabled = if loop_requested_for_key {
                        host.models_mut()
                            .read(&slides_for_key, |slides| {
                                if slides.len() != items_len {
                                    return true;
                                }
                                let slides = slides
                                    .iter()
                                    .map(|s| headless_embla::slide_looper::Slide1D {
                                        start: s.start.0,
                                        size: s.size.0,
                                    })
                                    .collect::<Vec<_>>();
                                headless_embla::slide_looper::can_loop(&slides, view_size_for_loop)
                            })
                            .ok()
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    let delta = if down.key == prev_key { -1 } else { 1 };
                    let target_index = if loop_enabled {
                        headless_snap_points::step_index_wrapped(snaps.len(), index, delta)
                    } else {
                        headless_snap_points::step_index_clamped(snaps.len(), index, delta)
                    };
                    let Some(target_index) = target_index else {
                        return true;
                    };
                    if target_index == index {
                        return true;
                    }

                    let target = snaps[target_index];
                    let cur = host
                        .models_mut()
                        .read(&offset_for_key, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));

                    if embla_engine_enabled_for_key {
                        let view_size = host
                            .models_mut()
                            .read(&extent_for_key, |v| v.0.max(0.0))
                            .ok()
                            .unwrap_or(0.0);
                        let max_offset = host
                            .models_mut()
                            .read(&max_offset_for_key, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let content_size =
                            (max_offset.0.max(0.0) + view_size.max(0.0)).max(0.0);
                        let mut scroll_snaps = snaps.iter().map(|px| -px.0).collect::<Vec<_>>();
                        if scroll_snaps.is_empty() {
                            scroll_snaps.push(0.0);
                        }

                        let mut engine = headless_embla::engine::Engine::new(
                            scroll_snaps,
                            content_size,
                            headless_embla::engine::EngineConfig {
                                loop_enabled,
                                drag_free: drag_free_for_key,
                                skip_snaps: skip_snaps_for_key,
                                duration: embla_duration_for_key.max(0.0),
                                base_friction: 0.68,
                                view_size,
                                start_snap: index,
                            },
                        );
                        let loc = -cur.0;
                        engine.scroll_body.set_location(loc);
                        engine.scroll_body.set_target(loc);
                        engine.scroll_target.set_target_vector(loc);

                        let select = engine.scroll_to_index(target_index, headless_embla::utils::DIRECTION_NONE);

                        let _ = host.models_mut().update(&embla_engine_for_key, |v| {
                            *v = Some(engine);
                        });
                        if let Some(select) = select {
                            let _ = host
                                .models_mut()
                                .update(&index_for_key, |v| *v = select.target_snap);
                        }
                        let _ = host.models_mut().update(&runtime_for_key, |st| {
                            st.drag = headless_carousel::CarouselDragState::default();
                            st.settling = false;
                            st.embla_settling = true;
                            st.selection_initialized = true;
                        });
                        host.push_effect(Effect::RequestAnimationFrame(cx.window));
                        host.request_redraw(cx.window);
                        return true;
                    }

                    let _ = host
                        .models_mut()
                        .update(&index_for_key, |v| *v = target_index);
                    let _ = host.models_mut().update(&runtime_for_key, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                        st.settling = true;
                        st.settle_from = cur;
                        st.settle_to = target;
                        st.settle_generation = st.settle_generation.saturating_add(1);
                    });
                    host.request_redraw(cx.window);
                    true
                },
            );

            let slides_prev: Arc<[headless_carousel::CarouselSlide1D]> = cx
                .watch_model(&slides_model)
                .layout()
                .cloned()
                .unwrap_or_else(|| Arc::from(Vec::<headless_carousel::CarouselSlide1D>::new()));
            let view_size_prev = cx.watch_model(&extent_model).copied().unwrap_or(Px(0.0));
            let content_size_prev = slides_prev
                .iter()
                .map(|s| s.start.0 + s.size.0)
                .fold(0.0f32, |a, b| a.max(b));
            let loop_translates = if options.loop_enabled
                && embla_engine_enabled
                && slides_prev.len() == items_len
                && content_size_prev > 0.0
                && view_size_prev.0 > 0.0
            {
                let slides = slides_prev
                    .iter()
                    .map(|s| headless_embla::slide_looper::Slide1D {
                        start: s.start.0,
                        size: s.size.0,
                    })
                    .collect::<Vec<_>>();
                headless_embla::slide_looper::compute_slide_translates_if_can_loop(
                    &slides,
                    axis_offset.0,
                    content_size_prev,
                    view_size_prev.0,
                )
            } else {
                Vec::new()
            };
            let loop_translates_for_items = loop_translates.clone();

            let mut item_ids = Vec::with_capacity(items_len);
            let item_ids_ref = &mut item_ids;
            let mut slide_content_ids = Vec::with_capacity(items_len);
            let slide_content_ids_ref = &mut slide_content_ids;
            let track = cx.flex(
                FlexProps {
                    layout: track_layout,
                    direction: track_direction,
                    align: CrossAlign::Stretch,
                    wrap: false,
                    ..Default::default()
                },
                move |cx| {
                    items
                        .into_iter()
                        .enumerate()
                        .map(|(idx, item)| {
                            let per_item_layout_patch = item.layout;
                            let per_item_basis = per_item_layout_patch
                                .flex_item
                                .as_ref()
                                .and_then(|f| f.basis.as_ref())
                                .is_some();

                            let item_pad_space =
                                item.padding_start.unwrap_or(item_pad_default_space);
                            let item_pad = decl_style::space(&theme_for_items, item_pad_space);

                            let content = item.child;
                            slide_content_ids_ref.push(content.id);

                            let mut item_layout = LayoutRefinement::default()
                                .flex_none()
                                .min_w(MetricRef::Px(Px(0.0)))
                                .merge(item_layout_patch.clone())
                                .merge(per_item_layout_patch);

                            if !per_item_basis {
                                if let Some(basis) = item_basis {
                                    item_layout = item_layout
                                        .basis(LengthRefinement::Px(MetricRef::Px(basis)));

                                    // When an explicit item basis is provided, treat it as the
                                    // authoritative snap extent and clamp the item's main-axis size
                                    // to match. This keeps `item_basis_main_px` deterministic even
                                    // when children would otherwise expand the flex item.
                                    if track_direction == fret_core::Axis::Vertical {
                                        item_layout = item_layout
                                            .h_px(MetricRef::Px(basis))
                                            .min_h(MetricRef::Px(basis))
                                            .max_h(MetricRef::Px(basis));
                                    }
                                } else if item_layout
                                    .flex_item
                                    .as_ref()
                                    .and_then(|f| f.basis.as_ref())
                                    .is_none()
                                {
                                    // Match shadcn/ui v4 `basis-full` default.
                                    item_layout = item_layout.basis_fraction(1.0);
                                }
                            }

                            let item_layout =
                                decl_style::layout_style(&theme_for_items, item_layout);
                            let test_id = Arc::from(format!(
                                "{}-item-{}",
                                root_test_id_for_items.as_ref(),
                                idx + 1
                            ));

                            let padding = match track_direction {
                                fret_core::Axis::Horizontal => Edges {
                                    left: item_pad,
                                    ..Edges::all(Px(0.0))
                                },
                                fret_core::Axis::Vertical => Edges {
                                    top: item_pad,
                                    ..Edges::all(Px(0.0))
                                },
                            };

                            let item = cx.container(
                                fret_ui::element::ContainerProps {
                                    layout: item_layout,
                                    padding: padding.into(),
                                    ..Default::default()
                                },
                                move |_cx| vec![content],
                            );

                            item_ids_ref.push(item.id);

                            let item = item.attach_semantics(
                                SemanticsDecoration::default()
                                    .role(SemanticsRole::Group)
                                    .label(Arc::from(format!("Slide {} of {}", idx + 1, items_len)))
                                    .test_id(test_id),
                            );

                            let translate =
                                loop_translates_for_items.get(idx).copied().unwrap_or(0.0);
                            if translate == 0.0 {
                                return item;
                            }

                            let transform = match orientation {
                                CarouselOrientation::Horizontal => fret_core::Transform2D::translation(
                                    Point::new(Px(translate), Px(0.0)),
                                ),
                                CarouselOrientation::Vertical => fret_core::Transform2D::translation(
                                    Point::new(Px(0.0), Px(translate)),
                                ),
                            };
                            cx.render_transform_props(
                                RenderTransformProps {
                                    layout: LayoutStyle::default(),
                                    transform,
                                },
                                move |_cx| vec![item],
                            )
                        })
                        .collect::<Vec<_>>()
                },
            );

            let track = cx.render_transform_props(
                RenderTransformProps {
                    layout: LayoutStyle::default(),
                    transform,
                },
                move |_cx| vec![track],
            );

            let pointer_layout =
                decl_style::layout_style(&theme, LayoutRefinement::default().size_full());

            let drag_enabled = items_len > 1 && options.draggable;
            let pointer_region = cx.pointer_region(
                PointerRegionProps {
                    layout: pointer_layout,
                    enabled: drag_enabled,
                    capture_phase_pointer_moves: true,
                },
                move |cx| {
                    cx.pointer_region_on_pointer_down(on_down);
                    cx.pointer_region_on_pointer_move(on_move);
                    cx.pointer_region_on_pointer_up(on_up);
                    cx.pointer_region_on_pointer_cancel(on_cancel);
                    vec![track]
                },
            );

            let (viewport_id, viewport) = cx.keyed((root_test_id.clone(), "viewport"), |cx| {
                let id = cx.root_id();
                (
                    id,
                    AnyElement::new(
                        id,
                        ElementKind::Container(ContainerProps {
                            layout: viewport_layout,
                            ..Default::default()
                        }),
                        vec![pointer_region],
                    ),
                )
            });

            let snaps_prev: Arc<[Px]> = cx
                .watch_model(&snaps_model)
                .cloned()
                .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
            let max_offset_prev = cx.watch_model(&max_offset_model).copied().unwrap_or(Px(0.0));
            let view_size_prev = cx.watch_model(&extent_model).copied().unwrap_or(Px(0.0));

            let viewport_bounds = cx.last_bounds_for_element(viewport_id);
            let viewport_width_now = viewport_bounds.map(|b| b.size.width).unwrap_or(Px(0.0));
            let _ = cx
                .app
                .models_mut()
                .update(&viewport_width_model, |v| *v = viewport_width_now);
            let view_size_now = viewport_bounds
                .map(|b| match orientation {
                    CarouselOrientation::Horizontal => b.size.width,
                    CarouselOrientation::Vertical => b.size.height,
                })
                .unwrap_or(Px(0.0));
            let _ = cx
                .app
                .models_mut()
                .update(&extent_model, |v| *v = view_size_now);

            let mut snaps_now: Vec<Px> = vec![Px(0.0)];
            let mut max_offset_now = Px(0.0);
            let mut slides_now: Vec<headless_carousel::CarouselSlide1D> = Vec::new();
            let mut snap_by_slide_now: Vec<usize> = Vec::new();

            if view_size_now.0 > 0.0 {
                if let Some(viewport_bounds) = viewport_bounds {
                    let mut slides = Vec::with_capacity(items_len);
                    for id in &item_ids {
                        let Some(bounds) = cx.last_bounds_for_element(*id) else {
                            continue;
                        };

                         let (start, size) = match orientation {
                             CarouselOrientation::Horizontal => (
                                Px(bounds.origin.x.0 - viewport_bounds.origin.x.0),
                                 bounds.size.width,
                             ),
                             CarouselOrientation::Vertical => (
                                Px(bounds.origin.y.0 - viewport_bounds.origin.y.0),
                                 bounds.size.height,
                             ),
                         };
                         slides.push(headless_carousel::CarouselSlide1D { start, size });
                     }

                    let mut start_gap = Px(0.0);
                    if slides.len() == items_len {
                        let min_start = slides
                            .iter()
                            .map(|s| s.start)
                            .fold(Px(f32::INFINITY), |a, b| Px(a.0.min(b.0)));
                        if min_start.0.is_finite() && min_start.0 > 0.0 {
                            start_gap = min_start;
                            for s in &mut slides {
                                s.start = Px(s.start.0 - min_start.0);
                            }
                        }
                    }

                    let slides_ready =
                        slides.len() == items_len && slides.iter().all(|s| s.size.0 > 0.0);

                    if slides_ready {
                        slides_now = slides.clone();
                        let model = headless_carousel::snap_model_1d(
                            view_size_now,
                            &slides,
                            start_gap,
                            Px(0.0),
                            options.slides_to_scroll.to_headless(),
                            false,
                            options.align.to_headless(),
                            options.contain_scroll.to_headless(),
                            options.pixel_tolerance_px.max(0.0),
                        );
                        snaps_now = model.snaps_px;
                        max_offset_now = model.max_offset_px;
                        snap_by_slide_now = model.snap_by_slide;
                    } else if let Some(first_item_id) = item_ids.first().copied()
                        && let Some(first_bounds) = cx.last_bounds_for_element(first_item_id)
                    {
                        // Fallback: keep the previous "uniform item" approximation when we don't
                        // have stable bounds for all items yet (e.g. first frame).
                        let (first_start, item_main_size) = match orientation {
                            CarouselOrientation::Horizontal => (
                                Px(first_bounds.origin.x.0 - viewport_bounds.origin.x.0),
                                first_bounds.size.width,
                            ),
                            CarouselOrientation::Vertical => (
                                Px(first_bounds.origin.y.0 - viewport_bounds.origin.y.0),
                                first_bounds.size.height,
                            ),
                        };

                        if item_main_size.0 > 0.0 {
                            let slides = (0..items_len)
                                .map(|i| headless_carousel::CarouselSlide1D {
                                    start: Px(first_start.0 + (i as f32) * item_main_size.0),
                                    size: item_main_size,
                                })
                                .collect::<Vec<_>>();
                            slides_now = slides.clone();

                            let model = headless_carousel::snap_model_1d(
                                view_size_now,
                                &slides,
                                Px(0.0),
                                Px(0.0),
                                options.slides_to_scroll.to_headless(),
                                false,
                                options.align.to_headless(),
                                options.contain_scroll.to_headless(),
                                options.pixel_tolerance_px.max(0.0),
                            );
                            snaps_now = model.snaps_px;
                            max_offset_now = model.max_offset_px;
                            snap_by_slide_now = model.snap_by_slide;
                        }
                    }
                };
            }

            if options.watch_focus && view_size_now.0 > 0.0 && !snap_by_slide_now.is_empty() {
                let pointer_down = runtime_snapshot.drag.armed || runtime_snapshot.drag.dragging;
                let focused_now = cx.focused_element();
                let focus_changed = focused_now != runtime_snapshot.focus_last_focused_element;
                if !pointer_down && focus_changed {
                    let tab_pending = runtime_snapshot.focus_tab_generation
                        != runtime_snapshot.focus_last_handled_tab_generation;
                    let maybe_slide_index_and_center = (|| {
                        let focused = focused_now?;
                        let focused_bounds = cx
                            .last_visual_bounds_for_element(focused)
                            .or_else(|| cx.last_bounds_for_element(focused))?;
                        let center = Point::new(
                            Px(focused_bounds.origin.x.0 + focused_bounds.size.width.0 * 0.5),
                            Px(focused_bounds.origin.y.0 + focused_bounds.size.height.0 * 0.5),
                        );

                        for (idx, item_id) in item_ids.iter().enumerate() {
                            let bounds = cx
                                .last_visual_bounds_for_element(*item_id)
                                .or_else(|| cx.last_bounds_for_element(*item_id));
                            if bounds.is_some_and(|b| b.contains(center)) {
                                return Some((idx, center));
                            }
                        }
                        None
                    })();

                    if let Some((slide_index, center)) = maybe_slide_index_and_center {
                        let focus_offscreen = viewport_bounds.is_some_and(|b| !b.contains(center));
                        let snap_index = snap_by_slide_now
                            .get(slide_index)
                            .copied()
                            .unwrap_or(slide_index)
                            .min(snaps_now.len().saturating_sub(1));
                        if (tab_pending || focus_offscreen)
                            && let Some(target_snap) = snaps_now.get(snap_index).copied()
                        {
                            let loop_enabled_effective = if options.loop_enabled {
                                let slides_prev: Arc<[headless_carousel::CarouselSlide1D]> = cx
                                    .watch_model(&slides_model)
                                    .layout()
                                    .cloned()
                                    .unwrap_or_else(|| {
                                        Arc::from(Vec::<headless_carousel::CarouselSlide1D>::new())
                                    });
                                if slides_prev.len() == items_len {
                                    let slides = slides_prev
                                        .iter()
                                        .map(|s| headless_embla::slide_looper::Slide1D {
                                            start: s.start.0,
                                            size: s.size.0,
                                        })
                                        .collect::<Vec<_>>();
                                    headless_embla::slide_looper::can_loop(
                                        &slides,
                                        view_size_now.0.max(0.0),
                                    )
                                } else {
                                    true
                                }
                            } else {
                                false
                            };

                            let offset_max = if loop_enabled_effective {
                                (max_offset_now.0 + view_size_now.0).max(0.0)
                            } else {
                                max_offset_now.0.max(0.0)
                            };
                            let target_offset = Px(target_snap.0.clamp(0.0, offset_max));

                            if embla_engine_enabled {
                                let content_size =
                                    (max_offset_now.0.max(0.0) + view_size_now.0.max(0.0)).max(0.0);
                                let mut scroll_snaps = snaps_now.iter().map(|px| -px.0).collect::<Vec<_>>();
                                if scroll_snaps.is_empty() {
                                    scroll_snaps.push(0.0);
                                }

                                let cur = cx.watch_model(&offset_model).copied().unwrap_or(Px(0.0));
                                let start_snap = cx.watch_model(&index_model).copied().unwrap_or(0);

                                let mut next_selected = None;
                                let mut next_offset = None;
                                let _ = cx.app.models_mut().update(&embla_engine_model, |v| {
                                    if let Some(engine) = v.as_mut() {
                                        engine.set_options(
                                            loop_enabled_effective,
                                            options.drag_free,
                                            options.skip_snaps,
                                            options.embla_duration,
                                        );
                                        engine.scroll_body.use_duration(0.0);
                                        let _ = engine.scroll_to_index(
                                            snap_index,
                                            headless_embla::utils::DIRECTION_NONE,
                                        );
                                        engine.tick(false);
                                        engine.scroll_body.use_base_duration();

                                        next_selected = Some(engine.index_current);
                                        next_offset = Some(Px((-engine.scroll_body.location()).clamp(
                                            0.0,
                                            offset_max,
                                        )));
                                        return;
                                    }

                                    let mut engine = headless_embla::engine::Engine::new(
                                        scroll_snaps,
                                        content_size,
                                        headless_embla::engine::EngineConfig {
                                            loop_enabled: loop_enabled_effective,
                                            drag_free: options.drag_free,
                                            skip_snaps: options.skip_snaps,
                                            duration: 0.0,
                                            base_friction: 0.68,
                                            view_size: view_size_now.0.max(0.0),
                                            start_snap,
                                        },
                                    );
                                    let loc = -cur.0;
                                    engine.scroll_body.set_location(loc);
                                    engine.scroll_body.set_target(loc);
                                    engine.scroll_target.set_target_vector(loc);
                                    let _ = engine.scroll_to_index(
                                        snap_index,
                                        headless_embla::utils::DIRECTION_NONE,
                                    );
                                    engine.tick(false);
                                    engine.scroll_body.use_base_duration();

                                    next_selected = Some(engine.index_current);
                                    next_offset = Some(Px((-engine.scroll_body.location()).clamp(
                                        0.0,
                                        offset_max,
                                    )));

                                    *v = Some(engine);
                                });

                                if let Some(selected) = next_selected {
                                    let _ = cx.app.models_mut().update(&index_model, |v| *v = selected);
                                } else {
                                    let _ = cx.app.models_mut().update(&index_model, |v| *v = snap_index);
                                }
                                if let Some(next_offset) = next_offset {
                                    let _ = cx.app.models_mut().update(&offset_model, |v| *v = next_offset);
                                } else {
                                    let _ = cx.app.models_mut().update(&offset_model, |v| *v = target_offset);
                                }
                                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                                    st.drag = headless_carousel::CarouselDragState::default();
                                    st.settling = false;
                                    st.embla_settling = false;
                                    st.selection_initialized = true;
                                });
                            } else {
                                let _ = cx.app.models_mut().update(&embla_engine_model, |v| *v = None);
                                let _ = cx.app.models_mut().update(&index_model, |v| *v = snap_index);
                                let _ = cx.app.models_mut().update(&offset_model, |v| *v = target_offset);
                                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                                    st.drag = headless_carousel::CarouselDragState::default();
                                    st.settling = false;
                                    st.embla_settling = false;
                                    st.selection_initialized = true;
                                });
                            }

                            cx.request_frame();
                        }
                    }

                    if tab_pending {
                        let _ = cx.app.models_mut().update(&runtime_model, |st| {
                            st.focus_last_handled_tab_generation =
                                runtime_snapshot.focus_tab_generation;
                        });
                    }
                }
            }

            let focused_now = cx.focused_element();
            if focused_now != runtime_snapshot.focus_last_focused_element {
                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                    st.focus_last_focused_element = focused_now;
                });
            }

            if let Some(snapshot_model) = slides_in_view_snapshot_model.clone() {
                if view_size_now.0 > 0.0 && !slides_now.is_empty() {
                    let tracker_model = carousel_slides_in_view_tracker_model(cx);
                    let threshold = options.in_view_threshold;
                    let margin_px = options.in_view_margin_px.0;
                    let snapshot_prev_generation = cx
                        .watch_model(&snapshot_model)
                        .layout()
                        .read_ref(|v| v.generation)
                        .ok()
                        .unwrap_or(0);

                    let slides_for_view = slides_now
                        .iter()
                        .enumerate()
                        .map(|(idx, s)| {
                            let translate = loop_translates.get(idx).copied().unwrap_or(0.0);
                            headless_embla::slide_looper::Slide1D {
                                start: s.start.0 + translate,
                                size: s.size.0,
                            }
                        })
                        .collect::<Vec<_>>();

                    let mut update_out = None;
                    let mut generation = 0;
                    let _ = cx.app.models_mut().update(&tracker_model, |tracker| {
                        let out = tracker.update(
                            &slides_for_view,
                            axis_offset.0.max(0.0),
                            view_size_now.0.max(0.0),
                            threshold,
                            margin_px,
                        );
                        generation = tracker.generation();
                        update_out = Some(out);
                    });

                    if let Some(update_out) = update_out {
                        let should_emit = update_out.changed || snapshot_prev_generation == 0;
                        if should_emit {
                            let headless_embla::slides_in_view::SlidesInViewUpdate {
                                slides_in_view,
                                slides_enter_view,
                                slides_left_view,
                                changed: _,
                            } = update_out;

                            let _ = cx.app.models_mut().update(&snapshot_model, |v| {
                                v.slides_in_view = Arc::from(slides_in_view.into_boxed_slice());
                                v.slides_enter_view =
                                    Arc::from(slides_enter_view.into_boxed_slice());
                                v.slides_left_view =
                                    Arc::from(slides_left_view.into_boxed_slice());
                                v.generation = generation;
                            });
                        }
                    }
                }
            }

            let snaps_arc: Arc<[Px]> = Arc::from(snaps_now.clone().into_boxed_slice());
            let _ = cx
                .app
                .models_mut()
                .update(&snaps_model, |v| *v = snaps_arc.clone());
            let slides_arc: Arc<[headless_carousel::CarouselSlide1D]> =
                Arc::from(slides_now.into_boxed_slice());
            let _ = cx
                .app
                .models_mut()
                .update(&slides_model, |v| *v = slides_arc.clone());
            let _ = cx
                .app
                .models_mut()
                .update(&max_offset_model, |v| *v = max_offset_now);

            let eps = 0.001;
            let snaps_changed = snaps_prev.len() != snaps_now.len()
                || snaps_prev
                    .iter()
                    .zip(snaps_now.iter())
                    .any(|(a, b)| (a.0 - b.0).abs() > eps);
            let max_offset_changed = (max_offset_prev.0 - max_offset_now.0).abs() > eps;
            let view_size_changed = (view_size_prev.0 - view_size_now.0).abs() > eps;

            let mut did_reinit = false;
            if (snaps_changed || max_offset_changed || view_size_changed) && view_size_now.0 > 0.0 {
                did_reinit = true;
            }
            let options_changed = options_prev.loop_enabled != options.loop_enabled
                || options_prev.skip_snaps != options.skip_snaps
                || options_prev.drag_free != options.drag_free
                || options_prev.embla_engine != options.embla_engine
                || (options_prev.embla_duration - options.embla_duration).abs() > eps;
            if options_changed && view_size_now.0 > 0.0 {
                did_reinit = true;
            }

            if view_size_now.0 > 0.0 && !slide_content_ids.is_empty() {
                let slide_content_ids_model = carousel_slide_content_ids_model(cx);
                let prev: Arc<[fret_ui::elements::GlobalElementId]> = cx
                    .watch_model(&slide_content_ids_model)
                    .cloned()
                    .unwrap_or_else(|| Arc::from(Vec::<fret_ui::elements::GlobalElementId>::new()));
                let now: Arc<[fret_ui::elements::GlobalElementId]> =
                    Arc::from(slide_content_ids.clone().into_boxed_slice());

                let content_ids_changed = prev.len() != now.len()
                    || prev
                        .iter()
                        .zip(now.iter())
                        .any(|(a, b)| a != b);
                if content_ids_changed {
                    did_reinit = true;
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&slide_content_ids_model, |v| *v = now);
                }
            }

            if did_reinit && snaps_now.len() > 1 {
                let scroll_snaps = snaps_now.iter().map(|px| -px.0).collect::<Vec<_>>();
                let content_size = (max_offset_now.0 + view_size_now.0).max(0.0);
                let view_size = view_size_now.0.max(0.0);
                let loop_enabled_effective = if options.loop_enabled && slides_arc.len() == items_len {
                    let slides = slides_arc
                        .iter()
                        .map(|s| headless_embla::slide_looper::Slide1D {
                            start: s.start.0,
                            size: s.size.0,
                        })
                        .collect::<Vec<_>>();
                    headless_embla::slide_looper::can_loop(&slides, view_size)
                } else {
                    options.loop_enabled
                };

                let pointer_down =
                    runtime_snapshot.drag.armed || runtime_snapshot.drag.dragging;

                let mut selected = None;
                let _ = cx.app.models_mut().update(&embla_engine_model, |v| {
                    let Some(engine) = v.as_mut() else {
                        return;
                    };

                    engine.set_options(
                        loop_enabled_effective,
                        options.drag_free,
                        options.skip_snaps,
                        options.embla_duration,
                    );

                    let _ev = engine.reinit(scroll_snaps, content_size, view_size);
                    engine.constrain_bounds(pointer_down);
                    selected = Some(engine.index_current);
                });

                if let Some(selected) = selected {
                    let _ = cx.app.models_mut().update(&index_model, |v| *v = selected);
                    let _ = cx.app.models_mut().update(&runtime_model, |st| {
                        st.settling = false;
                    });
                }
            }

            // Clamp index/offset when snaps change (e.g. window resize).
            let snaps_len = snaps_now.len();
            let selection_source_index = if !runtime_snapshot.selection_initialized
                && !runtime_snapshot.settling
                && !runtime_snapshot.embla_settling
                && !runtime_snapshot.drag.dragging
            {
                options.start_snap
            } else {
                index_now
            };
            let clamped_index = if snaps_len == 0 {
                // Keep the requested selection until we have measurable snaps to clamp against.
                selection_source_index
            } else {
                selection_source_index.min(snaps_len.saturating_sub(1))
            };
            if snaps_len > 0 && clamped_index != index_now {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&index_model, |v| *v = clamped_index);
            }
            let loop_enabled_effective = if options.loop_enabled && slides_arc.len() == items_len {
                let slides = slides_arc
                    .iter()
                    .map(|s| headless_embla::slide_looper::Slide1D {
                        start: s.start.0,
                        size: s.size.0,
                    })
                    .collect::<Vec<_>>();
                headless_embla::slide_looper::can_loop(&slides, view_size_now.0.max(0.0))
            } else {
                options.loop_enabled
            };
            let offset_max = if loop_enabled_effective {
                (max_offset_now.0 + view_size_now.0).max(0.0)
            } else {
                max_offset_now.0.max(0.0)
            };
            let offset_clamped = Px(offset_now.0.clamp(0.0, offset_max));
            if offset_clamped != offset_now {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&offset_model, |v| *v = offset_clamped);
            }

            // Match upstream behavior: controls are disabled until the carousel has a measurable
            // viewport/item extent (Embla initializes canScrollPrev/Next to false until it
            // measures and emits `select`/`reInit`).
            let extent_ready = view_size_now.0 > 0.0 && snaps_len > 0;

            let now_frame = cx.app.frame_id().0;
            const REINIT_EMIT_MIN_FRAME_DELTA: u64 = 4;
            let reinit_pending = runtime_snapshot.api_reinit_pending || did_reinit;
            let should_emit_reinit = if reinit_pending && !did_reinit {
                // Geometry has stabilized after a change burst: emit once immediately so app state
                // (counters, button states) can converge.
                true
            } else {
                // Continuous churn: throttle to at most once per N frames.
                did_reinit
                    && runtime_snapshot
                        .api_last_reinit_emit_frame
                        .map_or(true, |last| {
                            now_frame.saturating_sub(last) >= REINIT_EMIT_MIN_FRAME_DELTA
                        })
            };

            if should_emit_reinit
                || (extent_ready && clamped_index != runtime_snapshot.api_last_selected_index)
            {
                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                    st.api_reinit_pending = reinit_pending && !should_emit_reinit;
                    if should_emit_reinit {
                        st.api_reinit_generation = st.api_reinit_generation.saturating_add(1);
                        st.api_last_reinit_emit_frame = Some(now_frame);
                    }
                    if extent_ready && clamped_index != st.api_last_selected_index {
                        st.api_last_selected_index = clamped_index;
                        st.api_select_generation = st.api_select_generation.saturating_add(1);
                    }
                });
            } else if did_reinit && !runtime_snapshot.api_reinit_pending {
                // Ensure we don't drop a re-init that was detected but throttled this frame.
                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                    st.api_reinit_pending = true;
                });
            }

            // Embla's `startSnap` selects the initial snap before the user interacts. Because this
            // recipe derives snaps from measured geometry (available after at least one layout
            // pass), we apply the initial snap once snaps become available.
            if extent_ready
                && !runtime_snapshot.selection_initialized
                && !runtime_snapshot.settling
                && !runtime_snapshot.embla_settling
                && !runtime_snapshot.drag.dragging
                {
                    let target = snaps_now
                        .get(clamped_index)
                        .copied()
                        .unwrap_or(Px(0.0));
                let offset_max = if loop_enabled_effective {
                    (max_offset_now.0 + view_size_now.0).max(0.0)
                } else {
                    max_offset_now.0.max(0.0)
                };
                let target = Px(target.0.clamp(0.0, offset_max));
                let _ = cx.app.models_mut().update(&offset_model, |v| *v = target);
                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                    st.selection_initialized = true;
                });
                cx.request_frame();
            }

            let prev_disabled =
                !extent_ready || snaps_len <= 1 || (!loop_enabled_effective && clamped_index == 0);
            let next_disabled = !extent_ready
                || snaps_len <= 1
                || (!loop_enabled_effective && clamped_index + 1 >= snaps_len);

            if let Some(api_snapshot) = api_snapshot_model.as_ref() {
                let runtime_now = cx.watch_model(&runtime_model).copied().unwrap_or_default();
                let (embla_engine_present, embla_moving, embla_scroll_duration) = cx
                    .app
                    .models_mut()
                    .read(&embla_engine_model, |v| {
                        let Some(engine) = v.as_ref() else {
                            return (false, false, 0.0);
                        };
                        (
                            true,
                            !engine.scroll_body.settled(),
                            engine.scroll_body.duration(),
                        )
                    })
                    .ok()
                    .unwrap_or((false, false, 0.0));
                let at_selected_snap = extent_ready
                    && snaps_now
                        .get(clamped_index)
                        .is_some_and(|snap| (snap.0 - offset_now.0).abs() <= 0.5);
                let selected_snap_px = if extent_ready {
                    snaps_now.get(clamped_index).map(|s| s.0).unwrap_or(0.0)
                } else {
                    0.0
                };
                let snapshot = CarouselApiSnapshot {
                    selected_index: clamped_index,
                    snap_count: if extent_ready { snaps_len } else { 0 },
                    can_scroll_prev: !prev_disabled,
                    can_scroll_next: !next_disabled,
                    settling: runtime_now.settling || runtime_now.embla_settling || embla_moving,
                    recipe_settling: runtime_now.settling,
                    embla_settling: runtime_now.embla_settling,
                    at_selected_snap,
                    offset_px: offset_now.0,
                    selected_snap_px,
                    embla_engine_enabled,
                    embla_duration: options.embla_duration.max(0.0),
                    embla_engine_present,
                    embla_scroll_duration,
                    select_generation: runtime_now.api_select_generation,
                    reinit_generation: runtime_now.api_reinit_generation,
                };
                let _ = cx.app.models_mut().update(api_snapshot, |v| *v = snapshot);
            }

            let (prev_part, next_part) = match controls {
                CarouselControls::None => (None, None),
                CarouselControls::BuiltIn => (Some(CarouselPrevious::new()), Some(CarouselNext::new())),
                CarouselControls::Parts { previous, next } => (Some(previous), Some(next)),
            };

            let (prev_wrapper, next_wrapper) = if let (Some(prev_part), Some(next_part)) =
                (prev_part, next_part)
            {
                let rotate_controls = orientation == CarouselOrientation::Vertical;
                let arrow_rotation = if rotate_controls { 90.0 } else { 0.0 };
                let arrow_center = Point::new(Px(8.0), Px(8.0));
                let arrow_transform =
                    fret_core::Transform2D::rotation_about_degrees(arrow_rotation, arrow_center);
                let arrow_layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(Px(16.0))
                        .h_px(Px(16.0))
                        .flex_shrink_0(),
                );

                let rtl_controls = layout_direction == LayoutDirection::Rtl
                    && orientation == CarouselOrientation::Horizontal;
                let (prev_icon, next_icon) = if rtl_controls {
                    (ids::ui::ARROW_RIGHT, ids::ui::ARROW_LEFT)
                } else {
                    (ids::ui::ARROW_LEFT, ids::ui::ARROW_RIGHT)
                };

                let prev_test_id = prev_part.test_id.clone().unwrap_or_else(|| {
                    Arc::from(format!("{}-previous", root_test_id.as_ref()))
                });
                let next_test_id = next_part.test_id.clone().unwrap_or_else(|| {
                    Arc::from(format!("{}-next", root_test_id.as_ref()))
                });

                let prev_label = prev_part
                    .label
                    .clone()
                    .unwrap_or_else(|| Arc::from("Previous slide"));
                let next_label = next_part
                    .label
                    .clone()
                    .unwrap_or_else(|| Arc::from("Next slide"));

                let prev_button = Button::new(prev_label)
                    .variant(prev_part.variant)
                    .size(prev_part.size)
                    .disabled(prev_disabled)
                    .test_id(prev_test_id)
                    .refine_style(prev_part.chrome)
                    .children([cx.visual_transform_props(
                        VisualTransformProps {
                            layout: arrow_layout,
                            transform: arrow_transform,
                        },
                        move |cx| vec![decl_icon::icon(cx, prev_icon)],
                    )])
                    .on_activate(on_prev)
                    .into_element(cx);

                let next_button = Button::new(next_label)
                    .variant(next_part.variant)
                    .size(next_part.size)
                    .disabled(next_disabled)
                    .test_id(next_test_id)
                    .refine_style(next_part.chrome)
                    .children([cx.visual_transform_props(
                        VisualTransformProps {
                            layout: arrow_layout,
                            transform: arrow_transform,
                        },
                        move |cx| vec![decl_icon::icon(cx, next_icon)],
                    )])
                    .on_activate(on_next)
                    .into_element(cx);

                // Upstream shadcn uses `-left-12` / `-right-12` which maps to 48px in Tailwind.
                let offset = MetricRef::Px(Px(48.0));
                let button_size = MetricRef::Px(Px(32.0));

                let (prev_layout, next_layout) = match orientation {
                    CarouselOrientation::Horizontal => {
                        if rtl_controls {
                            (
                                LayoutRefinement::default()
                                    .absolute()
                                    .top(Space::N0)
                                    .bottom(Space::N0)
                                    .right_neg_px(offset.clone())
                                    .w_px(button_size.clone())
                                    .merge(prev_part.layout),
                                LayoutRefinement::default()
                                    .absolute()
                                    .top(Space::N0)
                                    .bottom(Space::N0)
                                    .left_neg_px(offset)
                                    .w_px(button_size)
                                    .merge(next_part.layout),
                            )
                        } else {
                            (
                                LayoutRefinement::default()
                                    .absolute()
                                    .top(Space::N0)
                                    .bottom(Space::N0)
                                    .left_neg_px(offset.clone())
                                    .w_px(button_size.clone())
                                    .merge(prev_part.layout),
                                LayoutRefinement::default()
                                    .absolute()
                                    .top(Space::N0)
                                    .bottom(Space::N0)
                                    .right_neg_px(offset)
                                    .w_px(button_size)
                                    .merge(next_part.layout),
                            )
                        }
                    }
                    CarouselOrientation::Vertical => (
                        LayoutRefinement::default()
                            .absolute()
                            .left(Space::N0)
                            .right(Space::N0)
                            .top_neg_px(offset.clone())
                            .h_px(button_size.clone())
                            .merge(prev_part.layout),
                        LayoutRefinement::default()
                            .absolute()
                            .left(Space::N0)
                            .right(Space::N0)
                            .bottom_neg_px(offset)
                            .h_px(button_size)
                            .merge(next_part.layout),
                    ),
                };

                let prev_layout = decl_style::layout_style(&theme, prev_layout);
                let next_layout = decl_style::layout_style(&theme, next_layout);

                let prev_wrapper = cx.flex(
                    FlexProps {
                        layout: prev_layout,
                        direction: button_axis,
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                        wrap: false,
                        ..Default::default()
                    },
                    move |_cx| vec![prev_button],
                );

                let next_wrapper = cx.flex(
                    FlexProps {
                        layout: next_layout,
                        direction: button_axis,
                        justify: MainAlign::Center,
                        align: CrossAlign::Center,
                        wrap: false,
                        ..Default::default()
                    },
                    move |_cx| vec![next_button],
                );

                (Some(prev_wrapper), Some(next_wrapper))
            } else {
                (None, None)
            };

            let pointer_can_hover =
                fret_ui_kit::declarative::primary_pointer_can_hover(cx, Invalidation::Layout, true);
            let runtime_for_hover = runtime_model.clone();
            let snaps_for_hover = snaps_model.clone();
            let theme_for_hover = theme.clone();
            let root = cx.container(
                ContainerProps {
                    layout: root_layout,
                    ..Default::default()
                },
                move |cx| {
                    let hover_overlay = autoplay_cfg.map(|cfg| {
                        cx.hover_region(
                            HoverRegionProps {
                                layout: decl_style::layout_style(
                                    &theme_for_hover,
                                    LayoutRefinement::default()
                                        .absolute()
                                        .top(Space::N0)
                                        .right(Space::N0)
                                        .bottom(Space::N0)
                                        .left(Space::N0),
                                ),
                            },
                            move |cx, hovered| {
                                let hovered = hovered && pointer_can_hover;

                                let runtime_snapshot = cx
                                    .watch_model(&runtime_for_hover)
                                    .copied()
                                    .unwrap_or_default();

                                let was_hovered = runtime_snapshot.autoplay_hovered;
                                let left_hover = was_hovered && !hovered;
                                let entered_hover = !was_hovered && hovered;

                                if was_hovered != hovered {
                                    let _ = cx.app.models_mut().update(&runtime_for_hover, |st| {
                                        st.autoplay_hovered = hovered;
                                    });
                                }

                                if entered_hover && cfg.pause_on_hover {
                                    if let Some(token) = runtime_snapshot.autoplay_token {
                                        cx.app.push_effect(Effect::CancelTimer { token });
                                    }
                                    let _ = cx.app.models_mut().update(&runtime_for_hover, |st| {
                                        st.autoplay_token = None;
                                    });
                                }

                                if left_hover
                                    && cfg.reset_on_hover_leave
                                    && !runtime_snapshot.autoplay_paused
                                {
                                    let snaps: Arc<[Px]> = cx
                                        .watch_model(&snaps_for_hover)
                                        .layout()
                                        .cloned()
                                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));

                                    if snaps.len() > 1 && runtime_snapshot.autoplay_token.is_none()
                                    {
                                        let token = cx.app.next_timer_token();
                                        let _ =
                                            cx.app.models_mut().update(&runtime_for_hover, |st| {
                                                st.autoplay_token = Some(token);
                                            });
                                        cx.app.push_effect(Effect::SetTimer {
                                            window: Some(cx.window),
                                            token,
                                            after: cfg.delay,
                                            repeat: None,
                                        });
                                    }
                                }

                                if !hovered
                                    && !runtime_snapshot.autoplay_paused
                                    && runtime_snapshot.autoplay_token.is_none()
                                {
                                    let snaps: Arc<[Px]> = cx
                                        .watch_model(&snaps_for_hover)
                                        .layout()
                                        .cloned()
                                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));

                                    if snaps.len() > 1 {
                                        let token = cx.app.next_timer_token();
                                        let _ =
                                            cx.app.models_mut().update(&runtime_for_hover, |st| {
                                                st.autoplay_token = Some(token);
                                            });
                                        cx.app.push_effect(Effect::SetTimer {
                                            window: Some(cx.window),
                                            token,
                                            after: cfg.delay,
                                            repeat: None,
                                        });
                                    }
                                }

                                Vec::new()
                            },
                        )
                    });

                    let mut children = vec![viewport];
                    if let Some(prev_wrapper) = prev_wrapper {
                        children.push(prev_wrapper);
                    }
                    if let Some(next_wrapper) = next_wrapper {
                        children.push(next_wrapper);
                    }
                    if let Some(hover_overlay) = hover_overlay {
                        children.push(hover_overlay);
                    }
                    children
                },
            );

                if let Some(cfg) = autoplay_cfg {
                    let runtime_for_timer = runtime_model.clone();
                    let snaps_for_timer = snaps_model.clone();
                    let index_for_timer = index_model.clone();
                    let offset_for_timer = offset_model.clone();
                    let loop_requested_for_timer = options.loop_enabled;
                    let extent_for_timer = extent_model.clone();
                    let slides_for_timer = slides_model.clone();
                    cx.timer_on_timer_for(
                        root.id,
                        Arc::new(move |host, action_cx, token| {
                        let runtime = host
                            .models_mut()
                            .read(&runtime_for_timer, |st| *st)
                            .ok()
                            .unwrap_or_default();
                        if runtime.autoplay_token.as_ref() != Some(&token) {
                            return false;
                        }
                        if runtime.autoplay_paused
                            || (cfg.pause_on_hover && runtime.autoplay_hovered)
                        {
                            let _ = host.models_mut().update(&runtime_for_timer, |st| {
                                st.autoplay_token = None;
                            });
                            return true;
                        }

                        let snaps: Arc<[Px]> = host
                            .models_mut()
                            .read(&snaps_for_timer, |v| v.clone())
                            .ok()
                            .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
                        if snaps.len() <= 1 {
                            let _ = host.models_mut().update(&runtime_for_timer, |st| {
                                st.autoplay_token = None;
                            });
                            return true;
                        }

                        let index: usize = host
                            .models_mut()
                            .read(&index_for_timer, |v| *v)
                            .ok()
                            .unwrap_or(0);
                        let view_size_for_loop = host
                            .models_mut()
                            .read(&extent_for_timer, |v| v.0.max(0.0))
                            .ok()
                            .unwrap_or(0.0);
                        let loop_enabled = if loop_requested_for_timer {
                            host.models_mut()
                                .read(&slides_for_timer, |slides| {
                                    if slides.len() != items_len {
                                        return true;
                                    }
                                    let slides = slides
                                        .iter()
                                        .map(|s| headless_embla::slide_looper::Slide1D {
                                            start: s.start.0,
                                            size: s.size.0,
                                        })
                                        .collect::<Vec<_>>();
                                    headless_embla::slide_looper::can_loop(&slides, view_size_for_loop)
                                })
                                .ok()
                                .unwrap_or(false)
                        } else {
                            false
                        };
                        let target_index = if index + 1 < snaps.len() {
                            index + 1
                        } else if loop_enabled {
                            0
                        } else {
                            let _ = host.models_mut().update(&runtime_for_timer, |st| {
                                st.autoplay_token = None;
                            });
                            return true;
                        };

                        let target = snaps[target_index];
                        let cur = host
                            .models_mut()
                            .read(&offset_for_timer, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let _ = host
                            .models_mut()
                            .update(&index_for_timer, |v| *v = target_index);
                        let _ = host.models_mut().update(&runtime_for_timer, |st| {
                            st.drag = headless_carousel::CarouselDragState::default();
                            st.settling = true;
                            st.settle_from = cur;
                            st.settle_to = target;
                            st.settle_generation = st.settle_generation.saturating_add(1);
                        });
                        host.request_redraw(action_cx.window);

                        host.push_effect(Effect::SetTimer {
                            window: Some(action_cx.window),
                            token,
                            after: cfg.delay,
                            repeat: None,
                        });
                        true
                    }),
                );
            }

            cx.key_add_on_key_down_capture_for(root.id, on_key_down);

            let orientation_semantics = match orientation {
                CarouselOrientation::Horizontal => SemanticsOrientation::Horizontal,
                CarouselOrientation::Vertical => SemanticsOrientation::Vertical,
            };
            root.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Panel)
                    .label("Carousel")
                    .orientation(orientation_semantics)
                    .test_id(root_test_id),
            )
        })
    }
}

/// shadcn/ui `CarouselContent` (v4).
#[derive(Debug)]
pub struct CarouselContent {
    items: Vec<CarouselItem>,
    viewport_layout: LayoutRefinement,
    track_layout: LayoutRefinement,
    item_layout: LayoutRefinement,
    track_start_neg_margin: Option<Space>,
}

impl CarouselContent {
    pub fn new(items: impl IntoIterator<Item = CarouselItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            viewport_layout: LayoutRefinement::default(),
            track_layout: LayoutRefinement::default(),
            item_layout: LayoutRefinement::default(),
            track_start_neg_margin: None,
        }
    }

    pub fn refine_viewport_layout(mut self, layout: LayoutRefinement) -> Self {
        self.viewport_layout = self.viewport_layout.merge(layout);
        self
    }

    pub fn refine_track_layout(mut self, layout: LayoutRefinement) -> Self {
        self.track_layout = self.track_layout.merge(layout);
        self
    }

    pub fn refine_item_layout(mut self, layout: LayoutRefinement) -> Self {
        self.item_layout = self.item_layout.merge(layout);
        self
    }

    /// Match shadcn's `CarouselContent className="-ml-4"` / `"-mt-4"` spacing approach.
    ///
    /// This is a convenience surface that maps to [`Carousel::track_start_neg_margin`].
    pub fn track_start_neg_margin(mut self, margin: Space) -> Self {
        self.track_start_neg_margin = Some(margin);
        self
    }
}

/// shadcn/ui `CarouselItem` (v4).
#[derive(Debug)]
pub struct CarouselItem {
    child: AnyElement,
    layout: LayoutRefinement,
    padding_start: Option<Space>,
}

impl CarouselItem {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
            layout: LayoutRefinement::default(),
            padding_start: None,
        }
    }

    /// Matches shadcn's per-item `className` surface (e.g. `md:basis-1/2`).
    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    /// Matches shadcn's per-item `className="pl-4"` / `pt-4` spacing approach.
    ///
    /// This is a convenience surface that maps to the recipe's start padding for each slide.
    pub fn padding_start(mut self, padding: Space) -> Self {
        self.padding_start = Some(padding);
        self
    }
}

impl From<AnyElement> for CarouselItem {
    fn from(child: AnyElement) -> Self {
        Self::new(child)
    }
}

/// shadcn/ui `CarouselPrevious` (v4).
#[derive(Debug, Clone)]
pub struct CarouselPrevious {
    label: Option<Arc<str>>,
    variant: ButtonVariant,
    size: ButtonSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl Default for CarouselPrevious {
    fn default() -> Self {
        Self {
            label: None,
            variant: ButtonVariant::Outline,
            size: ButtonSize::IconSm,
            chrome: ChromeRefinement::default().rounded(Radius::Full),
            layout: LayoutRefinement::default(),
            test_id: None,
        }
    }
}

impl CarouselPrevious {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }
}

/// shadcn/ui `CarouselNext` (v4).
#[derive(Debug, Clone)]
pub struct CarouselNext {
    label: Option<Arc<str>>,
    variant: ButtonVariant,
    size: ButtonSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl Default for CarouselNext {
    fn default() -> Self {
        Self {
            label: None,
            variant: ButtonVariant::Outline,
            size: ButtonSize::IconSm,
            chrome: ChromeRefinement::default().rounded(Radius::Full),
            layout: LayoutRefinement::default(),
            test_id: None,
        }
    }
}

impl CarouselNext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }
}

#[derive(Default)]
struct CarouselContextProviderState {
    current: Option<CarouselContext>,
}

#[derive(Debug, Clone)]
pub struct CarouselContext {
    pub api_handle: Model<Option<CarouselApi>>,
    pub api_snapshot: Model<CarouselApiSnapshot>,
    pub orientation: CarouselOrientation,
    pub options: CarouselOptions,
    pub root_test_id: Arc<str>,
}

pub fn carousel_context<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<CarouselContext> {
    cx.inherited_state_where::<CarouselContextProviderState>(|st| st.current.is_some())
        .and_then(|st| st.current.clone())
}

#[track_caller]
pub fn use_carousel<H: UiHost>(cx: &ElementContext<'_, H>) -> CarouselContext {
    carousel_context(cx).expect("use_carousel must be used within a `Carousel`")
}
