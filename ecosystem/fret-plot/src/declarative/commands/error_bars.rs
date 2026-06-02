//! Declarative error-bars path-command projection owner.

use fret_core::{PathCommand, Point, Px};

use crate::cartesian::{DataPoint, PlotTransform};
use crate::models::MarkerShape;

use super::super::model::PlotPanelErrorBars;

pub(in crate::declarative) fn error_bars_commands_from_series(
    transform: PlotTransform,
    data: &dyn crate::series::SeriesData,
    error_bars: &PlotPanelErrorBars,
) -> Vec<PathCommand> {
    let cap = if error_bars.show_caps {
        error_bars.cap_size.0.max(0.0)
    } else {
        0.0
    };
    let marker = if error_bars.show_markers {
        error_bars.marker_radius.0.max(0.0)
    } else {
        0.0
    };
    let mut out = Vec::new();

    let mut push_point = |idx: usize, point: DataPoint| {
        if !point.x.is_finite() || !point.y.is_finite() {
            return;
        }

        let Some(x_px) = transform.data_x_to_px(point.x) else {
            return;
        };

        if let Some(y_err) = error_bars
            .y_errors
            .as_ref()
            .and_then(|errors| errors.get(idx))
        {
            let y0 = point.y - y_err.neg.abs();
            let y1 = point.y + y_err.pos.abs();
            if let (Some(y0_px), Some(y1_px)) =
                (transform.data_y_to_px(y0), transform.data_y_to_px(y1))
            {
                out.push(PathCommand::MoveTo(Point::new(x_px, y0_px)));
                out.push(PathCommand::LineTo(Point::new(x_px, y1_px)));

                if cap > 0.0 {
                    let x0 = Px(x_px.0 - cap);
                    let x1 = Px(x_px.0 + cap);
                    out.push(PathCommand::MoveTo(Point::new(x0, y0_px)));
                    out.push(PathCommand::LineTo(Point::new(x1, y0_px)));
                    out.push(PathCommand::MoveTo(Point::new(x0, y1_px)));
                    out.push(PathCommand::LineTo(Point::new(x1, y1_px)));
                }
            }
        }

        if let Some(x_err) = error_bars
            .x_errors
            .as_ref()
            .and_then(|errors| errors.get(idx))
            && let Some(y_px) = transform.data_y_to_px(point.y)
        {
            let x0 = point.x - x_err.neg.abs();
            let x1 = point.x + x_err.pos.abs();
            if let (Some(x0_px), Some(x1_px)) =
                (transform.data_x_to_px(x0), transform.data_x_to_px(x1))
            {
                out.push(PathCommand::MoveTo(Point::new(x0_px, y_px)));
                out.push(PathCommand::LineTo(Point::new(x1_px, y_px)));

                if cap > 0.0 {
                    let y0 = Px(y_px.0 - cap);
                    let y1 = Px(y_px.0 + cap);
                    out.push(PathCommand::MoveTo(Point::new(x0_px, y0)));
                    out.push(PathCommand::LineTo(Point::new(x0_px, y1)));
                    out.push(PathCommand::MoveTo(Point::new(x1_px, y0)));
                    out.push(PathCommand::LineTo(Point::new(x1_px, y1)));
                }
            }
        }

        if marker > 0.0 {
            let Some(y_px) = transform.data_y_to_px(point.y) else {
                return;
            };
            push_line_plot_marker_commands(
                &mut out,
                x_px,
                y_px,
                Px(marker),
                error_bars.marker_shape,
            );
        }
    };

    if let Some(points) = data.as_slice() {
        for (idx, point) in points.iter().copied().enumerate() {
            push_point(idx, point);
        }
    } else {
        for idx in 0..data.len() {
            let Some(point) = data.get(idx) else {
                continue;
            };
            push_point(idx, point);
        }
    }

    out
}

fn push_line_plot_marker_commands(
    out: &mut Vec<PathCommand>,
    x: Px,
    y: Px,
    radius: Px,
    shape: MarkerShape,
) {
    let x = x.0;
    let y = y.0;
    let r = radius.0.max(0.0);
    if !x.is_finite() || !y.is_finite() || !r.is_finite() || r <= 0.0 {
        return;
    }

    match shape {
        MarkerShape::Plus => {
            out.push(PathCommand::MoveTo(Point::new(Px(x - r), Px(y))));
            out.push(PathCommand::LineTo(Point::new(Px(x + r), Px(y))));
            out.push(PathCommand::MoveTo(Point::new(Px(x), Px(y - r))));
            out.push(PathCommand::LineTo(Point::new(Px(x), Px(y + r))));
        }
        MarkerShape::X => {
            out.push(PathCommand::MoveTo(Point::new(Px(x - r), Px(y - r))));
            out.push(PathCommand::LineTo(Point::new(Px(x + r), Px(y + r))));
            out.push(PathCommand::MoveTo(Point::new(Px(x - r), Px(y + r))));
            out.push(PathCommand::LineTo(Point::new(Px(x + r), Px(y - r))));
        }
        MarkerShape::Square => {
            let p0 = Point::new(Px(x - r), Px(y - r));
            let p1 = Point::new(Px(x + r), Px(y - r));
            let p2 = Point::new(Px(x + r), Px(y + r));
            let p3 = Point::new(Px(x - r), Px(y + r));
            out.push(PathCommand::MoveTo(p0));
            out.push(PathCommand::LineTo(p1));
            out.push(PathCommand::LineTo(p2));
            out.push(PathCommand::LineTo(p3));
            out.push(PathCommand::LineTo(p0));
        }
        MarkerShape::Diamond => {
            let p0 = Point::new(Px(x), Px(y - r));
            let p1 = Point::new(Px(x + r), Px(y));
            let p2 = Point::new(Px(x), Px(y + r));
            let p3 = Point::new(Px(x - r), Px(y));
            out.push(PathCommand::MoveTo(p0));
            out.push(PathCommand::LineTo(p1));
            out.push(PathCommand::LineTo(p2));
            out.push(PathCommand::LineTo(p3));
            out.push(PathCommand::LineTo(p0));
        }
        MarkerShape::TriangleUp => {
            let p0 = Point::new(Px(x), Px(y - r));
            let p1 = Point::new(Px(x + r), Px(y + r));
            let p2 = Point::new(Px(x - r), Px(y + r));
            out.push(PathCommand::MoveTo(p0));
            out.push(PathCommand::LineTo(p1));
            out.push(PathCommand::LineTo(p2));
            out.push(PathCommand::LineTo(p0));
        }
        MarkerShape::TriangleDown => {
            let p0 = Point::new(Px(x), Px(y + r));
            let p1 = Point::new(Px(x + r), Px(y - r));
            let p2 = Point::new(Px(x - r), Px(y - r));
            out.push(PathCommand::MoveTo(p0));
            out.push(PathCommand::LineTo(p1));
            out.push(PathCommand::LineTo(p2));
            out.push(PathCommand::LineTo(p0));
        }
        MarkerShape::Circle => {
            let segments = 12usize;
            let step = (std::f32::consts::PI * 2.0) / segments as f32;
            let p0 = Point::new(Px(x + r), Px(y));
            out.push(PathCommand::MoveTo(p0));
            for i in 1..=segments {
                let t = step * i as f32;
                let px = x + r * t.cos();
                let py = y + r * t.sin();
                if !px.is_finite() || !py.is_finite() {
                    continue;
                }
                out.push(PathCommand::LineTo(Point::new(Px(px), Px(py))));
            }
        }
    }
}
