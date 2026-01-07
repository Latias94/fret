use fret_core::{Color, Px};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LineJoin {
    Miter,
    Bevel,
    Round,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LineCap {
    Butt,
    Square,
    Round,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DashPattern {
    pub segments: Vec<Px>,
    pub phase: Px,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StrokeStyleV2 {
    pub width: Px,
    pub join: LineJoin,
    pub cap: LineCap,
    pub miter_limit: f32,
    pub dash: Option<DashPattern>,
}

impl Default for StrokeStyleV2 {
    fn default() -> Self {
        Self {
            width: Px(1.0),
            join: LineJoin::Round,
            cap: LineCap::Round,
            miter_limit: 4.0,
            dash: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GradientStop {
    pub t: f32,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LinearGradient {
    pub x0: Px,
    pub y0: Px,
    pub x1: Px,
    pub y1: Px,
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RadialGradient {
    pub cx: Px,
    pub cy: Px,
    pub r: Px,
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Paint {
    Solid(Color),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
}
