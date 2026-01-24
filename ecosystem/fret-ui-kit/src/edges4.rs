use fret_core::Px;

use crate::style::MarginEdgeRefinement;
use crate::{MetricRef, SignedMetricRef, Space};

/// A 4-edge value (top/right/bottom/left) used for fluent authoring ergonomics.
///
/// This intentionally lives in the ecosystem layer (`fret-ui-kit`) and is **token-aware** via
/// `Space`/`MetricRef` conversions (unlike `fret_core::Edges`, which is Px-only).
#[derive(Debug, Clone, PartialEq)]
pub struct Edges4<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T> Edges4<T> {
    pub fn trbl(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Edges4<U> {
        Edges4 {
            top: f(self.top),
            right: f(self.right),
            bottom: f(self.bottom),
            left: f(self.left),
        }
    }
}

impl<T: Clone> Edges4<T> {
    pub fn all(value: T) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    pub fn symmetric(horizontal: T, vertical: T) -> Self {
        Self {
            top: vertical.clone(),
            right: horizontal.clone(),
            bottom: vertical,
            left: horizontal,
        }
    }
}

impl From<Edges4<Space>> for Edges4<MetricRef> {
    fn from(value: Edges4<Space>) -> Self {
        value.map(MetricRef::space)
    }
}

impl From<Edges4<Px>> for Edges4<MetricRef> {
    fn from(value: Edges4<Px>) -> Self {
        value.map(MetricRef::Px)
    }
}

impl From<Edges4<Space>> for Edges4<SignedMetricRef> {
    fn from(value: Edges4<Space>) -> Self {
        Edges4::<MetricRef>::from(value).pos()
    }
}

impl From<Edges4<Px>> for Edges4<SignedMetricRef> {
    fn from(value: Edges4<Px>) -> Self {
        Edges4::<MetricRef>::from(value).pos()
    }
}

impl Edges4<Space> {
    pub fn pos(self) -> Edges4<SignedMetricRef> {
        Edges4::<MetricRef>::from(self).pos()
    }

    pub fn neg(self) -> Edges4<SignedMetricRef> {
        Edges4::<MetricRef>::from(self).neg()
    }
}

impl Edges4<Px> {
    pub fn pos(self) -> Edges4<SignedMetricRef> {
        Edges4::<MetricRef>::from(self).pos()
    }

    pub fn neg(self) -> Edges4<SignedMetricRef> {
        Edges4::<MetricRef>::from(self).neg()
    }
}

impl Edges4<MetricRef> {
    pub fn pos(self) -> Edges4<SignedMetricRef> {
        self.map(SignedMetricRef::pos)
    }

    pub fn neg(self) -> Edges4<SignedMetricRef> {
        self.map(SignedMetricRef::neg)
    }
}

/// Margin edges support `auto` in addition to px/token lengths.
#[derive(Debug, Clone)]
pub enum MarginEdge {
    Px(SignedMetricRef),
    Auto,
}

impl MarginEdge {
    pub fn auto() -> Self {
        Self::Auto
    }

    pub fn px(metric: impl Into<SignedMetricRef>) -> Self {
        Self::Px(metric.into())
    }
}

impl From<SignedMetricRef> for MarginEdge {
    fn from(value: SignedMetricRef) -> Self {
        Self::Px(value)
    }
}

impl From<MetricRef> for MarginEdge {
    fn from(value: MetricRef) -> Self {
        Self::Px(SignedMetricRef::pos(value))
    }
}

impl From<Px> for MarginEdge {
    fn from(value: Px) -> Self {
        Self::from(MetricRef::Px(value))
    }
}

impl From<Space> for MarginEdge {
    fn from(value: Space) -> Self {
        Self::from(MetricRef::space(value))
    }
}

impl From<Edges4<Space>> for Edges4<MarginEdge> {
    fn from(value: Edges4<Space>) -> Self {
        value.map(MarginEdge::from)
    }
}

impl From<Edges4<Px>> for Edges4<MarginEdge> {
    fn from(value: Edges4<Px>) -> Self {
        value.map(MarginEdge::from)
    }
}

impl From<Edges4<MetricRef>> for Edges4<MarginEdge> {
    fn from(value: Edges4<MetricRef>) -> Self {
        value.map(MarginEdge::from)
    }
}

impl From<Edges4<SignedMetricRef>> for Edges4<MarginEdge> {
    fn from(value: Edges4<SignedMetricRef>) -> Self {
        value.map(MarginEdge::from)
    }
}

impl From<MarginEdge> for MarginEdgeRefinement {
    fn from(value: MarginEdge) -> Self {
        match value {
            MarginEdge::Px(m) => MarginEdgeRefinement::Px(m),
            MarginEdge::Auto => MarginEdgeRefinement::Auto,
        }
    }
}
