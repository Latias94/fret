use std::fmt;
use std::sync::Arc;

use crate::cartesian::AxisScale;

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
            Self::Number(f) => 0x4e55_4d00_0000_0000u64 ^ u64::from(f.key()),
            Self::TimeSeconds(f) => 0x5449_4d00_0000_0000u64 ^ f.key(),
        }
    }

    pub fn format(self, v: f64, span: f64) -> String {
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AxisTicks {
    #[default]
    Nice,
    Linear,
    Log10,
    TimeSeconds(TimeAxisFormat),
}

impl AxisTicks {
    pub fn key(self) -> u64 {
        match self {
            Self::Nice => 0x4e_0000_0000_0000u64,
            Self::Linear => 0x4c_0000_0000_0000u64,
            Self::Log10 => 0x4f_0000_0000_0000u64,
            Self::TimeSeconds(f) => 0x54_0000_0000_0000u64 ^ f.key(),
        }
    }
}

pub fn axis_ticks(min: f64, max: f64, tick_count: usize, ticks: AxisTicks) -> Vec<f64> {
    match ticks {
        AxisTicks::Nice => nice_ticks(min, max, tick_count),
        AxisTicks::Linear => linear_ticks(min, max, tick_count),
        AxisTicks::Log10 => log10_ticks(min, max, tick_count),
        AxisTicks::TimeSeconds(f) => time_ticks_seconds(min, max, tick_count, f),
    }
}

pub fn axis_ticks_scaled(
    min: f64,
    max: f64,
    tick_count: usize,
    ticks: AxisTicks,
    scale: AxisScale,
) -> Vec<f64> {
    if scale == AxisScale::Log10 {
        // For log axes, always generate log ticks (even if the caller requests "nice" ticks).
        return log10_ticks(min, max, tick_count);
    }
    axis_ticks(min, max, tick_count, ticks)
}

pub fn log10_ticks(min: f64, max: f64, tick_count: usize) -> Vec<f64> {
    if tick_count == 0 {
        return Vec::new();
    }
    if !min.is_finite() || !max.is_finite() {
        return Vec::new();
    }

    // Clamp the log domain to positive values. Non-positive values are not representable.
    const MIN_POS: f64 = 1.0e-12;
    let (min, max) = if min <= max { (min, max) } else { (max, min) };
    let min = min.max(MIN_POS);
    if max <= 0.0 || max <= min {
        return vec![min];
    }

    let e0 = min.log10().floor();
    let e1 = max.log10().ceil();
    if !e0.is_finite() || !e1.is_finite() || e0 > e1 {
        return vec![min, max]
            .into_iter()
            .filter(|v| v.is_finite() && *v > 0.0)
            .collect();
    }

    // Always include decade marks, then optionally add 2/5 ticks within each decade.
    let exp0 = e0 as i32;
    let exp1 = e1 as i32;
    let max_decades = 4096i32;

    let mut decades: Vec<f64> = Vec::new();
    for (i, exp) in (exp0..=exp1).enumerate() {
        if i as i32 >= max_decades {
            break;
        }
        let v = 10.0_f64.powi(exp);
        if v.is_finite() && v >= min && v <= max {
            decades.push(v);
        }
    }

    if decades.is_empty() {
        return vec![min, max]
            .into_iter()
            .filter(|v| v.is_finite() && *v > 0.0)
            .collect();
    }

    decades.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    decades.dedup_by(|a, b| (*a - *b).abs() <= 0.0);

    if tick_count <= decades.len() {
        if tick_count == 1 {
            return vec![decades[0]];
        }

        let len = decades.len();
        let mut out: Vec<f64> = Vec::with_capacity(tick_count);
        for i in 0..tick_count {
            let t = i as f64 / (tick_count.saturating_sub(1) as f64);
            let idx = (t * ((len - 1) as f64)).round() as usize;
            if let Some(v) = decades.get(idx).copied()
                && out.last().copied().is_none_or(|last| last != v)
            {
                out.push(v);
            }
        }
        return out;
    }

    let mut candidates: Vec<f64> = Vec::new();
    for (i, exp) in (exp0..=exp1).enumerate() {
        if i as i32 >= max_decades {
            break;
        }
        let base = 10.0_f64.powi(exp);
        for m in [2.0, 5.0] {
            let v = m * base;
            if v.is_finite() && v >= min && v <= max {
                candidates.push(v);
            }
        }
    }
    candidates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    candidates.dedup_by(|a, b| (*a - *b).abs() <= 0.0);

    let mut out = decades;
    for v in candidates {
        if out.len() >= tick_count {
            break;
        }
        if out
            .binary_search_by(|x| x.partial_cmp(&v).unwrap_or(std::cmp::Ordering::Equal))
            .is_err()
        {
            out.push(v);
        }
    }
    out.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    out.dedup_by(|a, b| (*a - *b).abs() <= 0.0);
    out
}

#[derive(Clone)]
pub struct AxisLabelFormatter {
    key: u64,
    f: Arc<dyn Fn(f64, f64) -> String + Send + Sync + 'static>,
}

impl fmt::Debug for AxisLabelFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AxisLabelFormatter")
            .field("key", &self.key)
            .finish()
    }
}

impl AxisLabelFormatter {
    /// Creates a custom label formatter with a stable cache key.
    ///
    /// The key is used by the plot widget to cache prepared text blobs for tick labels. If the
    /// key changes between frames, tick labels will be re-shaped and re-allocated even if the
    /// formatting logic is the same.
    pub fn custom(key: u64, f: impl Fn(f64, f64) -> String + Send + Sync + 'static) -> Self {
        Self {
            key,
            f: Arc::new(f),
        }
    }

    /// Creates a custom label formatter from a `'static` function pointer.
    ///
    /// This is a convenience helper for cases where capturing state in a closure is not needed.
    /// The cache key is derived from the function pointer address and is stable for the lifetime
    /// of the process.
    pub fn custom_static(f: fn(f64, f64) -> String) -> Self {
        let addr = f as usize as u64;
        // Mix the address into a tagged key. This does not need to be stable across processes.
        let key = 0x4355_5354_4d00_0000u64 ^ addr.wrapping_mul(0x9e3779b97f4a7c15);
        Self {
            key,
            f: Arc::new(f),
        }
    }

    pub fn number(fmt: AxisNumberFormat) -> Self {
        Self::custom(
            0x4e55_4d00_0000_0000u64 ^ u64::from(fmt.key()),
            move |v, span| format_number(v, span, fmt),
        )
    }

    pub fn time_seconds(fmt: TimeAxisFormat) -> Self {
        Self::custom(0x5449_4d00_0000_0000u64 ^ fmt.key(), move |v, span| {
            format_time_seconds(v, span, fmt)
        })
    }

    pub fn key(&self) -> u64 {
        self.key
    }

    pub fn is_number_auto(&self) -> bool {
        // AxisLabelFormatter::default() == number(Auto) uses this deterministic key.
        self.key == 0x4e55_4d00_0000_0000u64
    }

    pub fn format(&self, v: f64, span: f64) -> String {
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
        (self.base_seconds.to_bits() ^ (tag << 1)).wrapping_mul(0x9e3779b97f4a7c15)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeAxisPresentation {
    /// Format ticks as relative time offsets (mm:ss, hh:mm:ss, ...).
    Relative,
    /// Format ticks as absolute timestamps in UTC (YYYY-MM-DD, HH:MM, ...).
    UnixUtc,
}

fn format_number(v: f64, span: f64, fmt: AxisNumberFormat) -> String {
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

fn format_time_seconds(v: f64, span: f64, fmt: TimeAxisFormat) -> String {
    if !v.is_finite() {
        return "NA".to_string();
    }

    match fmt.presentation {
        TimeAxisPresentation::Relative => format_relative_time_seconds(v, span),
        TimeAxisPresentation::UnixUtc => {
            let abs = fmt.base_seconds + v;
            format_unix_utc_seconds(abs, span)
        }
    }
}

fn format_relative_time_seconds(t: f64, span: f64) -> String {
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

fn format_unix_utc_seconds(abs_seconds: f64, span: f64) -> String {
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

// Inverse of `civil_from_days`, based on Howard Hinnant's "days_from_civil" algorithm (public
// domain). Interprets day 0 as 1970-01-01 in the proleptic Gregorian calendar.
fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || day == 0 || day > 31 {
        return None;
    }

    let y = i64::from(year) - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 }.div_euclid(400);
    let yoe = y - era * 400; // [0, 399]
    let mp = i64::from(month) + if month > 2 { -3 } else { 9 }; // [0, 11]
    let doy = (153 * mp + 2).div_euclid(5) + i64::from(day) - 1; // [0, 365]
    let doe = yoe * 365 + yoe.div_euclid(4) - yoe.div_euclid(100) + doy; // [0, 146096]
    Some(era * 146_097 + doe - 719_468)
}

fn utc_parts_to_unix_seconds(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<f64> {
    if hour >= 24 || minute >= 60 || second >= 60 {
        return None;
    }
    let days = days_from_civil(year, month, day)?;
    let sec_of_day = i64::from(hour) * 3600 + i64::from(minute) * 60 + i64::from(second);
    let secs = days.checked_mul(86400)?.checked_add(sec_of_day)?;
    Some(secs as f64)
}

pub fn nice_ticks(min: f64, max: f64, tick_count: usize) -> Vec<f64> {
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
    let span = max - min;
    let target = if tick_count <= 1 {
        span
    } else {
        span / (tick_count.saturating_sub(1) as f64)
    };

    let step = nice_step_125(target).max(f64::EPSILON);
    let first = (min / step).ceil() * step;
    let last = (max / step).floor() * step;

    if !first.is_finite() || !last.is_finite() || first > last {
        return vec![min, max]
            .into_iter()
            .filter(|v| v.is_finite())
            .collect();
    }

    let mut out: Vec<f64> = Vec::new();
    let mut v = first;
    let max_steps = 4096usize;
    for _ in 0..max_steps {
        if v > last + step * 0.5 {
            break;
        }
        out.push(v);
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

pub fn time_ticks_seconds(min: f64, max: f64, tick_count: usize, fmt: TimeAxisFormat) -> Vec<f64> {
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
    let span = (max - min).abs();
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
    let min_abs = min + base;
    let max_abs = max + base;
    if !min_abs.is_finite() || !max_abs.is_finite() {
        return nice_ticks(min, max, tick_count);
    }

    if fmt.presentation == TimeAxisPresentation::UnixUtc {
        let abs = time_ticks_unix_utc_seconds(min_abs, max_abs, tick_count);
        return abs
            .into_iter()
            .map(|t| t - base)
            .filter(|v| v.is_finite())
            .collect();
    }

    let first_k = (min_abs / step).ceil();
    let last_k = (max_abs / step).floor();
    if !first_k.is_finite() || !last_k.is_finite() || first_k > last_k {
        return nice_ticks(min, max, tick_count);
    }

    let mut out: Vec<f64> = Vec::new();
    let mut k = first_k;
    let max_steps = 4096usize;
    for _ in 0..max_steps {
        if k > last_k + 0.5 {
            break;
        }
        let v = k * step - base;
        if v.is_finite() {
            out.push(v);
        }
        k += 1.0;
    }

    if out.is_empty() {
        nice_ticks(min, max, tick_count)
    } else {
        out
    }
}

fn time_ticks_unix_utc_seconds(min_abs: f64, max_abs: f64, tick_count: usize) -> Vec<f64> {
    if tick_count == 0 {
        return Vec::new();
    }
    if !min_abs.is_finite() || !max_abs.is_finite() {
        return Vec::new();
    }
    if min_abs == max_abs {
        return vec![min_abs];
    }

    let (min_abs, max_abs) = if min_abs <= max_abs {
        (min_abs, max_abs)
    } else {
        (max_abs, min_abs)
    };
    let span = (max_abs - min_abs).abs();
    if !span.is_finite() || span <= 0.0 {
        return vec![min_abs, max_abs]
            .into_iter()
            .filter(|v| v.is_finite())
            .collect();
    }

    let target = if tick_count <= 1 {
        span
    } else {
        span / (tick_count.saturating_sub(1) as f64)
    };

    // Prefer calendar-aligned steps for UTC time (day/month/year boundaries).
    #[derive(Clone, Copy)]
    enum Step {
        Seconds(i64),
        Minutes(i64),
        Hours(i64),
        Days(i64),
        Months(i32),
        Years(i32),
    }

    const CANDIDATES: &[Step] = &[
        Step::Seconds(1),
        Step::Seconds(2),
        Step::Seconds(5),
        Step::Seconds(10),
        Step::Seconds(15),
        Step::Seconds(30),
        Step::Minutes(1),
        Step::Minutes(2),
        Step::Minutes(5),
        Step::Minutes(10),
        Step::Minutes(15),
        Step::Minutes(30),
        Step::Hours(1),
        Step::Hours(2),
        Step::Hours(3),
        Step::Hours(6),
        Step::Hours(12),
        Step::Days(1),
        Step::Days(2),
        Step::Days(7),
        Step::Months(1),
        Step::Months(2),
        Step::Months(3),
        Step::Months(6),
        Step::Years(1),
        Step::Years(2),
        Step::Years(5),
        Step::Years(10),
    ];

    let step = CANDIDATES
        .iter()
        .copied()
        .find(|s| match *s {
            Step::Seconds(n) => (n as f64) >= target,
            Step::Minutes(n) => (n as f64) * 60.0 >= target,
            Step::Hours(n) => (n as f64) * 3600.0 >= target,
            Step::Days(n) => (n as f64) * 86400.0 >= target,
            Step::Months(n) => (n as f64) * 86400.0 * 30.0 >= target,
            Step::Years(n) => (n as f64) * 86400.0 * 365.0 >= target,
        })
        .unwrap_or(Step::Years(10));

    match step {
        Step::Seconds(step_s) => time_ticks_fixed_seconds(min_abs, max_abs, step_s),
        Step::Minutes(step_m) => time_ticks_fixed_seconds(min_abs, max_abs, step_m * 60),
        Step::Hours(step_h) => time_ticks_fixed_seconds(min_abs, max_abs, step_h * 3600),
        Step::Days(step_d) => time_ticks_fixed_seconds(min_abs, max_abs, step_d * 86400),
        Step::Months(step_m) => time_ticks_by_month(min_abs, max_abs, step_m),
        Step::Years(step_y) => time_ticks_by_year(min_abs, max_abs, step_y),
    }
}

fn time_ticks_fixed_seconds(min_abs: f64, max_abs: f64, step_seconds: i64) -> Vec<f64> {
    if step_seconds <= 0 {
        return Vec::new();
    }
    let min_s = min_abs.floor() as i64;
    let max_s = max_abs.floor() as i64;

    let first = if min_s.rem_euclid(step_seconds) == 0 {
        min_s
    } else {
        (min_s.div_euclid(step_seconds) + 1) * step_seconds
    };
    let last = max_s.div_euclid(step_seconds) * step_seconds;
    if first > last {
        return Vec::new();
    }

    let mut out: Vec<f64> = Vec::new();
    let mut t = first;
    let max_steps = 8192usize;
    for _ in 0..max_steps {
        if t > last {
            break;
        }
        out.push(t as f64);
        t = match t.checked_add(step_seconds) {
            Some(v) => v,
            None => break,
        };
    }
    out
}

fn time_ticks_by_month(min_abs: f64, max_abs: f64, step_months: i32) -> Vec<f64> {
    if step_months <= 0 {
        return Vec::new();
    }
    let Some((y0, m0, _d0, _h0, _min0, _s0)) = unix_seconds_to_utc_parts(min_abs) else {
        return Vec::new();
    };
    let Some((_y1, _m1, _d1, _h1, _min1, _s1)) = unix_seconds_to_utc_parts(max_abs) else {
        return Vec::new();
    };

    let month0 = (m0 as i32).saturating_sub(1);
    let mut idx0 = y0.saturating_mul(12).saturating_add(month0);
    let step = step_months;

    // Snap to the first month boundary at or after `min_abs`, then to the step grid.
    let start_of_month = utc_parts_to_unix_seconds(y0, m0, 1, 0, 0, 0).unwrap_or(min_abs);
    if start_of_month < min_abs - 0.5 {
        idx0 = idx0.saturating_add(1);
    }
    let rem = idx0.rem_euclid(step);
    if rem != 0 {
        idx0 = idx0.saturating_add(step - rem);
    }

    let mut out: Vec<f64> = Vec::new();
    let max_steps = 4096usize;
    let mut idx = idx0;
    for _ in 0..max_steps {
        let year = idx.div_euclid(12);
        let month = idx.rem_euclid(12) + 1;
        let Some(t) = utc_parts_to_unix_seconds(year, month as u32, 1, 0, 0, 0) else {
            break;
        };
        if t > max_abs + 0.5 {
            break;
        }
        if t >= min_abs - 0.5 {
            out.push(t);
        }
        idx = match idx.checked_add(step) {
            Some(v) => v,
            None => break,
        };
    }

    out
}

fn time_ticks_by_year(min_abs: f64, max_abs: f64, step_years: i32) -> Vec<f64> {
    if step_years <= 0 {
        return Vec::new();
    }
    let Some((y0, _m0, _d0, _h0, _min0, _s0)) = unix_seconds_to_utc_parts(min_abs) else {
        return Vec::new();
    };

    let mut year = y0;
    let start_of_year = utc_parts_to_unix_seconds(y0, 1, 1, 0, 0, 0).unwrap_or(min_abs);
    if start_of_year < min_abs - 0.5 {
        year = year.saturating_add(1);
    }
    let rem = year.rem_euclid(step_years);
    if rem != 0 {
        year = year.saturating_add(step_years - rem);
    }

    let mut out: Vec<f64> = Vec::new();
    let max_steps = 4096usize;
    for _ in 0..max_steps {
        let Some(t) = utc_parts_to_unix_seconds(year, 1, 1, 0, 0, 0) else {
            break;
        };
        if t > max_abs + 0.5 {
            break;
        }
        if t >= min_abs - 0.5 {
            out.push(t);
        }
        year = match year.checked_add(step_years) {
            Some(v) => v,
            None => break,
        };
    }
    out
}

pub fn linear_ticks(min: f64, max: f64, tick_count: usize) -> Vec<f64> {
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
    let denom = (n - 1) as f64;
    (0..n)
        .map(|i| {
            let t = (i as f64) / denom;
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
            .fold(None, |acc: Option<f64>, v| {
                Some(acc.map_or(v, |a| a.min(v)))
            })
            .unwrap_or(0.0);

        // For span ~1.0 and 6 ticks, target step ~0.2 => expect 0.2.
        assert!((step - 0.2).abs() <= 1e-8, "step={step}");
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

    #[test]
    fn unix_utc_ticks_align_to_month_boundaries() {
        // 2020-01-15 00:00:00 UTC.
        let base = utc_parts_to_unix_seconds(2020, 1, 15, 0, 0, 0).unwrap();
        let fmt = TimeAxisFormat {
            base_seconds: base,
            presentation: TimeAxisPresentation::UnixUtc,
        };

        // View spans ~70 days around the base.
        let min = 0.0;
        let max = 86400.0 * 70.0;
        let ticks = time_ticks_seconds(min, max, 5, fmt);
        assert!(!ticks.is_empty());

        // The first "calendar" tick should be at the start of the next month: 2020-02-01.
        let feb1 = utc_parts_to_unix_seconds(2020, 2, 1, 0, 0, 0).unwrap() - base;
        assert!(ticks.contains(&feb1), "ticks={ticks:?}, feb1={feb1}");
    }

    #[test]
    fn log10_ticks_include_decade_marks() {
        let ticks = log10_ticks(0.1, 1000.0, 8);
        assert!(ticks.contains(&0.1));
        assert!(ticks.contains(&1.0));
        assert!(ticks.contains(&10.0));
        assert!(ticks.contains(&100.0));
        assert!(ticks.contains(&1000.0));
    }
}
