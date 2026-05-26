use super::{ImUiHoveredFlags, ResponseExt};

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
}
