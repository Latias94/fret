use fret_authoring::Response;
use fret_core::{Modifiers, Point};
use fret_ui::GlobalElementId;

use super::super::drag::DragResponse;

/// A richer interaction result intended for immediate-mode facade helpers.
///
/// This is a ui-kit-level convenience wrapper: it extends the minimal `fret-authoring::Response`
/// contract with additional commonly requested signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResponseExt {
    pub(super) core: Response,
    pub(super) id: Option<GlobalElementId>,
    pub(super) enabled: bool,
    /// True on the frame an item enters its active or engaged state.
    pub(super) activated: bool,
    /// True on the frame an item leaves its active or engaged state.
    pub(super) deactivated: bool,
    /// True on the frame an item commits a meaningful value mutation.
    pub(super) edited: bool,
    /// True on the frame an item deactivates after having been edited during the same active
    /// session.
    pub(super) deactivated_after_edit: bool,
    /// Pointer-hover signal without ImGui-style disabled gating.
    ///
    /// When a widget is disabled, `core.hovered` is forced to `false` by
    /// `sanitize_response_for_enabled(...)`.
    /// This field can still carry the raw pointer-hover signal for query helpers like
    /// `is_hovered(ImUiHoveredFlags::ALLOW_WHEN_DISABLED)`.
    pub(super) pointer_hovered_raw: bool,
    /// Pointer-hover signal available even when popup policy blocks/suppresses hover (best-effort).
    ///
    /// This is primarily intended to support ImGui's `AllowWhenBlockedByPopup` hovered query flag.
    pub(super) pointer_hovered_raw_below_barrier: bool,
    /// True once the "stationary" dwell timer has elapsed while hovered (best-effort).
    pub(super) hover_stationary_met: bool,
    /// True once the short hover delay has elapsed while hovered.
    pub(super) hover_delay_short_met: bool,
    /// True once the normal hover delay has elapsed while hovered.
    pub(super) hover_delay_normal_met: bool,
    /// True once the short hover delay has elapsed (shared window-scoped timer, best-effort).
    pub(super) hover_delay_short_shared_met: bool,
    /// True once the normal hover delay has elapsed (shared window-scoped timer, best-effort).
    pub(super) hover_delay_normal_shared_met: bool,
    /// True when ImGui-style hover queries should be suppressed because another item is active.
    ///
    /// This is a facade-level policy knob intended to mirror `IsItemHovered()` behavior where
    /// hovered queries are suppressed while dragging another item, unless the query explicitly
    /// opts into active-item blocking.
    pub(super) hover_blocked_by_active_item: bool,
    /// True when the item is focused and the window's focus-visible policy indicates keyboard
    /// navigation is active.
    ///
    /// This is intended as an immediate-mode equivalent of ImGui's "nav highlight under nav"
    /// behavior used by `IsItemHovered()` when `NavHighlightItemUnderNav` is active.
    pub(super) nav_highlighted: bool,
    pub(super) secondary_clicked: bool,
    pub(super) double_clicked: bool,
    pub(super) long_pressed: bool,
    pub(super) press_holding: bool,
    pub(super) context_menu_requested: bool,
    pub(super) context_menu_anchor: Option<Point>,
    /// True when `clicked` was produced by a pointer click rather than keyboard activation.
    pub(super) pointer_clicked: bool,
    /// Best-effort modifier snapshot for the pointer click that produced `clicked`.
    ///
    /// Consumers should read this through `pointer_click_modifiers()` so keyboard activations map
    /// to `None`.
    pub(super) pointer_click_modifiers: Modifiers,
    pub(super) drag: DragResponse,
}
