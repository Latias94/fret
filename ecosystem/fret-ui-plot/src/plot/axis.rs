use std::fmt;
use std::sync::Arc;

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

    pub fn ticks(self) -> AxisTicks {
        match self {
            Self::Number(_) => AxisTicks::Nice,
            Self::TimeSeconds(f) => AxisTicks::TimeSeconds(f),
        }
    }

    pub fn labels(self) -> AxisLabelFormatter {
        match self {
            Self::Number(f) => AxisLabelFormatter::number(f),
            Self::TimeSeconds(f) => AxisLabelFormatter::time_seconds(f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisTicks {
    Nice,
    Linear,
    TimeSeconds(TimeAxisFormat),
}

impl Default for AxisTicks {
    fn default() -> Self {
        Self::Nice
    }
}

impl AxisTicks {
    pub fn key(self) -> u64 {
        match self {
            Self::Nice => 0x4e_0000_0000_0000u64,
            Self::Linear => 0x4c_0000_0000_0000u64,
            Self::TimeSeconds(f) => 0x54_0000_0000_0000u64 ^ f.key(),
        }
    }
}

pub fn axis_ticks(min: f32, max: f32, tick_count: usize, ticks: AxisTicks) -> Vec<f32> {
    match ticks {
        AxisTicks::Nice => nice_ticks(min, max, tick_count),
        AxisTicks::Linear => linear_ticks(min, max, tick_count),
        AxisTicks::TimeSeconds(f) => time_ticks_seconds(min, max, tick_count, f),
    }
}

#[derive(Clone)]
pub struct AxisLabelFormatter {
    key: u64,
    f: Arc<dyn Fn(f32, f32) -> String + Send + Sync + 'static>,
}

impl fmt::Debug for AxisLabelFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AxisLabelFormatter")
            .field("key", &self.key)
            .finish()
    }
}

impl AxisLabelFormatter {
    pub fn custom(key: u64, f: impl Fn(f32, f32) -> String + Send + Sync + 'static) -> Self {
        Self {
            key,
            f: Arc::new(f),
        }
    }

    pub fn number(fmt: AxisNumberFormat) -> Self {
        Self::custom(
            0x4e554d00_0000_0000u64 ^ u64::from(fmt.key()),
            move |v, span| format_number(v, span, fmt),
        )
    }

    pub fn time_seconds(fmt: TimeAxisFormat) -> Self {
        Self::custom(0x54494d00_0000_0000u64 ^ fmt.key(), move |v, span| {
            format_time_seconds(v, span, fmt)
        })
    }

    pub fn key(&self) -> u64 {
        self.key
    }

    pub fn format(&self, v: f32, span: f32) -> String {
        (self.f)(v, span)
    }
}

impl Default for AxisLabelFormatter {
    fn default() -> Self {
        Self::number(AxisNumberFormat::Auto)
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
    /// Base timestamp in Unix seconds.
    ///
    /// When formatting absolute time (e.g. UTC), tick labels are derived from
    /// `base_seconds + x`.
    pub base_seconds: f64,
    pub presentation: TimeAxisPresentation,
}

impl Default for TimeAxisFormat {
    fn default() -> Self {
        Self {
            base_seconds: 0.0,
            presentation: TimeAxisPresentation::Relative,
        }
    }
}

impl TimeAxisFormat {
    pub fn key(self) -> u64 {
        let tag = match self.presentation {
            TimeAxisPresentation::Relative => 0u64,
            TimeAxisPresentation::UnixUtc => 1u64,
        };
        (u64::from(self.base_seconds.to_bits()) ^ (tag << 1)).wrapping_mul(0x9e3779b97f4a7c15)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeAxisPresentation {
    /// Format ticks as relative time offsets (mm:ss, hh:mm:ss, ...).
    Relative,
    /// Format ticks as absolute timestamps in UTC (YYYY-MM-DD, HH:MM, ...).
    UnixUtc,
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

    match fmt.presentation {
        TimeAxisPresentation::Relative => format_relative_time_seconds(v as f64, span),
        TimeAxisPresentation::UnixUtc => {
            let abs = fmt.base_seconds + (v as f64);
            format_unix_utc_seconds(abs, span)
        }
    }
}

fn format_relative_time_seconds(t: f64, span: f32) -> String {
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

fn format_unix_utc_seconds(abs_seconds: f64, span: f32) -> String {
    let Some((year, month, day, hour, minute, second)) = unix_seconds_to_utc_parts(abs_seconds)
    else {
        return "NA".to_string();
    };

    let span = span.abs();

    if span.is_finite() && span < 60.0 {
        return format!("{hour:02}:{minute:02}:{second:02}");
    }
    if span.is_finite() && span < 3600.0 {
        return format!("{hour:02}:{minute:02}:{second:02}");
    }
    if span.is_finite() && span < 86400.0 {
        return format!("{hour:02}:{minute:02}");
    }

    if span.is_finite() && span < 86400.0 * 365.0 {
        return format!("{year:04}-{month:02}-{day:02}");
    }

    format!("{year:04}-{month:02}")
}

fn unix_seconds_to_utc_parts(abs_seconds: f64) -> Option<(i32, u32, u32, u32, u32, u32)> {
    if !abs_seconds.is_finite() {
        return None;
    }

    let secs = abs_seconds.floor() as i64;
    let days = secs.div_euclid(86400);
    let sec_of_day = secs.rem_euclid(86400) as u32;

    let (year, month, day) = civil_from_days(days);
    let hour = sec_of_day / 3600;
    let minute = (sec_of_day / 60) % 60;
    let second = sec_of_day % 60;

    Some((year, month, day, hour, minute, second))
}

// Based on Howard Hinnant's "civil_from_days" algorithm (public domain).
// Interprets day 0 as 1970-01-01 in the proleptic Gregorian calendar.
fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let d = doy - (153 * mp + 2).div_euclid(5) + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = (y + (m <= 2) as i64) as i32;
    (year, m as u32, d as u32)
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

    let base = fmt.base_seconds;
    let min_abs = (min as f64) + base;
    let max_abs = (max as f64) + base;
    if !min_abs.is_finite() || !max_abs.is_finite() {
        return nice_ticks(min, max, tick_count);
    }

    let first_k = (min_abs / step).ceil();
    let last_k = (max_abs / step).floor();
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
        let v = k * step - base;
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

    #[test]
    fn unix_utc_formatting_handles_epoch() {
        let fmt = TimeAxisFormat {
            base_seconds: 0.0,
            presentation: TimeAxisPresentation::UnixUtc,
        };
        assert_eq!(
            AxisLabelFormat::TimeSeconds(fmt).format(0.0, 86400.0),
            "1970-01-01"
        );
    }
}
