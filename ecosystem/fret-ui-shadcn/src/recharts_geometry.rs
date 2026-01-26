//! Internal helpers for matching Recharts chart geometry in shadcn/ui chart examples.
//!
//! This module is not part of the public shadcn surface; it exists to support golden-driven
//! parity tests and to concentrate any Recharts-specific math in one place.

use fret_core::{Px, Rect};

/// Returns a "nice" tick step for a given rough step size.
///
/// Matches the common 1/2/5 * 10^k scaling used by chart engines.
fn nice_step(raw_step: f32) -> f32 {
    if !raw_step.is_finite() || raw_step <= 0.0 {
        return 0.0;
    }

    // The shadcn/ui Recharts examples consistently match a "round up to the next
    // power-of-ten multiple" rule for tick steps (e.g. 305 with 5 ticks => 76.25 => 80).
    let exponent = raw_step.abs().log10().floor();
    let base = 10.0_f32.powf(exponent);
    (raw_step / base).ceil() * base
}

/// Computes an upper-bounded "nice" max for the Y axis domain.
///
/// Recharts generally picks a nice tick step and rounds the max up to the next tick.
pub fn nice_domain_max(max_value: f32, tick_count: usize) -> f32 {
    if !max_value.is_finite() || max_value <= 0.0 {
        return 0.0;
    }
    let tick_count = tick_count.max(2);
    let slots = (tick_count - 1) as f32;
    let step = nice_step(max_value / slots);
    if step <= 0.0 {
        return max_value;
    }
    (max_value / step).ceil() * step
}

#[derive(Debug, Clone, Copy)]
pub struct BarChartSeriesLayout {
    /// Fraction of the category step used for the bar width.
    pub bar_width_ratio: f32,
    /// Tick count used for deriving the nice Y domain.
    pub y_tick_count: usize,
}

impl Default for BarChartSeriesLayout {
    fn default() -> Self {
        Self {
            // Recharts default `barCategoryGap="10%"` but the effective single-series bar width
            // in the shadcn v4 examples matches ~80% of the band step at 1440x900@2x.
            bar_width_ratio: 0.8,
            y_tick_count: 5,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BarRect {
    pub rect: Rect,
    pub value: f32,
}

/// Computes the expected bar rectangles for a single-series Recharts bar chart given the plot rect.
pub fn bar_rects(plot: Rect, values: &[f32], layout: BarChartSeriesLayout) -> Vec<BarRect> {
    if values.is_empty() {
        return Vec::new();
    }

    let plot_w = plot.size.width.0;
    let plot_h = plot.size.height.0;
    if !(plot_w.is_finite() && plot_h.is_finite()) || plot_w <= 0.0 || plot_h <= 0.0 {
        return Vec::new();
    }

    let max_value = values
        .iter()
        .copied()
        .filter(|v| v.is_finite() && *v >= 0.0)
        .fold(0.0_f32, f32::max);
    let domain_max = nice_domain_max(max_value, layout.y_tick_count).max(1.0);

    let n = values.len() as f32;
    let step = plot_w / n;
    let bar_w = (step * layout.bar_width_ratio).max(0.0);
    let offset = (step - bar_w) / 2.0;
    let baseline_y = plot.origin.y.0 + plot_h;

    values
        .iter()
        .enumerate()
        .map(|(i, raw)| {
            let value = if raw.is_finite() && *raw >= 0.0 {
                *raw
            } else {
                0.0
            };
            let h = (value / domain_max) * plot_h;
            let x = plot.origin.x.0 + (i as f32) * step + offset;
            let y = baseline_y - h;

            BarRect {
                rect: Rect::new(
                    fret_core::Point::new(Px(x), Px(y)),
                    fret_core::Size::new(Px(bar_w), Px(h)),
                ),
                value,
            }
        })
        .collect()
}
