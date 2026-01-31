//! NavigationMenu primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing navigation menu behavior in
//! recipes. It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/navigation-menu/src/navigation-menu.tsx`

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use fret_core::{Modifiers, Point, PointerType, Px, Rect, Size, Transform2D};
use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{AnyElement, LayoutStyle};
use fret_ui::elements::ContinuousFrames;
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::Side;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::model_watch::ModelWatchExt;
use crate::headless::transition::TransitionTimeline;
use crate::overlay;
use crate::primitives::popper;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

/// Radix `delayDuration` default (milliseconds).
pub const DEFAULT_DELAY_DURATION_MS: u64 = 200;
/// Radix `skipDelayDuration` default (milliseconds).
pub const DEFAULT_SKIP_DELAY_DURATION_MS: u64 = 300;
/// Radix `startCloseTimer` default (milliseconds).
pub const DEFAULT_CLOSE_DELAY_MS: u64 = 150;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationMenuConfig {
    pub delay_duration: Duration,
    pub skip_delay_duration: Duration,
    pub close_delay_duration: Duration,
}

impl Default for NavigationMenuConfig {
    fn default() -> Self {
        Self {
            delay_duration: Duration::from_millis(DEFAULT_DELAY_DURATION_MS),
            skip_delay_duration: Duration::from_millis(DEFAULT_SKIP_DELAY_DURATION_MS),
            close_delay_duration: Duration::from_millis(DEFAULT_CLOSE_DELAY_MS),
        }
    }
}

impl NavigationMenuConfig {
    pub fn new(
        delay_duration: Duration,
        skip_delay_duration: Duration,
        close_delay_duration: Duration,
    ) -> Self {
        Self {
            delay_duration,
            skip_delay_duration,
            close_delay_duration,
        }
    }
}

/// Returns a selected-value model that behaves like Radix `useControllableState` (`value` /
/// `defaultValue`).
///
/// Radix uses an empty string to represent "closed". In Fret we use `Option<Arc<str>>` (`None`
/// means closed).
pub fn navigation_menu_use_value_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<Option<Arc<str>>>>,
    default_value: impl FnOnce() -> Option<Arc<str>>,
) -> crate::primitives::controllable_state::ControllableModel<Option<Arc<str>>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

#[derive(Default)]
struct TriggerIdRegistry {
    ids: HashMap<Arc<str>, GlobalElementId>,
}

/// Registers a rendered trigger element id for the given value.
///
/// Recipes can use this to position an indicator or viewport relative to the active trigger.
pub fn navigation_menu_register_trigger_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    value: Arc<str>,
    trigger_id: GlobalElementId,
) {
    cx.with_state_for(root_id, TriggerIdRegistry::default, |st| {
        st.ids.insert(value, trigger_id);
    });
}

/// Returns the last registered trigger element id for a value.
pub fn navigation_menu_trigger_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    value: &str,
) -> Option<GlobalElementId> {
    cx.with_state_for(root_id, TriggerIdRegistry::default, |st| {
        st.ids.get(value).copied()
    })
}

#[derive(Default)]
struct ViewportContentIdRegistry {
    ids: HashMap<Arc<str>, GlobalElementId>,
}

#[derive(Default)]
struct ViewportPanelIdRegistry {
    id: Option<GlobalElementId>,
}

#[derive(Default)]
struct IndicatorTrackIdRegistry {
    id: Option<GlobalElementId>,
}

#[derive(Default)]
struct IndicatorDiamondIdRegistry {
    id: Option<GlobalElementId>,
}
/// Registers the viewport content element id for a given value.
///
/// This mirrors Radix's internal "viewport content map" concept: each content instance is keyed by
/// `value` so other parts (viewport sizing, indicator, focus proxies) can look up the last known
/// element id without reaching into recipe-local state.
pub fn navigation_menu_register_viewport_content_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    value: Arc<str>,
    content_id: GlobalElementId,
) {
    cx.with_state_for(root_id, ViewportContentIdRegistry::default, |st| {
        st.ids.insert(value, content_id);
    });
}

/// Returns the last registered viewport content element id for a value.
pub fn navigation_menu_viewport_content_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    value: &str,
) -> Option<GlobalElementId> {
    cx.with_state_for(root_id, ViewportContentIdRegistry::default, |st| {
        st.ids.get(value).copied()
    })
}

/// Registers the viewport panel element id for the root.
///
/// This mirrors the Radix `NavigationMenuViewport` element: a single panel that hosts the active
/// content and animates its size while opening/closing.
pub fn navigation_menu_register_viewport_panel_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    viewport_panel_id: GlobalElementId,
) {
    cx.with_state_for(root_id, ViewportPanelIdRegistry::default, |st| {
        st.id = Some(viewport_panel_id);
    });
}

/// Returns the last registered viewport panel element id for the root.
pub fn navigation_menu_viewport_panel_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
) -> Option<GlobalElementId> {
    cx.with_state_for(root_id, ViewportPanelIdRegistry::default, |st| st.id)
}

/// Registers the indicator track element id for the root.
///
/// Recipes can use this to associate diagnostics / golden assertions with the rendered indicator.
pub fn navigation_menu_register_indicator_track_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    indicator_track_id: GlobalElementId,
) {
    cx.with_state_for(root_id, IndicatorTrackIdRegistry::default, |st| {
        st.id = Some(indicator_track_id);
    });
}

/// Returns the last registered indicator track element id for the root.
pub fn navigation_menu_indicator_track_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
) -> Option<GlobalElementId> {
    cx.with_state_for(root_id, IndicatorTrackIdRegistry::default, |st| st.id)
}

/// Registers the indicator diamond element id for the root.
pub fn navigation_menu_register_indicator_diamond_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    indicator_diamond_id: GlobalElementId,
) {
    cx.with_state_for(root_id, IndicatorDiamondIdRegistry::default, |st| {
        st.id = Some(indicator_diamond_id);
    });
}

/// Returns the last registered indicator diamond element id for the root.
pub fn navigation_menu_indicator_diamond_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
) -> Option<GlobalElementId> {
    cx.with_state_for(root_id, IndicatorDiamondIdRegistry::default, |st| st.id)
}

fn navigation_menu_viewport_content_semantics_id_in_scope<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &str,
) -> GlobalElementId {
    navigation_menu_viewport_content_pressable_with_id_props::<H>(cx, value, |_cx, _st, _id| {
        (
            fret_ui::element::PressableProps {
                layout: LayoutStyle::default(),
                enabled: true,
                focusable: false,
                ..Default::default()
            },
            Vec::new(),
        )
    })
    .id
}

/// Returns the stable semantics element id for a navigation-menu viewport content.
///
/// This mirrors Radix `NavigationMenuTrigger` / `NavigationMenuContent` behavior where the trigger
/// advertises a `controls` relationship (`aria-controls`) to the content element derived from the
/// root + `value`.
///
/// Callers should use this root-name-scoped helper rather than trying to capture the mounted
/// content id from the overlay subtree: triggers need stable ids even while the viewport is not
/// mounted yet.
pub fn navigation_menu_viewport_content_semantics_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    overlay_root_name: &str,
    value: &str,
) -> GlobalElementId {
    cx.with_root_name(overlay_root_name, |cx| {
        navigation_menu_viewport_content_semantics_id_in_scope::<H>(cx, value)
    })
}

/// Builds the viewport content wrapper using a stable call path keyed by `value`.
///
/// Use this instead of calling `ElementContext::pressable_with_id_props` directly when you need a
/// deterministic content element id (e.g. for trigger `aria-controls` relationships).
pub fn navigation_menu_viewport_content_pressable_with_id_props<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &str,
    f: impl FnOnce(
        &mut ElementContext<'_, H>,
        fret_ui::element::PressableState,
        GlobalElementId,
    ) -> (fret_ui::element::PressableProps, Vec<AnyElement>),
) -> AnyElement {
    cx.keyed(value, |cx| cx.pressable_with_id_props(f))
}

#[derive(Default)]
struct ViewportPresentSelectionState {
    last_present_selected: Option<Arc<str>>,
}

/// Returns a selection value that is stable while a viewport overlay is present.
///
/// Radix keeps the last selected content mounted while closing so that the viewport can animate
/// out without "snapping" to empty. Recipes pass `present=true` while the viewport overlay remains
/// mounted (e.g. during close presence animations).
pub fn navigation_menu_viewport_selected_value<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    selected: Option<Arc<str>>,
    present: bool,
) -> Option<Arc<str>> {
    cx.with_state_for(root_id, ViewportPresentSelectionState::default, |st| {
        if selected.is_some() {
            st.last_present_selected = selected.clone();
            return selected;
        }

        if present {
            return st.last_present_selected.clone();
        }

        None
    })
}

#[derive(Default)]
struct ViewportSizeRegistry {
    sizes: HashMap<Arc<str>, Size>,
    last_size: Option<Size>,
}

/// Registers the last measured viewport size for a given value.
///
/// This is a portable replacement for Radix's viewport CSS vars
/// `--radix-navigation-menu-viewport-{width,height}`: recipes can read these values and animate
/// their own overlay/layout policies accordingly.
pub fn navigation_menu_register_viewport_size<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    value: Arc<str>,
    size: Size,
) {
    cx.with_state_for(root_id, ViewportSizeRegistry::default, |st| {
        st.sizes.insert(value, size);
        st.last_size = Some(size);
    });
}

fn lerp_px(a: fret_core::Px, b: fret_core::Px, t: f32) -> fret_core::Px {
    let t = t.clamp(0.0, 1.0);
    fret_core::Px(a.0 + (b.0 - a.0) * t)
}

fn lerp_size(a: Size, b: Size, t: f32) -> Size {
    Size::new(lerp_px(a.width, b.width, t), lerp_px(a.height, b.height, t))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavigationMenuViewportSizeOutput {
    pub size: Size,
    pub from_size: Option<Size>,
    pub to_size: Option<Size>,
    pub progress: f32,
    pub animating: bool,
}

impl Default for NavigationMenuViewportSizeOutput {
    fn default() -> Self {
        Self {
            size: Size::default(),
            from_size: None,
            to_size: None,
            progress: 1.0,
            animating: false,
        }
    }
}

/// Returns the current viewport size, interpolating between the previous and next content sizes
/// when switching values.
///
/// This models Radix's viewport sizing behavior (CSS vars + CSS transitions) in a recipe-friendly
/// way: it exposes a single `Size` that can be fed into layout solvers or animated wrapper panels.
pub fn navigation_menu_viewport_size_for_transition<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    selected: Option<Arc<str>>,
    values: &[Arc<str>],
    transition: NavigationMenuContentTransitionOutput,
    fallback: Size,
) -> NavigationMenuViewportSizeOutput {
    let (active_size, last_size, from_size, to_size) =
        cx.with_state_for(root_id, ViewportSizeRegistry::default, |st| {
            let active_size = selected
                .as_ref()
                .and_then(|v| st.sizes.get(v).copied())
                .or(st.last_size)
                .unwrap_or(fallback);

            let from_size = transition
                .from_idx
                .and_then(|idx| values.get(idx))
                .and_then(|v| st.sizes.get(v).copied());
            let to_size = transition
                .to_idx
                .and_then(|idx| values.get(idx))
                .and_then(|v| st.sizes.get(v).copied());

            (active_size, st.last_size, from_size, to_size)
        });

    if !transition.switching {
        return NavigationMenuViewportSizeOutput {
            size: active_size,
            from_size: None,
            to_size: None,
            progress: 1.0,
            animating: false,
        };
    }

    let Some(from_idx) = transition.from_idx else {
        return NavigationMenuViewportSizeOutput {
            size: active_size,
            from_size: None,
            to_size: None,
            progress: 1.0,
            animating: false,
        };
    };
    let Some(to_idx) = transition.to_idx else {
        return NavigationMenuViewportSizeOutput {
            size: active_size,
            from_size: None,
            to_size: None,
            progress: 1.0,
            animating: false,
        };
    };
    if from_idx == to_idx {
        return NavigationMenuViewportSizeOutput {
            size: active_size,
            from_size: None,
            to_size: None,
            progress: 1.0,
            animating: false,
        };
    }

    let from_size = from_size.or(to_size).or(last_size).unwrap_or(fallback);
    let to_size = to_size
        .or(Some(from_size))
        .or(last_size)
        .unwrap_or(fallback);

    let progress = transition.progress.clamp(0.0, 1.0);
    let size = if transition.animating {
        lerp_size(from_size, to_size, progress)
    } else {
        to_size
    };

    NavigationMenuViewportSizeOutput {
        size,
        from_size: Some(from_size),
        to_size: Some(to_size),
        progress,
        animating: transition.animating,
    }
}

/// Computes the indicator rect aligned to the currently active trigger and the placed viewport.
///
/// shadcn/ui renders the indicator as a rotated square (diamond) that sits between the trigger row
/// and the viewport panel. Radix computes indicator offset/size via DOM measurement; in Fret we use
/// geometry instead (anchor + placed viewport rects).
pub fn navigation_menu_indicator_rect(
    anchor: Rect,
    viewport_rect: Rect,
    side: Side,
    indicator_thickness: Px,
) -> Rect {
    let thickness = indicator_thickness.0.max(0.0);

    match side {
        // Match Radix/shadcn behavior:
        // - width tracks the active trigger width,
        // - offset tracks the active trigger edge aligned to the viewport panel,
        // - thickness fills the gap between trigger row and viewport panel.
        Side::Bottom => Rect::new(
            Point::new(anchor.origin.x, Px(viewport_rect.origin.y.0 - thickness)),
            Size::new(anchor.size.width, Px(thickness)),
        ),
        Side::Top => Rect::new(
            Point::new(
                anchor.origin.x,
                Px(viewport_rect.origin.y.0 + viewport_rect.size.height.0),
            ),
            Size::new(anchor.size.width, Px(thickness)),
        ),
        // Vertical orientation (rare in shadcn skins): treat thickness as the gutter width and
        // track the active trigger height.
        Side::Right => Rect::new(
            Point::new(Px(viewport_rect.origin.x.0 - thickness), anchor.origin.y),
            Size::new(Px(thickness), anchor.size.height),
        ),
        Side::Left => Rect::new(
            Point::new(
                Px(viewport_rect.origin.x.0 + viewport_rect.size.width.0),
                anchor.origin.y,
            ),
            Size::new(Px(thickness), anchor.size.height),
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavigationMenuViewportOverlayLayout {
    pub anchor: Rect,
    pub placed: Rect,
    pub side: Side,
    pub transform_origin: Point,
    pub indicator_rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavigationMenuViewportOverlayRequestArgs {
    pub window_margin: Px,
    pub placement: popper::PopperContentPlacement,
    /// Optional override for the anchor element used to place the viewport panel.
    ///
    /// When `None`, placement uses the active trigger bounds.
    pub placement_anchor_override: Option<GlobalElementId>,
    pub content_size: Size,
    pub indicator_size: Px,
    /// When `true`, the viewport panel width matches the computed placement anchor width.
    ///
    /// This matches the upstream shadcn/ui mobile behavior (`w-full` on the viewport panel).
    pub width_tracks_anchor: bool,
}

#[derive(Debug, Clone)]
pub struct NavigationMenuViewportOverlayRenderOutput {
    pub opacity: f32,
    pub transform: Transform2D,
    pub children: Vec<AnyElement>,
}

/// Requests a dismissible popover overlay for a navigation menu viewport/indicator pair.
///
/// This is a policy helper: it computes popper placement from the active trigger id, builds an
/// overlay root name, and submits a `dismissible_popover` request. The caller provides the visual
/// children and animation parameters (opacity/transform) so skins can remain in recipe layers.
pub fn navigation_menu_request_viewport_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    value_model: Model<Option<Arc<str>>>,
    open_model: Model<bool>,
    presence: OverlayPresence,
    selected_value: Option<&str>,
    args: NavigationMenuViewportOverlayRequestArgs,
    render: impl FnOnce(
        &mut ElementContext<'_, H>,
        NavigationMenuViewportOverlayLayout,
    ) -> NavigationMenuViewportOverlayRenderOutput,
) -> Option<NavigationMenuViewportOverlayLayout> {
    if !presence.present {
        return None;
    }

    let overlay_root_name = OverlayController::popover_root_name(root_id);
    let mut computed_layout: Option<NavigationMenuViewportOverlayLayout> = None;

    let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
        let Some(value) = selected_value else {
            return Vec::new();
        };
        let trigger_anchor_id = navigation_menu_trigger_id(cx, root_id, value);
        let trigger_anchor = trigger_anchor_id.and_then(|id| {
            cx.last_bounds_for_element(id)
                .or_else(|| cx.last_visual_bounds_for_element(id))
        });
        let Some(trigger_anchor) = trigger_anchor else {
            return Vec::new();
        };

        let placement_anchor = args
            .placement_anchor_override
            .and_then(|id| {
                cx.last_bounds_for_element(id)
                    .or_else(|| cx.last_visual_bounds_for_element(id))
            })
            .map(|override_anchor| match args.placement.side {
                Side::Top | Side::Bottom => Rect::new(
                    Point::new(override_anchor.origin.x, trigger_anchor.origin.y),
                    Size::new(override_anchor.size.width, trigger_anchor.size.height),
                ),
                Side::Left | Side::Right => Rect::new(
                    Point::new(trigger_anchor.origin.x, override_anchor.origin.y),
                    Size::new(trigger_anchor.size.width, override_anchor.size.height),
                ),
            })
            .unwrap_or(trigger_anchor);

        let content_size = if args.width_tracks_anchor {
            Size::new(placement_anchor.size.width, args.content_size.height)
        } else {
            args.content_size
        };

        if std::env::var("FRET_DEBUG_NAV_MENU_OVERLAY").ok().as_deref() == Some("1") {
            eprintln!(
                "nav-menu overlay root={:?} selected={:?} trigger_anchor={:?} override={:?} placement_anchor={:?} content_size={:?} width_tracks_anchor={}",
                root_id,
                selected_value,
                trigger_anchor,
                args.placement_anchor_override.and_then(|id| cx.last_bounds_for_element(id)),
                placement_anchor,
                content_size,
                args.width_tracks_anchor
            );
        }

        let outer = overlay::outer_bounds_with_window_margin(cx.bounds, args.window_margin);
        let popper_layout = popper::popper_content_layout_unclamped(
            outer,
            placement_anchor,
            content_size,
            args.placement,
        );
        let placed = popper_layout.rect;

        if std::env::var("FRET_DEBUG_NAV_MENU_OVERLAY").ok().as_deref() == Some("1") {
            eprintln!(
                "nav-menu overlay outer={:?} window_margin={:?} placed={:?}",
                outer, args.window_margin, placed
            );
        }

        let transform_origin =
            popper::popper_content_transform_origin(&popper_layout, placement_anchor, None);
        let indicator_rect = navigation_menu_indicator_rect(
            trigger_anchor,
            placed,
            popper_layout.side,
            args.indicator_size,
        );

        let layout = NavigationMenuViewportOverlayLayout {
            anchor: trigger_anchor,
            placed,
            side: popper_layout.side,
            transform_origin,
            indicator_rect,
        };
        computed_layout = Some(layout);

        let out = render(cx, layout);

        let overlay_content = crate::declarative::overlay_motion::wrap_opacity_and_render_transform(
            cx,
            out.opacity,
            out.transform,
            out.children,
        );

        vec![overlay_content]
    });

    let open_model_for_request = open_model.clone();
    let open_model_for_dismiss = open_model.clone();
    let mut request = OverlayRequest::dismissible_popover(
        root_id,
        root_id,
        open_model_for_request,
        presence,
        overlay_children,
    );
    request.root_name = Some(overlay_root_name);
    request.dismissible_on_dismiss_request = Some(Arc::new({
        let open_model = open_model_for_dismiss;
        move |host, _cx, _reason| {
            let _ = host.models_mut().update(&open_model, |v| *v = false);
            let _ = host.models_mut().update(&value_model, |v| *v = None);
        }
    }));
    OverlayController::request(cx, request);

    computed_layout
}

/// A composable, Radix-shaped navigation-menu configuration surface.
///
/// This mirrors Radix's exported part names (`Root`, `List`, `Item`, `Trigger`, `Content`, `Link`,
/// `Indicator`, `Viewport`) but models outcomes rather than DOM APIs.
#[derive(Debug, Clone)]
pub struct NavigationMenuRoot {
    model: Model<Option<Arc<str>>>,
    config: NavigationMenuConfig,
    disabled: bool,
}

impl NavigationMenuRoot {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model,
            config: NavigationMenuConfig::default(),
            disabled: false,
        }
    }

    /// Creates a root with a controlled/uncontrolled selection model (Radix `value` /
    /// `defaultValue`).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        controlled: Option<Model<Option<Arc<str>>>>,
        default_value: impl FnOnce() -> Option<Arc<str>>,
    ) -> Self {
        let model = navigation_menu_use_value_model(cx, controlled, default_value).model();
        Self::new(model)
    }

    pub fn model(&self) -> Model<Option<Arc<str>>> {
        self.model.clone()
    }

    pub fn config(mut self, config: NavigationMenuConfig) -> Self {
        self.config = config;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn context<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        root_id: GlobalElementId,
    ) -> NavigationMenuContext {
        let root_state: Arc<Mutex<NavigationMenuRootState>> = cx.with_state_for(
            root_id,
            || Arc::new(Mutex::new(NavigationMenuRootState::default())),
            |s| s.clone(),
        );

        let value_model_for_timer = self.model.clone();
        let root_state_for_timer = root_state.clone();
        let cfg = self.config;
        cx.timer_on_timer_for(
            root_id,
            Arc::new(move |host, action_cx, token| {
                let mut st = root_state_for_timer
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                st.on_timer(host, action_cx, token, &value_model_for_timer, cfg)
            }),
        );

        NavigationMenuContext {
            root_id,
            model: self.model.clone(),
            config: self.config,
            disabled: self.disabled,
            root_state,
        }
    }

    pub fn trigger(&self, value: impl Into<Arc<str>>) -> NavigationMenuTrigger {
        NavigationMenuTrigger::new(value)
    }

    pub fn content(&self, value: impl Into<Arc<str>>) -> NavigationMenuContent {
        NavigationMenuContent::new(value)
    }

    pub fn link(&self) -> NavigationMenuLink {
        NavigationMenuLink::new()
    }
}

#[derive(Clone)]
pub struct NavigationMenuContext {
    pub root_id: GlobalElementId,
    pub model: Model<Option<Arc<str>>>,
    pub config: NavigationMenuConfig,
    pub disabled: bool,
    pub root_state: Arc<Mutex<NavigationMenuRootState>>,
}

impl NavigationMenuContext {
    pub fn selected<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Option<Arc<str>> {
        cx.watch_model(&self.model).layout().cloned().flatten()
    }
}

#[derive(Debug, Clone)]
pub struct NavigationMenuTrigger {
    value: Arc<str>,
    label: Option<Arc<str>>,
    disabled: bool,
}

impl NavigationMenuTrigger {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: None,
            disabled: false,
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Renders a trigger subtree wiring Radix-like hover and open/close behavior.
    ///
    /// This helper is skin-free: pass `PressableProps` and render children in `f`.
    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        ctx: &NavigationMenuContext,
        mut pressable: fret_ui::element::PressableProps,
        pointer_region: fret_ui::element::PointerRegionProps,
        f: impl FnOnce(
            &mut ElementContext<'_, H>,
            fret_ui::element::PressableState,
            bool,
        ) -> Vec<fret_ui::element::AnyElement>,
    ) -> fret_ui::element::AnyElement {
        use fret_core::KeyCode;

        let value_model = ctx.model.clone();
        let item_value = self.value.clone();
        let disabled = ctx.disabled || self.disabled;
        let cfg = ctx.config;
        let root_id = ctx.root_id;
        let root_state = ctx.root_state.clone();

        let is_open = cx
            .watch_model(&value_model)
            .layout()
            .cloned()
            .flatten()
            .as_deref()
            .is_some_and(|v| v == item_value.as_ref());

        let trigger_state: Arc<Mutex<NavigationMenuTriggerState>> = cx.with_state_for(
            cx.root_id(),
            || Arc::new(Mutex::new(NavigationMenuTriggerState::default())),
            |s| s.clone(),
        );

        pressable.enabled = !disabled;
        pressable.focusable = !disabled;

        if pressable.a11y.role.is_none() {
            pressable.a11y.role = Some(fret_core::SemanticsRole::Button);
        }
        if pressable.a11y.label.is_none() {
            pressable.a11y.label = self.label.clone();
        }
        pressable.a11y.expanded = Some(is_open);
        if pressable.a11y.controls_element.is_none() {
            let overlay_root_name = OverlayController::popover_root_name(root_id);
            let content_id = navigation_menu_viewport_content_semantics_id::<H>(
                cx,
                overlay_root_name.as_str(),
                item_value.as_ref(),
            );
            pressable.a11y.controls_element = Some(content_id.0);
        }

        cx.pointer_region(pointer_region, move |cx| {
            if !disabled {
                let trigger_state_for_pointer_move = trigger_state.clone();
                let root_state_for_pointer_move = root_state.clone();
                let value_for_pointer_move = value_model.clone();
                let item_value_for_pointer_move = item_value.clone();
                cx.pointer_region_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                    let mut trigger = trigger_state_for_pointer_move
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    match navigation_menu_trigger_pointer_move_action(
                        mv.pointer_type,
                        disabled,
                        *trigger,
                    ) {
                        NavigationMenuTriggerPointerMoveAction::Ignore => false,
                        NavigationMenuTriggerPointerMoveAction::Open => {
                            let mut root = root_state_for_pointer_move
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            root.on_trigger_enter(
                                host,
                                action_cx,
                                &value_for_pointer_move,
                                item_value_for_pointer_move.clone(),
                                cfg,
                            );
                            trigger.has_pointer_move_opened = true;
                            trigger.was_click_close = false;
                            trigger.was_escape_close = false;
                            false
                        }
                    }
                }));
            }

            let item_value_for_registry = item_value.clone();
            vec![cx.pressable_with_id(pressable, move |cx, st, trigger_id| {
                navigation_menu_register_trigger_id(
                    cx,
                    root_id,
                    item_value_for_registry.clone(),
                    trigger_id,
                );

                if !disabled {
                    let element = trigger_id;
                    let root_state_for_escape = root_state.clone();
                    let value_for_escape = value_model.clone();
                    let trigger_state_for_escape = trigger_state.clone();
                    cx.key_on_key_down_for(
                        element,
                        Arc::new(move |host, action_cx, it| {
                            if it.repeat || it.key != KeyCode::Escape {
                                return false;
                            }

                            let is_open = host
                                .models_mut()
                                .read(&value_for_escape, |v| v.is_some())
                                .ok()
                                .unwrap_or(false);
                            if !is_open {
                                return false;
                            }

                            let mut root = root_state_for_escape
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            root.on_item_dismiss(host, action_cx, &value_for_escape, cfg);

                            let mut trigger = trigger_state_for_escape
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            trigger.was_escape_close = true;
                            trigger.was_click_close = false;
                            trigger.has_pointer_move_opened = false;

                            true
                        }),
                    );

                    let root_state_for_activate = root_state.clone();
                    let value_for_activate = value_model.clone();
                    let trigger_state_for_activate = trigger_state.clone();
                    let item_value_for_activate = item_value.clone();
                    cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                        let mut root = root_state_for_activate
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        root.on_item_select(
                            host,
                            action_cx,
                            &value_for_activate,
                            item_value_for_activate.clone(),
                            cfg,
                        );

                        let now_open = host
                            .models_mut()
                            .read(&value_for_activate, |v| v.clone())
                            .ok()
                            .flatten()
                            .is_some_and(|v| v.as_ref() == item_value_for_activate.as_ref());

                        let mut trigger = trigger_state_for_activate
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        trigger.was_click_close = !now_open;
                        if now_open {
                            trigger.was_escape_close = false;
                        }
                        trigger.has_pointer_move_opened = false;
                    }));

                    let trigger_state_for_hover = trigger_state.clone();
                    let root_state_for_hover = root_state.clone();
                    let value_for_trigger = value_model.clone();
                    cx.pressable_on_hover_change(Arc::new(move |host, action_cx, hovered| {
                        if hovered {
                            return;
                        }
                        let mut trigger = trigger_state_for_hover
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        let mut root = root_state_for_hover
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        root.on_trigger_leave(host, action_cx, &value_for_trigger, cfg);
                        *trigger = NavigationMenuTriggerState::default();
                    }));
                }

                f(cx, st, is_open)
            })]
        })
    }
}

#[derive(Debug, Clone)]
pub struct NavigationMenuContent {
    value: Arc<str>,
    force_mount: bool,
}

impl NavigationMenuContent {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            force_mount: false,
        }
    }

    pub fn force_mount(mut self, force_mount: bool) -> Self {
        self.force_mount = force_mount;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        ctx: &NavigationMenuContext,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<fret_ui::element::AnyElement>,
    ) -> Option<fret_ui::element::AnyElement> {
        let selected = ctx.selected(cx);
        let active = selected.as_deref() == Some(self.value.as_ref());
        if !active && !self.force_mount {
            return None;
        }
        if self.force_mount {
            Some(cx.interactivity_gate(active, active, |cx| f(cx)))
        } else {
            Some(cx.interactivity_gate(true, true, |cx| f(cx)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavigationMenuLink {
    dismiss_on_select: bool,
    dismiss_on_ctrl_or_meta: bool,
}

impl NavigationMenuLink {
    pub fn new() -> Self {
        Self {
            dismiss_on_select: true,
            dismiss_on_ctrl_or_meta: false,
        }
    }

    pub fn dismiss_on_select(mut self, dismiss_on_select: bool) -> Self {
        self.dismiss_on_select = dismiss_on_select;
        self
    }

    /// When `false` (default), link activation with Ctrl/Meta pressed does not dismiss the root.
    ///
    /// This matches Radix's `NavigationMenuLink`: modified clicks are treated like "open in new tab"
    /// and should not close the menu.
    pub fn dismiss_on_ctrl_or_meta(mut self, dismiss_on_ctrl_or_meta: bool) -> Self {
        self.dismiss_on_ctrl_or_meta = dismiss_on_ctrl_or_meta;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        ctx: &NavigationMenuContext,
        mut pressable: fret_ui::element::PressableProps,
        f: impl FnOnce(
            &mut ElementContext<'_, H>,
            fret_ui::element::PressableState,
        ) -> Vec<fret_ui::element::AnyElement>,
    ) -> fret_ui::element::AnyElement {
        #[derive(Default)]
        struct LinkModifierState {
            suppress_dismiss_for_next_activate: bool,
        }

        let disabled = ctx.disabled;
        pressable.enabled = pressable.enabled && !disabled;
        pressable.focusable = pressable.focusable && !disabled;

        let root_state = ctx.root_state.clone();
        let value_model = ctx.model.clone();
        let cfg = ctx.config;
        let dismiss = self.dismiss_on_select;
        let dismiss_on_ctrl_or_meta = self.dismiss_on_ctrl_or_meta;
        cx.pressable(pressable, move |cx, st| {
            if dismiss && !disabled {
                let modifier_state: Arc<Mutex<LinkModifierState>> = cx.with_state_for(
                    cx.root_id(),
                    || Arc::new(Mutex::new(LinkModifierState::default())),
                    |s| s.clone(),
                );
                let modifier_state_for_pointer = modifier_state.clone();
                cx.pressable_add_on_pointer_down(Arc::new(move |_host, _cx, down| {
                    use fret_ui::action::PressablePointerDownResult as R;

                    let suppress = navigation_menu_link_suppresses_dismiss(
                        down.modifiers,
                        dismiss_on_ctrl_or_meta,
                    );
                    let mut st = modifier_state_for_pointer
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    st.suppress_dismiss_for_next_activate = suppress;
                    R::Continue
                }));

                let root_state_for_dismiss = root_state.clone();
                let value_for_dismiss = value_model.clone();
                cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                    let mut st = modifier_state.lock().unwrap_or_else(|e| e.into_inner());
                    let suppress = st.suppress_dismiss_for_next_activate;
                    st.suppress_dismiss_for_next_activate = false;
                    if suppress {
                        return;
                    }

                    let mut root = root_state_for_dismiss
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    root.on_item_dismiss(host, action_cx, &value_for_dismiss, cfg);
                }));
            }
            f(cx, st)
        })
    }
}

fn navigation_menu_link_suppresses_dismiss(
    modifiers: Modifiers,
    dismiss_on_ctrl_or_meta: bool,
) -> bool {
    (modifiers.ctrl || modifiers.meta) && !dismiss_on_ctrl_or_meta
}

fn cancel_timer(host: &mut dyn UiActionHost, token: &mut Option<TimerToken>) {
    if let Some(token) = token.take() {
        host.push_effect(Effect::CancelTimer { token });
    }
}

fn arm_timer(
    host: &mut dyn UiActionHost,
    window: fret_core::AppWindowId,
    after: Duration,
    token_out: &mut Option<TimerToken>,
) -> TimerToken {
    cancel_timer(host, token_out);
    let token = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window: Some(window),
        token,
        after,
        repeat: None,
    });
    *token_out = Some(token);
    token
}

#[derive(Debug, Clone)]
pub struct NavigationMenuRootState {
    open_timer: Option<TimerToken>,
    close_timer: Option<TimerToken>,
    skip_delay_timer: Option<TimerToken>,
    pending_open_value: Option<Arc<str>>,
    is_open_delayed: bool,
}

impl Default for NavigationMenuRootState {
    fn default() -> Self {
        Self {
            open_timer: None,
            close_timer: None,
            skip_delay_timer: None,
            pending_open_value: None,
            is_open_delayed: true,
        }
    }
}

impl NavigationMenuRootState {
    pub fn is_open_delayed(&self) -> bool {
        self.is_open_delayed
    }

    pub fn clear_timers(&mut self, host: &mut dyn UiActionHost) {
        cancel_timer(host, &mut self.open_timer);
        cancel_timer(host, &mut self.close_timer);
        cancel_timer(host, &mut self.skip_delay_timer);
        self.pending_open_value = None;
    }

    fn note_opened(&mut self, host: &mut dyn UiActionHost, cfg: NavigationMenuConfig) {
        cancel_timer(host, &mut self.skip_delay_timer);
        // Radix only skips open delays when `skipDelayDuration > 0`.
        self.is_open_delayed = cfg.skip_delay_duration.is_zero();
    }

    fn note_closed(
        &mut self,
        host: &mut dyn UiActionHost,
        window: fret_core::AppWindowId,
        cfg: NavigationMenuConfig,
    ) {
        cancel_timer(host, &mut self.skip_delay_timer);
        self.is_open_delayed = true;
        if cfg.skip_delay_duration.is_zero() {
            return;
        }
        // Mirror Radix: after `skipDelayDuration`, re-enable delayed opening.
        arm_timer(
            host,
            window,
            cfg.skip_delay_duration,
            &mut self.skip_delay_timer,
        );
        // While the timer is armed we keep `is_open_delayed=false` (immediate-open window).
        self.is_open_delayed = false;
    }

    pub fn on_trigger_enter(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        item_value: Arc<str>,
        cfg: NavigationMenuConfig,
    ) {
        cancel_timer(host, &mut self.open_timer);

        let current = host
            .models_mut()
            .read(value_model, |v| v.clone())
            .ok()
            .flatten();

        // Always cancel close when entering a trigger.
        cancel_timer(host, &mut self.close_timer);

        if !self.is_open_delayed {
            let _ = host
                .models_mut()
                .update(value_model, |v| *v = Some(item_value.clone()));
            self.note_opened(host, cfg);
            host.request_redraw(acx.window);
            return;
        }

        // Delayed open: if the item is already open, just clear the close timer (done above).
        if current.as_deref() == Some(item_value.as_ref()) {
            return;
        }

        self.pending_open_value = Some(item_value);
        arm_timer(host, acx.window, cfg.delay_duration, &mut self.open_timer);
        host.request_redraw(acx.window);
    }

    pub fn on_trigger_leave(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        cfg: NavigationMenuConfig,
    ) {
        cancel_timer(host, &mut self.open_timer);
        self.pending_open_value = None;
        self.start_close_timer(host, acx, value_model, cfg);
    }

    pub fn on_content_enter(&mut self, host: &mut dyn UiActionHost) {
        cancel_timer(host, &mut self.close_timer);
    }

    pub fn on_content_leave(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        cfg: NavigationMenuConfig,
    ) {
        self.start_close_timer(host, acx, value_model, cfg);
    }

    fn start_close_timer(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        cfg: NavigationMenuConfig,
    ) {
        if cfg.close_delay_duration.is_zero() {
            let _ = host.models_mut().update(value_model, |v| *v = None);
            self.note_closed(host, acx.window, cfg);
            host.request_redraw(acx.window);
            return;
        }
        arm_timer(
            host,
            acx.window,
            cfg.close_delay_duration,
            &mut self.close_timer,
        );
        host.request_redraw(acx.window);
    }

    pub fn on_item_select(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        item_value: Arc<str>,
        cfg: NavigationMenuConfig,
    ) {
        cancel_timer(host, &mut self.open_timer);
        cancel_timer(host, &mut self.close_timer);
        self.pending_open_value = None;

        let current = host
            .models_mut()
            .read(value_model, |v| v.clone())
            .ok()
            .flatten();
        if current.as_deref() == Some(item_value.as_ref()) {
            let _ = host.models_mut().update(value_model, |v| *v = None);
            self.note_closed(host, acx.window, cfg);
        } else {
            let _ = host
                .models_mut()
                .update(value_model, |v| *v = Some(item_value.clone()));
            self.note_opened(host, cfg);
        }

        host.request_redraw(acx.window);
    }

    pub fn on_item_dismiss(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        value_model: &Model<Option<Arc<str>>>,
        cfg: NavigationMenuConfig,
    ) {
        cancel_timer(host, &mut self.open_timer);
        cancel_timer(host, &mut self.close_timer);
        self.pending_open_value = None;

        let _ = host.models_mut().update(value_model, |v| *v = None);
        self.note_closed(host, acx.window, cfg);
        host.request_redraw(acx.window);
    }

    /// Handle a timer callback (open/close/skip-delay).
    ///
    /// Returns `true` when it updates state and a redraw should be requested by the caller.
    pub fn on_timer(
        &mut self,
        host: &mut dyn UiActionHost,
        acx: ActionCx,
        token: TimerToken,
        value_model: &Model<Option<Arc<str>>>,
        cfg: NavigationMenuConfig,
    ) -> bool {
        if self.open_timer == Some(token) {
            self.open_timer = None;
            let Some(value) = self.pending_open_value.take() else {
                return false;
            };
            cancel_timer(host, &mut self.close_timer);
            let _ = host.models_mut().update(value_model, |v| *v = Some(value));
            self.note_opened(host, cfg);
            host.request_redraw(acx.window);
            return true;
        }

        if self.close_timer == Some(token) {
            self.close_timer = None;
            let _ = host.models_mut().update(value_model, |v| *v = None);
            self.note_closed(host, acx.window, cfg);
            host.request_redraw(acx.window);
            return true;
        }

        if self.skip_delay_timer == Some(token) {
            self.skip_delay_timer = None;
            self.is_open_delayed = true;
            host.request_redraw(acx.window);
            return true;
        }

        false
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NavigationMenuTriggerState {
    pub has_pointer_move_opened: bool,
    pub was_click_close: bool,
    pub was_escape_close: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationMenuTriggerPointerMoveAction {
    Open,
    Ignore,
}

/// Mirror Radix `NavigationMenuTrigger` hover-open gating.
pub fn navigation_menu_trigger_pointer_move_action(
    pointer_type: PointerType,
    disabled: bool,
    state: NavigationMenuTriggerState,
) -> NavigationMenuTriggerPointerMoveAction {
    match pointer_type {
        PointerType::Touch | PointerType::Pen => NavigationMenuTriggerPointerMoveAction::Ignore,
        PointerType::Mouse | PointerType::Unknown => {
            if disabled
                || state.was_click_close
                || state.was_escape_close
                || state.has_pointer_move_opened
            {
                NavigationMenuTriggerPointerMoveAction::Ignore
            } else {
                NavigationMenuTriggerPointerMoveAction::Open
            }
        }
    }
}

/// Matches Radix `data-motion` values used by `NavigationMenuContent` when switching between
/// values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationMenuContentMotion {
    None,
    FromStart,
    FromEnd,
    ToStart,
    ToEnd,
}

impl NavigationMenuContentMotion {
    pub fn as_str(self) -> &'static str {
        match self {
            NavigationMenuContentMotion::None => "none",
            NavigationMenuContentMotion::FromStart => "from-start",
            NavigationMenuContentMotion::FromEnd => "from-end",
            NavigationMenuContentMotion::ToStart => "to-start",
            NavigationMenuContentMotion::ToEnd => "to-end",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavigationMenuContentTransitionOutput {
    pub from_idx: Option<usize>,
    pub to_idx: Option<usize>,
    pub switching: bool,
    /// Transition progress in `[0, 1]` where `0` is the beginning of the switch and `1` is the end.
    pub progress: f32,
    pub animating: bool,
    pub from_motion: NavigationMenuContentMotion,
    pub to_motion: NavigationMenuContentMotion,
}

/// A convenience wrapper around [`NavigationMenuContentTransitionOutput`] for recipes that need to
/// render "from" + "to" layers during a value switch (shadcn-style `data-motion` transitions).
///
/// This models Radix's behavior of only animating the selected and the previously selected content,
/// avoiding unnecessary mounts and interrupted animations outside of that range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavigationMenuContentSwitchOutput {
    pub from_idx: usize,
    pub to_idx: usize,
    pub progress: f32,
    pub animating: bool,
    pub forward: bool,
    pub from_motion: NavigationMenuContentMotion,
    pub to_motion: NavigationMenuContentMotion,
}

/// Returns a switch output when the content transition is actively animating between two indices.
pub fn navigation_menu_content_switch(
    transition: NavigationMenuContentTransitionOutput,
) -> Option<NavigationMenuContentSwitchOutput> {
    if !transition.switching || !transition.animating {
        return None;
    }
    let (Some(from_idx), Some(to_idx)) = (transition.from_idx, transition.to_idx) else {
        return None;
    };
    if from_idx == to_idx {
        return None;
    }
    Some(NavigationMenuContentSwitchOutput {
        from_idx,
        to_idx,
        progress: transition.progress.clamp(0.0, 1.0),
        animating: transition.animating,
        forward: to_idx > from_idx,
        from_motion: transition.from_motion,
        to_motion: transition.to_motion,
    })
}

impl Default for NavigationMenuContentTransitionOutput {
    fn default() -> Self {
        Self {
            from_idx: None,
            to_idx: None,
            switching: false,
            progress: 1.0,
            animating: false,
            from_motion: NavigationMenuContentMotion::None,
            to_motion: NavigationMenuContentMotion::None,
        }
    }
}

fn content_motion(
    from_idx: usize,
    to_idx: usize,
) -> (NavigationMenuContentMotion, NavigationMenuContentMotion) {
    if from_idx == to_idx {
        return (
            NavigationMenuContentMotion::None,
            NavigationMenuContentMotion::None,
        );
    }

    // Radix/shadcn direction semantics (LTR):
    // - Forward (increasing index): new content slides in from the end (right), old slides out to the start (left).
    // - Backward: new content slides in from the start (left), old slides out to the end (right).
    if to_idx > from_idx {
        (
            NavigationMenuContentMotion::ToStart,
            NavigationMenuContentMotion::FromEnd,
        )
    } else {
        (
            NavigationMenuContentMotion::ToEnd,
            NavigationMenuContentMotion::FromStart,
        )
    }
}

#[derive(Default)]
struct ContentTransitionState {
    last_selected: Option<Arc<str>>,
    last_selected_idx: Option<usize>,
    from_idx: Option<usize>,
    to_idx: Option<usize>,
    seq: u64,
}

#[derive(Default)]
struct ContentTransitionMotionState {
    seq: u64,
    last_app_tick: u64,
    last_frame_tick: u64,
    tick: u64,
    timeline: TransitionTimeline,
    lease: Option<ContinuousFrames>,
    configured_open_ticks: u64,
    configured_close_ticks: u64,
}

/// Drive a Radix-like `data-motion` content transition when switching between two open values.
///
/// This is a reusable substrate for shadcn-style recipes: it exposes the direction semantics
/// (`from-start`/`from-end`/`to-start`/`to-end`) plus a normalized `progress` that callers can map
/// to transforms/opacity.
///
/// Notes:
/// - This helper is skin-free: it does not prescribe distances or easing beyond the function you
///   provide.
/// - It keeps the transition mounted while animating; callers decide how to keep rendering the
///   "from" content while `animating=true`.
pub fn navigation_menu_content_transition_with_durations_and_easing<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: fret_ui::elements::GlobalElementId,
    open: bool,
    selected: Option<Arc<str>>,
    values: &[Arc<str>],
    open_ticks: u64,
    close_ticks: u64,
    ease: fn(f32) -> f32,
) -> NavigationMenuContentTransitionOutput {
    if !open {
        cx.with_state_for(id, ContentTransitionState::default, |st| {
            st.last_selected = None;
            st.last_selected_idx = None;
            st.from_idx = None;
            st.to_idx = None;
        });
        cx.with_state_for(id, ContentTransitionMotionState::default, |st| {
            st.seq = 0;
            st.tick = 0;
            st.timeline = TransitionTimeline::default();
            st.lease = None;
        });
        return NavigationMenuContentTransitionOutput::default();
    }

    let selected_idx = selected
        .as_deref()
        .and_then(|v| values.iter().position(|it| it.as_ref() == v));

    let (seq, from_idx, to_idx) = cx.with_state_for(id, ContentTransitionState::default, |st| {
        let changed = selected.is_some()
            && st.last_selected.is_some()
            && selected != st.last_selected
            && selected_idx.is_some()
            && st.last_selected_idx.is_some();

        if changed {
            st.from_idx = st.last_selected_idx;
            st.to_idx = selected_idx;
            st.seq = st.seq.saturating_add(1);
        }

        st.last_selected = selected.clone();
        st.last_selected_idx = selected_idx;

        (st.seq, st.from_idx, st.to_idx)
    });

    let (Some(from_idx), Some(to_idx)) = (from_idx, to_idx) else {
        return NavigationMenuContentTransitionOutput::default();
    };

    let app_tick = cx.app.tick_id().0;
    let frame_tick = cx.frame_id.0;

    let (out, start_lease, stop_lease) =
        cx.with_state_for(id, ContentTransitionMotionState::default, |st| {
            if st.configured_open_ticks != open_ticks || st.configured_close_ticks != close_ticks {
                st.configured_open_ticks = open_ticks;
                st.configured_close_ticks = close_ticks;
                st.timeline.set_durations(open_ticks, close_ticks);
            }

            if st.seq != seq {
                st.seq = seq;
                st.last_app_tick = app_tick;
                st.last_frame_tick = frame_tick;
                st.tick = 0;
                st.timeline = TransitionTimeline::default();
                st.timeline.set_durations(open_ticks, close_ticks);
            }

            if st.last_frame_tick != frame_tick {
                st.last_frame_tick = frame_tick;
                st.tick = st.tick.saturating_add(1);
            } else if st.last_app_tick != app_tick {
                st.last_app_tick = app_tick;
                st.tick = st.tick.saturating_add(1);
            } else {
                st.tick = st.tick.saturating_add(1);
            }

            let out = st.timeline.update_with_easing(true, st.tick, ease);
            let start_lease = out.animating && st.lease.is_none();
            let stop_lease = !out.animating && st.lease.is_some();
            (out, start_lease, stop_lease)
        });

    if start_lease {
        let lease = cx.begin_continuous_frames();
        cx.with_state_for(id, ContentTransitionMotionState::default, |st| {
            st.lease = Some(lease);
        });
    } else if stop_lease {
        cx.with_state_for(id, ContentTransitionMotionState::default, |st| {
            st.lease = None;
        });

        cx.with_state_for(id, ContentTransitionState::default, |st| {
            st.from_idx = None;
            st.to_idx = None;
        });
    }

    if out.animating {
        cx.request_frame();
    } else {
        // If no continuous-frames lease was acquired (e.g. a 1-tick transition), still clear the
        // switch state immediately.
        cx.with_state_for(id, ContentTransitionState::default, |st| {
            st.from_idx = None;
            st.to_idx = None;
        });
    }

    let (from_motion, to_motion) = content_motion(from_idx, to_idx);
    NavigationMenuContentTransitionOutput {
        from_idx: Some(from_idx),
        to_idx: Some(to_idx),
        switching: true,
        progress: out.progress,
        animating: out.animating,
        from_motion,
        to_motion,
    }
}

/// Convenience wrapper that uses shadcn-style defaults.
pub fn navigation_menu_content_transition<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: fret_ui::elements::GlobalElementId,
    open: bool,
    selected: Option<Arc<str>>,
    values: &[Arc<str>],
) -> NavigationMenuContentTransitionOutput {
    navigation_menu_content_transition_with_durations_and_easing(
        cx,
        id,
        open,
        selected,
        values,
        crate::declarative::overlay_motion::SHADCN_MOTION_TICKS_200,
        crate::declarative::overlay_motion::SHADCN_MOTION_TICKS_200,
        crate::declarative::overlay_motion::shadcn_ease,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::GlobalElementId;
    use fret_ui::action::UiActionHostAdapter;

    fn acx(window: AppWindowId) -> ActionCx {
        ActionCx {
            window,
            target: GlobalElementId(0x1),
        }
    }

    #[test]
    fn trigger_enter_is_delayed_by_default_and_opens_after_timer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let value = app.models_mut().insert(None::<Arc<str>>);
        let mut host = UiActionHostAdapter { app: &mut app };

        let mut st = NavigationMenuRootState::default();
        let cfg = NavigationMenuConfig::default();

        st.on_trigger_enter(&mut host, acx(window), &value, Arc::from("a"), cfg);
        assert_eq!(
            host.models_mut().read(&value, |v| v.clone()).ok().flatten(),
            None
        );

        let effects = host.app.flush_effects();
        let token = effects
            .iter()
            .find_map(|e| match e {
                Effect::SetTimer { token, after, .. } if *after == cfg.delay_duration => {
                    Some(*token)
                }
                _ => None,
            })
            .expect("expected open timer");

        assert!(st.on_timer(&mut host, acx(window), token, &value, cfg));
        assert_eq!(
            host.models_mut()
                .read(&value, |v| v.clone())
                .ok()
                .flatten()
                .as_deref(),
            Some("a")
        );
        assert!(
            !st.is_open_delayed(),
            "expected skip-delay window to be active"
        );
    }

    #[test]
    fn closing_enables_immediate_open_within_skip_delay_window() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let value = app.models_mut().insert(Some(Arc::from("a")));
        let mut host = UiActionHostAdapter { app: &mut app };

        let mut st = NavigationMenuRootState::default();
        let cfg = NavigationMenuConfig::default();

        // Mark as opened (Radix sets isOpenDelayed=false while open).
        st.note_opened(&mut host, cfg);
        assert!(!st.is_open_delayed());

        // Dismiss closes and arms the skip-delay timer, while keeping `is_open_delayed=false`
        // until it fires.
        st.on_item_dismiss(&mut host, acx(window), &value, cfg);
        assert_eq!(
            host.models_mut().read(&value, |v| v.clone()).ok().flatten(),
            None
        );
        assert!(!st.is_open_delayed());

        host.app.flush_effects();

        // Within the skip window: entering a trigger opens immediately (no open timer).
        st.on_trigger_enter(&mut host, acx(window), &value, Arc::from("b"), cfg);
        assert_eq!(
            host.models_mut()
                .read(&value, |v| v.clone())
                .ok()
                .flatten()
                .as_deref(),
            Some("b")
        );
        let effects = host.app.flush_effects();
        assert!(
            effects.iter().all(
                |e| !matches!(e, Effect::SetTimer { after, .. } if *after == cfg.delay_duration)
            ),
            "expected immediate open (no delayed-open timer)"
        );
    }

    #[test]
    fn trigger_leave_starts_close_timer_and_content_enter_cancels_it() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let value = app.models_mut().insert(Some(Arc::from("a")));
        let mut host = UiActionHostAdapter { app: &mut app };

        let mut st = NavigationMenuRootState::default();
        let cfg = NavigationMenuConfig::default();

        st.on_trigger_leave(&mut host, acx(window), &value, cfg);
        let effects = host.app.flush_effects();
        let close_token = effects
            .iter()
            .find_map(|e| match e {
                Effect::SetTimer { token, after, .. } if *after == cfg.close_delay_duration => {
                    Some(*token)
                }
                _ => None,
            })
            .expect("expected close timer");

        // Content enter cancels the close timer.
        st.on_content_enter(&mut host);
        let effects = host.app.flush_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::CancelTimer { token } if *token == close_token)),
            "expected close timer cancellation"
        );
    }

    #[test]
    fn trigger_pointer_move_gate_matches_radix_outcomes() {
        let st = NavigationMenuTriggerState::default();
        assert_eq!(
            navigation_menu_trigger_pointer_move_action(PointerType::Mouse, false, st),
            NavigationMenuTriggerPointerMoveAction::Open
        );
        assert_eq!(
            navigation_menu_trigger_pointer_move_action(PointerType::Touch, false, st),
            NavigationMenuTriggerPointerMoveAction::Ignore
        );

        let st = NavigationMenuTriggerState {
            has_pointer_move_opened: true,
            ..Default::default()
        };
        assert_eq!(
            navigation_menu_trigger_pointer_move_action(PointerType::Mouse, false, st),
            NavigationMenuTriggerPointerMoveAction::Ignore
        );
    }

    #[test]
    fn content_motion_matches_forward_and_backward_semantics() {
        let (from, to) = content_motion(0, 1);
        assert_eq!(from, NavigationMenuContentMotion::ToStart);
        assert_eq!(to, NavigationMenuContentMotion::FromEnd);

        let (from, to) = content_motion(2, 1);
        assert_eq!(from, NavigationMenuContentMotion::ToEnd);
        assert_eq!(to, NavigationMenuContentMotion::FromStart);
    }

    #[test]
    fn content_switch_exposes_from_to_and_direction() {
        let transition = NavigationMenuContentTransitionOutput {
            from_idx: Some(0),
            to_idx: Some(2),
            switching: true,
            progress: 0.25,
            animating: true,
            from_motion: NavigationMenuContentMotion::ToStart,
            to_motion: NavigationMenuContentMotion::FromEnd,
        };
        let out = navigation_menu_content_switch(transition).expect("switch");
        assert_eq!(out.from_idx, 0);
        assert_eq!(out.to_idx, 2);
        assert!(out.forward);
        assert_eq!(out.progress, 0.25);
        assert_eq!(out.from_motion, NavigationMenuContentMotion::ToStart);
        assert_eq!(out.to_motion, NavigationMenuContentMotion::FromEnd);
    }

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn viewport_size_interpolates_between_registered_sizes() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let root_id = cx.root_id();
            let a: Arc<str> = Arc::from("a");
            let b: Arc<str> = Arc::from("b");
            let values = vec![a.clone(), b.clone()];

            navigation_menu_register_viewport_size(
                cx,
                root_id,
                a.clone(),
                Size::new(Px(100.0), Px(50.0)),
            );
            navigation_menu_register_viewport_size(
                cx,
                root_id,
                b.clone(),
                Size::new(Px(200.0), Px(150.0)),
            );

            let transition = NavigationMenuContentTransitionOutput {
                from_idx: Some(0),
                to_idx: Some(1),
                switching: true,
                progress: 0.5,
                animating: true,
                from_motion: NavigationMenuContentMotion::ToStart,
                to_motion: NavigationMenuContentMotion::FromEnd,
            };

            let out = navigation_menu_viewport_size_for_transition(
                cx,
                root_id,
                Some(b.clone()),
                &values,
                transition,
                Size::new(Px(10.0), Px(10.0)),
            );
            assert_eq!(out.size, Size::new(Px(150.0), Px(100.0)));
        });
    }

    #[test]
    fn viewport_selected_value_is_stable_while_present() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let root_id = cx.root_id();

            let a: Arc<str> = Arc::from("a");
            let b: Arc<str> = Arc::from("b");

            assert_eq!(
                navigation_menu_viewport_selected_value(cx, root_id, Some(a.clone()), false)
                    .as_deref(),
                Some("a")
            );
            assert_eq!(
                navigation_menu_viewport_selected_value(cx, root_id, None, true).as_deref(),
                Some("a")
            );
            assert_eq!(
                navigation_menu_viewport_selected_value(cx, root_id, Some(b.clone()), true)
                    .as_deref(),
                Some("b")
            );
            assert_eq!(
                navigation_menu_viewport_selected_value(cx, root_id, None, false).as_deref(),
                None
            );
        });
    }

    #[test]
    fn indicator_rect_tracks_anchor_center_and_viewport_edge() {
        let anchor = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(100.0), Px(40.0)),
        );
        let viewport = Rect::new(
            Point::new(Px(0.0), Px(100.0)),
            Size::new(Px(200.0), Px(80.0)),
        );
        let out = navigation_menu_indicator_rect(anchor, viewport, Side::Bottom, Px(6.0));
        assert_eq!(out.origin.x, Px(10.0));
        assert_eq!(out.origin.y, Px(100.0 - 6.0));
        assert_eq!(out.size, Size::new(Px(100.0), Px(6.0)));
    }

    #[test]
    fn navigation_menu_link_suppresses_dismiss_on_ctrl_or_meta_by_default() {
        assert!(
            navigation_menu_link_suppresses_dismiss(
                Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                false
            ),
            "expected ctrl to suppress dismiss"
        );
        assert!(
            navigation_menu_link_suppresses_dismiss(
                Modifiers {
                    meta: true,
                    ..Default::default()
                },
                false
            ),
            "expected meta to suppress dismiss"
        );
        assert!(
            !navigation_menu_link_suppresses_dismiss(Modifiers::default(), false),
            "expected unmodified to not suppress dismiss"
        );
        assert!(
            !navigation_menu_link_suppresses_dismiss(
                Modifiers {
                    meta: true,
                    ..Default::default()
                },
                true
            ),
            "expected opt-in to allow dismiss on modified select"
        );
    }

    #[test]
    fn viewport_content_semantics_id_matches_mounted_content_id() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let overlay_root_name = "nav-menu-overlay";
            let value = "alpha";
            let expected =
                navigation_menu_viewport_content_semantics_id::<App>(cx, overlay_root_name, value);
            let actual = cx.with_root_name(overlay_root_name, |cx| {
                navigation_menu_viewport_content_pressable_with_id_props::<App>(
                    cx,
                    value,
                    |_cx, _st, _id| {
                        (
                            fret_ui::element::PressableProps {
                                layout: LayoutStyle::default(),
                                enabled: true,
                                focusable: false,
                                ..Default::default()
                            },
                            Vec::new(),
                        )
                    },
                )
                .id
            });
            assert_eq!(expected, actual);
        });
    }
}
