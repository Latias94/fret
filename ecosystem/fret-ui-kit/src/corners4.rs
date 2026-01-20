use fret_core::Px;

use crate::{MetricRef, Radius};

/// A 4-corner value (top-left/top-right/bottom-right/bottom-left) used for fluent authoring ergonomics.
///
/// This intentionally lives in the ecosystem layer (`fret-ui-kit`) and is token-aware via
/// `Radius`/`MetricRef` conversions (unlike `fret_core::Corners`, which is Px-only).
#[derive(Debug, Clone, PartialEq)]
pub struct Corners4<T> {
    pub top_left: T,
    pub top_right: T,
    pub bottom_right: T,
    pub bottom_left: T,
}

impl<T> Corners4<T> {
    pub fn tltrbrbl(top_left: T, top_right: T, bottom_right: T, bottom_left: T) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Corners4<U> {
        Corners4 {
            top_left: f(self.top_left),
            top_right: f(self.top_right),
            bottom_right: f(self.bottom_right),
            bottom_left: f(self.bottom_left),
        }
    }
}

impl<T: Clone> Corners4<T> {
    pub fn all(value: T) -> Self {
        Self {
            top_left: value.clone(),
            top_right: value.clone(),
            bottom_right: value.clone(),
            bottom_left: value,
        }
    }
}

impl From<Corners4<Radius>> for Corners4<MetricRef> {
    fn from(value: Corners4<Radius>) -> Self {
        value.map(MetricRef::radius)
    }
}

impl From<Corners4<Px>> for Corners4<MetricRef> {
    fn from(value: Corners4<Px>) -> Self {
        value.map(MetricRef::Px)
    }
}
