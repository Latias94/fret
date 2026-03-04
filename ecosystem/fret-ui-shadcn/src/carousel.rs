use std::sync::Arc;
use std::time::Duration;

use fret_core::time::Instant;
use fret_core::{
    Edges, KeyCode, LayoutDirection, MouseButton, Point, Px, SemanticsOrientation, SemanticsRole,
};
use fret_icons::ids;
use fret_runtime::{Effect, Model, ModelHost, ModelStore, TimerToken};
use fret_ui::action::{
    ActionCx, ActivateReason, KeyDownCx, OnKeyDown, OnWheel, UiActionHost, WheelCx,
};
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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CarouselAutoplayApiSnapshot {
    /// True when an autoplay plugin is attached and the carousel has more than one snap.
    pub active: bool,
    /// True while the carousel has an armed autoplay timer.
    pub playing: bool,
    /// True when the carousel is paused by an external caller (e.g. a wrapper hover region).
    pub paused_external: bool,
    /// True when autoplay was stopped due to a user interaction (`stop_on_interaction=true`).
    pub stopped_by_interaction: bool,
    /// True when autoplay stopped after reaching the last snap (`stop_on_last_snap=true`).
    pub stopped_by_last_snap: bool,
    /// True when the carousel is hovered (when hover is supported).
    pub hovered: bool,
    /// Remaining time until the next snap while autoplay is running.
    pub time_until_next: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct CarouselAutoplayApi {
    commands: Model<CarouselAutoplayCommandQueue>,
    snapshot: Model<CarouselAutoplayApiSnapshot>,
    delays: Model<Option<Arc<[Duration]>>>,
}

impl CarouselAutoplayApi {
    pub fn play(&self, host: &mut impl ModelHost) {
        let _ = host.update_model(&self.commands, |q, _| {
            q.pending.push(CarouselAutoplayCommand::Play);
        });
    }

    pub fn stop(&self, host: &mut impl ModelHost) {
        let _ = host.update_model(&self.commands, |q, _| {
            q.pending.push(CarouselAutoplayCommand::Stop);
        });
    }

    pub fn pause(&self, host: &mut impl ModelHost) {
        let _ = host.update_model(&self.commands, |q, _| {
            q.pending.push(CarouselAutoplayCommand::Pause);
        });
    }

    pub fn reset(&self, host: &mut impl ModelHost) {
        let _ = host.update_model(&self.commands, |q, _| {
            q.pending.push(CarouselAutoplayCommand::Reset);
        });
    }

    pub fn play_store(&self, store: &mut ModelStore) {
        let _ = store.update(&self.commands, |q| {
            q.pending.push(CarouselAutoplayCommand::Play);
        });
    }

    pub fn stop_store(&self, store: &mut ModelStore) {
        let _ = store.update(&self.commands, |q| {
            q.pending.push(CarouselAutoplayCommand::Stop);
        });
    }

    pub fn pause_store(&self, store: &mut ModelStore) {
        let _ = store.update(&self.commands, |q| {
            q.pending.push(CarouselAutoplayCommand::Pause);
        });
    }

    pub fn reset_store(&self, store: &mut ModelStore) {
        let _ = store.update(&self.commands, |q| {
            q.pending.push(CarouselAutoplayCommand::Reset);
        });
    }

    pub fn set_delays(&self, host: &mut impl ModelHost, delays: impl Into<Arc<[Duration]>>) {
        let delays = delays.into();
        let _ = host.update_model(&self.delays, |v, _| {
            *v = Some(delays.clone());
        });
    }

    pub fn clear_delays(&self, host: &mut impl ModelHost) {
        let _ = host.update_model(&self.delays, |v, _| {
            *v = None;
        });
    }

    pub fn set_delays_store(&self, store: &mut ModelStore, delays: impl Into<Arc<[Duration]>>) {
        let delays = delays.into();
        let _ = store.update(&self.delays, |v| {
            *v = Some(delays.clone());
        });
    }

    pub fn clear_delays_store(&self, store: &mut ModelStore) {
        let _ = store.update(&self.delays, |v| {
            *v = None;
        });
    }

    pub fn snapshot(&self, host: &mut impl ModelHost) -> CarouselAutoplayApiSnapshot {
        host.read(&self.snapshot, |_host, v| *v)
            .ok()
            .unwrap_or_default()
    }

    pub fn time_until_next(&self, host: &mut impl ModelHost) -> Option<Duration> {
        self.snapshot(host).time_until_next
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarouselAutoplayConfig {
    pub delay: Duration,
    pub instant: bool,
    pub stop_on_interaction: bool,
    pub stop_on_last_snap: bool,
    pub pause_on_hover: bool,
    pub reset_on_hover_leave: bool,
}

impl Default for CarouselAutoplayConfig {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(2000),
            instant: false,
            stop_on_interaction: true,
            stop_on_last_snap: false,
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

    pub fn instant(mut self, instant: bool) -> Self {
        self.instant = instant;
        self
    }

    pub fn stop_on_interaction(mut self, stop_on_interaction: bool) -> Self {
        self.stop_on_interaction = stop_on_interaction;
        self
    }

    pub fn stop_on_last_snap(mut self, stop_on_last_snap: bool) -> Self {
        self.stop_on_last_snap = stop_on_last_snap;
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarouselWheelGesturesConfig {
    /// Amount of accumulated wheel delta in the carousel main axis required to trigger a single
    /// snap step.
    pub step_threshold_px: Px,
    /// Maximum number of snap steps to apply per wheel event.
    pub max_steps_per_event: usize,
    /// When true, ignore wheel deltas that are not dominant on the carousel main axis.
    ///
    /// This avoids accidentally intercepting vertical page scrolling when a horizontal carousel
    /// is hovered.
    pub require_main_axis_dominant: bool,
    /// When true, allow `Shift` to swap the wheel axes (typical desktop "horizontal scroll" UX).
    pub allow_shift_to_swap_axes: bool,
}

impl Default for CarouselWheelGesturesConfig {
    fn default() -> Self {
        Self {
            // Tailwind `-left-12` / `-right-12` is 48px; use it as a conservative default step.
            step_threshold_px: Px(48.0),
            max_steps_per_event: 1,
            require_main_axis_dominant: true,
            allow_shift_to_swap_axes: true,
        }
    }
}

impl CarouselWheelGesturesConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn step_threshold_px(mut self, threshold: Px) -> Self {
        self.step_threshold_px = threshold;
        self
    }

    pub fn max_steps_per_event(mut self, max: usize) -> Self {
        self.max_steps_per_event = max;
        self
    }

    pub fn require_main_axis_dominant(mut self, enabled: bool) -> Self {
        self.require_main_axis_dominant = enabled;
        self
    }

    pub fn allow_shift_to_swap_axes(mut self, enabled: bool) -> Self {
        self.allow_shift_to_swap_axes = enabled;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CarouselPlugin {
    Autoplay(CarouselAutoplayConfig),
    WheelGestures(CarouselWheelGesturesConfig),
}

fn wheel_delta_main_axis(
    orientation: CarouselOrientation,
    wheel: WheelCx,
    config: CarouselWheelGesturesConfig,
) -> Option<f32> {
    let dx = wheel.delta.x.0;
    let dy = wheel.delta.y.0;
    if !dx.is_finite() || !dy.is_finite() {
        return None;
    }

    match orientation {
        CarouselOrientation::Horizontal => {
            let mut main = dx;
            if config.allow_shift_to_swap_axes && wheel.modifiers.shift && main.abs() <= 0.001 {
                main = dy;
            }
            if config.require_main_axis_dominant && dy.abs() > main.abs() {
                return None;
            }
            if main.abs() <= 0.001 {
                return None;
            }
            Some(main)
        }
        CarouselOrientation::Vertical => {
            let mut main = dy;
            if config.allow_shift_to_swap_axes && wheel.modifiers.shift && main.abs() <= 0.001 {
                main = dx;
            }
            if config.require_main_axis_dominant && dx.abs() > main.abs() {
                return None;
            }
            if main.abs() <= 0.001 {
                return None;
            }
            Some(main)
        }
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

#[derive(Debug, Clone, Copy)]
pub struct CarouselSpaceBreakpoint {
    pub min_width_px: Px,
    pub value: Space,
}

fn resolve_space_breakpoints(
    viewport_width_px: Px,
    base: Space,
    breakpoints: &[CarouselSpaceBreakpoint],
) -> Space {
    if viewport_width_px.0 <= 0.0 || breakpoints.is_empty() {
        return base;
    }

    let mut value = base;
    for bp in breakpoints {
        if viewport_width_px.0 >= bp.min_width_px.0 {
            value = bp.value;
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_space_breakpoints_returns_base_when_unmeasured() {
        let base = Space::N2;
        let out = resolve_space_breakpoints(
            Px(0.0),
            base,
            &[CarouselSpaceBreakpoint {
                min_width_px: Px(10.0),
                value: Space::N4,
            }],
        );
        assert_eq!(out, base);
    }

    #[test]
    fn resolve_space_breakpoints_selects_last_matching_breakpoint() {
        let out = resolve_space_breakpoints(
            Px(320.0),
            Space::N1,
            &[
                CarouselSpaceBreakpoint {
                    min_width_px: Px(200.0),
                    value: Space::N2,
                },
                CarouselSpaceBreakpoint {
                    min_width_px: Px(300.0),
                    value: Space::N4,
                },
            ],
        );
        assert_eq!(out, Space::N4);
    }
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
    /// Enable Embla-style loop behavior (`loop: true`).
    ///
    /// Notes:
    /// - When slide geometry is measurable, this follows Embla's `canLoop` downgrade (it behaves
    ///   like `loop=false` when content cannot physically loop).
    /// - Seamless loop visuals require the Embla-style engine (`embla_engine=true`). When the
    ///   engine is disabled, loop still uses the same `canLoop` gating but falls back to the
    ///   deterministic v1 settle path (no momentum physics).
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
    track_start_neg_margin_viewport_breakpoints: Vec<CarouselSpaceBreakpoint>,
    track_start_neg_margin_layout_breakpoints: Vec<CarouselSpaceBreakpoint>,
    item_padding_start: Space,
    item_padding_start_viewport_breakpoints: Vec<CarouselSpaceBreakpoint>,
    item_padding_start_layout_breakpoints: Vec<CarouselSpaceBreakpoint>,
    item_basis_main_px: Option<Px>,
    breakpoints: Vec<CarouselBreakpoint>,
    options: CarouselOptions,
    drag_config: headless_carousel::CarouselDragConfig,
    api_snapshot: Option<Model<CarouselApiSnapshot>>,
    api_handle: Option<Model<Option<CarouselApi>>>,
    autoplay_api_handle: Option<Model<Option<CarouselAutoplayApi>>>,
    slides_in_view_snapshot: Option<Model<CarouselSlidesInViewSnapshot>>,
    plugins: Vec<CarouselPlugin>,
    controls: CarouselControls,
    test_id: Option<Arc<str>>,
}

#[derive(Debug, Clone, Copy)]
struct CarouselRuntime {
    drag: headless_carousel::CarouselDragState,
    settling: bool,
    embla_settling: bool,
    prevent_click: bool,
    wheel_accum_main_px: f32,
    settle_from: Px,
    settle_to: Px,
    settle_generation: u64,
    selection_initialized: bool,
    autoplay_token: Option<TimerToken>,
    autoplay_timer_started_at: Option<Instant>,
    autoplay_timer_delay: Option<Duration>,
    autoplay_pause_delay: Option<Duration>,
    autoplay_paused_external: bool,
    autoplay_stopped_by_interaction: bool,
    autoplay_stopped_by_last_snap: bool,
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
            wheel_accum_main_px: 0.0,
            settle_from: Px(0.0),
            settle_to: Px(0.0),
            settle_generation: 0,
            selection_initialized: false,
            autoplay_token: None,
            autoplay_timer_started_at: None,
            autoplay_timer_delay: None,
            autoplay_pause_delay: None,
            autoplay_paused_external: false,
            autoplay_stopped_by_interaction: false,
            autoplay_stopped_by_last_snap: false,
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
    autoplay_commands: Option<Model<CarouselAutoplayCommandQueue>>,
    autoplay_snapshot: Option<Model<CarouselAutoplayApiSnapshot>>,
    autoplay_delays: Option<Model<Option<Arc<[Duration]>>>>,
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

#[derive(Debug, Clone, Default)]
struct CarouselAutoplayCommandQueue {
    pending: Vec<CarouselAutoplayCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CarouselAutoplayCommand {
    Play,
    Stop,
    Pause,
    Reset,
}

fn autoplay_time_until_next(now: Instant, runtime: CarouselRuntime) -> Option<Duration> {
    if runtime.autoplay_token.is_none() {
        return None;
    }
    let started_at = runtime.autoplay_timer_started_at?;
    let delay = runtime.autoplay_timer_delay?;
    let elapsed = now.saturating_duration_since(started_at);
    Some(delay.saturating_sub(elapsed))
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

fn carousel_autoplay_commands_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<CarouselAutoplayCommandQueue> {
    let existing = cx.with_state(CarouselState::default, |st| st.autoplay_commands.clone());
    if let Some(existing) = existing {
        return existing;
    }

    let model = cx
        .app
        .models_mut()
        .insert(CarouselAutoplayCommandQueue::default());
    cx.with_state(CarouselState::default, |st| {
        st.autoplay_commands = Some(model.clone());
    });
    model
}

fn carousel_autoplay_snapshot_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<CarouselAutoplayApiSnapshot> {
    let existing = cx.with_state(CarouselState::default, |st| st.autoplay_snapshot.clone());
    if let Some(existing) = existing {
        return existing;
    }

    let model = cx
        .app
        .models_mut()
        .insert(CarouselAutoplayApiSnapshot::default());
    cx.with_state(CarouselState::default, |st| {
        st.autoplay_snapshot = Some(model.clone());
    });
    model
}

fn carousel_autoplay_delays_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Arc<[Duration]>>> {
    let existing = cx.with_state(CarouselState::default, |st| st.autoplay_delays.clone());
    if let Some(existing) = existing {
        return existing;
    }

    let model = cx.app.models_mut().insert(None::<Arc<[Duration]>>);
    cx.with_state(CarouselState::default, |st| {
        st.autoplay_delays = Some(model.clone());
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
            track_start_neg_margin_viewport_breakpoints: Vec::new(),
            track_start_neg_margin_layout_breakpoints: Vec::new(),
            item_padding_start: Space::N4,
            item_padding_start_viewport_breakpoints: Vec::new(),
            item_padding_start_layout_breakpoints: Vec::new(),
            item_basis_main_px: None,
            breakpoints: Vec::new(),
            options: CarouselOptions::default(),
            drag_config: headless_carousel::CarouselDragConfig::default(),
            api_snapshot: None,
            api_handle: None,
            autoplay_api_handle: None,
            slides_in_view_snapshot: None,
            plugins: Vec::new(),
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

    /// Exposes a Rust-native handle to control the autoplay plugin (play/stop/reset) when the
    /// plugin is attached.
    pub fn autoplay_api_handle_model(mut self, model: Model<Option<CarouselAutoplayApi>>) -> Self {
        self.autoplay_api_handle = Some(model);
        self
    }

    pub fn slides_in_view_snapshot_model(
        mut self,
        model: Model<CarouselSlidesInViewSnapshot>,
    ) -> Self {
        self.slides_in_view_snapshot = Some(model);
        self
    }

    /// Embla-style plugin surface aligned with shadcn/ui `plugins`.
    pub fn plugin(mut self, plugin: CarouselPlugin) -> Self {
        self.plugins.push(plugin);
        self
    }

    pub fn plugins(mut self, plugins: impl IntoIterator<Item = CarouselPlugin>) -> Self {
        self.plugins.extend(plugins);
        self
    }

    /// Adds an Embla-style autoplay policy surface (shadcn `carousel-plugin` outcome).
    pub fn autoplay(self, config: CarouselAutoplayConfig) -> Self {
        self.plugin(CarouselPlugin::Autoplay(config))
    }

    /// Adds wheel/trackpad gesture navigation (Embla wheel-gestures-style outcome).
    pub fn wheel_gestures(self, config: CarouselWheelGesturesConfig) -> Self {
        self.plugin(CarouselPlugin::WheelGestures(config))
    }

    pub fn track_start_neg_margin(mut self, margin: Space) -> Self {
        self.track_start_neg_margin = margin;
        self
    }

    /// Breakpoint-based track start negative margin patches for the **window viewport** width
    /// (Tailwind `md:` / `lg:`).
    ///
    /// This matches shadcn's responsive `md:-ml-*` / `md:-mt-*` semantics on `CarouselContent`.
    pub fn viewport_track_start_neg_margin_breakpoint(
        mut self,
        min_width_px: Px,
        margin: Space,
    ) -> Self {
        self.track_start_neg_margin_viewport_breakpoints
            .push(CarouselSpaceBreakpoint {
                min_width_px,
                value: margin,
            });
        self.track_start_neg_margin_viewport_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    /// Breakpoint-based track start negative margin patches for the **carousel viewport** width
    /// (container-query style).
    pub fn track_start_neg_margin_breakpoint(mut self, min_width_px: Px, margin: Space) -> Self {
        self.track_start_neg_margin_layout_breakpoints
            .push(CarouselSpaceBreakpoint {
                min_width_px,
                value: margin,
            });
        self.track_start_neg_margin_layout_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    pub fn item_padding_start(mut self, padding: Space) -> Self {
        self.item_padding_start = padding;
        self
    }

    /// Breakpoint-based default item start padding patches for the **window viewport** width
    /// (Tailwind `md:` / `lg:`).
    ///
    /// This matches shadcn's responsive `md:pl-*` / `md:pt-*` semantics on `CarouselItem` when
    /// the same value applies to all items.
    pub fn viewport_item_padding_start_breakpoint(
        mut self,
        min_width_px: Px,
        padding: Space,
    ) -> Self {
        self.item_padding_start_viewport_breakpoints
            .push(CarouselSpaceBreakpoint {
                min_width_px,
                value: padding,
            });
        self.item_padding_start_viewport_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    /// Breakpoint-based default item start padding patches for the **carousel viewport** width
    /// (container-query style).
    pub fn item_padding_start_breakpoint(mut self, min_width_px: Px, padding: Space) -> Self {
        self.item_padding_start_layout_breakpoints
            .push(CarouselSpaceBreakpoint {
                min_width_px,
                value: padding,
            });
        self.item_padding_start_layout_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
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
        this.track_start_neg_margin_viewport_breakpoints
            .extend(content.track_start_neg_margin_viewport_breakpoints);
        this.track_start_neg_margin_viewport_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        this.track_start_neg_margin_layout_breakpoints
            .extend(content.track_start_neg_margin_layout_breakpoints);
        this.track_start_neg_margin_layout_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));

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
            let plugins = self.plugins;
            let mut autoplay_cfg: Option<CarouselAutoplayConfig> = None;
            let mut wheel_cfg: Option<CarouselWheelGesturesConfig> = None;
            for plugin in plugins {
                match plugin {
                    CarouselPlugin::Autoplay(cfg) => autoplay_cfg = Some(cfg),
                    CarouselPlugin::WheelGestures(cfg) => wheel_cfg = Some(cfg),
                }
            }
            let autoplay_stop_on_interaction =
                autoplay_cfg.is_some_and(|cfg| cfg.stop_on_interaction);
            let root_test_id = self.test_id.unwrap_or_else(|| Arc::from("carousel"));
            let slides_in_view_snapshot_model = self.slides_in_view_snapshot;
            let api_handle_model = self.api_handle;
            let api_snapshot_model = self.api_snapshot;
            let autoplay_api_handle_model = self.autoplay_api_handle;
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

            let autoplay_commands_model = autoplay_api_handle_model
                .as_ref()
                .map(|_| carousel_autoplay_commands_model(cx));
            let autoplay_snapshot_model = autoplay_api_handle_model
                .as_ref()
                .map(|_| carousel_autoplay_snapshot_model(cx));
            let autoplay_delays_model = autoplay_api_handle_model
                .as_ref()
                .map(|_| carousel_autoplay_delays_model(cx));
            if let (Some(autoplay_api_handle_model), Some(commands), Some(snapshot), Some(delays)) = (
                autoplay_api_handle_model.as_ref(),
                autoplay_commands_model.as_ref(),
                autoplay_snapshot_model.as_ref(),
                autoplay_delays_model.as_ref(),
            ) {
                let api = CarouselAutoplayApi {
                    commands: commands.clone(),
                    snapshot: snapshot.clone(),
                    delays: delays.clone(),
                };
                let _ = cx.app.models_mut().update(autoplay_api_handle_model, |v| {
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

            let viewport_width_for_viewport_breakpoints =
                cx.environment_viewport_width(Invalidation::Layout);
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
                    .ml_neg({
                        let space = resolve_space_breakpoints(
                            viewport_width_for_viewport_breakpoints,
                            self.track_start_neg_margin,
                            &self.track_start_neg_margin_viewport_breakpoints,
                        );
                        resolve_space_breakpoints(
                            view_width_for_breakpoints,
                            space,
                            &self.track_start_neg_margin_layout_breakpoints,
                        )
                    })
                    .merge(self.track_layout),
                CarouselOrientation::Vertical => LayoutRefinement::default()
                    .w_full()
                    .mt_neg({
                        let space = resolve_space_breakpoints(
                            viewport_width_for_viewport_breakpoints,
                            self.track_start_neg_margin,
                            &self.track_start_neg_margin_viewport_breakpoints,
                        );
                        resolve_space_breakpoints(
                            view_width_for_breakpoints,
                            space,
                            &self.track_start_neg_margin_layout_breakpoints,
                        )
                    })
                    .merge(self.track_layout),
            };
            let track_layout = decl_style::layout_style(&theme, track_layout);

            let item_pad_default_space = {
                let space = resolve_space_breakpoints(
                    viewport_width_for_viewport_breakpoints,
                    self.item_padding_start,
                    &self.item_padding_start_viewport_breakpoints,
                );
                resolve_space_breakpoints(
                    view_width_for_breakpoints,
                    space,
                    &self.item_padding_start_layout_breakpoints,
                )
            };

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
            let viewport_width_for_item_breakpoints = view_width_for_breakpoints;

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

            if let Some(autoplay_commands_model) = autoplay_commands_model.as_ref() {
                let commands = cx
                    .app
                    .models_mut()
                    .update(autoplay_commands_model, |q| std::mem::take(&mut q.pending))
                    .unwrap_or_default();
                if let Some(cmd) = commands.last().copied() {
                    let token = cx
                        .app
                        .models_mut()
                        .read(&runtime_model, |st| st.autoplay_token)
                        .ok()
                        .flatten();
                    if let Some(token) = token {
                        cx.cancel_timer(token);
                    }

                    let now = Instant::now();
                    let time_until_next = cx
                        .app
                        .models_mut()
                        .read(&runtime_model, |st| autoplay_time_until_next(now, *st))
                        .ok()
                        .flatten();

                    let _ = cx.app.models_mut().update(&runtime_model, |st| {
                        st.autoplay_token = None;
                        st.autoplay_timer_started_at = None;
                        st.autoplay_timer_delay = None;
                        match cmd {
                            CarouselAutoplayCommand::Play => {
                                st.autoplay_paused_external = false;
                                st.autoplay_stopped_by_interaction = false;
                                st.autoplay_stopped_by_last_snap = false;
                            }
                            CarouselAutoplayCommand::Stop => {
                                st.autoplay_paused_external = true;
                                st.autoplay_pause_delay = None;
                            }
                            CarouselAutoplayCommand::Pause => {
                                st.autoplay_paused_external = true;
                                st.autoplay_pause_delay = time_until_next;
                            }
                            CarouselAutoplayCommand::Reset => {
                                st.autoplay_paused_external = false;
                                st.autoplay_stopped_by_last_snap = false;
                                st.autoplay_pause_delay = None;
                            }
                        }
                    });

                    cx.request_frame();
                }
            }

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
                                cx.cancel_timer(token);
                            }
                            let _ = cx.app.models_mut().update(&runtime_model, |st| {
                                st.autoplay_token = None;
                                st.autoplay_timer_started_at = None;
                                st.autoplay_timer_delay = None;
                                st.autoplay_pause_delay = None;
                                st.autoplay_stopped_by_interaction = true;
                            });
                        }

                        let index: usize = cx
                            .app
                            .models_mut()
                            .read(&index_model, |v| *v)
                            .ok()
                            .unwrap_or(0);
                        let view_size_for_loop = cx
                            .app
                            .models_mut()
                            .read(&extent_model, |v| v.0.max(0.0))
                            .ok()
                            .unwrap_or(0.0);
                        let slides_for_loop: Arc<[headless_carousel::CarouselSlide1D]> = cx
                            .app
                            .models_mut()
                            .read(&slides_model, |slides| slides.clone())
                            .ok()
                            .unwrap_or_else(|| {
                                Arc::from(Vec::<headless_carousel::CarouselSlide1D>::new())
                            });
                        let loop_enabled_effective = resolve_loop_enabled_effective(
                            options.loop_enabled,
                            items_len,
                            view_size_for_loop,
                            &slides_for_loop,
                        );
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
                                let cur = cx
                                    .app
                                    .models_mut()
                                    .read(&offset_model, |v| *v)
                                    .ok()
                                    .unwrap_or(Px(0.0));
                                let start_snap = resolve_nearest_snap_index(snaps.as_ref(), cur);

                                let mut select = None;
                                let _ = cx.app.models_mut().update(&embla_engine_model, |v| {
                                    if let Some(engine) = v.as_mut() {
                                        engine.set_options(
                                            loop_enabled_effective,
                                            options.drag_free,
                                            options.skip_snaps,
                                            options.embla_duration,
                                        );
                                        select = engine.scroll_to_index(target_index, 0);
                                        return;
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
                                            start_snap,
                                        },
                                    );
                                    let loc = -cur.0;
                                    engine.scroll_body.set_location(loc);
                                    engine.scroll_body.set_target(loc);
                                    engine.scroll_target.set_target_vector(loc);
                                    select = engine.scroll_to_index(target_index, 0);
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
                        st.autoplay_timer_started_at = None;
                        st.autoplay_timer_delay = None;
                        st.autoplay_pause_delay = None;
                        st.autoplay_stopped_by_interaction = true;
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
                let loop_enabled = if can_use_embla_engine {
                    host.models_mut()
                        .read(&slides_for_down, |slides| {
                            resolve_loop_enabled_effective(
                                loop_requested_for_down,
                                items_len,
                                view_size,
                                slides,
                            )
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
            let extent_for_up = extent_model.clone();
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

                let view_size_for_loop = host
                    .models_mut()
                    .read(&extent_for_up, |v| v.0.max(0.0))
                    .ok()
                    .unwrap_or(0.0);
                let loop_enabled_effective = if loop_requested_for_up {
                    host.models_mut()
                        .read(&slides_for_up, |slides| {
                            resolve_loop_enabled_effective(
                                true,
                                items_len,
                                view_size_for_loop,
                                slides,
                            )
                        })
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                };

                let can_use_embla_engine = embla_engine_enabled_for_up && snaps.len() > 1;

                if can_use_embla_engine {
                    let view_size = view_size_for_loop;

                    let content_size = if max_offset.0 > 0.0 {
                        max_offset.0 + view_size
                    } else {
                        let extent = match item_basis {
                            Some(px) => px,
                            None => Px(view_size),
                        };
                        (extent.0 * (items_len as f32)).max(0.0)
                    };

                    let mut scroll_snaps = snaps.iter().map(|px| -px.0).collect::<Vec<_>>();
                    if scroll_snaps.is_empty() {
                        scroll_snaps.push(0.0);
                    }

                    let release = headless_carousel::on_pointer_up_with_snaps_options(
                        drag_config,
                        &mut drag,
                        track_direction,
                        direction_for_up,
                        up.position,
                        snaps.as_ref(),
                        max_offset,
                        loop_enabled_effective,
                        skip_snaps_for_up,
                        drag_free_for_up,
                    )
                    .expect("release output when dragging");

                    let index: usize = host
                        .models_mut()
                        .read(&index_for_up, |v| *v)
                        .ok()
                        .unwrap_or(0);

                    let mut engine = headless_embla::engine::Engine::new(
                        scroll_snaps,
                        content_size,
                        headless_embla::engine::EngineConfig {
                            loop_enabled: loop_enabled_effective,
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

                    // Decide the target snap index using our headless drag contract so the Embla
                    // path matches `CarouselDragConfig::snap_threshold_fraction` (and remains
                    // deterministic in tests).
                    let select = if !drag_free_for_up {
                        engine.scroll_to_index(release.next_index, headless_embla::utils::DIRECTION_NONE)
                    } else {
                        None
                    };

                    let _ = host.models_mut().update(&embla_engine_for_up, |v| {
                        *v = Some(engine);
                    });

                    let next_index = select.map(|s| s.target_snap).unwrap_or(release.next_index);
                    let _ = host
                        .models_mut()
                        .update(&index_for_up, |v| *v = next_index);
                    if drag_free_for_up {
                        let _ = host
                            .models_mut()
                            .update(&offset_for_up, |v| *v = release.target_offset);
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
                        loop_enabled_effective,
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
                            st.autoplay_timer_started_at = None;
                            st.autoplay_timer_delay = None;
                            st.autoplay_pause_delay = None;
                            st.autoplay_stopped_by_interaction = true;
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

                    let view_size = host
                        .models_mut()
                        .read(&extent_for_prev, |v| v.0.max(0.0))
                        .ok()
                        .unwrap_or(0.0);
                    let loop_enabled_effective = if loop_requested_for_prev {
                        host.models_mut()
                            .read(&slides_for_prev, |slides| {
                                resolve_loop_enabled_effective(true, items_len, view_size, slides)
                            })
                            .ok()
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    if embla_engine_enabled_for_prev {
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
                        let cur = host
                            .models_mut()
                            .read(&offset_for_prev, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let start_snap = resolve_nearest_snap_index(snaps.as_ref(), cur);

                        let mut select = None;
                        let _ = host.models_mut().update(&embla_engine_for_prev, |v| {
                            if let Some(engine) = v.as_mut() {
                                engine.set_options(
                                    loop_enabled_effective,
                                    drag_free_for_prev,
                                    skip_snaps_for_prev,
                                    embla_duration_for_prev,
                                );
                                select = engine.scroll_to_prev();
                                return;
                            }

                            let mut engine = headless_embla::engine::Engine::new(
                                scroll_snaps,
                                content_size,
                                headless_embla::engine::EngineConfig {
                                    loop_enabled: loop_enabled_effective,
                                    drag_free: drag_free_for_prev,
                                    skip_snaps: skip_snaps_for_prev,
                                    duration: embla_duration_for_prev.max(0.0),
                                    base_friction: 0.68,
                                    view_size,
                                    start_snap,
                                },
                            );
                            let loc = -cur.0;
                            engine.scroll_body.set_location(loc);
                            engine.scroll_body.set_target(loc);
                            engine.scroll_target.set_target_vector(loc);
                            select = engine.scroll_to_prev();
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

                    let target_index = if loop_enabled_effective {
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
                            st.autoplay_timer_started_at = None;
                            st.autoplay_timer_delay = None;
                            st.autoplay_pause_delay = None;
                            st.autoplay_stopped_by_interaction = true;
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

                    let view_size = host
                        .models_mut()
                        .read(&extent_for_next, |v| v.0.max(0.0))
                        .ok()
                        .unwrap_or(0.0);
                    let loop_enabled_effective = if loop_requested_for_next {
                        host.models_mut()
                            .read(&slides_for_next, |slides| {
                                resolve_loop_enabled_effective(true, items_len, view_size, slides)
                            })
                            .ok()
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    if embla_engine_enabled_for_next {
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
                        let cur = host
                            .models_mut()
                            .read(&offset_for_next, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let start_snap = resolve_nearest_snap_index(snaps.as_ref(), cur);

                        let mut select = None;
                        let _ = host.models_mut().update(&embla_engine_for_next, |v| {
                            if let Some(engine) = v.as_mut() {
                                engine.set_options(
                                    loop_enabled_effective,
                                    drag_free_for_next,
                                    skip_snaps_for_next,
                                    embla_duration_for_next,
                                );
                                select = engine.scroll_to_next();
                                return;
                            }

                            let mut engine = headless_embla::engine::Engine::new(
                                scroll_snaps,
                                content_size,
                                headless_embla::engine::EngineConfig {
                                    loop_enabled: loop_enabled_effective,
                                    drag_free: drag_free_for_next,
                                    skip_snaps: skip_snaps_for_next,
                                    duration: embla_duration_for_next.max(0.0),
                                    base_friction: 0.68,
                                    view_size,
                                    start_snap,
                                },
                            );
                            let loc = -cur.0;
                            engine.scroll_body.set_location(loc);
                            engine.scroll_body.set_target(loc);
                            engine.scroll_target.set_target_vector(loc);
                            select = engine.scroll_to_next();
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

                    let target_index = if loop_enabled_effective {
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
                            st.autoplay_timer_started_at = None;
                            st.autoplay_timer_delay = None;
                            st.autoplay_pause_delay = None;
                            st.autoplay_stopped_by_interaction = true;
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
                                resolve_loop_enabled_effective(
                                    true,
                                    items_len,
                                    view_size_for_loop,
                                    slides,
                                )
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
                        let start_snap = resolve_nearest_snap_index(snaps.as_ref(), cur);

                        let mut select = None;
                        let _ = host.models_mut().update(&embla_engine_for_key, |v| {
                            if let Some(engine) = v.as_mut() {
                                engine.set_options(
                                    loop_enabled,
                                    drag_free_for_key,
                                    skip_snaps_for_key,
                                    embla_duration_for_key,
                                );
                                select = engine.scroll_to_index(
                                    target_index,
                                    headless_embla::utils::DIRECTION_NONE,
                                );
                                return;
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
                                    start_snap,
                                },
                            );
                            let loc = -cur.0;
                            engine.scroll_body.set_location(loc);
                            engine.scroll_body.set_target(loc);
                            engine.scroll_target.set_target_vector(loc);

                            select = engine.scroll_to_index(
                                target_index,
                                headless_embla::utils::DIRECTION_NONE,
                            );
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
            let loop_enabled_for_translates = embla_engine_enabled
                && resolve_loop_enabled_effective(
                    options.loop_enabled,
                    items_len,
                    view_size_prev.0.max(0.0),
                    &slides_prev,
                );
            let loop_translates =
                if loop_enabled_for_translates && content_size_prev > 0.0 && view_size_prev.0 > 0.0
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
                            let per_item_layout_patch = item
                                .layout
                                .merge(resolve_item_viewport_layout_breakpoints(
                                    viewport_width_for_viewport_breakpoints,
                                    &item.viewport_layout_breakpoints,
                                ))
                                .merge(resolve_item_layout_breakpoints(
                                    viewport_width_for_item_breakpoints,
                                    &item.layout_breakpoints,
                                ));
                            let per_item_basis = per_item_layout_patch
                                .flex_item
                                .as_ref()
                                .and_then(|f| f.basis.as_ref())
                                .is_some();

                            let item_pad_space = {
                                let space = item.padding_start.unwrap_or(item_pad_default_space);
                                let space = resolve_space_breakpoints(
                                    viewport_width_for_viewport_breakpoints,
                                    space,
                                    &item.viewport_padding_start_breakpoints,
                                );
                                resolve_space_breakpoints(
                                    viewport_width_for_item_breakpoints,
                                    space,
                                    &item.padding_start_breakpoints,
                                )
                            };
                            let item_pad = decl_style::space(&theme_for_items, item_pad_space);

                            let content = item.child;
                            slide_content_ids_ref.push(content.id);

                            let mut item_layout = LayoutRefinement::default()
                                .flex_grow(0.0)
                                .flex_shrink(0.0)
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
                                    .role_description("slide")
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

            // Pointer interactions (drag + wheel) should be available across the full viewport,
            // even when slide content has no intrinsic height (e.g. empty slides in tests).
            let pointer_layout =
                decl_style::layout_style(&theme, LayoutRefinement::default().size_full());

            let on_wheel: Option<OnWheel> = wheel_cfg.map(|cfg| {
                let snaps_for_wheel = snaps_model.clone();
                let runtime_for_wheel = runtime_model.clone();
                let on_prev_for_wheel = on_prev.clone();
                let on_next_for_wheel = on_next.clone();
                let handler: OnWheel = Arc::new(
                    move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                          cx: ActionCx,
                          wheel: WheelCx| {
                        let snaps_len = host
                            .models_mut()
                            .read(&snaps_for_wheel, |v: &Arc<[Px]>| v.len())
                            .ok()
                            .unwrap_or(0);
                        if snaps_len <= 1 {
                            return false;
                        }

                        let Some(delta_main) = wheel_delta_main_axis(orientation, wheel, cfg) else {
                            return false;
                        };

                        let threshold = cfg.step_threshold_px.0.max(1.0);
                        let max_steps = cfg.max_steps_per_event.max(1).min(8);

                        let (prev_steps, next_steps) = host
                            .models_mut()
                            .update(&runtime_for_wheel, |st| {
                                let mut accum = st.wheel_accum_main_px;
                                if accum != 0.0 && accum.signum() != delta_main.signum() {
                                    accum = 0.0;
                                }
                                accum += delta_main;

                                let mut prev = 0usize;
                                let mut next = 0usize;
                                while accum >= threshold && prev < max_steps {
                                    prev = prev.saturating_add(1);
                                    accum -= threshold;
                                }
                                while accum <= -threshold && next < max_steps {
                                    next = next.saturating_add(1);
                                    accum += threshold;
                                }

                                st.wheel_accum_main_px = accum;
                                (prev, next)
                            })
                            .unwrap_or((0usize, 0usize));

                        let host_action: &mut dyn UiActionHost = host;
                        for _ in 0..prev_steps {
                            on_prev_for_wheel(host_action, cx, ActivateReason::Pointer);
                        }
                        for _ in 0..next_steps {
                            on_next_for_wheel(host_action, cx, ActivateReason::Pointer);
                        }

                        true
                    },
                );
                handler
            });

            let drag_enabled = items_len > 1 && options.draggable;
            let pointer_region = cx.pointer_region(
                PointerRegionProps {
                    layout: pointer_layout,
                    enabled: drag_enabled || on_wheel.is_some(),
                    capture_phase_pointer_moves: true,
                },
                move |cx| {
                    cx.pointer_region_on_pointer_down(on_down);
                    cx.pointer_region_on_pointer_move(on_move);
                    cx.pointer_region_on_pointer_up(on_up);
                    cx.pointer_region_on_pointer_cancel(on_cancel);
                    if let Some(on_wheel) = on_wheel.clone() {
                        cx.pointer_region_on_wheel(on_wheel);
                    }
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

            let pointer_down = runtime_snapshot.drag.armed || runtime_snapshot.drag.dragging;
            let focused_now = cx.focused_element();
            let focus_changed = focused_now != runtime_snapshot.focus_last_focused_element;
            let maybe_focused_slide_index_and_center = if !pointer_down && focus_changed {
                (|| {
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
                })()
            } else {
                None
            };

            // Embla autoplay defaultInteraction stops playback when a slide receives focus
            // (e.g. keyboard navigation). We mirror the outcome when stopOnInteraction is enabled.
            if autoplay_stop_on_interaction
                && autoplay_cfg.is_some()
                && runtime_snapshot.autoplay_token.is_some()
                && !runtime_snapshot.autoplay_paused_external
                && !runtime_snapshot.autoplay_stopped_by_interaction
                && !runtime_snapshot.autoplay_stopped_by_last_snap
                && maybe_focused_slide_index_and_center.is_some()
            {
                if let Some(token) = runtime_snapshot.autoplay_token {
                    cx.cancel_timer(token);
                }
                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                    st.autoplay_token = None;
                    st.autoplay_timer_started_at = None;
                    st.autoplay_timer_delay = None;
                    st.autoplay_pause_delay = None;
                    st.autoplay_stopped_by_interaction = true;
                });
                cx.request_frame();
            }

            if options.watch_focus && view_size_now.0 > 0.0 && !snap_by_slide_now.is_empty() {
                if let Some((slide_index, center)) = maybe_focused_slide_index_and_center {
                    let tab_pending = runtime_snapshot.focus_tab_generation
                        != runtime_snapshot.focus_last_handled_tab_generation;
                    let focus_offscreen = viewport_bounds.is_some_and(|b| !b.contains(center));
                    let snap_index = snap_by_slide_now
                        .get(slide_index)
                        .copied()
                        .unwrap_or(slide_index)
                        .min(snaps_now.len().saturating_sub(1));
                    if (tab_pending || focus_offscreen)
                        && let Some(target_snap) = snaps_now.get(snap_index).copied()
                    {
                            let slides_prev: Arc<[headless_carousel::CarouselSlide1D]> = cx
                                .watch_model(&slides_model)
                                .layout()
                                .cloned()
                                .unwrap_or_else(|| {
                                    Arc::from(Vec::<headless_carousel::CarouselSlide1D>::new())
                                });
                            let loop_enabled_effective = resolve_loop_enabled_effective(
                                options.loop_enabled,
                                items_len,
                                view_size_now.0.max(0.0),
                                &slides_prev,
                            );

                            let offset_max = if embla_engine_enabled && loop_enabled_effective {
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

                    if tab_pending {
                        let _ = cx.app.models_mut().update(&runtime_model, |st| {
                            st.focus_last_handled_tab_generation =
                                runtime_snapshot.focus_tab_generation;
                        });
                    }
                }
            }

            if focus_changed {
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
                let loop_enabled_effective = resolve_loop_enabled_effective(
                    options.loop_enabled,
                    items_len,
                    view_size,
                    &slides_arc,
                );

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
            let loop_enabled_effective = resolve_loop_enabled_effective(
                options.loop_enabled,
                items_len,
                view_size_now.0.max(0.0),
                &slides_arc,
            );
            let offset_max = if embla_engine_enabled && loop_enabled_effective {
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
                let offset_max = if embla_engine_enabled && loop_enabled_effective {
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
                    CarouselOrientation::Horizontal => (
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
                    ),
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
            let index_for_hover = index_model.clone();
            let autoplay_delays_for_hover = autoplay_delays_model.clone();
            let theme_for_hover = theme.clone();
            let root = cx.container(
                ContainerProps {
                    layout: root_layout,
                    ..Default::default()
                },
                move |cx| {
                    let timer_target = cx.root_id();
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
                                        cx.cancel_timer(token);
                                    }
                                    let _ = cx.app.models_mut().update(&runtime_for_hover, |st| {
                                        st.autoplay_token = None;
                                        st.autoplay_timer_started_at = None;
                                        st.autoplay_timer_delay = None;
                                    });
                                }

                                if left_hover
                                    && cfg.pause_on_hover
                                    && cfg.reset_on_hover_leave
                                    && !runtime_snapshot.autoplay_paused_external
                                    && !runtime_snapshot.autoplay_stopped_by_interaction
                                    && !runtime_snapshot.autoplay_stopped_by_last_snap
                                {
                                    let snaps: Arc<[Px]> = cx
                                        .watch_model(&snaps_for_hover)
                                        .layout()
                                        .cloned()
                                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));

                                    if snaps.len() > 1 && runtime_snapshot.autoplay_token.is_none()
                                    {
                                        let selected_index = cx
                                            .watch_model(&index_for_hover)
                                            .layout()
                                            .copied()
                                            .unwrap_or(0);
                                        let per_snap_delay = autoplay_delays_for_hover
                                            .as_ref()
                                            .and_then(|m| {
                                                cx.watch_model(m).layout().cloned().flatten()
                                            })
                                            .and_then(|delays| delays.get(selected_index).copied());

                                        let now = Instant::now();
                                        let delay = per_snap_delay.unwrap_or(cfg.delay);
                                        let token = cx.app.next_timer_token();
                                        let _ =
                                            cx.app.models_mut().update(&runtime_for_hover, |st| {
                                                st.autoplay_token = Some(token);
                                                st.autoplay_timer_started_at = Some(now);
                                                st.autoplay_timer_delay = Some(delay);
                                                st.autoplay_pause_delay = None;
                                            });
                                        cx.set_timer_for(timer_target, token, delay);
                                    }
                                }

                                if (!hovered || !cfg.pause_on_hover)
                                    && !runtime_snapshot.autoplay_paused_external
                                    && !runtime_snapshot.autoplay_stopped_by_interaction
                                    && !runtime_snapshot.autoplay_stopped_by_last_snap
                                    && runtime_snapshot.autoplay_token.is_none()
                                {
                                    let snaps: Arc<[Px]> = cx
                                        .watch_model(&snaps_for_hover)
                                        .layout()
                                        .cloned()
                                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));

                                    if snaps.len() > 1 {
                                        let selected_index = cx
                                            .watch_model(&index_for_hover)
                                            .layout()
                                            .copied()
                                            .unwrap_or(0);
                                        let per_snap_delay = autoplay_delays_for_hover
                                            .as_ref()
                                            .and_then(|m| {
                                                cx.watch_model(m).layout().cloned().flatten()
                                            })
                                            .and_then(|delays| delays.get(selected_index).copied());

                                        let now = Instant::now();
                                        let delay = runtime_snapshot
                                            .autoplay_pause_delay
                                            .or(per_snap_delay)
                                            .unwrap_or(cfg.delay);
                                        let token = cx.app.next_timer_token();
                                        let _ =
                                            cx.app.models_mut().update(&runtime_for_hover, |st| {
                                                st.autoplay_token = Some(token);
                                                st.autoplay_timer_started_at = Some(now);
                                                st.autoplay_timer_delay = Some(delay);
                                                st.autoplay_pause_delay = None;
                                            });
                                        cx.set_timer_for(timer_target, token, delay);
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
                let autoplay_delays_for_timer = autoplay_delays_model.clone();
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
                        if runtime.autoplay_paused_external
                            || runtime.autoplay_stopped_by_interaction
                            || runtime.autoplay_stopped_by_last_snap
                            || (cfg.pause_on_hover && runtime.autoplay_hovered)
                        {
                            let _ = host.models_mut().update(&runtime_for_timer, |st| {
                                st.autoplay_token = None;
                                st.autoplay_timer_started_at = None;
                                st.autoplay_timer_delay = None;
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
                                st.autoplay_timer_started_at = None;
                                st.autoplay_timer_delay = None;
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
                                    resolve_loop_enabled_effective(
                                        true,
                                        items_len,
                                        view_size_for_loop,
                                        slides,
                                    )
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
                                st.autoplay_timer_started_at = None;
                                st.autoplay_timer_delay = None;
                            });
                            return true;
                        };

                        let last_index = snaps.len().saturating_sub(1);
                        let kill_after_select =
                            cfg.stop_on_last_snap && index.saturating_add(1) == last_index;

                        let target = snaps[target_index];
                        let cur = host
                            .models_mut()
                            .read(&offset_for_timer, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let _ = host
                            .models_mut()
                            .update(&index_for_timer, |v| *v = target_index);

                        if cfg.instant {
                            let _ = host
                                .models_mut()
                                .update(&offset_for_timer, |v| *v = target);
                            let _ = host.models_mut().update(&runtime_for_timer, |st| {
                                st.drag = headless_carousel::CarouselDragState::default();
                                st.settling = false;
                                st.embla_settling = false;
                                st.settle_from = target;
                                st.settle_to = target;
                                st.settle_generation = st.settle_generation.saturating_add(1);
                            });
                        } else {
                            let _ = host.models_mut().update(&runtime_for_timer, |st| {
                                st.drag = headless_carousel::CarouselDragState::default();
                                st.settling = true;
                                st.settle_from = cur;
                                st.settle_to = target;
                                st.settle_generation = st.settle_generation.saturating_add(1);
                            });
                        }
                        host.request_redraw(action_cx.window);

                        if kill_after_select {
                            let _ = host.models_mut().update(&runtime_for_timer, |st| {
                                st.autoplay_token = None;
                                st.autoplay_timer_started_at = None;
                                st.autoplay_timer_delay = None;
                                st.autoplay_pause_delay = None;
                                st.autoplay_stopped_by_last_snap = true;
                            });
                            return true;
                        }

                        let per_snap_delay = autoplay_delays_for_timer
                            .as_ref()
                            .and_then(|m| {
                                host.models_mut()
                                    .read(m, |v| v.clone())
                                    .ok()
                                    .flatten()
                            })
                            .and_then(|delays| delays.get(target_index).copied());
                        let now = Instant::now();
                        let delay = per_snap_delay.unwrap_or(cfg.delay);
                        let _ = host.models_mut().update(&runtime_for_timer, |st| {
                            st.autoplay_timer_started_at = Some(now);
                            st.autoplay_timer_delay = Some(delay);
                            st.autoplay_pause_delay = None;
                        });
                        host.push_effect(Effect::SetTimer {
                            window: Some(action_cx.window),
                            token,
                            after: delay,
                            repeat: None,
                        });
                        true
                    }),
                );
            }

            if let Some(autoplay_snapshot_model) = autoplay_snapshot_model.as_ref() {
                let runtime = cx.watch_model(&runtime_model).copied().unwrap_or_default();
                let time_until_next = autoplay_time_until_next(Instant::now(), runtime);
                let snaps: Arc<[Px]> = cx
                    .watch_model(&snaps_model)
                    .cloned()
                    .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
                let extent = cx.watch_model(&extent_model).copied().unwrap_or(Px(0.0));
                let snapshot = CarouselAutoplayApiSnapshot {
                    active: autoplay_cfg.is_some() && extent.0 > 0.0 && snaps.len() > 1,
                    playing: runtime.autoplay_token.is_some(),
                    paused_external: runtime.autoplay_paused_external,
                    stopped_by_interaction: runtime.autoplay_stopped_by_interaction,
                    stopped_by_last_snap: runtime.autoplay_stopped_by_last_snap,
                    hovered: runtime.autoplay_hovered,
                    time_until_next,
                };
                let _ = cx
                    .app
                    .models_mut()
                    .update(autoplay_snapshot_model, |v| *v = snapshot);
            }

            cx.key_add_on_key_down_capture_for(root.id, on_key_down);

            let orientation_semantics = match orientation {
                CarouselOrientation::Horizontal => SemanticsOrientation::Horizontal,
                CarouselOrientation::Vertical => SemanticsOrientation::Vertical,
            };
            root.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Region)
                    .role_description("carousel")
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
    track_start_neg_margin_viewport_breakpoints: Vec<CarouselSpaceBreakpoint>,
    track_start_neg_margin_layout_breakpoints: Vec<CarouselSpaceBreakpoint>,
}

impl CarouselContent {
    pub fn new(items: impl IntoIterator<Item = CarouselItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            viewport_layout: LayoutRefinement::default(),
            track_layout: LayoutRefinement::default(),
            item_layout: LayoutRefinement::default(),
            track_start_neg_margin: None,
            track_start_neg_margin_viewport_breakpoints: Vec::new(),
            track_start_neg_margin_layout_breakpoints: Vec::new(),
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

    /// Breakpoint-based track start negative margin patches for the **window viewport** width
    /// (Tailwind `md:` / `lg:`).
    pub fn viewport_track_start_neg_margin_breakpoint(
        mut self,
        min_width_px: Px,
        margin: Space,
    ) -> Self {
        self.track_start_neg_margin_viewport_breakpoints
            .push(CarouselSpaceBreakpoint {
                min_width_px,
                value: margin,
            });
        self.track_start_neg_margin_viewport_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    pub fn viewport_track_start_neg_margin_breakpoints(
        mut self,
        breakpoints: impl IntoIterator<Item = CarouselSpaceBreakpoint>,
    ) -> Self {
        self.track_start_neg_margin_viewport_breakpoints
            .extend(breakpoints.into_iter().collect::<Vec<_>>());
        self.track_start_neg_margin_viewport_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    /// Breakpoint-based track start negative margin patches for the **carousel viewport** width
    /// (container-query style).
    pub fn track_start_neg_margin_breakpoint(mut self, min_width_px: Px, margin: Space) -> Self {
        self.track_start_neg_margin_layout_breakpoints
            .push(CarouselSpaceBreakpoint {
                min_width_px,
                value: margin,
            });
        self.track_start_neg_margin_layout_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    pub fn track_start_neg_margin_breakpoints(
        mut self,
        breakpoints: impl IntoIterator<Item = CarouselSpaceBreakpoint>,
    ) -> Self {
        self.track_start_neg_margin_layout_breakpoints
            .extend(breakpoints.into_iter().collect::<Vec<_>>());
        self.track_start_neg_margin_layout_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }
}

/// shadcn/ui `CarouselItem` (v4).
#[derive(Debug)]
pub struct CarouselItem {
    child: AnyElement,
    layout: LayoutRefinement,
    padding_start: Option<Space>,
    viewport_padding_start_breakpoints: Vec<CarouselSpaceBreakpoint>,
    padding_start_breakpoints: Vec<CarouselSpaceBreakpoint>,
    viewport_layout_breakpoints: Vec<CarouselItemLayoutBreakpoint>,
    layout_breakpoints: Vec<CarouselItemLayoutBreakpoint>,
}

impl CarouselItem {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
            layout: LayoutRefinement::default(),
            padding_start: None,
            viewport_padding_start_breakpoints: Vec::new(),
            padding_start_breakpoints: Vec::new(),
            viewport_layout_breakpoints: Vec::new(),
            layout_breakpoints: Vec::new(),
        }
    }

    /// Matches shadcn's per-item `className` surface (e.g. `md:basis-1/2`).
    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    /// Breakpoint-based layout patches for the **window viewport** width (Tailwind `md:` / `lg:`).
    ///
    /// This matches shadcn's responsive `md:` / `lg:` semantics (device/viewport-driven), but uses
    /// explicit pixel thresholds and typed layout patches instead of Tailwind strings.
    pub fn viewport_layout_breakpoint(mut self, min_width_px: Px, patch: LayoutRefinement) -> Self {
        self.viewport_layout_breakpoints
            .push(CarouselItemLayoutBreakpoint {
                min_width_px,
                patch,
            });
        self.viewport_layout_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    pub fn viewport_layout_breakpoints(
        mut self,
        breakpoints: impl IntoIterator<Item = CarouselItemLayoutBreakpoint>,
    ) -> Self {
        self.viewport_layout_breakpoints
            .extend(breakpoints.into_iter().collect::<Vec<_>>());
        self.viewport_layout_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    /// Breakpoint-based layout patches for the **carousel viewport** width (container-query style).
    ///
    /// This is useful for panel-width responsiveness inside docking/panels. For Tailwind-aligned
    /// device/viewport breakpoints, prefer [`CarouselItem::viewport_layout_breakpoint`].
    ///
    /// Note: breakpoints are evaluated against the measured carousel viewport width. The value is
    /// frame-lagged (ADR 0231). On initial mount, the carousel viewport width is discovered after
    /// the first layout pass, so breakpoint patches may apply starting from the third frame.
    pub fn layout_breakpoint(mut self, min_width_px: Px, patch: LayoutRefinement) -> Self {
        self.layout_breakpoints.push(CarouselItemLayoutBreakpoint {
            min_width_px,
            patch,
        });
        self.layout_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    pub fn layout_breakpoints(
        mut self,
        breakpoints: impl IntoIterator<Item = CarouselItemLayoutBreakpoint>,
    ) -> Self {
        self.layout_breakpoints
            .extend(breakpoints.into_iter().collect::<Vec<_>>());
        self.layout_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    /// Matches shadcn's per-item `className="pl-4"` / `pt-4` spacing approach.
    ///
    /// This is a convenience surface that maps to the recipe's start padding for each slide.
    pub fn padding_start(mut self, padding: Space) -> Self {
        self.padding_start = Some(padding);
        self
    }

    /// Breakpoint-based item start padding patches for the **window viewport** width
    /// (Tailwind `md:` / `lg:`).
    pub fn viewport_padding_start_breakpoint(mut self, min_width_px: Px, padding: Space) -> Self {
        self.viewport_padding_start_breakpoints
            .push(CarouselSpaceBreakpoint {
                min_width_px,
                value: padding,
            });
        self.viewport_padding_start_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    pub fn viewport_padding_start_breakpoints(
        mut self,
        breakpoints: impl IntoIterator<Item = CarouselSpaceBreakpoint>,
    ) -> Self {
        self.viewport_padding_start_breakpoints
            .extend(breakpoints.into_iter().collect::<Vec<_>>());
        self.viewport_padding_start_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    /// Breakpoint-based item start padding patches for the **carousel viewport** width
    /// (container-query style).
    pub fn padding_start_breakpoint(mut self, min_width_px: Px, padding: Space) -> Self {
        self.padding_start_breakpoints
            .push(CarouselSpaceBreakpoint {
                min_width_px,
                value: padding,
            });
        self.padding_start_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }

    pub fn padding_start_breakpoints(
        mut self,
        breakpoints: impl IntoIterator<Item = CarouselSpaceBreakpoint>,
    ) -> Self {
        self.padding_start_breakpoints
            .extend(breakpoints.into_iter().collect::<Vec<_>>());
        self.padding_start_breakpoints
            .sort_by(|a, b| a.min_width_px.0.total_cmp(&b.min_width_px.0));
        self
    }
}

impl From<AnyElement> for CarouselItem {
    fn from(child: AnyElement) -> Self {
        Self::new(child)
    }
}

#[derive(Debug, Clone)]
pub struct CarouselItemLayoutBreakpoint {
    pub min_width_px: Px,
    pub patch: LayoutRefinement,
}

fn resolve_item_viewport_layout_breakpoints(
    viewport_width_px: Px,
    breakpoints: &[CarouselItemLayoutBreakpoint],
) -> LayoutRefinement {
    let mut patch = LayoutRefinement::default();
    for bp in breakpoints {
        if viewport_width_px.0 >= bp.min_width_px.0 {
            patch = patch.merge(bp.patch.clone());
        }
    }
    patch
}

fn resolve_item_layout_breakpoints(
    viewport_width_px: Px,
    breakpoints: &[CarouselItemLayoutBreakpoint],
) -> LayoutRefinement {
    let mut patch = LayoutRefinement::default();
    for bp in breakpoints {
        if viewport_width_px.0 >= bp.min_width_px.0 {
            patch = patch.merge(bp.patch.clone());
        }
    }
    patch
}

fn resolve_loop_enabled_effective(
    loop_requested: bool,
    items_len: usize,
    view_size: f32,
    slides: &[headless_carousel::CarouselSlide1D],
) -> bool {
    if !loop_requested {
        return false;
    }

    // Embla downgrades `loop=true` when content cannot physically loop. We mirror that outcome
    // when geometry is measurable. Until then, keep loop requested to avoid flicker.
    if view_size <= 0.0 {
        return true;
    }
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
}

fn resolve_nearest_snap_index(snaps: &[Px], offset: Px) -> usize {
    let mut best = 0usize;
    let mut best_dist = f32::INFINITY;
    for (idx, snap) in snaps.iter().enumerate() {
        let dist = (snap.0 - offset.0).abs();
        if dist < best_dist {
            best_dist = dist;
            best = idx;
        }
    }
    best
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
