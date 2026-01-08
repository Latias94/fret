#[derive(Debug, Clone)]
pub(crate) struct HistogramBins {
    pub(crate) x_min: f64,
    pub(crate) x_max: f64,
    pub(crate) bin_width: f64,
    pub(crate) counts: Vec<f64>,
}

impl HistogramBins {
    pub(crate) fn len(&self) -> usize {
        self.counts.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    pub(crate) fn center_x(&self, bin: usize) -> f64 {
        self.x_min + (bin as f64 + 0.5) * self.bin_width
    }

    pub(crate) fn max_count(&self) -> f64 {
        self.counts
            .iter()
            .copied()
            .fold(0.0, |acc, v| if v > acc { v } else { acc })
    }
}

pub(crate) fn histogram_bins(
    values: &[f64],
    bin_count: usize,
    range: Option<(f64, f64)>,
) -> Option<HistogramBins> {
    let bin_count = bin_count.clamp(1, 16_384);

    let (x_min, x_max) = if let Some((min, max)) = range {
        (min, max)
    } else {
        let mut min: Option<f64> = None;
        let mut max: Option<f64> = None;
        for v in values.iter().copied() {
            if !v.is_finite() {
                continue;
            }
            min = Some(min.map_or(v, |m| m.min(v)));
            max = Some(max.map_or(v, |m| m.max(v)));
        }
        (min?, max?)
    };

    if !x_min.is_finite() || !x_max.is_finite() || x_max <= x_min {
        return None;
    }

    let width = (x_max - x_min) / bin_count as f64;
    if !width.is_finite() || width <= 0.0 {
        return None;
    }

    let mut counts = vec![0.0f64; bin_count];
    for v in values.iter().copied() {
        if !v.is_finite() {
            continue;
        }
        if v < x_min || v > x_max {
            continue;
        }

        let mut idx = ((v - x_min) / width).floor() as isize;
        if idx == bin_count as isize {
            idx = bin_count.saturating_sub(1) as isize;
        }
        if idx < 0 {
            continue;
        }
        let idx = idx as usize;
        if idx >= bin_count {
            continue;
        }
        counts[idx] += 1.0;
    }

    Some(HistogramBins {
        x_min,
        x_max,
        bin_width: width,
        counts,
    })
}
