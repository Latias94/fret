use crate::geometry::Px;

/// Dash pattern for stroke-like primitives.
///
/// Units are logical pixels (scale-aware; the renderer multiplies by the current scale factor).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DashPatternV1 {
    pub dash: Px,
    pub gap: Px,
    pub phase: Px,
}

impl DashPatternV1 {
    pub const fn new(dash: Px, gap: Px, phase: Px) -> Self {
        Self { dash, gap, phase }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StrokeStyleV1 {
    pub dash: Option<DashPatternV1>,
}
