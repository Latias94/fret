use fret_core::PathCommand;
use fret_core::geometry::Point;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StrokeStyle {
    #[default]
    Linear,
    StepAfter,
}

pub fn stroke_commands(points: &[Point], style: StrokeStyle) -> Vec<PathCommand> {
    if points.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(points.len().saturating_mul(2));
    out.push(PathCommand::MoveTo(points[0]));

    match style {
        StrokeStyle::Linear => {
            for p in &points[1..] {
                out.push(PathCommand::LineTo(*p));
            }
        }
        StrokeStyle::StepAfter => {
            for pair in points.windows(2) {
                let a = pair[0];
                let b = pair[1];
                out.push(PathCommand::LineTo(Point::new(b.x, a.y)));
                out.push(PathCommand::LineTo(b));
            }
        }
    }

    out
}
