use fret_core::Px;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutSize<T> {
    pub width: T,
    pub height: T,
}

impl<T> LayoutSize<T> {
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn map<U>(self, f: impl FnMut(T) -> U) -> LayoutSize<U> {
        let LayoutSize { width, height } = self;
        let mut f = f;
        LayoutSize {
            width: f(width),
            height: f(height),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum AvailableSpace {
    Definite(Px),
    #[default]
    MinContent,
    MaxContent,
}

impl AvailableSpace {
    pub const fn is_definite(self) -> bool {
        matches!(self, Self::Definite(_))
    }

    pub const fn definite(self) -> Option<Px> {
        match self {
            Self::Definite(px) => Some(px),
            Self::MinContent | Self::MaxContent => None,
        }
    }

    pub fn shrink_by(self, delta_px: f32) -> Self {
        match self {
            Self::Definite(px) => Self::Definite(Px((px.0 - delta_px).max(0.0))),
            Self::MinContent => Self::MinContent,
            Self::MaxContent => Self::MaxContent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutConstraints {
    pub known: LayoutSize<Option<Px>>,
    pub available: LayoutSize<AvailableSpace>,
}

impl LayoutConstraints {
    pub const fn new(known: LayoutSize<Option<Px>>, available: LayoutSize<AvailableSpace>) -> Self {
        Self { known, available }
    }
}
