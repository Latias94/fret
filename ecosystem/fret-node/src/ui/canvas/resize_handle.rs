//! Shared node resize handle vocabulary.
//!
//! This is intentionally split from the retained canvas interaction state so that presenter
//! contracts can use `NodeResizeHandle` without depending on the retained bridge.

/// Node resize handle locations (8-way).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeResizeHandle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

impl NodeResizeHandle {
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

    pub fn affects_left(self) -> bool {
        matches!(self, Self::TopLeft | Self::Left | Self::BottomLeft)
    }

    pub fn affects_right(self) -> bool {
        matches!(self, Self::TopRight | Self::Right | Self::BottomRight)
    }

    pub fn affects_top(self) -> bool {
        matches!(self, Self::TopLeft | Self::Top | Self::TopRight)
    }

    pub fn affects_bottom(self) -> bool {
        matches!(self, Self::BottomLeft | Self::Bottom | Self::BottomRight)
    }
}

