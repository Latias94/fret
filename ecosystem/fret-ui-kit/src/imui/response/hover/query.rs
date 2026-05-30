use super::{ImUiHoveredFlags, ResponseExt};

mod delay;
mod pointer;

impl ResponseExt {
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

        if pointer::nav_override_satisfied(self, flags) {
            return true;
        }

        if !pointer::pointer_hovered_for_query(self, flags) {
            return false;
        }

        if !pointer::active_item_allows_hover(self, flags) {
            return false;
        }

        if !delay::delay_requirements_met(self, flags) {
            return false;
        }

        true
    }
}
