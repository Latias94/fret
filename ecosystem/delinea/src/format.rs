use crate::engine::window::DataWindow;

pub fn nice_step(raw_step: f64) -> f64 {
    if !raw_step.is_finite() || raw_step <= 0.0 {
        return 0.0;
    }

    let exponent = raw_step.abs().log10().floor();
    let base = 10f64.powf(exponent);
    if !base.is_finite() || base <= 0.0 {
        return raw_step;
    }

    let fraction = raw_step / base;
    let nice_fraction = if fraction <= 1.0 {
        1.0
    } else if fraction <= 2.0 {
        2.0
    } else if fraction <= 5.0 {
        5.0
    } else {
        10.0
    };

    nice_fraction * base
}

pub fn nice_ticks(window: DataWindow, target_count: usize) -> Vec<f64> {
    let mut out = Vec::new();

    let mut window = window;
    window.clamp_non_degenerate();
    let span = window.span();

    if !span.is_finite() || span <= 0.0 || target_count == 0 {
        out.push(window.min);
        out.push(window.max);
        return out;
    }

    if target_count == 1 {
        out.push(0.5 * (window.min + window.max));
        return out;
    }

    let step = nice_step(span / (target_count as f64 - 1.0));
    if !step.is_finite() || step <= 0.0 {
        out.push(window.min);
        out.push(window.max);
        return out;
    }

    let start = (window.min / step).floor() * step;
    let end = (window.max / step).ceil() * step;
    if !start.is_finite() || !end.is_finite() || end < start {
        out.push(window.min);
        out.push(window.max);
        return out;
    }

    let mut v = start;
    // Prevent pathological loops due to floating rounding.
    let max_steps = 10_000usize;
    for _ in 0..max_steps {
        if v > end + step * 0.5 {
            break;
        }
        let clamped = v.clamp(window.min, window.max);
        if out
            .last()
            .is_none_or(|prev| (clamped - *prev).abs() > step * 0.1)
        {
            out.push(clamped);
        }
        v += step;
    }

    if out.is_empty() {
        out.push(window.min);
        out.push(window.max);
    } else {
        let first = *out.first().unwrap_or(&window.min);
        let last = *out.last().unwrap_or(&window.max);
        if (first - window.min).abs() > step * 0.1 {
            out.insert(0, window.min);
        }
        if (last - window.max).abs() > step * 0.1 {
            out.push(window.max);
        }
    }

    out
}

pub fn format_tick_value(window: DataWindow, value: f64) -> String {
    let mut window = window;
    window.clamp_non_degenerate();

    let span = window.span().abs();
    if !span.is_finite() || span <= 0.0 {
        return format!("{value}");
    }

    let step = nice_step(span / 4.0);
    let digits = if step.is_finite() && step > 0.0 {
        let log10 = step.abs().log10();
        (-log10).ceil().clamp(0.0, 8.0) as usize
    } else {
        3
    };

    let mut s = format!("{value:.digits$}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_ticks_include_endpoints() {
        let window = DataWindow { min: 0.2, max: 9.7 };
        let ticks = nice_ticks(window, 5);
        assert!(!ticks.is_empty());
        assert_eq!(*ticks.first().unwrap(), window.min);
        assert_eq!(*ticks.last().unwrap(), window.max);
    }
}
