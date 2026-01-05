#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisLabelFormat {
    Number(AxisNumberFormat),
    TimeSeconds(TimeAxisFormat),
}

impl Default for AxisLabelFormat {
    fn default() -> Self {
        Self::Number(AxisNumberFormat::Auto)
    }
}

impl AxisLabelFormat {
    pub fn key(self) -> u64 {
        match self {
            Self::Number(f) => 0x4e554d00_0000_0000u64 ^ u64::from(f.key()),
            Self::TimeSeconds(f) => 0x54494d00_0000_0000u64 ^ f.key(),
        }
    }

    pub fn format(self, v: f32, span: f32) -> String {
        match self {
            Self::Number(f) => format_number(v, span, f),
            Self::TimeSeconds(f) => format_time_seconds(v, span, f),
        }
    }
}

pub fn axis_ticks(min: f32, max: f32, tick_count: usize, format: AxisLabelFormat) -> Vec<f32> {
    match format {
        AxisLabelFormat::Number(_) => nice_ticks(min, max, tick_count),
        AxisLabelFormat::TimeSeconds(f) => time_ticks_seconds(min, max, tick_count, f),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisNumberFormat {
    /// A heuristic chosen for plot tick labels (similar to ImPlot defaults).
    Auto,
    /// Fixed number of decimals.
    Fixed(u8),
}

impl AxisNumberFormat {
    pub fn key(self) -> u8 {
        match self {
            Self::Auto => 0,
            Self::Fixed(p) => p.saturating_add(1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeAxisFormat {
    /// Origin used for formatting. Labels show `x - origin_seconds`.
    pub origin_seconds: f64,
}

impl Default for TimeAxisFormat {
    fn default() -> Self {
        Self {
            origin_seconds: 0.0,
        }
    }
}

impl TimeAxisFormat {
    pub fn key(self) -> u64 {
        u64::from(self.origin_seconds.to_bits())
    }
}

fn format_number(v: f32, span: f32, fmt: AxisNumberFormat) -> String {
    if !v.is_finite() {
        return "NA".to_string();
    }

    match fmt {
        AxisNumberFormat::Fixed(p) => {
            let p = p as usize;
            format!("{v:.p$}")
        }
        AxisNumberFormat::Auto => {
            let abs = v.abs();
            let span = span.abs();

            // Prefer higher precision when the view is narrow.
            if span.is_finite() && span < 1.0 {
                return format!("{v:.4}");
            }

            if abs < 1.0 {
                format!("{v:.3}")
            } else if abs < 10.0 {
                format!("{v:.2}")
            } else {
                format!("{v:.1}")
            }
        }
    }
}

fn format_time_seconds(v: f32, span: f32, fmt: TimeAxisFormat) -> String {
    if !v.is_finite() {
        return "NA".to_string();
    }

    let t = (v as f64) - fmt.origin_seconds;
    if !t.is_finite() {
        return "NA".to_string();
    }

    let sign = if t < 0.0 { "-" } else { "" };
    let t = t.abs();

    let span = span.abs();

    if span.is_finite() && span < 1.0 {
        return format!("{sign}{t:.3}s");
    }
    if span.is_finite() && span < 10.0 {
        return format!("{sign}{t:.2}s");
    }
    if span.is_finite() && span < 60.0 {
        return format!("{sign}{t:.1}s");
    }

    let total_seconds = t.floor() as u64;
    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = total_seconds / 3600;

    if hours > 0 {
        format!("{sign}{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{sign}{minutes:02}:{seconds:02}")
    }
}

pub fn nice_ticks(min: f32, max: f32, tick_count: usize) -> Vec<f32> {
    if tick_count == 0 {
        return Vec::new();
    }
    if !min.is_finite() || !max.is_finite() {
        return Vec::new();
    }
    if min == max {
        return vec![min];
    }

    let (min, max) = if min <= max { (min, max) } else { (max, min) };
    let span = (max - min) as f64;
    let target = if tick_count <= 1 {
        span
    } else {
        span / (tick_count.saturating_sub(1) as f64)
    };

    let step = nice_step_125(target).max(f64::EPSILON);
    let first = (min as f64 / step).ceil() * step;
    let last = (max as f64 / step).floor() * step;

    if !first.is_finite() || !last.is_finite() || first > last {
        return vec![min, max]
            .into_iter()
            .filter(|v| v.is_finite())
            .collect();
    }

    let mut out: Vec<f32> = Vec::new();
    let mut v = first;
    let max_steps = 4096usize;
    for _ in 0..max_steps {
        if v > last + step * 0.5 {
            break;
        }
        out.push(v as f32);
        v += step;
    }

    if out.is_empty() {
        vec![min, max]
            .into_iter()
            .filter(|v| v.is_finite())
            .collect()
    } else {
        out
    }
}

fn nice_step_125(x: f64) -> f64 {
    if !x.is_finite() || x <= 0.0 {
        return 1.0;
    }

    let exp = x.abs().log10().floor();
    let base = 10f64.powf(exp);
    let f = x / base;

    let nf = if f <= 1.0 {
        1.0
    } else if f <= 2.0 {
        2.0
    } else if f <= 5.0 {
        5.0
    } else {
        10.0
    };

    nf * base
}

pub fn time_ticks_seconds(min: f32, max: f32, tick_count: usize, fmt: TimeAxisFormat) -> Vec<f32> {
    if tick_count == 0 {
        return Vec::new();
    }
    if !min.is_finite() || !max.is_finite() {
        return Vec::new();
    }
    if min == max {
        return vec![min];
    }

    let (min, max) = if min <= max { (min, max) } else { (max, min) };
    let span = (max - min).abs() as f64;
    if !span.is_finite() || span <= 0.0 {
        return vec![min, max]
            .into_iter()
            .filter(|v| v.is_finite())
            .collect();
    }

    let target = if tick_count <= 1 {
        span
    } else {
        span / (tick_count.saturating_sub(1) as f64)
    };

    const STEPS: &[f64] = &[
        0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0,
        60.0, 120.0, 300.0, 600.0, 900.0, 1800.0, 3600.0, 7200.0, 14400.0, 21600.0, 43200.0,
        86400.0, 172800.0, 604800.0,
    ];

    let mut step = *STEPS.last().unwrap_or(&1.0);
    for s in STEPS {
        if *s >= target {
            step = *s;
            break;
        }
    }

    let origin = fmt.origin_seconds;
    let min_rel = (min as f64) - origin;
    let max_rel = (max as f64) - origin;
    if !min_rel.is_finite() || !max_rel.is_finite() {
        return nice_ticks(min, max, tick_count);
    }

    let first_k = (min_rel / step).ceil();
    let last_k = (max_rel / step).floor();
    if !first_k.is_finite() || !last_k.is_finite() || first_k > last_k {
        return nice_ticks(min, max, tick_count);
    }

    let mut out: Vec<f32> = Vec::new();
    let mut k = first_k;
    let max_steps = 4096usize;
    for _ in 0..max_steps {
        if k > last_k + 0.5 {
            break;
        }
        let v = origin + k * step;
        if v.is_finite() {
            out.push(v as f32);
        }
        k += 1.0;
    }

    if out.is_empty() {
        nice_ticks(min, max, tick_count)
    } else {
        out
    }
}

pub fn linear_ticks(min: f32, max: f32, tick_count: usize) -> Vec<f32> {
    if tick_count == 0 {
        return Vec::new();
    }
    if !min.is_finite() || !max.is_finite() {
        return Vec::new();
    }
    let n = tick_count.max(1);
    if n == 1 {
        return vec![min];
    }
    let denom = (n - 1) as f32;
    (0..n)
        .map(|i| {
            let t = (i as f32) / denom;
            min + (max - min) * t
        })
        .filter(|v| v.is_finite())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_ticks_uses_125_steps() {
        let ticks = nice_ticks(0.12, 1.12, 6);
        assert!(!ticks.is_empty());

        let step = ticks
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(None, |acc: Option<f32>, v| {
                Some(acc.map_or(v, |a| a.min(v)))
            })
            .unwrap_or(0.0);

        // For span ~1.0 and 6 ticks, target step ~0.2 => expect 0.2.
        assert!((step - 0.2).abs() <= 1e-4, "step={step}");
    }

    #[test]
    fn time_ticks_pick_human_steps() {
        let fmt = TimeAxisFormat::default();
        let ticks = time_ticks_seconds(0.0, 10.0, 6, fmt);
        assert!(!ticks.is_empty());
        // For a small range, we should still generate second-based ticks (not HH:MM yet).
        assert!(ticks.iter().all(|t| t.is_finite()));
    }
}
