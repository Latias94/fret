use crate::engine::window::DataWindow;

use time::{Date, Duration, Month, OffsetDateTime, Time};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimeTickUnit {
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Month,
    Year,
}

#[derive(Debug, Clone, Copy)]
struct TimeTickSpec {
    unit: TimeTickUnit,
    step: i64,
}

pub fn ticks(window: DataWindow, target_count: usize) -> Vec<f64> {
    let mut window = window;
    window.clamp_non_degenerate();

    let min = window.min;
    let max = window.max;
    if !min.is_finite() || !max.is_finite() || target_count == 0 {
        return vec![min, max];
    }
    if (max - min).abs() <= f64::EPSILON {
        return vec![min];
    }

    let spec = tick_spec(window, target_count);
    let Some(min_ms) = ms_from_value(min) else {
        return vec![min, max];
    };
    let Some(max_ms) = ms_from_value(max) else {
        return vec![min, max];
    };

    let mut out: Vec<i64> = Vec::new();
    out.push(min_ms);

    let mut internal = internal_ticks(min_ms, max_ms, spec);
    out.append(&mut internal);

    if out.last().copied() != Some(max_ms) {
        out.push(max_ms);
    }

    out.into_iter().map(|v| v as f64).collect()
}

pub fn format_tick(window: DataWindow, value: f64) -> String {
    format_tick_with_target(window, value, 5)
}

pub fn format_tick_with_target(window: DataWindow, value: f64, target_count: usize) -> String {
    let mut window = window;
    window.clamp_non_degenerate();

    let spec = tick_spec(window, target_count);
    let Some(ms) = ms_from_value(value) else {
        return value.to_string();
    };
    let Some(dt) = utc_from_ms(ms) else {
        return value.to_string();
    };

    let span_ms = (window.max - window.min).abs();
    let show_date = span_ms.is_finite() && span_ms >= 86_400_000.0;

    let year = dt.year();
    let month = dt.month() as u8;
    let day = dt.day();
    let hour = dt.hour();
    let minute = dt.minute();
    let second = dt.second();
    let millis = (dt.nanosecond() / 1_000_000) as u16;

    match (spec.unit, show_date) {
        (TimeTickUnit::Year, _) => format!("{year:04}"),
        (TimeTickUnit::Month, _) => format!("{year:04}-{month:02}"),
        (TimeTickUnit::Day, true) => format!("{year:04}-{month:02}-{day:02}"),
        (TimeTickUnit::Day, false) => format!("{month:02}-{day:02}"),
        (TimeTickUnit::Hour, true) => format!("{year:04}-{month:02}-{day:02} {hour:02}:00"),
        (TimeTickUnit::Hour, false) => format!("{hour:02}:00"),
        (TimeTickUnit::Minute, true) => {
            format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
        }
        (TimeTickUnit::Minute, false) => format!("{hour:02}:{minute:02}"),
        (TimeTickUnit::Second, true) => {
            format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
        }
        (TimeTickUnit::Second, false) => format!("{hour:02}:{minute:02}:{second:02}"),
        (TimeTickUnit::Millisecond, true) => {
            format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}.{millis:03}")
        }
        (TimeTickUnit::Millisecond, false) => {
            format!("{hour:02}:{minute:02}:{second:02}.{millis:03}")
        }
    }
}

fn tick_spec(window: DataWindow, target_count: usize) -> TimeTickSpec {
    let span_ms = window.span().abs();
    let approx_tick_num = target_count.max(2) as f64;
    let approx_ms = if span_ms.is_finite() && span_ms > 0.0 {
        span_ms / approx_tick_num
    } else {
        1.0
    };

    if approx_ms < 1_000.0 {
        let step = crate::format::nice_step(approx_ms).round().max(1.0) as i64;
        return TimeTickSpec {
            unit: TimeTickUnit::Millisecond,
            step,
        };
    }

    if approx_ms < 60_000.0 {
        return TimeTickSpec {
            unit: TimeTickUnit::Second,
            step: select_step(approx_ms / 1_000.0, &[1, 2, 5, 10, 15, 20, 30]),
        };
    }

    if approx_ms < 3_600_000.0 {
        return TimeTickSpec {
            unit: TimeTickUnit::Minute,
            step: select_step(approx_ms / 60_000.0, &[1, 2, 5, 10, 15, 20, 30]),
        };
    }

    if approx_ms < 86_400_000.0 {
        return TimeTickSpec {
            unit: TimeTickUnit::Hour,
            step: select_step(approx_ms / 3_600_000.0, &[1, 2, 3, 4, 6, 12]),
        };
    }

    let day_ms = 86_400_000.0;
    if approx_ms < 31.0 * day_ms {
        return TimeTickSpec {
            unit: TimeTickUnit::Day,
            step: select_step(approx_ms / day_ms, &[1, 2, 3, 5, 7, 14]),
        };
    }

    if approx_ms < 366.0 * day_ms {
        return TimeTickSpec {
            unit: TimeTickUnit::Month,
            step: select_step(approx_ms / (30.0 * day_ms), &[1, 2, 3, 6]),
        };
    }

    let years = (approx_ms / (365.0 * day_ms)).max(1.0);
    let step = crate::format::nice_step(years).round().max(1.0) as i64;
    TimeTickSpec {
        unit: TimeTickUnit::Year,
        step,
    }
}

fn select_step(approx: f64, candidates: &[i64]) -> i64 {
    if !approx.is_finite() || approx <= 0.0 {
        return candidates.first().copied().unwrap_or(1);
    }
    for &c in candidates {
        if approx <= c as f64 {
            return c;
        }
    }
    *candidates.last().unwrap_or(&1)
}

fn ms_from_value(value: f64) -> Option<i64> {
    if !value.is_finite() {
        return None;
    }
    let rounded = value.round();
    if rounded < i64::MIN as f64 || rounded > i64::MAX as f64 {
        return None;
    }
    Some(rounded as i64)
}

fn utc_from_ms(ms: i64) -> Option<OffsetDateTime> {
    let nanos = i128::from(ms) * 1_000_000i128;
    OffsetDateTime::from_unix_timestamp_nanos(nanos).ok()
}

fn internal_ticks(min_ms: i64, max_ms: i64, spec: TimeTickSpec) -> Vec<i64> {
    let (min_ms, max_ms) = if min_ms <= max_ms {
        (min_ms, max_ms)
    } else {
        (max_ms, min_ms)
    };

    let Some(min_dt) = utc_from_ms(min_ms) else {
        return Vec::new();
    };

    let upper = upper_unit(spec.unit);
    let mut cursor = floor_to_unit(min_dt, upper);
    cursor = add_unit(cursor, spec.unit, spec.step);

    let mut out = Vec::new();
    let mut steps = 0usize;
    while steps < 10_000 {
        let ms = (cursor.unix_timestamp_nanos() / 1_000_000) as i64;
        if ms >= max_ms {
            break;
        }
        if ms > min_ms && out.last().copied() != Some(ms) {
            out.push(ms);
        }
        cursor = add_unit(cursor, spec.unit, spec.step);
        steps += 1;
    }
    out
}

fn upper_unit(unit: TimeTickUnit) -> TimeTickUnit {
    match unit {
        TimeTickUnit::Millisecond => TimeTickUnit::Second,
        TimeTickUnit::Second => TimeTickUnit::Minute,
        TimeTickUnit::Minute => TimeTickUnit::Hour,
        TimeTickUnit::Hour => TimeTickUnit::Day,
        TimeTickUnit::Day => TimeTickUnit::Month,
        TimeTickUnit::Month => TimeTickUnit::Year,
        TimeTickUnit::Year => TimeTickUnit::Year,
    }
}

fn floor_to_unit(dt: OffsetDateTime, unit: TimeTickUnit) -> OffsetDateTime {
    let year = dt.year();
    let month = dt.month();
    let day = dt.day();

    match unit {
        TimeTickUnit::Year => {
            let date = Date::from_calendar_date(year, Month::January, 1).ok();
            date.and_then(|d| OffsetDateTime::new_utc(d, Time::MIDNIGHT).into())
                .unwrap_or(dt)
        }
        TimeTickUnit::Month => {
            let date = Date::from_calendar_date(year, month, 1).ok();
            date.and_then(|d| OffsetDateTime::new_utc(d, Time::MIDNIGHT).into())
                .unwrap_or(dt)
        }
        TimeTickUnit::Day => {
            let date = Date::from_calendar_date(year, month, day).ok();
            date.and_then(|d| OffsetDateTime::new_utc(d, Time::MIDNIGHT).into())
                .unwrap_or(dt)
        }
        TimeTickUnit::Hour => dt
            .replace_minute(0)
            .and_then(|dt| dt.replace_second(0))
            .and_then(|dt| dt.replace_nanosecond(0))
            .unwrap_or(dt),
        TimeTickUnit::Minute => dt
            .replace_second(0)
            .and_then(|dt| dt.replace_nanosecond(0))
            .unwrap_or(dt),
        TimeTickUnit::Second => dt.replace_nanosecond(0).unwrap_or(dt),
        TimeTickUnit::Millisecond => {
            let nanos = dt.nanosecond();
            let ms = nanos / 1_000_000;
            dt.replace_nanosecond(ms * 1_000_000).unwrap_or(dt)
        }
    }
}

fn add_unit(dt: OffsetDateTime, unit: TimeTickUnit, step: i64) -> OffsetDateTime {
    let step = step.max(1);
    match unit {
        TimeTickUnit::Millisecond => dt.saturating_add(Duration::milliseconds(step)),
        TimeTickUnit::Second => dt.saturating_add(Duration::seconds(step)),
        TimeTickUnit::Minute => dt.saturating_add(Duration::minutes(step)),
        TimeTickUnit::Hour => dt.saturating_add(Duration::hours(step)),
        TimeTickUnit::Day => dt.saturating_add(Duration::days(step)),
        TimeTickUnit::Month => add_months(dt, step as i32),
        TimeTickUnit::Year => add_months(dt, (step as i32).saturating_mul(12)),
    }
}

fn add_months(dt: OffsetDateTime, months: i32) -> OffsetDateTime {
    let date = dt.date();
    let year = date.year();
    let month0 = date.month() as i32 - 1;

    let total = year * 12 + month0 + months;
    let year = total.div_euclid(12);
    let month0 = total.rem_euclid(12);
    let month = Month::try_from((month0 + 1) as u8).unwrap_or(Month::January);

    let date = Date::from_calendar_date(year, month, 1)
        .ok()
        .unwrap_or(date);
    OffsetDateTime::new_utc(date, Time::MIDNIGHT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tick_with_target_uses_coarse_units_for_large_spans() {
        let day_ms = 86_400_000.0;
        let window = DataWindow {
            min: 0.0,
            max: 400.0 * day_ms,
        };

        // For large spans, default labels should be date-like (year/month/day), not time-of-day.
        let labels: Vec<String> = ticks(window, 3)
            .into_iter()
            .map(|v| format_tick_with_target(window, v, 3))
            .collect();
        assert!(!labels.is_empty());
        for label in labels {
            assert!(
                !label.contains(':'),
                "unexpected time-of-day label: {label}"
            );
        }
    }

    #[test]
    fn ticks_include_endpoints() {
        let window = DataWindow {
            min: 1_000.0,
            max: 10_000.0,
        };
        let ticks = ticks(window, 5);
        assert_eq!(ticks.first().copied(), Some(window.min));
        assert_eq!(ticks.last().copied(), Some(window.max));
    }

    #[test]
    fn month_ticks_are_monotonic() {
        let jan_2024 = 1_704_067_200_000i64;
        let may_2024 = 1_714_608_000_000i64;
        let window = DataWindow {
            min: jan_2024 as f64,
            max: may_2024 as f64,
        };
        let ticks = ticks(window, 5);
        for w in ticks.windows(2) {
            assert!(w[0] < w[1]);
        }
    }
}
