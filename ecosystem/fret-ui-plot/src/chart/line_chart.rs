use fret_runtime::Model;
use fret_runtime::ModelHost;

use crate::cartesian::{DataPoint, DataRect};
use crate::retained::{LinePlotCanvas, LinePlotModel};
use crate::series::Series;

pub struct LineChart<T> {
    data: Vec<T>,
    x: Box<dyn Fn(&T) -> Option<f32>>,
    y: Box<dyn Fn(&T) -> Option<f32>>,
    clamp_empty_domain: bool,
}

impl<T> LineChart<T> {
    pub fn new(data: impl IntoIterator<Item = T>) -> Self {
        Self {
            data: data.into_iter().collect(),
            x: Box::new(|_| None),
            y: Box::new(|_| None),
            clamp_empty_domain: true,
        }
    }

    pub fn x(mut self, f: impl Fn(&T) -> Option<f32> + 'static) -> Self {
        self.x = Box::new(f);
        self
    }

    pub fn y(mut self, f: impl Fn(&T) -> Option<f32> + 'static) -> Self {
        self.y = Box::new(f);
        self
    }

    /// If the data has no valid points, clamp the domain to a small non-zero box to avoid
    /// degenerate transforms.
    pub fn clamp_empty_domain(mut self, enabled: bool) -> Self {
        self.clamp_empty_domain = enabled;
        self
    }

    pub fn build_model(self) -> LinePlotModel {
        let mut points: Vec<DataPoint> = Vec::with_capacity(self.data.len());
        let mut x_min: Option<f32> = None;
        let mut x_max: Option<f32> = None;
        let mut y_min: Option<f32> = None;
        let mut y_max: Option<f32> = None;
        let mut sorted_by_x = true;
        let mut last_x: Option<f32> = None;

        for item in &self.data {
            let Some(x) = (self.x)(item) else {
                continue;
            };
            let Some(y) = (self.y)(item) else {
                continue;
            };
            if !x.is_finite() || !y.is_finite() {
                continue;
            }

            x_min = Some(x_min.map_or(x, |v| v.min(x)));
            x_max = Some(x_max.map_or(x, |v| v.max(x)));
            y_min = Some(y_min.map_or(y, |v| v.min(y)));
            y_max = Some(y_max.map_or(y, |v| v.max(y)));

            if let Some(prev) = last_x {
                if x < prev {
                    sorted_by_x = false;
                }
            }
            last_x = Some(x);

            points.push(DataPoint { x, y });
        }

        let bounds = match (x_min, x_max, y_min, y_max) {
            (Some(x_min), Some(x_max), Some(y_min), Some(y_max)) => DataRect {
                x_min,
                x_max,
                y_min,
                y_max,
            },
            _ if self.clamp_empty_domain => DataRect {
                x_min: 0.0,
                x_max: 1.0,
                y_min: 0.0,
                y_max: 1.0,
            },
            _ => DataRect {
                x_min: 0.0,
                x_max: 0.0,
                y_min: 0.0,
                y_max: 0.0,
            },
        };

        LinePlotModel {
            data_bounds: bounds,
            points: Series::from_points_sorted(points, sorted_by_x),
        }
    }

    pub fn install(self, host: &mut impl ModelHost) -> Model<LinePlotModel> {
        host.models_mut().insert(self.build_model())
    }

    pub fn into_canvas(self, host: &mut impl ModelHost) -> LinePlotCanvas {
        let model = self.install(host);
        LinePlotCanvas::new(model)
    }
}
