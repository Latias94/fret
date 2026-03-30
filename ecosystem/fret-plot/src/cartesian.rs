use fret_core::PathCommand;
use fret_core::geometry::{Point, Px, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AxisScale {
    #[default]
    Linear,
    Log10,
}

impl AxisScale {
    pub fn key(self) -> u64 {
        match self {
            Self::Linear => 0x4c49_4e45_4152_0000u64,
            Self::Log10 => 0x4c4f_4731_3000_0000u64,
        }
    }

    pub fn to_axis(self, v: f64) -> Option<f64> {
        if !v.is_finite() {
            return None;
        }
        match self {
            Self::Linear => Some(v),
            Self::Log10 => {
                if v <= 0.0 {
                    return None;
                }
                let out = v.log10();
                out.is_finite().then_some(out)
            }
        }
    }

    pub fn from_axis(self, v: f64) -> Option<f64> {
        if !v.is_finite() {
            return None;
        }
        match self {
            Self::Linear => Some(v),
            Self::Log10 => {
                let out = 10.0_f64.powf(v);
                out.is_finite().then_some(out)
            }
        }
    }

    pub fn sanitize_bounds(self, min: f64, max: f64) -> (f64, f64) {
        let (mut min, mut max) = if min <= max { (min, max) } else { (max, min) };

        if !min.is_finite() || !max.is_finite() {
            return (0.0, 1.0);
        }

        match self {
            Self::Linear => (min, max),
            Self::Log10 => {
                // Log axes require positive ranges. If the domain is invalid, clamp to a small,
                // deterministic range so transforms remain well-defined.
                const MIN_POS: f64 = 1.0e-12;
                if max <= 0.0 {
                    min = MIN_POS;
                    max = MIN_POS * 10.0;
                } else {
                    min = min.max(MIN_POS);
                    if max <= min {
                        max = min * 10.0;
                    }
                }
                (min, max)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataRect {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl DataRect {
    pub fn width(self) -> f64 {
        self.x_max - self.x_min
    }

    pub fn height(self) -> f64 {
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
        let mut x_min: Option<f64> = None;
        let mut x_max: Option<f64> = None;
        let mut y_min: Option<f64> = None;
        let mut y_max: Option<f64> = None;

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
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreparedPlotTransform {
    viewport_origin_x: f64,
    viewport_origin_y: f64,
    viewport_w: f64,
    viewport_h: f64,
    x0: f64,
    inv_w: f64,
    y0: f64,
    inv_h: f64,
    x_scale: AxisScale,
    y_scale: AxisScale,
}

impl PreparedPlotTransform {
    pub fn data_x_to_px(self, x: f64) -> Option<Px> {
        let vx = self.x_scale.to_axis(x)?;
        let nx = (vx - self.x0) * self.inv_w;
        if !nx.is_finite() {
            return None;
        }
        let px = self.viewport_origin_x + (nx * self.viewport_w);
        px.is_finite().then_some(Px(px as f32))
    }

    pub fn data_y_to_px(self, y: f64) -> Option<Px> {
        let vy = self.y_scale.to_axis(y)?;
        let ny = 1.0 - (vy - self.y0) * self.inv_h;
        if !ny.is_finite() {
            return None;
        }
        let px = self.viewport_origin_y + (ny * self.viewport_h);
        px.is_finite().then_some(Px(px as f32))
    }

    pub fn data_to_px(self, p: DataPoint) -> Point {
        let vx = self.x_scale.to_axis(p.x).unwrap_or(f64::NAN);
        let vy = self.y_scale.to_axis(p.y).unwrap_or(f64::NAN);

        let nx = (vx - self.x0) * self.inv_w;
        let ny = 1.0 - (vy - self.y0) * self.inv_h;

        let x = self.viewport_origin_x + (nx * self.viewport_w);
        let y = self.viewport_origin_y + (ny * self.viewport_h);
        Point::new(Px(x as f32), Px(y as f32))
    }
}

impl PlotTransform {
    pub fn prepare(self) -> Option<PreparedPlotTransform> {
        let (x_min, x_max) = self
            .x_scale
            .sanitize_bounds(self.data.x_min, self.data.x_max);
        let x0 = self.x_scale.to_axis(x_min)?;
        let x1 = self.x_scale.to_axis(x_max)?;
        let w = x1 - x0;
        if !w.is_finite() || w == 0.0 {
            return None;
        }

        let (y_min, y_max) = self
            .y_scale
            .sanitize_bounds(self.data.y_min, self.data.y_max);
        let y0 = self.y_scale.to_axis(y_min)?;
        let y1 = self.y_scale.to_axis(y_max)?;
        let h = y1 - y0;
        if !h.is_finite() || h == 0.0 {
            return None;
        }

        let viewport_w = f64::from(self.viewport.size.width.0);
        let viewport_h = f64::from(self.viewport.size.height.0);
        if !viewport_w.is_finite() || viewport_w <= 0.0 {
            return None;
        }
        if !viewport_h.is_finite() || viewport_h <= 0.0 {
            return None;
        }

        Some(PreparedPlotTransform {
            viewport_origin_x: f64::from(self.viewport.origin.x.0),
            viewport_origin_y: f64::from(self.viewport.origin.y.0),
            viewport_w,
            viewport_h,
            x0,
            inv_w: 1.0 / w,
            y0,
            inv_h: 1.0 / h,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
        })
    }

    pub fn data_x_to_px(self, x: f64) -> Option<Px> {
        let (x_min, x_max) = self
            .x_scale
            .sanitize_bounds(self.data.x_min, self.data.x_max);
        let x0 = self.x_scale.to_axis(x_min)?;
        let x1 = self.x_scale.to_axis(x_max)?;
        let w = x1 - x0;
        if !w.is_finite() || w == 0.0 {
            return None;
        }

        let viewport_w = f64::from(self.viewport.size.width.0);
        if !viewport_w.is_finite() || viewport_w <= 0.0 {
            return None;
        }

        let vx = self.x_scale.to_axis(x)?;
        let nx = (vx - x0) / w;
        if !nx.is_finite() {
            return None;
        }

        let px = f64::from(self.viewport.origin.x.0) + (nx * viewport_w);
        px.is_finite().then_some(Px(px as f32))
    }

    pub fn data_y_to_px(self, y: f64) -> Option<Px> {
        let (y_min, y_max) = self
            .y_scale
            .sanitize_bounds(self.data.y_min, self.data.y_max);
        let y0 = self.y_scale.to_axis(y_min)?;
        let y1 = self.y_scale.to_axis(y_max)?;
        let h = y1 - y0;
        if !h.is_finite() || h == 0.0 {
            return None;
        }

        let viewport_h = f64::from(self.viewport.size.height.0);
        if !viewport_h.is_finite() || viewport_h <= 0.0 {
            return None;
        }

        let vy = self.y_scale.to_axis(y)?;
        let ny = 1.0 - (vy - y0) / h;
        if !ny.is_finite() {
            return None;
        }

        let px = f64::from(self.viewport.origin.y.0) + (ny * viewport_h);
        px.is_finite().then_some(Px(px as f32))
    }

    pub fn data_to_px(self, p: DataPoint) -> Point {
        let (x_min, x_max) = self
            .x_scale
            .sanitize_bounds(self.data.x_min, self.data.x_max);
        let (y_min, y_max) = self
            .y_scale
            .sanitize_bounds(self.data.y_min, self.data.y_max);

        let x0 = self.x_scale.to_axis(x_min);
        let x1 = self.x_scale.to_axis(x_max);
        let y0 = self.y_scale.to_axis(y_min);
        let y1 = self.y_scale.to_axis(y_max);

        let data_w = x0.zip(x1).map(|(a, b)| b - a).unwrap_or(0.0);
        let data_h = y0.zip(y1).map(|(a, b)| b - a).unwrap_or(0.0);

        let viewport_w = f64::from(self.viewport.size.width.0);
        let viewport_h = f64::from(self.viewport.size.height.0);

        let px = self.x_scale.to_axis(p.x);
        let py = self.y_scale.to_axis(p.y);

        let nx = if data_w.is_finite() && data_w != 0.0 {
            match (px, x0) {
                (Some(px), Some(x0)) => (px - x0) / data_w,
                _ => f64::NAN,
            }
        } else {
            0.0
        };

        // Data-space +Y is up; screen-space +Y is down.
        let ny = if data_h.is_finite() && data_h != 0.0 {
            match (py, y0) {
                (Some(py), Some(y0)) => 1.0 - (py - y0) / data_h,
                _ => f64::NAN,
            }
        } else {
            0.0
        };

        let x = f64::from(self.viewport.origin.x.0) + (nx * viewport_w);
        let y = f64::from(self.viewport.origin.y.0) + (ny * viewport_h);
        Point::new(Px(x as f32), Px(y as f32))
    }

    pub fn px_to_data(self, p: Point) -> DataPoint {
        let (x_min, x_max) = self
            .x_scale
            .sanitize_bounds(self.data.x_min, self.data.x_max);
        let (y_min, y_max) = self
            .y_scale
            .sanitize_bounds(self.data.y_min, self.data.y_max);

        let x0 = self.x_scale.to_axis(x_min);
        let x1 = self.x_scale.to_axis(x_max);
        let y0 = self.y_scale.to_axis(y_min);
        let y1 = self.y_scale.to_axis(y_max);

        let data_w = x0.zip(x1).map(|(a, b)| b - a).unwrap_or(0.0);
        let data_h = y0.zip(y1).map(|(a, b)| b - a).unwrap_or(0.0);

        let viewport_w = f64::from(self.viewport.size.width.0);
        let viewport_h = f64::from(self.viewport.size.height.0);

        let nx = if viewport_w.is_finite() && viewport_w != 0.0 {
            (f64::from(p.x.0) - f64::from(self.viewport.origin.x.0)) / viewport_w
        } else {
            0.0
        };

        let ny = if viewport_h.is_finite() && viewport_h != 0.0 {
            (f64::from(p.y.0) - f64::from(self.viewport.origin.y.0)) / viewport_h
        } else {
            0.0
        };

        let x_axis = x0.zip(Some(nx)).map(|(x0, nx)| x0 + (nx * data_w));
        let y_axis = y0.zip(Some(ny)).map(|(y0, ny)| y0 + ((1.0 - ny) * data_h));

        let x = x_axis
            .and_then(|v| self.x_scale.from_axis(v))
            .unwrap_or(f64::NAN);
        let y = y_axis
            .and_then(|v| self.y_scale.from_axis(v))
            .unwrap_or(f64::NAN);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log10_scale_round_trips_positive_values() {
        let t = PlotTransform {
            viewport: Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                fret_core::geometry::Size::new(Px(100.0), Px(100.0)),
            ),
            data: DataRect {
                x_min: 1.0,
                x_max: 1000.0,
                y_min: 1.0e-3,
                y_max: 1.0e3,
            },
            x_scale: AxisScale::Log10,
            y_scale: AxisScale::Log10,
        };

        let p = DataPoint { x: 10.0, y: 1.0 };
        let px = t.data_to_px(p);
        assert!(px.x.0.is_finite() && px.y.0.is_finite());
        let back = t.px_to_data(px);
        assert!((back.x - p.x).abs() / p.x <= 1e-6, "back.x={}", back.x);
        assert!((back.y - p.y).abs() / p.y <= 1e-6, "back.y={}", back.y);
    }
}
