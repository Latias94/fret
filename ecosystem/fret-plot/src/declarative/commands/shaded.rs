//! Declarative shaded-band path-command projection owner.

use fret_core::{PathCommand, Point};

use crate::cartesian::{DataPoint, PlotTransform};

pub(in crate::declarative) fn line_plot_shaded_lower_path_key(series_id: u64) -> u64 {
    0x706c_6f74_7368_6164_u64 ^ series_id
}

pub(in crate::declarative) fn shaded_band_commands_from_series(
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
