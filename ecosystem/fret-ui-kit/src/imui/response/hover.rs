use fret_authoring::Response;
use fret_core::{Modifiers, Point};
use fret_ui::GlobalElementId;

use super::drag::DragResponse;

mod core_state;
mod flags;
mod hover_state;
mod lifecycle;
mod press_context;
mod query;

pub use flags::ImUiHoveredFlags;

/// A richer interaction result intended for immediate-mode facade helpers.
///
/// This is a ui-kit-level convenience wrapper: it extends the minimal `fret-authoring::Response`
/// contract with additional commonly requested signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResponseExt {
    core: Response,
    id: Option<GlobalElementId>,
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
    /// hovered queries are suppressed while dragging another item, unless the query explicitly
    /// opts into active-item blocking.
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
