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
