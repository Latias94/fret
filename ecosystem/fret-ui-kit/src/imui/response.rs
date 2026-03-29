//! Immediate-mode response and interaction helper types.

use std::rc::Rc;

use fret_authoring::Response;
use fret_core::{Point, Rect, Size};
use fret_ui::GlobalElementId;

/// A richer interaction result intended for immediate-mode facade helpers.
///
/// This is a ui-kit-level convenience wrapper: it extends the minimal `fret-authoring::Response`
/// contract with additional commonly requested signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct DragResponse {
    pub started: bool,
    pub dragging: bool,
    pub stopped: bool,
    pub delta: Point,
    pub total: Point,
}

/// Published state for an immediate drag source helper.
#[derive(Debug, Clone, Copy, Default)]
pub struct DragSourceResponse {
    pub active: bool,
    pub cross_window: bool,
}

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
    pub enabled: bool,
    /// Pointer-hover signal without ImGui-style disabled gating.
    ///
    /// When a widget is disabled, `core.hovered` is forced to `false` by
    /// `sanitize_response_for_enabled(...)`.
    /// This field can still carry the raw pointer-hover signal for query helpers like
    /// `is_hovered(ImUiHoveredFlags::ALLOW_WHEN_DISABLED)`.
    pub pointer_hovered_raw: bool,
    /// Pointer-hover signal available even when popup policy blocks/suppresses hover (best-effort).
    ///
    /// This is primarily intended to support ImGui's `AllowWhenBlockedByPopup` hovered query flag.
    pub pointer_hovered_raw_below_barrier: bool,
    /// True once the "stationary" dwell timer has elapsed while hovered (best-effort).
    pub hover_stationary_met: bool,
    /// True once the short hover delay has elapsed while hovered.
    pub hover_delay_short_met: bool,
    /// True once the normal hover delay has elapsed while hovered.
    pub hover_delay_normal_met: bool,
    /// True once the short hover delay has elapsed (shared window-scoped timer, best-effort).
    pub hover_delay_short_shared_met: bool,
    /// True once the normal hover delay has elapsed (shared window-scoped timer, best-effort).
    pub hover_delay_normal_shared_met: bool,
    /// True when ImGui-style hover queries should be suppressed because another item is active.
    ///
    /// This is a facade-level policy knob intended to mirror `IsItemHovered()` behavior where
    /// hovered queries are suppressed while dragging another item, unless explicitly overridden
    /// with `ImUiHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM`.
    pub hover_blocked_by_active_item: bool,
    /// True when the item is focused and the window's focus-visible policy indicates keyboard
    /// navigation is active.
    ///
    /// This is intended as an immediate-mode equivalent of ImGui's "nav highlight under nav"
    /// behavior used by `IsItemHovered()` when `NavHighlightItemUnderNav` is active.
    pub nav_highlighted: bool,
    pub secondary_clicked: bool,
    pub double_clicked: bool,
    pub long_pressed: bool,
    pub press_holding: bool,
    pub context_menu_requested: bool,
    pub context_menu_anchor: Option<Point>,
    pub drag: DragResponse,
}

/// Immediate drag/drop target readout for a typed payload.
pub struct DropTargetResponse<T: 'static> {
    pub active: bool,
    pub over: bool,
    pub delivered: bool,
    pub source_id: Option<GlobalElementId>,
    pub session_id: Option<fret_runtime::DragSessionId>,
    pub(super) preview_payload: Option<Rc<T>>,
    pub(super) delivered_payload: Option<Rc<T>>,
}

#[derive(Debug, Clone, Copy)]
pub struct FloatingAreaResponse {
    pub id: GlobalElementId,
    pub rect: Option<Rect>,
    pub position: Point,
    pub dragging: bool,
    pub drag_kind: fret_runtime::DragKindId,
}

impl FloatingAreaResponse {
    pub fn rect(self) -> Option<Rect> {
        self.rect
    }

    pub fn position(self) -> Point {
        self.position
    }

    pub fn dragging(self) -> bool {
        self.dragging
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FloatingWindowResponse {
    pub area: FloatingAreaResponse,
    pub size: Option<Size>,
    pub resizing: bool,
    pub collapsed: bool,
}

impl FloatingWindowResponse {
    pub fn rect(self) -> Option<Rect> {
        self.area.rect
    }

    pub fn position(self) -> Point {
        self.area.position
    }

    pub fn size(self) -> Option<Size> {
        self.size
    }

    pub fn dragging(self) -> bool {
        self.area.dragging
    }

    pub fn resizing(self) -> bool {
        self.resizing
    }

    pub fn collapsed(self) -> bool {
        self.collapsed
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DisclosureResponse {
    pub trigger: ResponseExt,
    pub open: bool,
    pub toggled: bool,
}

impl DisclosureResponse {
    pub fn id(self) -> Option<GlobalElementId> {
        self.trigger.id
    }

    pub fn open(self) -> bool {
        self.open
    }

    pub fn toggled(self) -> bool {
        self.toggled
    }

    pub fn clicked(self) -> bool {
        self.trigger.clicked()
    }

    pub fn hovered_like_imgui(self) -> bool {
        self.trigger.hovered_like_imgui()
    }
}

impl DragSourceResponse {
    pub fn active(self) -> bool {
        self.active
    }

    pub fn cross_window(self) -> bool {
        self.cross_window
    }
}

impl<T: 'static> Default for DropTargetResponse<T> {
    fn default() -> Self {
        Self {
            active: false,
            over: false,
            delivered: false,
            source_id: None,
            session_id: None,
            preview_payload: None,
            delivered_payload: None,
        }
    }
}

impl<T: 'static> DropTargetResponse<T> {
    pub fn active(&self) -> bool {
        self.active
    }

    pub fn over(&self) -> bool {
        self.over
    }

    pub fn delivered(&self) -> bool {
        self.delivered
    }

    pub fn preview_payload(&self) -> Option<Rc<T>> {
        self.preview_payload.clone()
    }

    pub fn delivered_payload(&self) -> Option<Rc<T>> {
        self.delivered_payload.clone()
    }

    pub fn source_id(&self) -> Option<GlobalElementId> {
        self.source_id
    }

    pub fn session_id(&self) -> Option<fret_runtime::DragSessionId> {
        self.session_id
    }
}

impl ResponseExt {
    pub fn clicked(self) -> bool {
        self.core.clicked()
    }

    pub fn changed(self) -> bool {
        self.core.changed()
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

    pub fn context_menu_anchor(self) -> Option<Point> {
        self.context_menu_anchor
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
        } else if self.enabled {
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
        self.drag.started
    }

    pub fn dragging(self) -> bool {
        self.drag.dragging
    }

    pub fn drag_stopped(self) -> bool {
        self.drag.stopped
    }

    pub fn drag_delta(self) -> Point {
        self.drag.delta
    }

    pub fn drag_total(self) -> Point {
        self.drag.total
    }
}
