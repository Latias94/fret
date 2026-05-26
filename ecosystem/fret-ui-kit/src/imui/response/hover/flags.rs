/// ImGui-style hovered query flags for immediate-mode hover convenience helpers.
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
