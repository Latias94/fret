use crate::{Point, Px, Rect, ids::PathId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadTo {
        ctrl: Point,
        to: Point,
    },
    CubicTo {
        ctrl1: Point,
        ctrl2: Point,
        to: Point,
    },
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FillStyle {
    pub rule: FillRule,
}

impl Default for FillStyle {
    fn default() -> Self {
        Self {
            rule: FillRule::NonZero,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeStyle {
    pub width: Px,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self { width: Px(1.0) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathStyle {
    Fill(FillStyle),
    Stroke(StrokeStyle),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathConstraints {
    /// Window/device scale factor used for tessellation and caching.
    pub scale_factor: f32,
}

impl Default for PathConstraints {
    fn default() -> Self {
        Self { scale_factor: 1.0 }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PathMetrics {
    pub bounds: Rect,
}

pub trait PathService {
    fn prepare(
        &mut self,
        commands: &[PathCommand],
        style: PathStyle,
        constraints: PathConstraints,
    ) -> (PathId, PathMetrics);

    fn measure(
        &mut self,
        commands: &[PathCommand],
        style: PathStyle,
        constraints: PathConstraints,
    ) -> PathMetrics {
        let (id, metrics) = self.prepare(commands, style, constraints);
        self.release(id);
        metrics
    }

    fn release(&mut self, path: PathId);
}
