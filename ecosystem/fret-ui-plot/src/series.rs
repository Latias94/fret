use std::fmt;
use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::cartesian::{DataPoint, DataRect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeriesId(pub u64);

impl SeriesId {
    /// Returns a deterministic 64-bit ID derived from the label bytes.
    ///
    /// This mirrors the common plot ecosystem convention (ImPlot / egui_plot) where "item identity"
    /// is derived from user-provided labels/IDs so that interaction state (hidden items, hover, etc)
    /// can be stored outside the dataset.
    pub fn from_label(label: &str) -> Self {
        // 64-bit FNV-1a
        const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
        const PRIME: u64 = 0x100000001b3;

        let mut hash = OFFSET_BASIS;
        for b in label.as_bytes() {
            hash ^= u64::from(*b);
            hash = hash.wrapping_mul(PRIME);
        }
        Self(hash)
    }
}

/// A data source for a single XY series.
///
/// This is intentionally ImPlot-like: callers can provide a slice-backed series or a getter-based
/// series to avoid allocating/duplicating large datasets in UI code.
pub trait SeriesData: Send + Sync + 'static {
    /// Total number of logical points in the series.
    fn len(&self) -> usize;

    /// Returns the point at `index`, or `None` if the point is missing.
    ///
    /// Conventions:
    /// - `None` behaves like a discontinuity (breaks the polyline).
    /// - Non-finite values (NaN/Inf) also behave like discontinuities.
    fn get(&self, index: usize) -> Option<DataPoint>;

    /// Optional fast-path for slice-backed data.
    fn as_slice(&self) -> Option<&[DataPoint]> {
        None
    }

    /// Optional bounds hint for the series.
    ///
    /// This is used to avoid scanning large datasets (especially getter-backed series) when
    /// computing plot domains. Implementations should ignore non-finite points when computing the
    /// hint.
    fn bounds_hint(&self) -> Option<DataRect> {
        None
    }

    /// Optional hint that points are sorted by X.
    ///
    /// This can enable faster hit testing and downsampling strategies.
    fn is_sorted_by_x(&self) -> bool {
        false
    }

    /// Optional view-range sampling hook.
    ///
    /// This is intended for generator-like series or data sources that can provide fast,
    /// view-dependent sampling (e.g. functions, streaming time-series windows, chunked stores).
    ///
    /// Returning `Some(vec)` indicates the series can be represented by the returned points within
    /// the provided `x_range`. The plot layer may still apply further decimation in screen space.
    ///
    /// The returned vector is expected to be sorted by X (monotonic) if it represents a polyline.
    fn sample_range(
        &self,
        _x_range: RangeInclusive<f64>,
        _budget: usize,
    ) -> Option<Vec<DataPoint>> {
        None
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A shared handle to series data.
#[derive(Clone)]
pub struct Series(Arc<dyn SeriesData>);

impl Series {
    pub fn new(data: impl SeriesData) -> Self {
        Self(Arc::new(data))
    }

    pub fn from_points(points: Vec<DataPoint>) -> Self {
        Self::new(OwnedSeriesData::new(points))
    }

    pub fn from_points_sorted(points: Vec<DataPoint>, sorted_by_x: bool) -> Self {
        let bounds = DataRect::from_points(points.iter().copied());
        Self::new(OwnedSeriesData {
            points,
            sorted_by_x,
            bounds,
        })
    }

    pub fn from_shared_points(points: Arc<[DataPoint]>) -> Self {
        Self::new(SharedSeriesData::new(points))
    }

    pub fn from_shared_points_sorted(points: Arc<[DataPoint]>, sorted_by_x: bool) -> Self {
        let bounds = DataRect::from_points(points.iter().copied());
        Self::new(SharedSeriesData {
            points,
            sorted_by_x,
            bounds,
        })
    }

    pub fn from_getter(
        len: usize,
        get: impl Fn(usize) -> Option<DataPoint> + Send + Sync + 'static,
    ) -> Self {
        Self::new(GetterSeriesData::new(len, get))
    }

    /// Draw a line based on a function `y=f(x)` with an (optionally infinite) X range and a point
    /// budget.
    ///
    /// This mirrors `egui_plot::PlotPoints::from_explicit_callback` and is the recommended way to
    /// plot function-like series without allocating a full backing buffer.
    pub fn from_explicit_callback(
        function: impl Fn(f64) -> f64 + Send + Sync + 'static,
        x_range: RangeInclusive<f64>,
        points: usize,
    ) -> Self {
        Self::new(GeneratedSeriesData::new(function, x_range, points))
    }
}

impl fmt::Debug for Series {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Series")
            .field("len", &self.len())
            .field("sorted_by_x", &self.is_sorted_by_x())
            .finish()
    }
}

impl std::ops::Deref for Series {
    type Target = dyn SeriesData;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

/// A slice-backed series.
#[derive(Debug, Clone)]
pub struct OwnedSeriesData {
    points: Vec<DataPoint>,
    sorted_by_x: bool,
    bounds: Option<DataRect>,
}

impl OwnedSeriesData {
    pub fn new(points: Vec<DataPoint>) -> Self {
        let bounds = DataRect::from_points(points.iter().copied());
        Self {
            points,
            sorted_by_x: false,
            bounds,
        }
    }
}

impl SeriesData for OwnedSeriesData {
    fn len(&self) -> usize {
        self.points.len()
    }

    fn get(&self, index: usize) -> Option<DataPoint> {
        self.points.get(index).copied()
    }

    fn as_slice(&self) -> Option<&[DataPoint]> {
        Some(&self.points)
    }

    fn bounds_hint(&self) -> Option<DataRect> {
        self.bounds
    }

    fn is_sorted_by_x(&self) -> bool {
        self.sorted_by_x
    }
}

/// A shared slice-backed series (zero-copy via `Arc<[DataPoint]>`).
#[derive(Debug, Clone)]
pub struct SharedSeriesData {
    points: Arc<[DataPoint]>,
    sorted_by_x: bool,
    bounds: Option<DataRect>,
}

impl SharedSeriesData {
    pub fn new(points: Arc<[DataPoint]>) -> Self {
        let bounds = DataRect::from_points(points.iter().copied());
        Self {
            points,
            sorted_by_x: false,
            bounds,
        }
    }

    pub fn sorted_by_x(mut self, sorted: bool) -> Self {
        self.sorted_by_x = sorted;
        self
    }

    pub fn bounds_hint(mut self, bounds: DataRect) -> Self {
        self.bounds = Some(bounds);
        self
    }
}

impl SeriesData for SharedSeriesData {
    fn len(&self) -> usize {
        self.points.len()
    }

    fn get(&self, index: usize) -> Option<DataPoint> {
        self.points.get(index).copied()
    }

    fn as_slice(&self) -> Option<&[DataPoint]> {
        Some(&self.points)
    }

    fn bounds_hint(&self) -> Option<DataRect> {
        self.bounds
    }

    fn is_sorted_by_x(&self) -> bool {
        self.sorted_by_x
    }
}

/// A getter-backed series (zero-copy from caller data).
pub struct GetterSeriesData {
    len: usize,
    get: Arc<dyn Fn(usize) -> Option<DataPoint> + Send + Sync + 'static>,
    sorted_by_x: bool,
    bounds: Option<DataRect>,
}

impl GetterSeriesData {
    pub fn new(
        len: usize,
        get: impl Fn(usize) -> Option<DataPoint> + Send + Sync + 'static,
    ) -> Self {
        Self {
            len,
            get: Arc::new(get),
            sorted_by_x: false,
            bounds: None,
        }
    }

    pub fn sorted_by_x(mut self, sorted: bool) -> Self {
        self.sorted_by_x = sorted;
        self
    }

    pub fn bounds_hint(mut self, bounds: DataRect) -> Self {
        self.bounds = Some(bounds);
        self
    }
}

impl fmt::Debug for GetterSeriesData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GetterSeriesData")
            .field("len", &self.len)
            .field("sorted_by_x", &self.sorted_by_x)
            .field("bounds", &self.bounds)
            .finish()
    }
}

impl SeriesData for GetterSeriesData {
    fn len(&self) -> usize {
        self.len
    }

    fn get(&self, index: usize) -> Option<DataPoint> {
        (index < self.len).then(|| (self.get)(index)).flatten()
    }

    fn bounds_hint(&self) -> Option<DataRect> {
        self.bounds
    }

    fn is_sorted_by_x(&self) -> bool {
        self.sorted_by_x
    }
}

/// A generator-backed series (explicit callback `y=f(x)`).
#[derive(Clone)]
pub struct GeneratedSeriesData {
    function: Arc<dyn Fn(f64) -> f64 + Send + Sync + 'static>,
    x_range: RangeInclusive<f64>,
    points: usize,
    bounds: Option<DataRect>,
}

impl GeneratedSeriesData {
    pub fn new(
        function: impl Fn(f64) -> f64 + Send + Sync + 'static,
        x_range: RangeInclusive<f64>,
        points: usize,
    ) -> Self {
        let function = Arc::new(function);
        let points = points.max(2);
        let bounds = estimate_generated_bounds(&*function, &x_range);
        Self {
            function,
            x_range,
            points,
            bounds,
        }
    }

    fn intersection(
        a: &RangeInclusive<f64>,
        b: &RangeInclusive<f64>,
    ) -> Option<RangeInclusive<f64>> {
        let (a0, a1) = (*a.start(), *a.end());
        let (b0, b1) = (*b.start(), *b.end());
        // Allow +/-Infinity (for "unbounded" ranges) but reject NaN.
        if a0.is_nan() || a1.is_nan() || b0.is_nan() || b1.is_nan() {
            return None;
        }

        let a_min = a0.min(a1);
        let a_max = a0.max(a1);
        let b_min = b0.min(b1);
        let b_max = b0.max(b1);

        let lo = a_min.max(b_min);
        let hi = a_max.min(b_max);
        (lo <= hi).then_some(lo..=hi)
    }
}

impl SeriesData for GeneratedSeriesData {
    fn len(&self) -> usize {
        self.points
    }

    fn get(&self, index: usize) -> Option<DataPoint> {
        if index >= self.points {
            return None;
        }

        let x0 = *self.x_range.start();
        let x1 = *self.x_range.end();
        if !x0.is_finite() || !x1.is_finite() {
            return None;
        }

        let denom = (self.points - 1) as f64;
        let t = (index as f64) / denom;
        let x = x0 + (x1 - x0) * t;
        let y = (self.function)(x);
        Some(DataPoint { x, y })
    }

    fn bounds_hint(&self) -> Option<DataRect> {
        self.bounds
    }

    fn is_sorted_by_x(&self) -> bool {
        let x0 = *self.x_range.start();
        let x1 = *self.x_range.end();
        x0.is_finite() && x1.is_finite()
    }

    fn sample_range(&self, x_range: RangeInclusive<f64>, budget: usize) -> Option<Vec<DataPoint>> {
        let Some(view) = Self::intersection(&self.x_range, &x_range) else {
            // Returning `Some(empty)` means "handled by generator but nothing to draw here".
            return Some(Vec::new());
        };

        let x0 = *view.start();
        let x1 = *view.end();
        if !x0.is_finite() || !x1.is_finite() {
            return Some(Vec::new());
        }

        let n = budget.max(2).min(self.points).min(8192);
        if n == 0 {
            return Some(Vec::new());
        }

        let denom = (n - 1) as f64;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let t = (i as f64) / denom;
            let x = x0 + (x1 - x0) * t;
            let y = (self.function)(x);
            out.push(DataPoint { x, y });
        }
        Some(out)
    }
}

fn estimate_generated_bounds(
    f: &dyn Fn(f64) -> f64,
    x_range: &RangeInclusive<f64>,
) -> Option<DataRect> {
    let x0 = *x_range.start();
    let x1 = *x_range.end();
    if !x0.is_finite() || !x1.is_finite() {
        return None;
    }
    let (min_x, max_x) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
    if min_x == max_x {
        let y = f(min_x);
        if min_x.is_finite() && y.is_finite() {
            return Some(DataRect {
                x_min: min_x,
                x_max: max_x,
                y_min: y,
                y_max: y,
            });
        }
        return None;
    }

    let mut x_min: Option<f64> = None;
    let mut x_max: Option<f64> = None;
    let mut y_min: Option<f64> = None;
    let mut y_max: Option<f64> = None;

    let mut add = |x: f64| {
        if !x.is_finite() {
            return;
        }
        let y = f(x);
        if !y.is_finite() {
            return;
        }
        x_min = Some(x_min.map_or(x, |v| v.min(x)));
        x_max = Some(x_max.map_or(x, |v| v.max(x)));
        y_min = Some(y_min.map_or(y, |v| v.min(y)));
        y_max = Some(y_max.map_or(y, |v| v.max(y)));
    };

    add(min_x);
    add(max_x);

    // Sample a small number of points for an initial auto-bounds estimate.
    const N: usize = 8;
    for i in 1..N {
        let t = (i as f64) / ((N - 1) as f64);
        add(min_x + (max_x - min_x) * t);
    }

    Some(DataRect {
        x_min: x_min?,
        x_max: x_max?,
        y_min: y_min?,
        y_max: y_max?,
    })
}
