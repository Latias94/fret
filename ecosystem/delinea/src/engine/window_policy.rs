use crate::engine::window::DataWindow;
use crate::spec::AxisRange;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct AxisFilter1D {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl AxisFilter1D {
    pub fn contains(&self, v: f64) -> bool {
        if let Some(min) = self.min
            && v < min
        {
            return false;
        }
        if let Some(max) = self.max
            && v > max
        {
            return false;
        }
        true
    }

    pub fn as_window(self) -> Option<DataWindow> {
        let (Some(min), Some(max)) = (self.min, self.max) else {
            return None;
        };
        let mut w = DataWindow { min, max };
        w.clamp_non_degenerate();
        Some(w)
    }
}

pub fn axis_filter_1d(range: AxisRange, state_window: Option<DataWindow>) -> AxisFilter1D {
    let mut out = AxisFilter1D::default();

    match range {
        AxisRange::Fixed { min, max } => {
            out.min = Some(min);
            out.max = Some(max);
        }
        AxisRange::Auto | AxisRange::LockMin { .. } | AxisRange::LockMax { .. } => {
            if let Some(mut w) = state_window {
                w.clamp_non_degenerate();
                out.min = Some(w.min);
                out.max = Some(w.max);
            }
        }
    }

    if let Some(min) = range.locked_min() {
        out.min = Some(min);
    }
    if let Some(max) = range.locked_max() {
        out.max = Some(max);
    }

    if let (Some(a), Some(b)) = (out.min, out.max)
        && b < a
    {
        out.min = Some(b);
        out.max = Some(a);
    }

    out
}

pub fn axis_mapping_window_1d(
    range: AxisRange,
    state_window: Option<DataWindow>,
) -> Option<DataWindow> {
    let filter = axis_filter_1d(range, state_window);

    match range {
        AxisRange::Fixed { .. } => filter.as_window(),
        AxisRange::Auto | AxisRange::LockMin { .. } | AxisRange::LockMax { .. } => {
            if state_window.is_some() {
                filter.as_window()
            } else {
                None
            }
        }
    }
}
