use fret_core::PathCommand;
use fret_core::geometry::{Point, Px};

use crate::cartesian::{DataPoint, PlotTransform};
use crate::series::{SeriesData, SeriesId};

use super::SamplePoint;

#[derive(Debug, Clone, Copy, PartialEq)]
struct BandPoint {
    index: usize,
    upper: DataPoint,
    lower: DataPoint,
    upper_px: Point,
    lower_px: Point,
}

pub(crate) fn decimate_shaded_band(
    transform: PlotTransform,
    upper: &dyn SeriesData,
    lower: &dyn SeriesData,
    scale_factor: f32,
    series_id: SeriesId,
) -> (
    Vec<PathCommand>,
    Vec<PathCommand>,
    Vec<PathCommand>,
    Vec<SamplePoint>,
) {
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

        fn next_x(&self) -> Option<f64> {
            self.next.map(|p| p.x)
        }

        fn is_segment_start_at_x(&self, x: f64) -> bool {
            self.prev.is_none() && self.next_x().is_some_and(|nx| nx == x)
        }

        fn fetch_next(&mut self, series: &dyn SeriesData) -> Option<DataPoint> {
            while self.idx < series.len() {
                let idx = self.idx;
                self.idx += 1;

                let Some(p) = series.get(idx) else {
                    self.prev = None;
                    continue;
                };
                if !p.x.is_finite() || !p.y.is_finite() {
                    self.prev = None;
                    continue;
                }
                return Some(p);
            }
            None
        }

        fn ensure_next(&mut self, series: &dyn SeriesData) {
            if self.next.is_none() {
                self.next = self.fetch_next(series);
            }
        }

        fn advance_if_at_x(&mut self, series: &dyn SeriesData, x: f64) {
            if self.next_x().is_some_and(|nx| nx == x) {
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

    struct BandDecimator {
        series_id: SeriesId,
        scale_factor: f32,
        fill_commands: Vec<PathCommand>,
        upper_commands: Vec<PathCommand>,
        lower_commands: Vec<PathCommand>,
        samples: Vec<SamplePoint>,
        decimated: Vec<BandPoint>,
        current_bucket: Option<i32>,
        min_upper: Option<BandPoint>,
        max_upper: Option<BandPoint>,
        min_lower: Option<BandPoint>,
        max_lower: Option<BandPoint>,
        last_emitted_idx: Option<usize>,
        last_emitted_upper_px: Option<Point>,
        last_emitted_lower_px: Option<Point>,
    }

    impl BandDecimator {
        fn new(series_id: SeriesId, scale_factor: f32) -> Self {
            Self {
                series_id,
                scale_factor,
                fill_commands: Vec::new(),
                upper_commands: Vec::new(),
                lower_commands: Vec::new(),
                samples: Vec::new(),
                decimated: Vec::new(),
                current_bucket: None,
                min_upper: None,
                max_upper: None,
                min_lower: None,
                max_lower: None,
                last_emitted_idx: None,
                last_emitted_upper_px: None,
                last_emitted_lower_px: None,
            }
        }

        fn bucket_of(&self, x: Px) -> i32 {
            let x = x.0 * self.scale_factor.max(1.0);
            if !x.is_finite() { 0 } else { x.floor() as i32 }
        }

        fn emit_segment(&mut self) {
            if self.decimated.len() < 2 {
                self.decimated.clear();
                return;
            }

            let first = self.decimated[0];
            self.upper_commands
                .push(PathCommand::MoveTo(first.upper_px));
            self.lower_commands
                .push(PathCommand::MoveTo(first.lower_px));

            for p in self.decimated.iter().copied().skip(1) {
                self.upper_commands.push(PathCommand::LineTo(p.upper_px));
                self.lower_commands.push(PathCommand::LineTo(p.lower_px));
            }

            self.fill_commands
                .push(PathCommand::MoveTo(self.decimated[0].upper_px));
            for p in self.decimated.iter().copied().skip(1) {
                self.fill_commands.push(PathCommand::LineTo(p.upper_px));
            }
            for p in self.decimated.iter().rev().copied() {
                self.fill_commands.push(PathCommand::LineTo(p.lower_px));
            }
            self.fill_commands.push(PathCommand::Close);

            for p in self.decimated.iter().copied() {
                let connects_to_prev = p.index != first.index;
                self.samples.push(SamplePoint {
                    series_id: self.series_id,
                    index: p.index,
                    data: p.upper,
                    plot_px: p.upper_px,
                    connects_to_prev,
                });
            }

            for p in self.decimated.iter().copied() {
                let connects_to_prev = p.index != first.index;
                self.samples.push(SamplePoint {
                    series_id: self.series_id,
                    index: p.index,
                    data: p.lower,
                    plot_px: p.lower_px,
                    connects_to_prev,
                });
            }

            self.decimated.clear();
        }

        fn emit_decimated_point(&mut self, p: BandPoint) {
            if self.last_emitted_idx.is_some_and(|idx| p.index <= idx) {
                return;
            }
            if self
                .last_emitted_upper_px
                .is_some_and(|px| px == p.upper_px)
                && self
                    .last_emitted_lower_px
                    .is_some_and(|px| px == p.lower_px)
            {
                self.last_emitted_idx = Some(p.index);
                return;
            }

            self.decimated.push(p);
            self.last_emitted_idx = Some(p.index);
            self.last_emitted_upper_px = Some(p.upper_px);
            self.last_emitted_lower_px = Some(p.lower_px);
        }

        fn flush_bucket(&mut self) {
            let mut candidates: Vec<BandPoint> = Vec::new();
            for p in [
                self.min_upper,
                self.max_upper,
                self.min_lower,
                self.max_lower,
            ]
            .into_iter()
            .flatten()
            {
                candidates.push(p);
            }

            candidates.sort_by_key(|p| p.index);
            candidates.dedup_by_key(|p| p.index);

            for p in candidates {
                self.emit_decimated_point(p);
            }

            self.min_upper = None;
            self.max_upper = None;
            self.min_lower = None;
            self.max_lower = None;
        }

        fn flush_current_segment(&mut self) {
            if self.current_bucket.is_some() {
                self.flush_bucket();
            }
            self.current_bucket = None;
            self.emit_segment();
            self.last_emitted_idx = None;
            self.last_emitted_upper_px = None;
            self.last_emitted_lower_px = None;
        }

        fn push_point(&mut self, p: BandPoint) {
            let b = self.bucket_of(p.upper_px.x);
            if self.current_bucket != Some(b) {
                if self.current_bucket.is_some() {
                    self.flush_bucket();
                }
                self.current_bucket = Some(b);
                self.min_upper = Some(p);
                self.max_upper = Some(p);
                self.min_lower = Some(p);
                self.max_lower = Some(p);
                self.emit_decimated_point(p);
                return;
            }

            if let Some(m) = self.min_upper
                && p.upper_px.y.0.is_finite()
                && m.upper_px.y.0.is_finite()
                && p.upper_px.y.0 < m.upper_px.y.0
            {
                self.min_upper = Some(p);
            }
            if let Some(m) = self.max_upper
                && p.upper_px.y.0.is_finite()
                && m.upper_px.y.0.is_finite()
                && p.upper_px.y.0 > m.upper_px.y.0
            {
                self.max_upper = Some(p);
            }
            if let Some(m) = self.min_lower
                && p.lower_px.y.0.is_finite()
                && m.lower_px.y.0.is_finite()
                && p.lower_px.y.0 < m.lower_px.y.0
            {
                self.min_lower = Some(p);
            }
            if let Some(m) = self.max_lower
                && p.lower_px.y.0.is_finite()
                && m.lower_px.y.0.is_finite()
                && p.lower_px.y.0 > m.lower_px.y.0
            {
                self.max_lower = Some(p);
            }
        }

        fn finish(
            mut self,
        ) -> (
            Vec<PathCommand>,
            Vec<PathCommand>,
            Vec<PathCommand>,
            Vec<SamplePoint>,
        ) {
            self.flush_current_segment();
            (
                self.fill_commands,
                self.upper_commands,
                self.lower_commands,
                self.samples,
            )
        }
    }

    let mut decimator = BandDecimator::new(series_id, scale_factor);

    if !(upper.is_sorted_by_x() && lower.is_sorted_by_x()) {
        // Fallback: index-aligned shaded band. This expects both series to share X values at each
        // index. Callers should prefer sorted-by-x series for correct interpolation + resampling.
        let len = upper.len().min(lower.len());
        let mut sample_index: usize = 0;

        for idx in 0..len {
            let (Some(upper_dp), Some(lower_dp)) = (upper.get(idx), lower.get(idx)) else {
                decimator.flush_current_segment();
                continue;
            };
            if !upper_dp.x.is_finite()
                || !upper_dp.y.is_finite()
                || !lower_dp.x.is_finite()
                || !lower_dp.y.is_finite()
            {
                decimator.flush_current_segment();
                continue;
            }
            if upper_dp.x != lower_dp.x {
                decimator.flush_current_segment();
                continue;
            }

            let upper_px = transform.data_to_px(upper_dp);
            let lower_px = transform.data_to_px(lower_dp);
            if !upper_px.x.0.is_finite()
                || !upper_px.y.0.is_finite()
                || !lower_px.x.0.is_finite()
                || !lower_px.y.0.is_finite()
            {
                decimator.flush_current_segment();
                continue;
            }

            decimator.push_point(BandPoint {
                index: sample_index,
                upper: upper_dp,
                lower: lower_dp,
                upper_px,
                lower_px,
            });
            sample_index = sample_index.wrapping_add(1);
        }

        return decimator.finish();
    }

    let mut upper_cursor = Cursor::new();
    let mut lower_cursor = Cursor::new();
    upper_cursor.ensure_next(upper);
    lower_cursor.ensure_next(lower);

    let mut sample_index: usize = 0;

    loop {
        let x = match (upper_cursor.next_x(), lower_cursor.next_x()) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => break,
        };

        if !x.is_finite() {
            decimator.flush_current_segment();
            break;
        }

        if x < transform.data.x_min || x > transform.data.x_max {
            upper_cursor.advance_if_at_x(upper, x);
            lower_cursor.advance_if_at_x(lower, x);
            continue;
        }

        let starts_new_segment =
            upper_cursor.is_segment_start_at_x(x) || lower_cursor.is_segment_start_at_x(x);
        if starts_new_segment && !decimator.decimated.is_empty() {
            decimator.flush_current_segment();
        }

        let (Some(upper_y), Some(lower_y)) = (upper_cursor.sample_y(x), lower_cursor.sample_y(x))
        else {
            decimator.flush_current_segment();
            upper_cursor.advance_if_at_x(upper, x);
            lower_cursor.advance_if_at_x(lower, x);
            continue;
        };

        let upper_dp = DataPoint { x, y: upper_y };
        let lower_dp = DataPoint { x, y: lower_y };

        let upper_px = transform.data_to_px(upper_dp);
        let lower_px = transform.data_to_px(lower_dp);
        if !upper_px.x.0.is_finite()
            || !upper_px.y.0.is_finite()
            || !lower_px.x.0.is_finite()
            || !lower_px.y.0.is_finite()
        {
            decimator.flush_current_segment();
            upper_cursor.advance_if_at_x(upper, x);
            lower_cursor.advance_if_at_x(lower, x);
            continue;
        }

        decimator.push_point(BandPoint {
            index: sample_index,
            upper: upper_dp,
            lower: lower_dp,
            upper_px,
            lower_px,
        });
        sample_index = sample_index.wrapping_add(1);

        upper_cursor.advance_if_at_x(upper, x);
        lower_cursor.advance_if_at_x(lower, x);
        upper_cursor.ensure_next(upper);
        lower_cursor.ensure_next(lower);
    }

    decimator.finish()
}
