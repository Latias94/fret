//! Headless 2D resize handle vocabulary for canvas-like surfaces.
//!
//! This module owns only generic handle geometry. Domain-specific resize policy, snapping, and
//! constraints stay in the consuming crate.

/// 8-way resize handle locations for a rectangular canvas item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeHandle2D {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

impl ResizeHandle2D {
    pub const ALL: [Self; 8] = [
        Self::TopLeft,
        Self::Top,
        Self::TopRight,
        Self::Right,
        Self::BottomRight,
        Self::Bottom,
        Self::BottomLeft,
        Self::Left,
    ];

    pub const fn affects_left(self) -> bool {
        matches!(self, Self::TopLeft | Self::Left | Self::BottomLeft)
    }

    pub const fn affects_right(self) -> bool {
        matches!(self, Self::TopRight | Self::Right | Self::BottomRight)
    }

    pub const fn affects_top(self) -> bool {
        matches!(self, Self::TopLeft | Self::Top | Self::TopRight)
    }

    pub const fn affects_bottom(self) -> bool {
        matches!(self, Self::BottomLeft | Self::Bottom | Self::BottomRight)
    }
}

/// Compact bitset for enabled 2D resize handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResizeHandleSet2D {
    bits: u16,
}

impl ResizeHandleSet2D {
    const fn mask(handle: ResizeHandle2D) -> u16 {
        match handle {
            ResizeHandle2D::TopLeft => 1 << 0,
            ResizeHandle2D::Top => 1 << 1,
            ResizeHandle2D::TopRight => 1 << 2,
            ResizeHandle2D::Right => 1 << 3,
            ResizeHandle2D::BottomRight => 1 << 4,
            ResizeHandle2D::Bottom => 1 << 5,
            ResizeHandle2D::BottomLeft => 1 << 6,
            ResizeHandle2D::Left => 1 << 7,
        }
    }

    pub const NONE: Self = Self { bits: 0 };
    pub const ALL: Self = Self { bits: (1 << 8) - 1 };

    pub const fn none() -> Self {
        Self::NONE
    }

    pub const fn all() -> Self {
        Self::ALL
    }

    pub const fn from_bits(bits: u16) -> Self {
        Self { bits }
    }

    pub const fn bits(self) -> u16 {
        self.bits
    }

    pub const fn contains(self, handle: ResizeHandle2D) -> bool {
        (self.bits & Self::mask(handle)) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub fn insert(&mut self, handle: ResizeHandle2D) {
        self.bits |= Self::mask(handle);
    }

    pub fn remove(&mut self, handle: ResizeHandle2D) {
        self.bits &= !Self::mask(handle);
    }
}

impl Default for ResizeHandleSet2D {
    fn default() -> Self {
        Self::all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_axis_flags_match_location() {
        assert!(ResizeHandle2D::TopLeft.affects_left());
        assert!(ResizeHandle2D::TopLeft.affects_top());
        assert!(!ResizeHandle2D::TopLeft.affects_right());
        assert!(!ResizeHandle2D::TopLeft.affects_bottom());

        assert!(ResizeHandle2D::Right.affects_right());
        assert!(!ResizeHandle2D::Right.affects_left());
        assert!(!ResizeHandle2D::Right.affects_top());
        assert!(!ResizeHandle2D::Right.affects_bottom());

        assert!(ResizeHandle2D::Bottom.affects_bottom());
        assert!(!ResizeHandle2D::Bottom.affects_top());
    }

    #[test]
    fn handle_set_mutations_are_stable() {
        let mut set = ResizeHandleSet2D::none();
        assert!(set.is_empty());
        assert!(!set.contains(ResizeHandle2D::Left));

        set.insert(ResizeHandle2D::Left);
        set.insert(ResizeHandle2D::TopRight);
        assert!(set.contains(ResizeHandle2D::Left));
        assert!(set.contains(ResizeHandle2D::TopRight));
        assert!(!set.contains(ResizeHandle2D::BottomRight));

        set.remove(ResizeHandle2D::Left);
        assert!(!set.contains(ResizeHandle2D::Left));
        assert!(set.contains(ResizeHandle2D::TopRight));
    }
}
