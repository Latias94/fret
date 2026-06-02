//! Declarative line-plot path-command projection owner.

use fret_core::{PathCommand, Point, Px};

use crate::cartesian::{DataPoint, PlotTransform};
use crate::models::StepMode;

use super::model::{PlotPanelBars, PlotPanelCandlestick, PlotPanelErrorBars, PlotPanelHistogram};

pub(super) fn line_plot_series_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6c69_6e65_u64 ^ series_id
}

pub(super) fn line_plot_area_fill_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6172_6561_u64 ^ series_id
}

pub(super) fn line_plot_shaded_lower_path_key(series_id: u64) -> u64 {
    0x706c_6f74_7368_6164_u64 ^ series_id
}

pub(super) fn line_plot_candlestick_down_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6364_6f77_u64 ^ series_id
}

fn line_plot_device_point_budget(transform: PlotTransform, scale_factor: f32) -> usize {
    let width = transform.viewport.size.width.0.max(0.0);
    let device_width = (width * scale_factor.max(1.0)).max(1.0);
    device_width as usize * 2
}

pub(super) fn candlestick_commands_from_series(
    transform: PlotTransform,
    candlestick: &PlotPanelCandlestick,
    stroke_width: Px,
    scale_factor: f32,
) -> (Vec<PathCommand>, Vec<PathCommand>, Vec<PathCommand>) {
    let points = &candlestick.points;
    if points.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let candle_width = candlestick.candle_width.abs();
    if !candle_width.is_finite() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let view_x_min = transform.data.x_min.min(transform.data.x_max);
    let view_x_max = transform.data.x_min.max(transform.data.x_max);
    let half_w = candle_width * 0.5;
    let max_count = line_plot_device_point_budget(transform, scale_factor).max(8);

    let mut wick = Vec::new();
    let mut body_up = Vec::new();
    let mut body_down = Vec::new();

    let push_rect = |out: &mut Vec<PathCommand>, x0: Px, x1: Px, y0: Px, y1: Px| {
        let left = x0.0.min(x1.0);
        let right = x0.0.max(x1.0);
        let top = y0.0.min(y1.0);
        let bottom = y0.0.max(y1.0);
        if !left.is_finite() || !right.is_finite() || !top.is_finite() || !bottom.is_finite() {
            return;
        }

        let width = (right - left).max(stroke_width.0.max(1.0));
        let height = (bottom - top).max(stroke_width.0.max(1.0));
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return;
        }

        let p0 = Point::new(Px(left), Px(top));
        let p1 = Point::new(Px(left + width), Px(top));
        let p2 = Point::new(Px(left + width), Px(top + height));
        let p3 = Point::new(Px(left), Px(top + height));

        out.push(PathCommand::MoveTo(p0));
        out.push(PathCommand::LineTo(p1));
        out.push(PathCommand::LineTo(p2));
        out.push(PathCommand::LineTo(p3));
        out.push(PathCommand::Close);
    };

    let mut push_point = |point: crate::models::OhlcPoint| {
        if !point.is_finite() || point.x < view_x_min || point.x > view_x_max {
            return;
        }

        let Some(x_px) = transform.data_x_to_px(point.x) else {
            return;
        };
        let Some(high_px) = transform.data_y_to_px(point.high) else {
            return;
        };
        let Some(low_px) = transform.data_y_to_px(point.low) else {
            return;
        };
        wick.push(PathCommand::MoveTo(Point::new(x_px, high_px)));
        wick.push(PathCommand::LineTo(Point::new(x_px, low_px)));

        let Some(x0_px) = transform.data_x_to_px(point.x - half_w) else {
            return;
        };
        let Some(x1_px) = transform.data_x_to_px(point.x + half_w) else {
            return;
        };
        let Some(open_px) = transform.data_y_to_px(point.open) else {
            return;
        };
        let Some(close_px) = transform.data_y_to_px(point.close) else {
            return;
        };

        if point.close >= point.open {
            push_rect(&mut body_up, x0_px, x1_px, open_px, close_px);
        } else {
            push_rect(&mut body_down, x0_px, x1_px, open_px, close_px);
        }
    };

    if points.len() <= max_count {
        for point in points.iter().copied() {
            push_point(point);
        }
    } else {
        let stride = points.len().div_ceil(max_count).max(1);
        for point in points.iter().copied().step_by(stride) {
            push_point(point);
        }
    }

    (wick, body_up, body_down)
}

pub(super) fn histogram_commands_from_series(
    transform: PlotTransform,
    data: &dyn crate::series::SeriesData,
    histogram: &PlotPanelHistogram,
) -> Vec<PathCommand> {
    let bin_width = histogram.bin_width;
    if !bin_width.is_finite() || bin_width <= 0.0 {
        return Vec::new();
    }

    let gap = histogram.bar_gap_fraction.clamp(0.0, 0.95);
    let bar_width = bin_width * f64::from(1.0 - gap);
    if !bar_width.is_finite() || bar_width <= 0.0 {
        return Vec::new();
    }

    let mut out = Vec::new();
    for index in 0..data.len() {
        let Some(point) = data.get(index) else {
            continue;
        };
        if !point.x.is_finite() || !point.y.is_finite() || point.y <= 0.0 {
            continue;
        }

        let x0 = point.x - bar_width * 0.5;
        let x1 = point.x + bar_width * 0.5;
        let p00 = transform.data_to_px(DataPoint { x: x0, y: 0.0 });
        let p10 = transform.data_to_px(DataPoint { x: x1, y: 0.0 });
        let p01 = transform.data_to_px(DataPoint { x: x0, y: point.y });
        let p11 = transform.data_to_px(DataPoint { x: x1, y: point.y });

        if !p00.x.0.is_finite()
            || !p00.y.0.is_finite()
            || !p10.x.0.is_finite()
            || !p10.y.0.is_finite()
            || !p01.x.0.is_finite()
            || !p01.y.0.is_finite()
            || !p11.x.0.is_finite()
            || !p11.y.0.is_finite()
        {
            continue;
        }

        let left = p00.x.0.min(p10.x.0).min(p01.x.0).min(p11.x.0);
        let right = p00.x.0.max(p10.x.0).max(p01.x.0).max(p11.x.0);
        let top = p00.y.0.min(p10.y.0).min(p01.y.0).min(p11.y.0);
        let bottom = p00.y.0.max(p10.y.0).max(p01.y.0).max(p11.y.0);

        if !left.is_finite()
            || !right.is_finite()
            || !top.is_finite()
            || !bottom.is_finite()
            || right <= left
            || bottom <= top
        {
            continue;
        }

        let a = Point::new(Px(left), Px(top));
        let b = Point::new(Px(right), Px(top));
        let c = Point::new(Px(right), Px(bottom));
        let d = Point::new(Px(left), Px(bottom));

        out.push(PathCommand::MoveTo(a));
        out.push(PathCommand::LineTo(b));
        out.push(PathCommand::LineTo(c));
        out.push(PathCommand::LineTo(d));
        out.push(PathCommand::Close);
    }

    out
}

pub(super) fn bars_commands_from_series(
    transform: PlotTransform,
    data: &dyn crate::series::SeriesData,
    bars: &PlotPanelBars,
) -> Vec<PathCommand> {
    let bar_width = bars.bar_width.abs();
    if !bar_width.is_finite() || bar_width <= 0.0 {
        return Vec::new();
    }

    let mut out = Vec::new();
    for index in 0..data.len() {
        let Some(point) = data.get(index) else {
            continue;
        };
        let baseline = bars
            .baselines
            .as_deref()
            .and_then(|baselines| baselines.get(index).copied())
            .unwrap_or(bars.baseline);
        if !point.x.is_finite()
            || !point.y.is_finite()
            || !baseline.is_finite()
            || point.y == baseline
        {
            continue;
        }

        let x0 = point.x - bar_width * 0.5;
        let x1 = point.x + bar_width * 0.5;
        let p00 = transform.data_to_px(DataPoint { x: x0, y: baseline });
        let p10 = transform.data_to_px(DataPoint { x: x1, y: baseline });
        let p01 = transform.data_to_px(DataPoint { x: x0, y: point.y });
        let p11 = transform.data_to_px(DataPoint { x: x1, y: point.y });

        if !p00.x.0.is_finite()
            || !p00.y.0.is_finite()
            || !p10.x.0.is_finite()
            || !p10.y.0.is_finite()
            || !p01.x.0.is_finite()
            || !p01.y.0.is_finite()
            || !p11.x.0.is_finite()
            || !p11.y.0.is_finite()
        {
            continue;
        }

        let left = p00.x.0.min(p10.x.0).min(p01.x.0).min(p11.x.0);
        let right = p00.x.0.max(p10.x.0).max(p01.x.0).max(p11.x.0);
        let top = p00.y.0.min(p10.y.0).min(p01.y.0).min(p11.y.0);
        let bottom = p00.y.0.max(p10.y.0).max(p01.y.0).max(p11.y.0);

        if !left.is_finite()
            || !right.is_finite()
            || !top.is_finite()
            || !bottom.is_finite()
            || right <= left
            || bottom <= top
        {
            continue;
        }

        let a = Point::new(Px(left), Px(top));
        let b = Point::new(Px(right), Px(top));
        let c = Point::new(Px(right), Px(bottom));
        let d = Point::new(Px(left), Px(bottom));

        out.push(PathCommand::MoveTo(a));
        out.push(PathCommand::LineTo(b));
        out.push(PathCommand::LineTo(c));
        out.push(PathCommand::LineTo(d));
        out.push(PathCommand::Close);
    }

    out
}

pub(super) fn error_bars_commands_from_series(
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
    shape: crate::models::MarkerShape,
) {
    let x = x.0;
    let y = y.0;
    let r = radius.0.max(0.0);
    if !x.is_finite() || !y.is_finite() || !r.is_finite() || r <= 0.0 {
        return;
    }

    match shape {
        crate::models::MarkerShape::Plus => {
            out.push(PathCommand::MoveTo(Point::new(Px(x - r), Px(y))));
            out.push(PathCommand::LineTo(Point::new(Px(x + r), Px(y))));
            out.push(PathCommand::MoveTo(Point::new(Px(x), Px(y - r))));
            out.push(PathCommand::LineTo(Point::new(Px(x), Px(y + r))));
        }
        crate::models::MarkerShape::X => {
            out.push(PathCommand::MoveTo(Point::new(Px(x - r), Px(y - r))));
            out.push(PathCommand::LineTo(Point::new(Px(x + r), Px(y + r))));
            out.push(PathCommand::MoveTo(Point::new(Px(x - r), Px(y + r))));
            out.push(PathCommand::LineTo(Point::new(Px(x + r), Px(y - r))));
        }
        crate::models::MarkerShape::Square => {
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
        crate::models::MarkerShape::Diamond => {
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
        crate::models::MarkerShape::TriangleUp => {
            let p0 = Point::new(Px(x), Px(y - r));
            let p1 = Point::new(Px(x + r), Px(y + r));
            let p2 = Point::new(Px(x - r), Px(y + r));
            out.push(PathCommand::MoveTo(p0));
            out.push(PathCommand::LineTo(p1));
            out.push(PathCommand::LineTo(p2));
            out.push(PathCommand::LineTo(p0));
        }
        crate::models::MarkerShape::TriangleDown => {
            let p0 = Point::new(Px(x), Px(y + r));
            let p1 = Point::new(Px(x + r), Px(y - r));
            let p2 = Point::new(Px(x - r), Px(y - r));
            out.push(PathCommand::MoveTo(p0));
            out.push(PathCommand::LineTo(p1));
            out.push(PathCommand::LineTo(p2));
            out.push(PathCommand::LineTo(p0));
        }
        crate::models::MarkerShape::Circle => {
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

pub(super) fn area_fill_commands_from_polyline(
    polyline: &[PathCommand],
    baseline_y: Px,
) -> Vec<PathCommand> {
    if polyline.is_empty() || !baseline_y.0.is_finite() {
        return Vec::new();
    }

    let mut out: Vec<PathCommand> = Vec::new();
    let mut segment: Vec<Point> = Vec::new();

    let mut flush_segment = |segment: &mut Vec<Point>| {
        if segment.len() < 2 {
            segment.clear();
            return;
        }

        let first = segment[0];
        let last = *segment.last().expect("len>=2");
        out.push(PathCommand::MoveTo(Point::new(first.x, baseline_y)));
        out.push(PathCommand::LineTo(first));
        for point in segment.iter().copied().skip(1) {
            out.push(PathCommand::LineTo(point));
        }
        out.push(PathCommand::LineTo(Point::new(last.x, baseline_y)));
        out.push(PathCommand::Close);
        segment.clear();
    };

    for command in polyline {
        match *command {
            PathCommand::MoveTo(point) => {
                flush_segment(&mut segment);
                segment.push(point);
            }
            PathCommand::LineTo(point) => {
                segment.push(point);
            }
            _ => {}
        }
    }

    flush_segment(&mut segment);
    out
}

pub(super) fn shaded_band_commands_from_series(
    transform: PlotTransform,
    upper: &dyn crate::series::SeriesData,
    lower: &dyn crate::series::SeriesData,
) -> (Vec<PathCommand>, Vec<PathCommand>, Vec<PathCommand>) {
    let mut upper_commands = Vec::<PathCommand>::new();
    let mut lower_commands = Vec::<PathCommand>::new();
    let mut fill_commands = Vec::<PathCommand>::new();
    let mut segment = Vec::<(Point, Point)>::new();

    let mut flush_segment = |segment: &mut Vec<(Point, Point)>| {
        if segment.len() < 2 {
            segment.clear();
            return;
        }

        let first = segment[0];
        upper_commands.push(PathCommand::MoveTo(first.0));
        lower_commands.push(PathCommand::MoveTo(first.1));
        for (upper, lower) in segment.iter().copied().skip(1) {
            upper_commands.push(PathCommand::LineTo(upper));
            lower_commands.push(PathCommand::LineTo(lower));
        }

        fill_commands.push(PathCommand::MoveTo(first.0));
        for (upper, _) in segment.iter().copied().skip(1) {
            fill_commands.push(PathCommand::LineTo(upper));
        }
        for (_, lower) in segment.iter().rev().copied() {
            fill_commands.push(PathCommand::LineTo(lower));
        }
        fill_commands.push(PathCommand::Close);
        segment.clear();
    };

    if upper.is_sorted_by_x() && lower.is_sorted_by_x() {
        #[derive(Clone, Copy)]
        struct Cursor {
            idx: usize,
            prev: Option<DataPoint>,
            next: Option<DataPoint>,
        }

        impl Cursor {
            fn new() -> Self {
                Self {
                    idx: 0,
                    prev: None,
                    next: None,
                }
            }

            fn fetch_next(&mut self, series: &dyn crate::series::SeriesData) -> Option<DataPoint> {
                while self.idx < series.len() {
                    let idx = self.idx;
                    self.idx = self.idx.saturating_add(1);
                    let Some(point) = series.get(idx) else {
                        self.prev = None;
                        continue;
                    };
                    if !point.x.is_finite() || !point.y.is_finite() {
                        self.prev = None;
                        continue;
                    }
                    return Some(point);
                }
                None
            }

            fn ensure_next(&mut self, series: &dyn crate::series::SeriesData) {
                if self.next.is_none() {
                    self.next = self.fetch_next(series);
                }
            }

            fn next_x(&self) -> Option<f64> {
                self.next.map(|point| point.x)
            }

            fn starts_segment_at(&self, x: f64) -> bool {
                self.prev.is_none() && self.next_x().is_some_and(|next_x| next_x == x)
            }

            fn advance_if_at_x(&mut self, series: &dyn crate::series::SeriesData, x: f64) {
                if self.next_x().is_some_and(|next_x| next_x == x) {
                    self.prev = self.next;
                    self.next = self.fetch_next(series);
                }
            }

            fn sample_y(&self, x: f64) -> Option<f64> {
                if !x.is_finite() {
                    return None;
                }

                if let Some(next) = self.next
                    && next.x == x
                {
                    return Some(next.y);
                }

                match (self.prev, self.next) {
                    (Some(a), Some(b)) => {
                        if x < a.x || x > b.x {
                            return None;
                        }
                        let dx = b.x - a.x;
                        if dx == 0.0 || !dx.is_finite() {
                            return Some(b.y);
                        }
                        let t = (x - a.x) / dx;
                        if !t.is_finite() {
                            return None;
                        }
                        let y = a.y + (b.y - a.y) * t;
                        y.is_finite().then_some(y)
                    }
                    (Some(a), None) => (a.x == x).then_some(a.y),
                    (None, Some(b)) => (b.x == x).then_some(b.y),
                    (None, None) => None,
                }
            }
        }

        let mut upper_cursor = Cursor::new();
        let mut lower_cursor = Cursor::new();
        upper_cursor.ensure_next(upper);
        lower_cursor.ensure_next(lower);

        loop {
            let x = match (upper_cursor.next_x(), lower_cursor.next_x()) {
                (Some(a), Some(b)) => a.min(b),
                (Some(a), None) => a,
                (None, Some(b)) => b,
                (None, None) => break,
            };

            if !x.is_finite() {
                flush_segment(&mut segment);
                break;
            }

            if x < transform.data.x_min || x > transform.data.x_max {
                upper_cursor.advance_if_at_x(upper, x);
                lower_cursor.advance_if_at_x(lower, x);
                upper_cursor.ensure_next(upper);
                lower_cursor.ensure_next(lower);
                continue;
            }

            let starts_new_segment =
                upper_cursor.starts_segment_at(x) || lower_cursor.starts_segment_at(x);
            if starts_new_segment && !segment.is_empty() {
                flush_segment(&mut segment);
            }

            let (Some(upper_y), Some(lower_y)) =
                (upper_cursor.sample_y(x), lower_cursor.sample_y(x))
            else {
                flush_segment(&mut segment);
                upper_cursor.advance_if_at_x(upper, x);
                lower_cursor.advance_if_at_x(lower, x);
                upper_cursor.ensure_next(upper);
                lower_cursor.ensure_next(lower);
                continue;
            };
            let upper_px = transform.data_to_px(DataPoint { x, y: upper_y });
            let lower_px = transform.data_to_px(DataPoint { x, y: lower_y });
            if upper_px.x.0.is_finite()
                && upper_px.y.0.is_finite()
                && lower_px.x.0.is_finite()
                && lower_px.y.0.is_finite()
            {
                segment.push((upper_px, lower_px));
            } else {
                flush_segment(&mut segment);
            }

            upper_cursor.advance_if_at_x(upper, x);
            lower_cursor.advance_if_at_x(lower, x);
            upper_cursor.ensure_next(upper);
            lower_cursor.ensure_next(lower);
        }

        flush_segment(&mut segment);
        return (fill_commands, upper_commands, lower_commands);
    }

    let len = upper.len().min(lower.len());
    for index in 0..len {
        let (Some(upper_point), Some(lower_point)) = (upper.get(index), lower.get(index)) else {
            flush_segment(&mut segment);
            continue;
        };
        if !upper_point.x.is_finite()
            || !upper_point.y.is_finite()
            || !lower_point.x.is_finite()
            || !lower_point.y.is_finite()
            || upper_point.x != lower_point.x
        {
            flush_segment(&mut segment);
            continue;
        }
        let upper_px = transform.data_to_px(upper_point);
        let lower_px = transform.data_to_px(lower_point);
        if upper_px.x.0.is_finite()
            && upper_px.y.0.is_finite()
            && lower_px.x.0.is_finite()
            && lower_px.y.0.is_finite()
        {
            segment.push((upper_px, lower_px));
        } else {
            flush_segment(&mut segment);
        }
    }
    flush_segment(&mut segment);

    (fill_commands, upper_commands, lower_commands)
}

pub(super) fn stems_commands_from_points(
    transform: PlotTransform,
    points: &[DataPoint],
    baseline: f32,
) -> Vec<PathCommand> {
    let Some(baseline_y) = transform.data_y_to_px(f64::from(baseline)) else {
        return Vec::new();
    };
    if !baseline_y.0.is_finite() {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(points.len().saturating_mul(2));
    for point in points {
        if !point.x.is_finite() || !point.y.is_finite() {
            continue;
        }
        let px = transform.data_to_px(*point);
        if !px.x.0.is_finite() || !px.y.0.is_finite() {
            continue;
        }
        out.push(PathCommand::MoveTo(Point::new(px.x, baseline_y)));
        out.push(PathCommand::LineTo(px));
    }
    out
}

pub(super) fn step_commands_from_polyline(
    polyline: &[PathCommand],
    step_mode: StepMode,
) -> Vec<PathCommand> {
    if polyline.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<PathCommand> = Vec::with_capacity(polyline.len().saturating_mul(2));
    let mut last: Option<Point> = None;

    for cmd in polyline {
        match *cmd {
            PathCommand::MoveTo(p) => {
                out.push(PathCommand::MoveTo(p));
                last = Some(p);
            }
            PathCommand::LineTo(p) => {
                let Some(prev) = last else {
                    out.push(PathCommand::MoveTo(p));
                    last = Some(p);
                    continue;
                };

                let mid = match step_mode {
                    StepMode::Pre => Point::new(prev.x, p.y),
                    StepMode::Post => Point::new(p.x, prev.y),
                };

                if mid != prev {
                    out.push(PathCommand::LineTo(mid));
                }
                if p != mid {
                    out.push(PathCommand::LineTo(p));
                }
                last = Some(p);
            }
            _ => {}
        }
    }

    out
}
