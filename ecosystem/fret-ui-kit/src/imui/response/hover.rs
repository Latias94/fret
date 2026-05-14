use fret_authoring::Response;
use fret_core::{Modifiers, Point};
use fret_ui::GlobalElementId;

use super::drag::DragResponse;

/// ImGui-style hovered query flags for `ResponseExt` convenience helpers.
///
/// This is a facade-level surface intended to keep `fret-authoring::Response` minimal/stable while
/// still allowing editor-grade hover policies (e.g. tooltip hover over disabled items).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ImUiHoveredFlags(u32);

impl ImUiHoveredFlags {
    pub const NONE: Self = Self(0);

    /// Return true even when the item is disabled.
    pub const ALLOW_WHEN_DISABLED: Self = Self(1 << 0);

    /// Return true even when a popup/modal barrier is blocking underlay hit-testing.
    ///
    /// This maps to ImGui's `ImGuiHoveredFlags_AllowWhenBlockedByPopup` in the common case where a
    /// modal barrier is active but the pointer is not currently over any active (non-blocked)
    /// layer.
    pub const ALLOW_WHEN_BLOCKED_BY_POPUP: Self = Self(1 << 1);

    /// Disable nav-highlight participation in hovered queries; always query pointer hover.
    pub const NO_NAV_OVERRIDE: Self = Self(1 << 2);

    /// Tooltip-style hover query preset (ImGui `ForTooltip`).
    ///
    /// This is a convenience shorthand that expands to:
    /// - `STATIONARY`
    /// - `DELAY_SHORT`
    /// - `ALLOW_WHEN_DISABLED`
    pub const FOR_TOOLTIP: Self = Self(1 << 3);

    /// Require a short stationary dwell before reporting hovered.
    pub const STATIONARY: Self = Self(1 << 4);

    /// Return true immediately (default).
    pub const DELAY_NONE: Self = Self(1 << 5);

    /// Return true after a short delay (ImGui-style, ~150ms by default).
    pub const DELAY_SHORT: Self = Self(1 << 6);

    /// Return true after a normal delay (ImGui-style, ~400ms by default).
    pub const DELAY_NORMAL: Self = Self(1 << 7);

    /// Disable the "shared delay" behavior between adjacent hovered items.
    /// (ImGui-style).
    ///
    /// This is best-effort and applies to pointer hover only (nav-tooltip delay parity is not
    /// implemented).
    pub const NO_SHARED_DELAY: Self = Self(1 << 8);

    /// Return true even when another item is active (e.g. while dragging an item).
    ///
    /// This is intended to model ImGui's `ImGuiHoveredFlags_AllowWhenBlockedByActiveItem`.
    pub const ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM: Self = Self(1 << 9);

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for ImUiHoveredFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ImUiHoveredFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// A richer interaction result intended for immediate-mode facade helpers.
///
/// This is a ui-kit-level convenience wrapper: it extends the minimal `fret-authoring::Response`
/// contract with additional commonly requested signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResponseExt {
    pub core: Response,
    pub id: Option<GlobalElementId>,
    enabled: bool,
    /// True on the frame an item enters its active or engaged state.
    activated: bool,
    /// True on the frame an item leaves its active or engaged state.
    deactivated: bool,
    /// True on the frame an item commits a meaningful value mutation.
    edited: bool,
    /// True on the frame an item deactivates after having been edited during the same active
    /// session.
    deactivated_after_edit: bool,
    /// Pointer-hover signal without ImGui-style disabled gating.
    ///
    /// When a widget is disabled, `core.hovered` is forced to `false` by
    /// `sanitize_response_for_enabled(...)`.
    /// This field can still carry the raw pointer-hover signal for query helpers like
    /// `is_hovered(ImUiHoveredFlags::ALLOW_WHEN_DISABLED)`.
    pointer_hovered_raw: bool,
    /// Pointer-hover signal available even when popup policy blocks/suppresses hover (best-effort).
    ///
    /// This is primarily intended to support ImGui's `AllowWhenBlockedByPopup` hovered query flag.
    pointer_hovered_raw_below_barrier: bool,
    /// True once the "stationary" dwell timer has elapsed while hovered (best-effort).
    hover_stationary_met: bool,
    /// True once the short hover delay has elapsed while hovered.
    hover_delay_short_met: bool,
    /// True once the normal hover delay has elapsed while hovered.
    hover_delay_normal_met: bool,
    /// True once the short hover delay has elapsed (shared window-scoped timer, best-effort).
    hover_delay_short_shared_met: bool,
    /// True once the normal hover delay has elapsed (shared window-scoped timer, best-effort).
    hover_delay_normal_shared_met: bool,
    /// True when ImGui-style hover queries should be suppressed because another item is active.
    ///
    /// This is a facade-level policy knob intended to mirror `IsItemHovered()` behavior where
    /// hovered queries are suppressed while dragging another item, unless explicitly overridden
    /// with `ImUiHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM`.
    hover_blocked_by_active_item: bool,
    /// True when the item is focused and the window's focus-visible policy indicates keyboard
    /// navigation is active.
    ///
    /// This is intended as an immediate-mode equivalent of ImGui's "nav highlight under nav"
    /// behavior used by `IsItemHovered()` when `NavHighlightItemUnderNav` is active.
    nav_highlighted: bool,
    secondary_clicked: bool,
    double_clicked: bool,
    long_pressed: bool,
    press_holding: bool,
    context_menu_requested: bool,
    context_menu_anchor: Option<Point>,
    /// True when `clicked` was produced by a pointer click rather than keyboard activation.
    pointer_clicked: bool,
    /// Best-effort modifier snapshot for the pointer click that produced `clicked`.
    ///
    /// Consumers should read this through `pointer_click_modifiers()` so keyboard activations map
    /// to `None`.
    pointer_click_modifiers: Modifiers,
    pub(crate) drag: DragResponse,
}

impl ResponseExt {
    pub(crate) fn drag_mut(&mut self) -> &mut DragResponse {
        &mut self.drag
    }

    pub fn drag(self) -> DragResponse {
        self.drag
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub(crate) fn set_activated(&mut self, activated: bool) {
        self.activated = activated;
    }

    pub(crate) fn set_deactivated(&mut self, deactivated: bool) {
        self.deactivated = deactivated;
    }

    pub(crate) fn set_edited(&mut self, edited: bool) {
        self.edited = edited;
    }

    pub(crate) fn set_deactivated_after_edit(&mut self, deactivated_after_edit: bool) {
        self.deactivated_after_edit = deactivated_after_edit;
    }

    pub(crate) fn merge_activated(&mut self, activated: bool) {
        self.activated |= activated;
    }

    pub(crate) fn merge_deactivated(&mut self, deactivated: bool) {
        self.deactivated |= deactivated;
    }

    pub(crate) fn merge_edited(&mut self, edited: bool) {
        self.edited |= edited;
    }

    pub(crate) fn merge_deactivated_after_edit(&mut self, deactivated_after_edit: bool) {
        self.deactivated_after_edit |= deactivated_after_edit;
    }

    pub(crate) fn clear_lifecycle_signals(&mut self) {
        self.activated = false;
        self.deactivated = false;
        self.edited = false;
        self.deactivated_after_edit = false;
    }

    pub(crate) fn set_pointer_hovered_raw(&mut self, pointer_hovered_raw: bool) {
        self.pointer_hovered_raw = pointer_hovered_raw;
    }

    pub(crate) fn set_pointer_hovered_raw_below_barrier(
        &mut self,
        pointer_hovered_raw_below_barrier: bool,
    ) {
        self.pointer_hovered_raw_below_barrier = pointer_hovered_raw_below_barrier;
    }

    pub(crate) fn set_hover_stationary_met(&mut self, hover_stationary_met: bool) {
        self.hover_stationary_met = hover_stationary_met;
    }

    pub(crate) fn set_hover_delay_short_met(&mut self, hover_delay_short_met: bool) {
        self.hover_delay_short_met = hover_delay_short_met;
    }

    pub(crate) fn set_hover_delay_normal_met(&mut self, hover_delay_normal_met: bool) {
        self.hover_delay_normal_met = hover_delay_normal_met;
    }

    pub(crate) fn set_hover_delay_short_shared_met(&mut self, hover_delay_short_shared_met: bool) {
        self.hover_delay_short_shared_met = hover_delay_short_shared_met;
    }

    pub(crate) fn set_hover_delay_normal_shared_met(
        &mut self,
        hover_delay_normal_shared_met: bool,
    ) {
        self.hover_delay_normal_shared_met = hover_delay_normal_shared_met;
    }

    pub(crate) fn set_hover_blocked_by_active_item(&mut self, hover_blocked_by_active_item: bool) {
        self.hover_blocked_by_active_item = hover_blocked_by_active_item;
    }

    pub(crate) fn set_nav_highlighted(&mut self, nav_highlighted: bool) {
        self.nav_highlighted = nav_highlighted;
    }

    pub(crate) fn set_secondary_clicked(&mut self, secondary_clicked: bool) {
        self.secondary_clicked = secondary_clicked;
    }

    pub(crate) fn set_double_clicked(&mut self, double_clicked: bool) {
        self.double_clicked = double_clicked;
    }

    pub(crate) fn set_long_pressed(&mut self, long_pressed: bool) {
        self.long_pressed = long_pressed;
    }

    pub(crate) fn set_press_holding(&mut self, press_holding: bool) {
        self.press_holding = press_holding;
    }

    pub(crate) fn set_context_menu_requested(&mut self, context_menu_requested: bool) {
        self.context_menu_requested = context_menu_requested;
    }

    pub(crate) fn set_context_menu_anchor(&mut self, context_menu_anchor: Option<Point>) {
        self.context_menu_anchor = context_menu_anchor;
    }

    pub(crate) fn set_pointer_clicked(&mut self, pointer_clicked: bool) {
        self.pointer_clicked = pointer_clicked;
    }

    pub(crate) fn set_pointer_click_modifiers(&mut self, pointer_click_modifiers: Modifiers) {
        self.pointer_click_modifiers = pointer_click_modifiers;
    }

    pub(crate) fn clear_press_context_signals(&mut self) {
        self.secondary_clicked = false;
        self.double_clicked = false;
        self.long_pressed = false;
        self.press_holding = false;
        self.context_menu_requested = false;
        self.context_menu_anchor = None;
        self.pointer_clicked = false;
        self.pointer_click_modifiers = Modifiers::default();
    }

    pub fn activated(self) -> bool {
        self.activated
    }

    pub fn deactivated(self) -> bool {
        self.deactivated
    }

    pub fn edited(self) -> bool {
        self.edited
    }

    pub fn deactivated_after_edit(self) -> bool {
        self.deactivated_after_edit
    }

    pub fn clicked(self) -> bool {
        self.core.clicked()
    }

    pub fn changed(self) -> bool {
        self.core.changed()
    }

    pub fn enabled(self) -> bool {
        self.enabled
    }

    pub fn secondary_clicked(self) -> bool {
        self.secondary_clicked
    }

    pub fn double_clicked(self) -> bool {
        self.double_clicked
    }

    pub fn long_pressed(self) -> bool {
        self.long_pressed
    }

    pub fn press_holding(self) -> bool {
        self.press_holding
    }

    pub fn context_menu_requested(self) -> bool {
        self.context_menu_requested
    }

    pub fn pointer_clicked(self) -> bool {
        self.pointer_clicked
    }

    pub fn pointer_click_modifiers(self) -> Option<Modifiers> {
        self.pointer_clicked.then_some(self.pointer_click_modifiers)
    }

    pub fn context_menu_anchor(self) -> Option<Point> {
        self.context_menu_anchor
    }

    pub fn pointer_hovered_raw(self) -> bool {
        self.pointer_hovered_raw
    }

    pub fn pointer_hovered_raw_below_barrier(self) -> bool {
        self.pointer_hovered_raw_below_barrier
    }

    pub fn hover_stationary_met(self) -> bool {
        self.hover_stationary_met
    }

    pub fn hover_delay_short_met(self) -> bool {
        self.hover_delay_short_met
    }

    pub fn hover_delay_normal_met(self) -> bool {
        self.hover_delay_normal_met
    }

    pub fn hover_delay_short_shared_met(self) -> bool {
        self.hover_delay_short_shared_met
    }

    pub fn hover_delay_normal_shared_met(self) -> bool {
        self.hover_delay_normal_shared_met
    }

    pub fn hover_blocked_by_active_item(self) -> bool {
        self.hover_blocked_by_active_item
    }

    pub fn nav_highlighted(self) -> bool {
        self.nav_highlighted
    }

    /// ImGui-style "hovered" default: pointer-hover OR nav-highlight.
    ///
    /// Note: for ImGui-style hovered query flags, use `is_hovered(...)`.
    pub fn hovered_like_imgui(self) -> bool {
        self.is_hovered(ImUiHoveredFlags::NONE)
    }

    /// ImGui-style `IsItemHovered(flags)` convenience helper.
    ///
    /// This is intentionally a facade-only helper: `fret-authoring::Response` remains a minimal,
    /// stable contract.
    ///
    /// Implemented flags:
    /// - `ALLOW_WHEN_DISABLED`
    /// - `ALLOW_WHEN_BLOCKED_BY_POPUP` (best-effort; supports popup pointer-occlusion and modal barriers)
    /// - `ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM` (best-effort; suppress hover while another item is active)
    /// - `NO_NAV_OVERRIDE`
    /// - `FOR_TOOLTIP` (expands to `STATIONARY | DELAY_SHORT | ALLOW_WHEN_DISABLED`)
    /// - `STATIONARY` / `DELAY_SHORT` / `DELAY_NORMAL` (best-effort; uses timers)
    /// - `NO_SHARED_DELAY` (best-effort; disables shared delay for the query)
    pub fn is_hovered(self, mut flags: ImUiHoveredFlags) -> bool {
        if flags.contains(ImUiHoveredFlags::FOR_TOOLTIP) {
            flags |= ImUiHoveredFlags::STATIONARY;
            flags |= ImUiHoveredFlags::DELAY_SHORT;
            flags |= ImUiHoveredFlags::ALLOW_WHEN_DISABLED;
        }

        let allow_disabled = flags.contains(ImUiHoveredFlags::ALLOW_WHEN_DISABLED);
        let allow_blocked_by_popup = flags.contains(ImUiHoveredFlags::ALLOW_WHEN_BLOCKED_BY_POPUP);
        let allow_blocked_by_active_item =
            flags.contains(ImUiHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM);
        let nav_override = !flags.contains(ImUiHoveredFlags::NO_NAV_OVERRIDE);

        if nav_override && self.nav_highlighted {
            return true;
        }

        let mut pointer_hovered = if allow_disabled {
            self.pointer_hovered_raw
        } else if self.enabled() {
            self.core.hovered
        } else {
            false
        };

        if allow_blocked_by_popup {
            let below = if allow_disabled || self.enabled {
                self.pointer_hovered_raw_below_barrier
            } else {
                false
            };
            pointer_hovered |= below;
        }

        if !pointer_hovered {
            return false;
        }

        if self.hover_blocked_by_active_item && !allow_blocked_by_active_item {
            return false;
        }

        let delay_normal = flags.contains(ImUiHoveredFlags::DELAY_NORMAL);
        let delay_short = flags.contains(ImUiHoveredFlags::DELAY_SHORT);
        let stationary = flags.contains(ImUiHoveredFlags::STATIONARY);
        let no_shared_delay = flags.contains(ImUiHoveredFlags::NO_SHARED_DELAY);

        if delay_normal {
            let delay_met = if no_shared_delay {
                self.hover_delay_normal_met
            } else {
                self.hover_delay_normal_shared_met || self.hover_delay_normal_met
            };
            if !self.hover_stationary_met || !delay_met {
                return false;
            }
        } else if delay_short {
            let delay_met = if no_shared_delay {
                self.hover_delay_short_met
            } else {
                self.hover_delay_short_shared_met || self.hover_delay_short_met
            };
            if !self.hover_stationary_met || !delay_met {
                return false;
            }
        } else if stationary && !self.hover_stationary_met {
            return false;
        }

        true
    }

    pub fn drag_started(self) -> bool {
        self.drag.started()
    }

    pub fn dragging(self) -> bool {
        self.drag.dragging()
    }

    pub fn drag_stopped(self) -> bool {
        self.drag.stopped()
    }

    pub fn drag_delta(self) -> Point {
        self.drag.delta()
    }

    pub fn drag_total(self) -> Point {
        self.drag.total()
    }
}
