use std::fmt;
use std::sync::Arc;

use crate::cartesian::{DataPoint, DataRect};

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

/// A getter-backed series (zero-copy from caller data).
pub struct GetterSeriesData {
    len: usize,
    get: Arc<dyn Fn(usize) -> DataPoint + Send + Sync + 'static>,
    sorted_by_x: bool,
    bounds: Option<DataRect>,
}

impl GetterSeriesData {
    pub fn new(len: usize, get: impl Fn(usize) -> DataPoint + Send + Sync + 'static) -> Self {
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
        (index < self.len).then(|| (self.get)(index))
    }

    fn bounds_hint(&self) -> Option<DataRect> {
        self.bounds
    }

    fn is_sorted_by_x(&self) -> bool {
        self.sorted_by_x
    }
}
