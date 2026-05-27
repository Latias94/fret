#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugDrawRoundCorners(u8);

impl DebugDrawRoundCorners {
    pub const NONE: Self = Self(0);
    pub const TOP_LEFT: Self = Self(1 << 0);
    pub const TOP_RIGHT: Self = Self(1 << 1);
    pub const BOTTOM_RIGHT: Self = Self(1 << 2);
    pub const BOTTOM_LEFT: Self = Self(1 << 3);
    pub const TOP: Self = Self(Self::TOP_LEFT.0 | Self::TOP_RIGHT.0);
    pub const BOTTOM: Self = Self(Self::BOTTOM_LEFT.0 | Self::BOTTOM_RIGHT.0);
    pub const LEFT: Self = Self(Self::TOP_LEFT.0 | Self::BOTTOM_LEFT.0);
    pub const RIGHT: Self = Self(Self::TOP_RIGHT.0 | Self::BOTTOM_RIGHT.0);
    pub const ALL: Self = Self(Self::TOP.0 | Self::BOTTOM.0);

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl Default for DebugDrawRoundCorners {
    fn default() -> Self {
        Self::ALL
    }
}

impl std::ops::BitOr for DebugDrawRoundCorners {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for DebugDrawRoundCorners {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
