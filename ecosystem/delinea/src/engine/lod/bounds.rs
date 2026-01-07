use crate::engine::window::DataWindowX;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct DataBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl DataBounds {
    pub fn is_valid(&self) -> bool {
        self.x_min.is_finite()
            && self.x_max.is_finite()
            && self.y_min.is_finite()
            && self.y_max.is_finite()
            && self.x_max > self.x_min
            && self.y_max > self.y_min
    }

    pub fn clamp_non_degenerate(&mut self) {
        if !self.x_min.is_finite() || !self.x_max.is_finite() || self.x_max <= self.x_min {
            self.x_min = 0.0;
            self.x_max = 1.0;
        }
        if !self.y_min.is_finite() || !self.y_max.is_finite() || self.y_max <= self.y_min {
            self.y_min = 0.0;
            self.y_max = 1.0;
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BoundsCursor {
    pub next_index: usize,
    pub saw_any: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct BoundsAccum {
    pub y_min: f64,
    pub y_max: f64,
}

impl BoundsAccum {
    pub fn reset(&mut self) {
        self.y_min = f64::INFINITY;
        self.y_max = f64::NEG_INFINITY;
    }

    pub fn is_valid(&self) -> bool {
        self.y_min.is_finite() && self.y_max.is_finite() && self.y_max > self.y_min
    }
}

pub fn compute_bounds_step(
    cursor: &mut BoundsCursor,
    accum: &mut BoundsAccum,
    x: &[f64],
    y: &[f64],
    window_x: Option<DataWindowX>,
    max_points_to_process: usize,
) -> Option<bool> {
    let len = x.len().min(y.len());
    if cursor.next_index >= len {
        return Some(true);
    }

    let end = (cursor.next_index + max_points_to_process).min(len);

    for i in cursor.next_index..end {
        let xi = x[i];
        let yi = y[i];
        if !xi.is_finite() || !yi.is_finite() {
            continue;
        }

        if let Some(w) = window_x
            && (xi < w.x_min || xi > w.x_max)
        {
            continue;
        }

        cursor.saw_any = true;
        accum.y_min = accum.y_min.min(yi);
        accum.y_max = accum.y_max.max(yi);
    }

    cursor.next_index = end;
    Some(cursor.next_index >= len)
}

pub fn finalize_bounds(accum: &BoundsAccum, window_x: Option<DataWindowX>) -> Option<DataBounds> {
    if !accum.y_min.is_finite() || !accum.y_max.is_finite() {
        return None;
    }

    let (x_min, x_max) = if let Some(mut w) = window_x {
        w.clamp_non_degenerate();
        (w.x_min, w.x_max)
    } else {
        // Caller should set x bounds from a separate pass if needed.
        (0.0, 1.0)
    };

    Some(DataBounds {
        x_min,
        x_max,
        y_min: accum.y_min,
        y_max: accum.y_max,
    })
}
