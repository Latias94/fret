use fret_core::PathCommand;
use fret_core::geometry::{Point, Px, Rect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataRect {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

impl DataRect {
    pub fn width(self) -> f32 {
        self.x_max - self.x_min
    }

    pub fn height(self) -> f32 {
        self.y_max - self.y_min
    }

    pub fn union(self, other: DataRect) -> DataRect {
        DataRect {
            x_min: self.x_min.min(other.x_min),
            x_max: self.x_max.max(other.x_max),
            y_min: self.y_min.min(other.y_min),
            y_max: self.y_max.max(other.y_max),
        }
    }

    pub fn from_points(points: impl IntoIterator<Item = DataPoint>) -> Option<DataRect> {
        let mut x_min: Option<f32> = None;
        let mut x_max: Option<f32> = None;
        let mut y_min: Option<f32> = None;
        let mut y_max: Option<f32> = None;

        for p in points {
            if !p.x.is_finite() || !p.y.is_finite() {
                continue;
            }

            x_min = Some(x_min.map_or(p.x, |v| v.min(p.x)));
            x_max = Some(x_max.map_or(p.x, |v| v.max(p.x)));
            y_min = Some(y_min.map_or(p.y, |v| v.min(p.y)));
            y_max = Some(y_max.map_or(p.y, |v| v.max(p.y)));
        }

        Some(DataRect {
            x_min: x_min?,
            x_max: x_max?,
            y_min: y_min?,
            y_max: y_max?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotTransform {
    /// Viewport in logical pixels (screen/layout space, +Y downward).
    pub viewport: Rect,
    /// Data-space bounds (+Y upward by convention).
    pub data: DataRect,
}

impl PlotTransform {
    pub fn data_to_px(self, p: DataPoint) -> Point {
        let data_w = self.data.width();
        let data_h = self.data.height();

        let viewport_w = self.viewport.size.width.0;
        let viewport_h = self.viewport.size.height.0;

        let nx = if data_w.is_finite() && data_w != 0.0 {
            (p.x - self.data.x_min) / data_w
        } else {
            0.0
        };

        // Data-space +Y is up; screen-space +Y is down.
        let ny = if data_h.is_finite() && data_h != 0.0 {
            1.0 - (p.y - self.data.y_min) / data_h
        } else {
            0.0
        };

        let x = self.viewport.origin.x.0 + (nx * viewport_w);
        let y = self.viewport.origin.y.0 + (ny * viewport_h);
        Point::new(Px(x), Px(y))
    }

    pub fn px_to_data(self, p: Point) -> DataPoint {
        let data_w = self.data.width();
        let data_h = self.data.height();

        let viewport_w = self.viewport.size.width.0;
        let viewport_h = self.viewport.size.height.0;

        let nx = if viewport_w.is_finite() && viewport_w != 0.0 {
            (p.x.0 - self.viewport.origin.x.0) / viewport_w
        } else {
            0.0
        };

        let ny = if viewport_h.is_finite() && viewport_h != 0.0 {
            (p.y.0 - self.viewport.origin.y.0) / viewport_h
        } else {
            0.0
        };

        let x = self.data.x_min + (nx * data_w);
        let y = self.data.y_min + ((1.0 - ny) * data_h);
        DataPoint { x, y }
    }
}

/// Maps a data point stream into a polyline path.
///
/// Conventions:
/// - Non-finite values (NaN/Inf) break the line; the next valid point starts a new subpath.
pub fn polyline_commands(transform: PlotTransform, points: &[DataPoint]) -> Vec<PathCommand> {
    let mut out = Vec::new();
    let mut active = false;

    for p in points {
        if !p.x.is_finite() || !p.y.is_finite() {
            active = false;
            continue;
        }

        let px = transform.data_to_px(*p);
        if !px.x.0.is_finite() || !px.y.0.is_finite() {
            active = false;
            continue;
        }

        if !active {
            out.push(PathCommand::MoveTo(px));
            active = true;
        } else {
            out.push(PathCommand::LineTo(px));
        }
    }

    out
}
