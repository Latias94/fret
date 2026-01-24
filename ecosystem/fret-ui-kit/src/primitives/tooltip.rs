//! Tooltip helpers (Radix `@radix-ui/react-tooltip` outcomes).
//!
//! Radix Tooltip composes three concerns:
//!
//! - Provider-scoped open delay behavior (`TooltipProvider` / "delay group")
//! - Floating placement (`Popper`)
//! - Dismiss / focus policy (handled by per-window overlay infrastructure in Fret)
//!
//! In `fret-ui-kit`, we keep the reusable delay mechanics split into:
//!
//! - `crate::headless::tooltip_delay_group` (pure, deterministic state machine), and
//! - `crate::tooltip_provider` (provider stack service for declarative trees).
//!
//! This module is the Radix-named facade that re-exports the pieces under a single entry point.

pub use crate::headless::tooltip_delay_group::{TooltipDelayGroupConfig, TooltipDelayGroupState};

use std::cmp::Ordering;
use std::sync::Arc;

use fret_core::{Point, PointerType, Px, Rect};
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Invalidation, UiHost};

pub use crate::tooltip_provider::{
    TooltipProviderConfig, current_config, is_pointer_in_transit, last_opened_tooltip, note_closed,
    note_opened_tooltip, open_delay_ticks, open_delay_ticks_with_base, pointer_in_transit_model,
    pointer_transit_geometry_model, set_pointer_in_transit, set_pointer_transit_geometry,
    with_tooltip_provider,
};

pub use crate::primitives::popper::{Align, ArrowOptions, LayoutDirection, Side};

use crate::declarative::ModelWatchExt;
use crate::headless::hover_intent::{HoverIntentConfig, HoverIntentState, HoverIntentUpdate};
use crate::headless::safe_hover;
use crate::primitives::popper;
use crate::primitives::trigger_a11y;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerMoveCx, UiActionHost};
use fret_ui::element::PointerRegionProps;

/// Stamps Radix-like trigger relationships:
/// - `described_by_element` mirrors `aria-describedby` (by element id).
///
/// In Radix Tooltip, the trigger advertises the tooltip content by id. In Fret we model this via
/// a portable element-id relationship that resolves into `SemanticsNode.described_by` when the
/// tooltip content is mounted.
pub fn apply_tooltip_trigger_a11y(
    trigger: AnyElement,
    open: bool,
    tooltip_element: GlobalElementId,
) -> AnyElement {
    trigger_a11y::apply_trigger_described_by(trigger, open.then_some(tooltip_element))
}

/// Stable per-overlay root naming convention for tooltip overlays.
pub fn tooltip_root_name(id: GlobalElementId) -> String {
    OverlayController::tooltip_root_name(id)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TooltipPopperVars {
    pub available_width: Px,
    pub available_height: Px,
    pub trigger_width: Px,
    pub trigger_height: Px,
}

pub fn tooltip_popper_desired_width(outer: Rect, anchor: Rect, min_width: Px) -> Px {
    popper::popper_desired_width(outer, anchor, min_width)
}

/// Compute Radix-like "tooltip popper vars" (`--radix-tooltip-*`) for recipes.
///
/// Upstream Radix re-namespaces these from `@radix-ui/react-popper`:
/// - `--radix-tooltip-content-available-width`
/// - `--radix-tooltip-content-available-height`
/// - `--radix-tooltip-trigger-width`
/// - `--radix-tooltip-trigger-height`
///
/// In Fret, we compute the same concepts as a structured return value so recipes can constrain
/// their content without relying on CSS variables.
pub fn tooltip_popper_vars(
    outer: Rect,
    anchor: Rect,
    min_width: Px,
    placement: popper::PopperContentPlacement,
) -> TooltipPopperVars {
    let metrics =
        popper::popper_available_metrics_for_placement(outer, anchor, min_width, placement);
    TooltipPopperVars {
        available_width: metrics.available_width,
        available_height: metrics.available_height,
        trigger_width: metrics.anchor_width,
        trigger_height: metrics.anchor_height,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TooltipInteractionConfig {
    pub disable_hoverable_content: bool,
    /// Overrides the provider-derived open delay (ticks).
    pub open_delay_ticks_override: Option<u64>,
    /// Overrides the hover-close delay (ticks).
    pub close_delay_ticks_override: Option<u64>,
    /// Pointer safe-hover corridor buffer.
    pub safe_hover_buffer: Px,
}

impl Default for TooltipInteractionConfig {
    fn default() -> Self {
        Self {
            disable_hoverable_content: false,
            open_delay_ticks_override: None,
            close_delay_ticks_override: None,
            safe_hover_buffer: Px(5.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooltipInteractionUpdate {
    pub open: bool,
    pub wants_continuous_ticks: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct TooltipFocusEdgeState {
    was_focused: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct TooltipOpenBroadcastState {
    last_seen_open_token: u64,
}

/// Returns a per-tooltip pointer tracking model stored in element state.
///
/// Tooltip pointer tracking is used to approximate Radix Tooltip's hoverable-content grace area:
/// while open, the tooltip remains open if the pointer lies within a "safe corridor" between the
/// trigger anchor and the floating content bounds.
pub fn tooltip_last_pointer_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Point>> {
    #[derive(Default)]
    struct State {
        model: Option<Model<Option<Point>>>,
    }

    let existing = cx.with_state(State::default, |st| st.model.clone());
    if let Some(model) = existing {
        return model;
    }

    let model = cx.app.models_mut().insert(None);
    cx.with_state(State::default, |st| st.model = Some(model.clone()));
    model
}

#[derive(Clone)]
pub struct TooltipTriggerEventModels {
    pub has_pointer_move_opened: Model<bool>,
    pub pointer_transit_geometry: Model<Option<(Rect, Rect)>>,
    pub suppress_hover_open: Model<bool>,
    pub suppress_focus_open: Model<bool>,
    pub close_requested: Model<bool>,
    pub open: Model<bool>,
}

/// Returns the per-tooltip trigger models used by Radix-aligned tooltip policies.
///
/// Recipes can use these models to:
/// - derive whether hover/focus should be treated as an "open affordance",
/// - request closes via `close_requested` (e.g. on outside press),
/// - keep `open` as an authoritative model so view-cache synthesis can remain stable.
pub fn tooltip_trigger_event_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> TooltipTriggerEventModels {
    #[derive(Default)]
    struct State {
        models: Option<TooltipTriggerEventModels>,
    }

    let existing = cx.with_state(State::default, |st| st.models.clone());
    if let Some(models) = existing {
        return models;
    }

    let models = TooltipTriggerEventModels {
        has_pointer_move_opened: cx.app.models_mut().insert(false),
        pointer_transit_geometry: pointer_transit_geometry_model(cx),
        suppress_hover_open: cx.app.models_mut().insert(false),
        suppress_focus_open: cx.app.models_mut().insert(false),
        close_requested: cx.app.models_mut().insert(false),
        open: cx.app.models_mut().insert(false),
    };

    cx.with_state(State::default, |st| st.models = Some(models.clone()));
    models
}

#[derive(Debug, Default, Clone, Copy)]
struct TooltipTriggerHoverEdgeState {
    was_hovered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooltipTriggerGatedSignals {
    pub trigger_hovered: bool,
    pub trigger_focused: bool,
    pub force_close: bool,
}

/// Applies Radix-aligned "open affordance" gating rules for tooltip triggers.
///
/// This is responsible for:
/// - suppressing immediate re-open after explicit dismiss (outside press / escape),
/// - requiring at least one pointer move (mouse) to allow hover-open,
/// - clearing suppression once the pointer leaves (hover edge) or focus is lost.
pub fn tooltip_trigger_update_gates<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hovered: bool,
    focused: bool,
    models: &TooltipTriggerEventModels,
) -> TooltipTriggerGatedSignals {
    let close_requested = cx
        .watch_model(&models.close_requested)
        .layout()
        .copied()
        .unwrap_or(false);
    let has_pointer_move_opened = cx
        .watch_model(&models.has_pointer_move_opened)
        .layout()
        .copied()
        .unwrap_or(false);
    let suppress_hover_open = cx
        .watch_model(&models.suppress_hover_open)
        .layout()
        .copied()
        .unwrap_or(false);
    let suppress_focus_open = cx
        .watch_model(&models.suppress_focus_open)
        .layout()
        .copied()
        .unwrap_or(false);

    let left_hover = cx.with_state(TooltipTriggerHoverEdgeState::default, |st| {
        let left = st.was_hovered && !hovered;
        st.was_hovered = hovered;
        left
    });

    if left_hover && (has_pointer_move_opened || suppress_hover_open) {
        let _ = cx
            .app
            .models_mut()
            .update(&models.has_pointer_move_opened, |v| *v = false);
        let _ = cx
            .app
            .models_mut()
            .update(&models.suppress_hover_open, |v| *v = false);
    }

    if !focused && suppress_focus_open {
        let _ = cx
            .app
            .models_mut()
            .update(&models.suppress_focus_open, |v| *v = false);
    }

    if close_requested {
        if has_pointer_move_opened && !suppress_hover_open {
            let _ = cx
                .app
                .models_mut()
                .update(&models.suppress_hover_open, |v| *v = true);
        }
        if focused && !suppress_focus_open {
            let _ = cx
                .app
                .models_mut()
                .update(&models.suppress_focus_open, |v| *v = true);
        }
        let _ = cx
            .app
            .models_mut()
            .update(&models.close_requested, |v| *v = false);
    }

    TooltipTriggerGatedSignals {
        trigger_hovered: hovered && has_pointer_move_opened && !suppress_hover_open,
        trigger_focused: focused && !suppress_focus_open,
        force_close: close_requested,
    }
}

/// Installs default Radix-aligned dismiss policies for a tooltip trigger.
///
/// This wires:
/// - pointer-down (close and suppress focus/hover re-open),
/// - activation (close and suppress focus re-open),
/// - Escape keydown (close and suppress focus re-open).
pub fn tooltip_install_default_trigger_dismiss_handlers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger: GlobalElementId,
    models: TooltipTriggerEventModels,
) {
    cx.pressable_add_on_pointer_down_for(
        trigger,
        Arc::new({
            let close_requested = models.close_requested.clone();
            let suppress_focus_open = models.suppress_focus_open.clone();
            let has_pointer_move_opened = models.has_pointer_move_opened.clone();
            let suppress_hover_open = models.suppress_hover_open.clone();
            move |host, acx, down| {
                if down.pointer_type != PointerType::Touch {
                    let _ = host.models_mut().update(&close_requested, |v| *v = true);
                }
                let _ = host
                    .models_mut()
                    .update(&suppress_focus_open, |v| *v = true);
                let gate = host
                    .models_mut()
                    .read(&has_pointer_move_opened, |v| *v)
                    .ok()
                    .unwrap_or(false);
                if gate {
                    let _ = host
                        .models_mut()
                        .update(&suppress_hover_open, |v| *v = true);
                }
                host.request_redraw(acx.window);
                fret_ui::action::PressablePointerDownResult::Continue
            }
        }),
    );

    cx.pressable_add_on_activate_for(
        trigger,
        Arc::new({
            let close_requested = models.close_requested.clone();
            let suppress_focus_open = models.suppress_focus_open.clone();
            move |host, acx, _reason| {
                let _ = host.models_mut().update(&close_requested, |v| *v = true);
                let _ = host
                    .models_mut()
                    .update(&suppress_focus_open, |v| *v = true);
                host.request_redraw(acx.window);
            }
        }),
    );

    cx.key_add_on_key_down_for(
        trigger,
        Arc::new({
            let close_requested = models.close_requested.clone();
            let suppress_focus_open = models.suppress_focus_open.clone();
            move |host, acx, down| {
                if down.repeat || down.key != fret_core::KeyCode::Escape {
                    return false;
                }
                let _ = host.models_mut().update(&close_requested, |v| *v = true);
                let _ = host
                    .models_mut()
                    .update(&suppress_focus_open, |v| *v = true);
                host.request_redraw(acx.window);
                true
            }
        }),
    );
}

/// Wraps a tooltip trigger with a pointer-move gate that enables hover-open.
///
/// This follows Radix Tooltip's "require pointer move" behavior to avoid opening on incidental
/// hover states, and also respects "pointer in transit" while the pointer moves between trigger
/// and content (so other tooltip triggers do not open during the safe corridor).
pub fn tooltip_wrap_trigger_with_pointer_move_open_gate<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger: AnyElement,
    models: TooltipTriggerEventModels,
    pointer_in_transit_buffer: Px,
) -> AnyElement {
    cx.pointer_region(PointerRegionProps::default(), move |cx| {
        cx.pointer_region_on_pointer_move(Arc::new({
            let has_pointer_move_opened = models.has_pointer_move_opened.clone();
            let pointer_transit_geometry = models.pointer_transit_geometry.clone();
            move |host, acx, mv| {
                if mv.pointer_type == PointerType::Touch {
                    return false;
                }

                let geometry = host
                    .models_mut()
                    .read(&pointer_transit_geometry, |v| *v)
                    .ok()
                    .flatten();
                if let Some((anchor, floating)) = geometry
                    && tooltip_pointer_in_transit(
                        mv.position,
                        anchor,
                        floating,
                        pointer_in_transit_buffer,
                    )
                {
                    return false;
                }

                let already = host
                    .models_mut()
                    .read(&has_pointer_move_opened, |v| *v)
                    .ok()
                    .unwrap_or(false);
                if !already {
                    let _ = host
                        .models_mut()
                        .update(&has_pointer_move_opened, |v| *v = true);
                    host.request_redraw(acx.window);
                }
                false
            }
        }));

        vec![trigger]
    })
}

fn tooltip_floating_side(anchor: Rect, floating: Rect) -> Option<fret_ui::overlay_placement::Side> {
    let anchor_left = anchor.origin.x.0;
    let anchor_right = anchor_left + anchor.size.width.0;
    let anchor_top = anchor.origin.y.0;
    let anchor_bottom = anchor_top + anchor.size.height.0;

    let floating_left = floating.origin.x.0;
    let floating_right = floating_left + floating.size.width.0;
    let floating_top = floating.origin.y.0;
    let floating_bottom = floating_top + floating.size.height.0;

    if floating_left >= anchor_right {
        return Some(fret_ui::overlay_placement::Side::Right);
    }
    if floating_right <= anchor_left {
        return Some(fret_ui::overlay_placement::Side::Left);
    }
    if floating_bottom <= anchor_top {
        return Some(fret_ui::overlay_placement::Side::Top);
    }
    if floating_top >= anchor_bottom {
        return Some(fret_ui::overlay_placement::Side::Bottom);
    }
    None
}

/// Returns `true` when the pointer should be considered "in transit" between the tooltip trigger
/// and content (Radix `isPointerInTransitRef` outcome).
///
/// Notes:
/// - This is a geometry-only approximation: it uses the safe-hover corridor between `anchor` and
///   `floating`.
/// - Unlike menus, Radix Tooltip's in-transit concept is used to *suppress other tooltip trigger
///   opens* while the pointer moves from trigger to content.
pub fn tooltip_pointer_in_transit(
    position: Point,
    anchor: Rect,
    floating: Rect,
    buffer: Px,
) -> bool {
    if anchor.contains(position) || floating.contains(position) {
        return false;
    }

    let Some(exit_side) = tooltip_floating_side(anchor, floating) else {
        return false;
    };

    let exit_point = tooltip_project_exit_point(anchor, position, exit_side);
    let padding = buffer;
    let exit_points = tooltip_padded_exit_points(exit_point, exit_side, padding);

    let mut points = Vec::with_capacity(6);
    points.extend_from_slice(&exit_points);
    points.extend_from_slice(&tooltip_rect_points(floating));

    let hull = tooltip_convex_hull(&mut points);
    tooltip_point_in_polygon(position, &hull)
}

fn tooltip_rect_points(rect: Rect) -> [Point; 4] {
    let left = rect.origin.x;
    let top = rect.origin.y;
    let right = rect.origin.x + rect.size.width;
    let bottom = rect.origin.y + rect.size.height;
    [
        Point::new(left, top),
        Point::new(right, top),
        Point::new(right, bottom),
        Point::new(left, bottom),
    ]
}

fn tooltip_clamp(v: Px, min: Px, max: Px) -> Px {
    Px(v.0.max(min.0).min(max.0))
}

fn tooltip_project_exit_point(
    anchor: Rect,
    position: Point,
    side: fret_ui::overlay_placement::Side,
) -> Point {
    let left = anchor.origin.x;
    let top = anchor.origin.y;
    let right = anchor.origin.x + anchor.size.width;
    let bottom = anchor.origin.y + anchor.size.height;

    match side {
        fret_ui::overlay_placement::Side::Right => {
            Point::new(right, tooltip_clamp(position.y, top, bottom))
        }
        fret_ui::overlay_placement::Side::Left => {
            Point::new(left, tooltip_clamp(position.y, top, bottom))
        }
        fret_ui::overlay_placement::Side::Top => {
            Point::new(tooltip_clamp(position.x, left, right), top)
        }
        fret_ui::overlay_placement::Side::Bottom => {
            Point::new(tooltip_clamp(position.x, left, right), bottom)
        }
    }
}

fn tooltip_padded_exit_points(
    exit_point: Point,
    side: fret_ui::overlay_placement::Side,
    padding: Px,
) -> [Point; 2] {
    match side {
        fret_ui::overlay_placement::Side::Top => [
            Point::new(exit_point.x - padding, exit_point.y + padding),
            Point::new(exit_point.x + padding, exit_point.y + padding),
        ],
        fret_ui::overlay_placement::Side::Bottom => [
            Point::new(exit_point.x - padding, exit_point.y - padding),
            Point::new(exit_point.x + padding, exit_point.y - padding),
        ],
        fret_ui::overlay_placement::Side::Left => [
            Point::new(exit_point.x + padding, exit_point.y - padding),
            Point::new(exit_point.x + padding, exit_point.y + padding),
        ],
        fret_ui::overlay_placement::Side::Right => [
            Point::new(exit_point.x - padding, exit_point.y - padding),
            Point::new(exit_point.x - padding, exit_point.y + padding),
        ],
    }
}

fn tooltip_point_in_polygon(point: Point, polygon: &[Point]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let x = point.x.0;
    let y = point.y.0;
    let mut inside = false;
    let mut j = polygon.len() - 1;

    for i in 0..polygon.len() {
        let xi = polygon[i].x.0;
        let yi = polygon[i].y.0;
        let xj = polygon[j].x.0;
        let yj = polygon[j].y.0;

        let intersect = (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi;
        if intersect {
            inside = !inside;
        }
        j = i;
    }

    inside
}

fn tooltip_cross(o: Point, a: Point, b: Point) -> f32 {
    (a.x.0 - o.x.0) * (b.y.0 - o.y.0) - (a.y.0 - o.y.0) * (b.x.0 - o.x.0)
}

fn tooltip_convex_hull(points: &mut [Point]) -> Vec<Point> {
    if points.len() <= 1 {
        return points.to_vec();
    }

    points.sort_by(|a, b| {
        a.x.0
            .partial_cmp(&b.x.0)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.y.0.partial_cmp(&b.y.0).unwrap_or(Ordering::Equal))
    });

    let mut lower: Vec<Point> = Vec::new();
    for &p in points.iter() {
        while lower.len() >= 2 {
            let len = lower.len();
            if tooltip_cross(lower[len - 2], lower[len - 1], p) <= 0.0 {
                lower.pop();
            } else {
                break;
            }
        }
        lower.push(p);
    }
    lower.pop();

    let mut upper: Vec<Point> = Vec::new();
    for &p in points.iter().rev() {
        while upper.len() >= 2 {
            let len = upper.len();
            if tooltip_cross(upper[len - 2], upper[len - 1], p) <= 0.0 {
                upper.pop();
            } else {
                break;
            }
        }
        upper.push(p);
    }
    upper.pop();

    if lower.len() == 1
        && upper.len() == 1
        && lower[0].x.0 == upper[0].x.0
        && lower[0].y.0 == upper[0].y.0
    {
        lower
    } else {
        lower.into_iter().chain(upper).collect()
    }
}

/// Installs a pointer-move observer that updates the tooltip's pointer tracking model.
///
/// Notes:
/// - This is intended for hoverable-content tooltips. When `disable_hoverable_content=true`, the
///   recipe should skip installing this observer.
/// - Touch pointers are ignored to match Radix Tooltip's "no hover on touch" behavior.
pub fn tooltip_install_pointer_move_tracker(
    request: &mut OverlayRequest,
    last_pointer: Model<Option<Point>>,
) {
    let last_pointer = last_pointer.clone();
    request.dismissible_on_pointer_move = Some(Arc::new(
        move |host: &mut dyn UiActionHost, _acx: ActionCx, mv: PointerMoveCx| {
            if mv.pointer_type == PointerType::Touch {
                return false;
            }
            let _ = host
                .models_mut()
                .update(&last_pointer, |v| *v = Some(mv.position));
            false
        },
    ));
}

/// Updates an internal hover-intent state machine using Radix-aligned tooltip timing rules.
///
/// This is a reusable policy helper for recipes:
/// - `open_delay_ticks` comes from the provider delay group, unless overridden.
/// - Blur closes immediately.
/// - Hoverable-content tooltips remain open while the pointer stays inside a safe-hover corridor
///   between `anchor_bounds` and `floating_bounds`.
pub fn tooltip_update_interaction<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_hovered: bool,
    trigger_focused: bool,
    force_close: bool,
    last_pointer: Model<Option<Point>>,
    anchor_bounds: Option<Rect>,
    floating_bounds: Option<Rect>,
    cfg: TooltipInteractionConfig,
) -> TooltipInteractionUpdate {
    let tooltip_id = cx.root_id();
    let (last_id, token) = last_opened_tooltip(cx).unwrap_or((tooltip_id, 0));
    let should_close_because_other_opened =
        cx.with_state(TooltipOpenBroadcastState::default, |st| {
            let should_close = token > st.last_seen_open_token && last_id != tooltip_id;
            st.last_seen_open_token = token;
            should_close
        });
    if should_close_because_other_opened {
        cx.with_state(HoverIntentState::default, |st| st.set_open(false));
    }

    let now = cx.app.frame_id().0;

    let was_open = cx.with_state(HoverIntentState::default, |st| st.is_open());

    let (close_delay_ticks, blurred) = cx.with_state(TooltipFocusEdgeState::default, |st| {
        let was = st.was_focused;
        st.was_focused = trigger_focused;
        let blurred = was && !trigger_focused;

        let close_delay_ticks = if blurred {
            0
        } else if trigger_focused {
            0
        } else {
            cfg.close_delay_ticks_override.unwrap_or(0)
        };

        (close_delay_ticks, blurred)
    });

    let open_delay_ticks = if trigger_focused {
        0
    } else if let Some(base_delay) = cfg.open_delay_ticks_override {
        open_delay_ticks_with_base(cx, now, base_delay)
    } else {
        open_delay_ticks(cx, now)
    };

    let intent_cfg = HoverIntentConfig::new(open_delay_ticks, close_delay_ticks);

    if force_close {
        cx.with_state(HoverIntentState::default, |st| st.set_open(false));
        if was_open {
            note_closed(cx, now);
        }
        if was_open {
            set_pointer_in_transit(cx, false);
            set_pointer_transit_geometry(cx, None);
        }
        return TooltipInteractionUpdate {
            open: false,
            wants_continuous_ticks: false,
        };
    }

    if was_open && !cfg.disable_hoverable_content && !blurred {
        cx.observe_model(&last_pointer, Invalidation::Paint);
    }

    let pointer_safe = if was_open && !cfg.disable_hoverable_content && !blurred {
        let pointer = cx.app.models().read(&last_pointer, |v| *v).ok().flatten();
        match (pointer, anchor_bounds, floating_bounds) {
            (Some(pointer), Some(anchor), Some(floating)) => {
                safe_hover::safe_hover_contains(pointer, anchor, floating, cfg.safe_hover_buffer)
            }
            _ => false,
        }
    } else {
        false
    };

    let HoverIntentUpdate {
        open,
        wants_continuous_ticks,
    } = cx.with_state(HoverIntentState::default, |st| {
        st.update(
            trigger_hovered || trigger_focused || pointer_safe,
            now,
            intent_cfg,
        )
    });

    if !was_open && open {
        let token = note_opened_tooltip(cx, tooltip_id);
        cx.with_state(TooltipOpenBroadcastState::default, |st| {
            st.last_seen_open_token = token;
        });
    }

    if was_open && !open {
        note_closed(cx, now);
    }

    if open {
        set_pointer_in_transit(cx, pointer_safe);
        if cfg.disable_hoverable_content || blurred {
            set_pointer_transit_geometry(cx, None);
        } else {
            match (anchor_bounds, floating_bounds) {
                (Some(anchor), Some(floating)) => {
                    set_pointer_transit_geometry(cx, Some((anchor, floating)));
                }
                _ => set_pointer_transit_geometry(cx, None),
            }
        }
    } else if was_open {
        set_pointer_in_transit(cx, false);
        set_pointer_transit_geometry(cx, None);
    }

    TooltipInteractionUpdate {
        open,
        wants_continuous_ticks,
    }
}

/// A Radix-shaped `Tooltip` root configuration surface (open state only).
///
/// Radix Tooltip supports a controlled/uncontrolled `open` state (`open` + `defaultOpen`). In
/// Fret, this root helper standardizes how recipes derive the open model before applying hover
/// intent or provider delay-group policy.
#[derive(Debug, Clone, Default)]
pub struct TooltipRoot {
    open: Option<Model<bool>>,
    default_open: bool,
}

impl TooltipRoot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the controlled `open` model (`Some`) or selects uncontrolled mode (`None`).
    pub fn open(mut self, open: Option<Model<bool>>) -> Self {
        self.open = open;
        self
    }

    /// Sets the uncontrolled initial open value (Radix `defaultOpen`).
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    /// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
    pub fn use_open_model<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
    ) -> crate::primitives::controllable_state::ControllableModel<bool> {
        tooltip_use_open_model(cx, self.open.clone(), || self.default_open)
    }

    pub fn open_model<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Model<bool> {
        self.use_open_model(cx).model()
    }

    /// Reads the current open value from the derived open model.
    pub fn is_open<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> bool {
        let open_model = self.open_model(cx);
        cx.watch_model(&open_model)
            .layout()
            .copied()
            .unwrap_or(false)
    }
}

/// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
///
/// This is a convenience helper for authoring Radix-shaped tooltip roots:
/// - if `controlled_open` is provided, it is used directly
/// - otherwise an internal model is created (once) using `default_open` (Radix `defaultOpen`)
pub fn tooltip_use_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled_open: Option<Model<bool>>,
    default_open: impl FnOnce() -> bool,
) -> crate::primitives::controllable_state::ControllableModel<bool> {
    crate::primitives::open_state::open_use_model(cx, controlled_open, default_open)
}

/// Builds an overlay request for a Radix-style tooltip.
pub fn tooltip_request(
    id: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
) -> OverlayRequest {
    let mut request = OverlayRequest::tooltip(id, open, presence, children);
    request.root_name = Some(tooltip_root_name(id));
    request
}

/// Requests a tooltip overlay for the current window.
pub fn request_tooltip<H: UiHost>(cx: &mut ElementContext<'_, H>, request: OverlayRequest) {
    OverlayController::request(cx, request);
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::Point as CorePoint;
    use fret_core::Size;
    use fret_ui::element::{ElementKind, LayoutStyle, PressableProps, SemanticsProps};

    #[test]
    fn tooltip_root_open_model_uses_controlled_model() {
        let window = Default::default();
        let mut app = App::new();

        let controlled = app.models_mut().insert(true);
        fret_ui::elements::with_element_cx(&mut app, window, Default::default(), "test", |cx| {
            let root = TooltipRoot::new()
                .open(Some(controlled.clone()))
                .default_open(false);
            assert_eq!(root.open_model(cx), controlled);
        });
    }

    #[test]
    fn tooltip_request_sets_default_root_name() {
        let mut app = App::new();
        let open = app.models_mut().insert(true);
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            move |_cx| {
                let id = GlobalElementId(0x123);
                let req =
                    tooltip_request(id, open.clone(), OverlayPresence::instant(true), Vec::new());
                let expected = tooltip_root_name(id);
                assert_eq!(req.root_name.as_deref(), Some(expected.as_str()));
            },
        );
    }

    #[test]
    fn apply_tooltip_trigger_a11y_sets_described_by_on_pressable() {
        let window = Default::default();
        let mut app = App::new();
        fret_ui::elements::with_element_cx(&mut app, window, Default::default(), "test", |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    layout: LayoutStyle::default(),
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );
            let tooltip = GlobalElementId(0xbeef);
            let trigger = apply_tooltip_trigger_a11y(trigger, true, tooltip);
            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable");
            };
            assert_eq!(a11y.described_by_element, Some(tooltip.0));
        });
    }

    #[test]
    fn apply_tooltip_trigger_a11y_sets_described_by_on_semantics() {
        let window = Default::default();
        let mut app = App::new();
        fret_ui::elements::with_element_cx(&mut app, window, Default::default(), "test", |cx| {
            let trigger = cx.semantics(SemanticsProps::default(), |_cx| Vec::new());
            let tooltip = GlobalElementId(0xbeef);
            let trigger = apply_tooltip_trigger_a11y(trigger, true, tooltip);
            let ElementKind::Semantics(props) = &trigger.kind else {
                panic!("expected semantics");
            };
            assert_eq!(props.described_by_element, Some(tooltip.0));
        });
    }

    #[test]
    fn apply_tooltip_trigger_a11y_clears_described_by_when_closed() {
        let window = Default::default();
        let mut app = App::new();
        fret_ui::elements::with_element_cx(&mut app, window, Default::default(), "test", |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    layout: LayoutStyle::default(),
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );
            let tooltip = GlobalElementId(0xbeef);
            let trigger = apply_tooltip_trigger_a11y(trigger, false, tooltip);
            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable");
            };
            assert_eq!(a11y.described_by_element, None);
        });
    }

    #[test]
    fn tooltip_popper_vars_available_height_tracks_flipped_side_space() {
        let outer = Rect::new(
            CorePoint::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let anchor = Rect::new(
            CorePoint::new(Px(10.0), Px(70.0)),
            Size::new(Px(30.0), Px(10.0)),
        );

        let placement = popper::PopperContentPlacement::new(
            popper::LayoutDirection::Ltr,
            popper::Side::Bottom,
            popper::Align::Start,
            Px(0.0),
        );
        let vars = tooltip_popper_vars(outer, anchor, Px(0.0), placement);
        assert!(vars.available_height.0 > 60.0 && vars.available_height.0 < 80.0);
    }
}
