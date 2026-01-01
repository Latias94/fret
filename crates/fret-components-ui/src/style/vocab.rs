use fret_ui::element::{CrossAlign, MainAlign, Overflow};

/// Tailwind-like `justify-*` vocabulary (component-layer).
///
/// This exists to avoid leaking runtime enums into recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Justify {
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
}

impl Justify {
    pub fn to_main_align(self) -> MainAlign {
        match self {
            Self::Start => MainAlign::Start,
            Self::Center => MainAlign::Center,
            Self::End => MainAlign::End,
            Self::Between => MainAlign::SpaceBetween,
            Self::Around => MainAlign::SpaceAround,
            Self::Evenly => MainAlign::SpaceEvenly,
        }
    }
}

/// Tailwind-like `items-*` vocabulary (component-layer).
///
/// This exists to avoid leaking runtime enums into recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Items {
    Start,
    Center,
    End,
    Stretch,
}

impl Items {
    pub fn to_cross_align(self) -> CrossAlign {
        match self {
            Self::Start => CrossAlign::Start,
            Self::Center => CrossAlign::Center,
            Self::End => CrossAlign::End,
            Self::Stretch => CrossAlign::Stretch,
        }
    }
}

/// Tailwind-like overflow vocabulary (component-layer).
///
/// Note: Fret deliberately separates clipping (`overflow_hidden`) from scrolling (explicit `Scroll`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowRefinement {
    Visible,
    Hidden,
}

impl OverflowRefinement {
    pub fn to_overflow(self) -> Overflow {
        match self {
            Self::Visible => Overflow::Visible,
            Self::Hidden => Overflow::Clip,
        }
    }
}
