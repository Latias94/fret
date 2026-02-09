//! Material 3 top app bar primitives (P1).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.top-app-bar.*` (Material Web v30).
//! - Minimal scroll surface: `scrolled: bool` (or a `TopAppBarScrollBehavior`) toggles the
//!   scrolled container treatment (Compose-style `TopAppBarColors.scrolledContainerColor`).
//! - Semantics: uses `SemanticsRole::Toolbar`.

use std::{cell::RefCell, rc::Rc, sync::Arc};

use fret_core::{Axis, Color, Edges, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_icons::IconId;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PositionStyle, SemanticsProps, SpacerProps, StackProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{Theme, UiHost};
use fret_ui_kit::{ColorRef, WidgetStateProperty};

use crate::foundation::surface::material_surface_style;
use crate::icon_button::{IconButton, IconButtonStyle, IconButtonVariant};
use crate::motion::{SpringAnimator, SpringSpec, ms_to_frames};
use crate::tokens::top_app_bar as top_app_bar_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TopAppBarVariant {
    #[default]
    Small,
    SmallCentered,
    Medium,
    Large,
}

#[derive(Debug, Clone)]
pub struct TopAppBarScrollBehavior {
    kind: TopAppBarScrollBehaviorKind,
    scroll_handle: ScrollHandle,
    enter_always_state: Option<Rc<RefCell<EnterAlwaysScrollState>>>,
    exit_until_collapsed_state: Option<Rc<RefCell<ExitUntilCollapsedScrollState>>>,
    settle_policy: TopAppBarSettlePolicy,
}

impl TopAppBarScrollBehavior {
    /// A minimal pinned behavior that treats any positive scroll offset as "scrolled behind".
    pub fn pinned(scroll_handle: ScrollHandle) -> Self {
        Self {
            kind: TopAppBarScrollBehaviorKind::Pinned,
            scroll_handle,
            enter_always_state: None,
            exit_until_collapsed_state: None,
            settle_policy: TopAppBarSettlePolicy::disabled(),
        }
    }

    /// A collapse behavior that hides the app bar as content is scrolled, and shows it again when
    /// the scroll direction reverses.
    ///
    /// This is a policy-only approximation of Compose's `TopAppBarDefaults.enterAlwaysScrollBehavior`:
    /// we derive the app bar height from scroll handle deltas rather than wiring a full nested
    /// scroll pipeline.
    pub fn enter_always(scroll_handle: ScrollHandle) -> Self {
        Self {
            kind: TopAppBarScrollBehaviorKind::EnterAlways,
            scroll_handle,
            enter_always_state: Some(Rc::new(RefCell::new(EnterAlwaysScrollState::default()))),
            exit_until_collapsed_state: None,
            settle_policy: TopAppBarSettlePolicy::disabled(),
        }
    }

    /// A collapse behavior that reduces medium/large app bars down to their collapsed height while
    /// keeping them visible.
    ///
    /// This is a policy-only approximation of Compose's
    /// `TopAppBarDefaults.exitUntilCollapsedScrollBehavior`.
    pub fn exit_until_collapsed(scroll_handle: ScrollHandle) -> Self {
        Self {
            kind: TopAppBarScrollBehaviorKind::ExitUntilCollapsed,
            scroll_handle,
            enter_always_state: None,
            exit_until_collapsed_state: Some(Rc::new(RefCell::new(
                ExitUntilCollapsedScrollState::default(),
            ))),
            settle_policy: TopAppBarSettlePolicy::disabled(),
        }
    }

    /// Enable a small policy-only settle behavior that snaps the app bar to a stable state after
    /// scroll has been idle for a short time.
    ///
    /// Note: this does not implement nested scroll consumption nor fling velocity. It is intended
    /// to reduce "half-collapsed" states after programmatic scroll jumps or short wheel scrolls.
    pub fn settle_on_idle(mut self) -> Self {
        self.settle_policy = TopAppBarSettlePolicy::enabled();
        self
    }

    pub fn is_scrolled(&self) -> bool {
        // Match the scrollability gate used by `resolve_layout` so callers can treat this as a
        // stable predicate for switching into "on-scroll" token branches.
        let viewport_h = self.scroll_handle.viewport_size().height.0;
        let content_h = self.scroll_handle.content_size().height.0;
        let sizes_known = viewport_h > 0.01 && content_h > 0.01;
        let can_scroll_y = if sizes_known {
            self.scroll_handle.max_offset().y.0 > 0.01
        } else {
            true
        };
        can_scroll_y && self.scroll_handle.offset().y.0 > 0.01
    }

    fn resolve_layout(&self, expanded_height: Px, collapsed_height: Px) -> TopAppBarScrollLayout {
        let initial_offset_y = self.scroll_handle.offset().y.0.max(0.0);
        // Prefer gating by actual scrollability, but treat "unknown" viewport/content sizes as
        // scrollable so policy-only behaviors still function before layout has populated metrics
        // (e.g. in unit tests or early frames).
        let viewport_h = self.scroll_handle.viewport_size().height.0;
        let content_h = self.scroll_handle.content_size().height.0;
        let sizes_known = viewport_h > 0.01 && content_h > 0.01;
        let can_scroll_y = if sizes_known {
            self.scroll_handle.max_offset().y.0 > 0.01
        } else {
            true
        };

        let (container_height, collapsed_fraction, effective_offset_y) = match self.kind {
            TopAppBarScrollBehaviorKind::Pinned => (expanded_height, 0.0, initial_offset_y),
            TopAppBarScrollBehaviorKind::ExitUntilCollapsed => {
                if !can_scroll_y {
                    return TopAppBarScrollLayout {
                        scrolled: false,
                        container_height: expanded_height,
                        collapsed_fraction: 0.0,
                    };
                }
                let collapse_range = (expanded_height.0 - collapsed_height.0).max(0.0);
                if collapse_range <= 0.01 {
                    (expanded_height, 0.0, initial_offset_y)
                } else {
                    let settled_offset_y = if self.settle_policy.enabled
                        && initial_offset_y <= collapse_range + 0.01
                    {
                        let state_rc = self
                            .exit_until_collapsed_state
                            .as_ref()
                            .expect("exit-until-collapsed behavior requires internal state")
                            .clone();
                        let mut state = state_rc.borrow_mut();
                        state.tick = state.tick.saturating_add(1);
                        state.observe_and_maybe_snap(
                            self.scroll_handle.clone(),
                            initial_offset_y,
                            collapse_range,
                            self.settle_policy,
                        )
                    } else {
                        initial_offset_y
                    };

                    let collapse = settled_offset_y.min(collapse_range);
                    let height = Px((expanded_height.0 - collapse).max(collapsed_height.0));
                    let fraction = (collapse / collapse_range).clamp(0.0, 1.0);
                    (height, fraction, settled_offset_y)
                }
            }
            TopAppBarScrollBehaviorKind::EnterAlways => {
                if !can_scroll_y {
                    return TopAppBarScrollLayout {
                        scrolled: false,
                        container_height: expanded_height,
                        collapsed_fraction: 0.0,
                    };
                }
                let state_rc = self
                    .enter_always_state
                    .as_ref()
                    .expect("enter-always behavior requires internal state")
                    .clone();
                let mut state = state_rc.borrow_mut();
                let (h, f) = state.resolve(
                    expanded_height,
                    collapsed_height,
                    initial_offset_y,
                    self.settle_policy,
                );
                (h, f, initial_offset_y)
            }
        };
        let scrolled = can_scroll_y && effective_offset_y > 0.01;

        TopAppBarScrollLayout {
            scrolled,
            container_height,
            collapsed_fraction,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TopAppBarScrollBehaviorKind {
    Pinned,
    EnterAlways,
    ExitUntilCollapsed,
}

#[derive(Debug, Clone, Copy)]
struct TopAppBarSettlePolicy {
    enabled: bool,
    idle_frames: u32,
    snap_threshold: f32,
    spring: SpringSpec,
}

impl TopAppBarSettlePolicy {
    fn disabled() -> Self {
        Self {
            enabled: false,
            idle_frames: 0,
            snap_threshold: 0.5,
            spring: SpringSpec::new(1.0, 800.0),
        }
    }

    fn enabled() -> Self {
        Self {
            enabled: true,
            idle_frames: ms_to_frames(200).min(u64::from(u32::MAX)) as u32,
            snap_threshold: 0.5,
            spring: SpringSpec::new(1.0, 800.0),
        }
    }
}

#[derive(Debug, Default)]
struct EnterAlwaysScrollState {
    last_offset_y: Option<f32>,
    height_offset: f32,
    idle_frames: u32,
    tick: u64,
    settle: SpringAnimator,
}

#[derive(Debug, Default)]
struct ExitUntilCollapsedScrollState {
    last_offset_y: Option<f32>,
    idle_frames: u32,
    tick: u64,
}

impl ExitUntilCollapsedScrollState {
    fn observe_and_maybe_snap(
        &mut self,
        scroll_handle: ScrollHandle,
        offset_y: f32,
        collapse_range: f32,
        settle_policy: TopAppBarSettlePolicy,
    ) -> f32 {
        let last = self.last_offset_y.unwrap_or(offset_y);
        let delta = offset_y - last;
        self.last_offset_y = Some(offset_y);

        if delta.abs() > 0.01 {
            self.idle_frames = 0;
            return offset_y;
        }
        self.idle_frames = self.idle_frames.saturating_add(1);
        if self.idle_frames < settle_policy.idle_frames {
            return offset_y;
        }

        // Policy-only snap:
        // - While within the collapse range, snap the scroll offset to either 0 or the full
        //   collapse range after the wheel has been idle for a short duration.
        // - This avoids introducing nested scroll consumption in v1, at the cost of moving content.
        let frac = if collapse_range <= 0.01 {
            0.0
        } else {
            (offset_y / collapse_range).clamp(0.0, 1.0)
        };
        let snap_to_collapsed = frac >= settle_policy.snap_threshold;
        let target = if snap_to_collapsed {
            collapse_range
        } else {
            0.0
        };

        let prev = scroll_handle.offset();
        scroll_handle.set_offset(fret_core::Point::new(prev.x, Px(target)));

        let snapped = scroll_handle.offset().y.0.max(0.0);
        self.last_offset_y = Some(snapped);
        self.idle_frames = 0;
        snapped
    }
}

impl EnterAlwaysScrollState {
    fn resolve(
        &mut self,
        expanded_height: Px,
        collapsed_height: Px,
        offset_y: f32,
        settle_policy: TopAppBarSettlePolicy,
    ) -> (Px, f32) {
        self.tick = self.tick.saturating_add(1);

        let last = self.last_offset_y.unwrap_or(offset_y);
        let delta = offset_y - last;
        self.last_offset_y = Some(offset_y);

        // Match Compose's `heightOffset` convention: 0 is fully expanded, negative collapses.
        let limit = -expanded_height.0.max(0.0);
        self.height_offset = (self.height_offset - delta).clamp(limit, 0.0);

        // Keep the settle animator aligned with the imperative height offset when the scroll is
        // actively changing.
        if delta.abs() > 0.01 {
            self.idle_frames = 0;
            self.settle.reset(self.tick, self.height_offset);
        } else {
            self.idle_frames = self.idle_frames.saturating_add(1);
        }

        if settle_policy.enabled && self.idle_frames >= settle_policy.idle_frames {
            let visible_h = (expanded_height.0 + self.height_offset).clamp(0.0, expanded_height.0);
            let visible_fraction = if expanded_height.0 <= 0.01 {
                1.0
            } else {
                (visible_h / expanded_height.0).clamp(0.0, 1.0)
            };
            let snap_to_expanded = visible_fraction >= settle_policy.snap_threshold;
            let target = if snap_to_expanded { 0.0 } else { limit };
            self.settle
                .set_target(self.tick, target, settle_policy.spring);
        }

        self.settle.advance(self.tick);
        if self.settle.is_initialized() {
            self.height_offset = self.settle.value().clamp(limit, 0.0);
        }

        let height = Px((expanded_height.0 + self.height_offset).max(0.0));

        let collapse_range = (expanded_height.0 - collapsed_height.0).max(0.0);
        let fraction = if collapse_range <= 0.01 {
            0.0
        } else {
            let clamped_height = height.0.max(collapsed_height.0).min(expanded_height.0);
            ((expanded_height.0 - clamped_height) / collapse_range).clamp(0.0, 1.0)
        };

        (height, fraction)
    }
}

#[derive(Debug, Clone, Copy)]
struct TopAppBarScrollLayout {
    scrolled: bool,
    container_height: Px,
    collapsed_fraction: f32,
}

#[derive(Clone)]
pub struct TopAppBarAction {
    icon: IconId,
    on_activate: Option<OnActivate>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for TopAppBarAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TopAppBarAction")
            .field("icon", &self.icon.as_str())
            .field("on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl TopAppBarAction {
    pub fn new(icon: IconId) -> Self {
        Self {
            icon,
            on_activate: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct TopAppBar {
    variant: TopAppBarVariant,
    title: Arc<str>,
    navigation_icon: Option<TopAppBarAction>,
    actions: Vec<TopAppBarAction>,
    scroll_behavior: Option<TopAppBarScrollBehavior>,
    scrolled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl TopAppBar {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            variant: TopAppBarVariant::default(),
            title: title.into(),
            navigation_icon: None,
            actions: Vec::new(),
            scroll_behavior: None,
            scrolled: false,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn variant(mut self, variant: TopAppBarVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn navigation_icon(mut self, action: TopAppBarAction) -> Self {
        self.navigation_icon = Some(action);
        self
    }

    pub fn actions(mut self, actions: Vec<TopAppBarAction>) -> Self {
        self.actions = actions;
        self
    }

    pub fn scroll_behavior(mut self, behavior: TopAppBarScrollBehavior) -> Self {
        self.scroll_behavior = Some(behavior);
        self
    }

    /// Marks the app bar as "scrolled behind".
    ///
    /// This is a deliberately small surface (v1) that stands in for Compose's `TopAppBarScrollBehavior`.
    /// A typical integration is `scrolled = scroll_offset_y > 0.0`.
    pub fn scrolled(mut self, scrolled: bool) -> Self {
        self.scrolled = scrolled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let (expanded_height, collapsed_height) = {
                let theme = Theme::global(&*cx.app);
                let expanded_height = top_app_bar_tokens::container_height(theme, self.variant);
                let collapsed_height = match self.variant {
                    TopAppBarVariant::Medium | TopAppBarVariant::Large => {
                        top_app_bar_tokens::container_height(theme, TopAppBarVariant::Small)
                    }
                    _ => expanded_height,
                };
                (expanded_height, collapsed_height)
            };

            let layout = self
                .scroll_behavior
                .as_ref()
                .map(|behavior| behavior.resolve_layout(expanded_height, collapsed_height));

            let scrolled = layout.as_ref().map_or(self.scrolled, |l| l.scrolled);
            let container_height = layout
                .as_ref()
                .map_or(expanded_height, |l| l.container_height);
            let collapsed_fraction = layout.as_ref().map_or(0.0, |l| l.collapsed_fraction);
            let top_row_height = Px(container_height.0.min(collapsed_height.0));
            let (
                surface,
                corner_radii,
                leading_icon_color,
                trailing_icon_color,
                title_style,
                title_color,
                collapsed_title_style,
                collapsed_title_color,
            ) = {
                let theme = Theme::global(&*cx.app);
                let background =
                    top_app_bar_tokens::container_background(theme, self.variant, scrolled);
                let elevation =
                    top_app_bar_tokens::container_elevation(theme, self.variant, scrolled);
                let corner_radii = top_app_bar_tokens::container_shape(theme, self.variant);
                let surface =
                    material_surface_style(theme, background, elevation, None, corner_radii);

                let leading_icon_color =
                    top_app_bar_tokens::leading_icon_color(theme, self.variant);
                let trailing_icon_color =
                    top_app_bar_tokens::trailing_icon_color(theme, self.variant);

                let title_style = top_app_bar_tokens::headline_text_style(theme, self.variant);
                let title_color = top_app_bar_tokens::headline_color(theme, self.variant);

                let collapsed_title_style =
                    top_app_bar_tokens::headline_text_style(theme, TopAppBarVariant::Small);
                let collapsed_title_color =
                    top_app_bar_tokens::headline_color(theme, TopAppBarVariant::Small);

                (
                    surface,
                    corner_radii,
                    leading_icon_color,
                    trailing_icon_color,
                    title_style,
                    title_color,
                    collapsed_title_style,
                    collapsed_title_color,
                )
            };

            let mut container = ContainerProps::default();
            container.background = Some(surface.background);
            container.shadow = surface.shadow;
            container.corner_radii = corner_radii;
            container.layout.size.width = Length::Fill;
            container.layout.size.height = Length::Px(container_height);
            // Compose uses 4dp horizontal padding for app bars; keep that as a stable default.
            container.padding = Edges {
                left: Px(4.0),
                right: Px(4.0),
                top: Px(0.0),
                bottom: Px(0.0),
            };

            let sem = SemanticsProps {
                role: SemanticsRole::Toolbar,
                label: self.a11y_label.clone(),
                test_id: self.test_id.clone(),
                ..Default::default()
            };

            cx.semantics(sem, move |cx| {
                vec![cx.container(container, move |cx| match self.variant {
                    TopAppBarVariant::Small | TopAppBarVariant::SmallCentered => {
                        vec![top_app_bar_single_row(
                            cx,
                            &self,
                            leading_icon_color,
                            trailing_icon_color,
                            title_style.clone(),
                            title_color,
                        )]
                    }
                    TopAppBarVariant::Medium | TopAppBarVariant::Large => {
                        vec![top_app_bar_two_rows(
                            cx,
                            &self,
                            leading_icon_color,
                            trailing_icon_color,
                            title_style.clone(),
                            title_color,
                            collapsed_title_style.clone(),
                            collapsed_title_color,
                            top_row_height,
                            collapsed_fraction,
                        )]
                    }
                })]
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px, Size};

    #[test]
    fn pinned_scroll_behavior_reports_scrolled_from_handle_offset() {
        let handle = ScrollHandle::default();
        let behavior = TopAppBarScrollBehavior::pinned(handle.clone());

        assert!(
            !behavior.is_scrolled(),
            "expected pinned behavior to start in the unscrolled state"
        );

        handle.set_offset(Point::new(Px(0.0), Px(12.0)));
        assert!(
            behavior.is_scrolled(),
            "expected pinned behavior to consider non-zero offsets as scrolled"
        );
    }

    #[test]
    fn exit_until_collapsed_clamps_height_to_collapsed_range() {
        let handle = ScrollHandle::default();
        let behavior = TopAppBarScrollBehavior::exit_until_collapsed(handle.clone());

        let expanded = Px(152.0);
        let collapsed = Px(64.0);
        let collapse_range = expanded.0 - collapsed.0;

        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, expanded);
        assert!((layout.collapsed_fraction - 0.0).abs() < 0.0001);

        handle.set_offset(Point::new(Px(0.0), Px(22.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!((layout.container_height.0 - (expanded.0 - 22.0)).abs() < 0.01);
        assert!((layout.collapsed_fraction - (22.0 / collapse_range)).abs() < 0.01);

        handle.set_offset(Point::new(Px(0.0), Px(999.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, collapsed);
        assert!((layout.collapsed_fraction - 1.0).abs() < 0.0001);
    }

    #[test]
    fn enter_always_tracks_scroll_direction_and_can_hide_fully() {
        let handle = ScrollHandle::default();
        let behavior = TopAppBarScrollBehavior::enter_always(handle.clone());

        let expanded = Px(64.0);
        let collapsed = Px(64.0);

        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, expanded);

        handle.set_offset(Point::new(Px(0.0), Px(20.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!((layout.container_height.0 - 44.0).abs() < 0.01);

        handle.set_offset(Point::new(Px(0.0), Px(50.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!((layout.container_height.0 - 14.0).abs() < 0.01);

        // Reverse scroll direction should bring the app bar back.
        handle.set_offset(Point::new(Px(0.0), Px(30.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!((layout.container_height.0 - 34.0).abs() < 0.01);

        // Large deltas can collapse fully.
        handle.set_offset(Point::new(Px(0.0), Px(10_000.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, Px(0.0));
    }

    #[test]
    fn enter_always_handles_programmatic_scroll_jumps_and_recovers() {
        let handle = ScrollHandle::default();
        let behavior = TopAppBarScrollBehavior::enter_always(handle.clone());

        let expanded = Px(64.0);
        let collapsed = Px(64.0);

        handle.set_offset(Point::new(Px(0.0), Px(0.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, expanded);

        // Programmatic jump down collapses fully.
        handle.set_offset(Point::new(Px(0.0), Px(10_000.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, Px(0.0));

        // Programmatic jump back to the top restores the bar.
        handle.set_offset(Point::new(Px(0.0), Px(0.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, expanded);

        // Subsequent deltas should behave normally (state not "stuck" after the jump).
        handle.set_offset(Point::new(Px(0.0), Px(20.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!((layout.container_height.0 - 44.0).abs() < 0.01);
    }

    #[test]
    fn enter_always_respects_can_scroll_gate_when_metrics_are_known() {
        let handle = ScrollHandle::default();
        let behavior = TopAppBarScrollBehavior::enter_always(handle.clone());

        let expanded = Px(64.0);
        let collapsed = Px(64.0);

        // Start in a scrollable configuration and collapse a bit.
        handle.set_viewport_size(Size::new(Px(100.0), Px(100.0)));
        handle.set_content_size(Size::new(Px(100.0), Px(400.0)));

        // Prime the internal delta baseline at the top.
        handle.set_offset(Point::new(Px(0.0), Px(0.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, expanded);

        handle.set_offset(Point::new(Px(0.0), Px(30.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!((layout.container_height.0 - 34.0).abs() < 0.01);

        // Switch to a non-scrollable configuration while keeping a positive offset; the bar should
        // not continue collapsing based on stale offsets.
        handle.set_content_size(Size::new(Px(100.0), Px(100.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, expanded);

        // Switch back to scrollable; the behavior should continue smoothly.
        handle.set_content_size(Size::new(Px(100.0), Px(400.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!((layout.container_height.0 - 34.0).abs() < 0.01);
    }

    #[test]
    fn enter_always_settles_to_expanded_after_idle_when_mostly_visible() {
        let handle = ScrollHandle::default();
        let behavior = TopAppBarScrollBehavior::enter_always(handle.clone()).settle_on_idle();

        let expanded = Px(64.0);
        let collapsed = Px(64.0);

        handle.set_viewport_size(Size::new(Px(100.0), Px(100.0)));
        handle.set_content_size(Size::new(Px(100.0), Px(400.0)));

        handle.set_offset(Point::new(Px(0.0), Px(0.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, expanded);

        // Collapse a bit (still more than 50% visible).
        handle.set_offset(Point::new(Px(0.0), Px(20.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!((layout.container_height.0 - 44.0).abs() < 0.01);

        // Hold the offset steady; the behavior should settle back to expanded.
        for _ in 0..240 {
            let layout = behavior.resolve_layout(expanded, collapsed);
            if (layout.container_height.0 - expanded.0).abs() < 0.5 {
                return;
            }
        }

        panic!("expected enter-always settle to expand fully after idle");
    }

    #[test]
    fn enter_always_settles_to_hidden_after_idle_when_mostly_collapsed() {
        let handle = ScrollHandle::default();
        let behavior = TopAppBarScrollBehavior::enter_always(handle.clone()).settle_on_idle();

        let expanded = Px(64.0);
        let collapsed = Px(64.0);

        handle.set_viewport_size(Size::new(Px(100.0), Px(100.0)));
        handle.set_content_size(Size::new(Px(100.0), Px(400.0)));

        handle.set_offset(Point::new(Px(0.0), Px(0.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, expanded);

        // Collapse deeply (less than 50% visible).
        handle.set_offset(Point::new(Px(0.0), Px(50.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!((layout.container_height.0 - 14.0).abs() < 0.01);

        // Hold the offset steady; the behavior should settle to fully hidden.
        for _ in 0..240 {
            let layout = behavior.resolve_layout(expanded, collapsed);
            if layout.container_height.0 < 0.5 {
                return;
            }
        }

        panic!("expected enter-always settle to hide fully after idle");
    }

    #[test]
    fn exit_until_collapsed_snaps_scroll_offset_to_nearest_state_after_idle() {
        let handle = ScrollHandle::default();
        let behavior =
            TopAppBarScrollBehavior::exit_until_collapsed(handle.clone()).settle_on_idle();

        let expanded = Px(152.0);
        let collapsed = Px(64.0);
        let collapse_range = expanded.0 - collapsed.0;

        handle.set_viewport_size(Size::new(Px(100.0), Px(100.0)));
        handle.set_content_size(Size::new(Px(100.0), Px(1000.0)));

        // Prime baseline.
        handle.set_offset(Point::new(Px(0.0), Px(0.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert_eq!(layout.container_height, expanded);

        // A small offset should settle back to 0.
        handle.set_offset(Point::new(Px(0.0), Px(20.0)));
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!((layout.container_height.0 - 132.0).abs() < 0.01);

        for _ in 0..60 {
            behavior.resolve_layout(expanded, collapsed);
        }
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!(
            handle.offset().y.0 < 0.5,
            "expected small offsets to snap back to 0 after idle"
        );
        assert!(
            !layout.scrolled,
            "expected a snapped-to-top scroll offset to clear the scrolled container state"
        );

        // A larger offset within the collapse range should settle to the full collapse range.
        handle.set_offset(Point::new(Px(0.0), Px(70.0)));
        for _ in 0..60 {
            behavior.resolve_layout(expanded, collapsed);
        }
        let layout = behavior.resolve_layout(expanded, collapsed);
        assert!(
            (handle.offset().y.0 - collapse_range).abs() < 0.5,
            "expected offsets past the threshold to snap to the collapse range after idle"
        );
        assert!(
            layout.scrolled,
            "expected a snapped-to-collapsed scroll offset to keep the scrolled container state"
        );
    }
}

fn top_app_bar_icon_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    action: &TopAppBarAction,
    icon_color: fret_core::Color,
) -> AnyElement {
    // Keep the IconButton interaction policy (state layer + ripple) but force the base icon color
    // to match top-app-bar tokens.
    let resolved_icon_color = if action.disabled {
        let mut c = icon_color;
        c.a *= 0.38;
        c
    } else {
        icon_color
    };

    let mut style = IconButtonStyle::default();
    style = style.icon_color(WidgetStateProperty::new(Some(ColorRef::Color(
        resolved_icon_color,
    ))));

    let mut btn = IconButton::new(action.icon.clone())
        .variant(IconButtonVariant::Standard)
        .style(style)
        .disabled(action.disabled);

    if let Some(handler) = action.on_activate.clone() {
        btn = btn.on_activate(handler);
    }
    if let Some(label) = action.a11y_label.clone() {
        btn = btn.a11y_label(label);
    }
    if let Some(id) = action.test_id.clone() {
        btn = btn.test_id(id);
    }

    btn.into_element(cx)
}

fn slot_container<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    min_width: Px,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.min_width = Some(min_width);
    props.layout.size.min_height = Some(Px(48.0));
    cx.container(props, move |_cx| children)
}

fn top_app_bar_title_element_for_variant<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: Arc<str>,
    style: fret_core::TextStyle,
    color: Color,
) -> AnyElement {
    let mut props = fret_ui::element::TextProps::new(title);
    props.style = Some(style);
    props.color = Some(color);
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Ellipsis;
    cx.text_props(props)
}

fn top_app_bar_single_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    bar: &TopAppBar,
    leading_color: Color,
    trailing_color: Color,
    title_style: fret_core::TextStyle,
    title_color: Color,
) -> AnyElement {
    let leading = bar
        .navigation_icon
        .as_ref()
        .map(|action| top_app_bar_icon_button(cx, action, leading_color));

    let trailing_buttons: Vec<AnyElement> = bar
        .actions
        .iter()
        .map(|action| top_app_bar_icon_button(cx, action, trailing_color))
        .collect();

    let title =
        top_app_bar_title_element_for_variant(cx, bar.title.clone(), title_style, title_color);

    match bar.variant {
        TopAppBarVariant::SmallCentered => {
            let leading_reserved = if leading.is_some() { Px(48.0) } else { Px(0.0) };
            let trailing_reserved = Px((trailing_buttons.len() as f32) * 48.0);
            let side_reserved = Px(leading_reserved.0.max(trailing_reserved.0));

            let leading_slot = match leading {
                Some(btn) => slot_container(cx, Px(48.0), vec![btn]),
                None => slot_container(cx, Px(0.0), Vec::new()),
            };
            let trailing_slot = match trailing_buttons.is_empty() {
                true => slot_container(cx, Px(0.0), Vec::new()),
                false => slot_container(cx, Px(0.0), trailing_buttons),
            };

            let mut icons_row = FlexProps::default();
            icons_row.direction = Axis::Horizontal;
            icons_row.align = CrossAlign::Center;
            icons_row.justify = MainAlign::Start;
            icons_row.wrap = false;
            icons_row.layout.size.width = Length::Fill;
            icons_row.layout.size.height = Length::Fill;

            let icons = cx.flex(icons_row, move |cx| {
                let spacer = cx.spacer(SpacerProps::default());
                vec![leading_slot, spacer, trailing_slot]
            });

            let mut title_layer = ContainerProps::default();
            title_layer.layout.size.width = Length::Fill;
            title_layer.layout.size.height = Length::Fill;
            title_layer.layout.position = PositionStyle::Absolute;
            title_layer.layout.inset.left = Some(side_reserved);
            title_layer.layout.inset.right = Some(side_reserved);
            title_layer.layout.inset.top = Some(Px(0.0));
            title_layer.layout.inset.bottom = Some(Px(0.0));

            let mut title_row = FlexProps::default();
            title_row.direction = Axis::Horizontal;
            title_row.align = CrossAlign::Center;
            title_row.justify = MainAlign::Center;
            title_row.wrap = false;
            title_row.layout.size.width = Length::Fill;
            title_row.layout.size.height = Length::Fill;

            let title_overlay = cx.container(title_layer, move |cx| {
                vec![cx.flex(title_row, move |_cx| vec![title.clone()])]
            });

            let mut stack_props = StackProps::default();
            stack_props.layout.size.width = Length::Fill;
            stack_props.layout.size.height = Length::Fill;

            cx.stack_props(stack_props, move |_cx| vec![icons, title_overlay])
        }
        _ => {
            let mut row = FlexProps::default();
            row.direction = Axis::Horizontal;
            row.align = CrossAlign::Center;
            row.justify = MainAlign::Start;
            row.wrap = false;
            row.layout.size.width = Length::Fill;
            row.layout.size.height = Length::Fill;

            let mut title_layout = LayoutStyle::default();
            title_layout.flex.grow = 1.0;
            title_layout.flex.basis = Length::Px(Px(0.0));

            let mut title_container_props = FlexProps::default();
            title_container_props.direction = Axis::Horizontal;
            title_container_props.align = CrossAlign::Center;
            title_container_props.justify = MainAlign::Start;
            title_container_props.wrap = false;
            title_container_props.layout = title_layout;
            let title_container = cx.flex(title_container_props, move |_cx| vec![title.clone()]);

            let leading_slot = match leading {
                Some(btn) => slot_container(cx, Px(48.0), vec![btn]),
                None => slot_container(cx, Px(12.0), Vec::new()),
            };
            let trailing_slot = slot_container(cx, Px(0.0), trailing_buttons);

            cx.flex(row, move |_cx| {
                vec![leading_slot, title_container, trailing_slot]
            })
        }
    }
}

fn top_app_bar_two_rows<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    bar: &TopAppBar,
    leading_color: Color,
    trailing_color: Color,
    expanded_title_style: fret_core::TextStyle,
    expanded_title_color: Color,
    collapsed_title_style: fret_core::TextStyle,
    collapsed_title_color: Color,
    top_row_height: Px,
    collapsed_fraction: f32,
) -> AnyElement {
    let leading = bar
        .navigation_icon
        .as_ref()
        .map(|action| top_app_bar_icon_button(cx, action, leading_color));

    let trailing_buttons: Vec<AnyElement> = bar
        .actions
        .iter()
        .map(|action| top_app_bar_icon_button(cx, action, trailing_color))
        .collect();

    let expanded_title = top_app_bar_title_element_for_variant(
        cx,
        bar.title.clone(),
        expanded_title_style,
        expanded_title_color,
    );
    let collapsed_title = top_app_bar_title_element_for_variant(
        cx,
        bar.title.clone(),
        collapsed_title_style,
        collapsed_title_color,
    );

    let mut column = FlexProps::default();
    column.direction = Axis::Vertical;
    column.align = CrossAlign::Stretch;
    column.justify = MainAlign::Start;
    column.wrap = false;
    column.layout.size.width = Length::Fill;
    column.layout.size.height = Length::Fill;

    let top_row = {
        let mut row = FlexProps::default();
        row.direction = Axis::Horizontal;
        row.align = CrossAlign::Center;
        row.justify = MainAlign::Start;
        row.wrap = false;
        row.layout.size.width = Length::Fill;
        row.layout.size.height = Length::Px(top_row_height);

        let leading_slot = match leading {
            Some(btn) => slot_container(cx, Px(48.0), vec![btn]),
            None => slot_container(cx, Px(48.0), Vec::new()),
        };

        let mut title_layout = LayoutStyle::default();
        title_layout.flex.grow = 1.0;
        title_layout.flex.basis = Length::Px(Px(0.0));
        let mut title_container = FlexProps::default();
        title_container.direction = Axis::Horizontal;
        title_container.align = CrossAlign::Center;
        title_container.justify = MainAlign::Start;
        title_container.wrap = false;
        title_container.layout = title_layout;
        let middle = cx.flex(title_container, move |cx| {
            vec![cx.opacity(collapsed_fraction, |_cx| vec![collapsed_title.clone()])]
        });

        let trailing_slot = slot_container(cx, Px(0.0), trailing_buttons);
        cx.flex(row, move |_cx| vec![leading_slot, middle, trailing_slot])
    };

    let base_bottom_padding = match bar.variant {
        TopAppBarVariant::Large => Px(28.0),
        _ => Px(24.0),
    };
    let bottom_padding = Px(base_bottom_padding.0 * (1.0 - collapsed_fraction).clamp(0.0, 1.0));

    let bottom_row = {
        let mut wrapper = ContainerProps::default();
        wrapper.layout.size.width = Length::Fill;
        wrapper.layout.size.height = Length::Fill;
        wrapper.layout.flex.grow = 1.0;
        wrapper.layout.flex.basis = Length::Px(Px(0.0));
        wrapper.padding = Edges {
            left: Px(12.0),
            right: Px(12.0),
            top: Px(0.0),
            bottom: bottom_padding,
        };

        cx.container(wrapper, move |cx| {
            let mut row = FlexProps::default();
            row.direction = Axis::Horizontal;
            row.align = CrossAlign::End;
            row.justify = MainAlign::Start;
            row.wrap = false;
            row.layout.size.width = Length::Fill;
            row.layout.size.height = Length::Fill;
            vec![cx.flex(row, move |cx| {
                vec![cx.opacity(1.0 - collapsed_fraction, |_cx| vec![expanded_title.clone()])]
            })]
        })
    };

    cx.flex(column, move |_cx| vec![top_row, bottom_row])
}
