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

/// Computes the expected bar rectangles for a grouped (multi-series) Recharts bar chart.
///
/// Uses `layout.bar_width_ratio` as the ratio of the *group* width within each category step.
pub fn grouped_bar_rects(
    plot: Rect,
    series: &[&[f32]],
    layout: BarChartSeriesLayout,
    bar_gap_px: f32,
) -> Vec<BarRect> {
    if series.is_empty() {
        return Vec::new();
    }

    let n = series[0].len();
    if n == 0 || series.iter().any(|s| s.len() != n) {
        return Vec::new();
    }

    let plot_w = plot.size.width.0;
    let plot_h = plot.size.height.0;
    if !(plot_w.is_finite() && plot_h.is_finite()) || plot_w <= 0.0 || plot_h <= 0.0 {
        return Vec::new();
    }

    let max_value = series
        .iter()
        .flat_map(|s| s.iter().copied())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .fold(0.0_f32, f32::max);
    let domain_max = nice_domain_max(max_value, layout.y_tick_count).max(1.0);

    let step = plot_w / n as f32;
    let group_w = (step * layout.bar_width_ratio).max(0.0);
    let bar_gap_px = bar_gap_px.max(0.0);
    let bar_w = if series.len() <= 1 {
        group_w
    } else {
        ((group_w - bar_gap_px * (series.len() as f32 - 1.0)) / series.len() as f32).max(0.0)
    };
    let offset = (step - group_w) / 2.0;
    let baseline_y = plot.origin.y.0 + plot_h;

    let mut out = Vec::with_capacity(n * series.len());
    for i in 0..n {
        for (j, s) in series.iter().enumerate() {
            let raw = s[i];
            let value = if raw.is_finite() && raw >= 0.0 {
                raw
            } else {
                0.0
            };
            let h = (value / domain_max) * plot_h;
            let x =
                plot.origin.x.0 + (i as f32) * step + offset + (j as f32) * (bar_w + bar_gap_px);
            let y = baseline_y - h;

            out.push(BarRect {
                rect: Rect::new(
                    fret_core::Point::new(Px(x), Px(y)),
                    fret_core::Size::new(Px(bar_w), Px(h)),
                ),
                value,
            });
        }
    }

    out
}

/// Computes expected rectangles for a stacked (multi-series) Recharts bar chart (single stack group).
pub fn stacked_bar_rects(
    plot: Rect,
    series: &[&[f32]],
    layout: BarChartSeriesLayout,
) -> Vec<BarRect> {
    if series.is_empty() {
        return Vec::new();
    }

    let n = series[0].len();
    if n == 0 || series.iter().any(|s| s.len() != n) {
        return Vec::new();
    }

    let plot_w = plot.size.width.0;
    let plot_h = plot.size.height.0;
    if !(plot_w.is_finite() && plot_h.is_finite()) || plot_w <= 0.0 || plot_h <= 0.0 {
        return Vec::new();
    }

    let max_total = (0..n)
        .map(|i| {
            series
                .iter()
                .map(|s| {
                    let v = s[i];
                    if v.is_finite() && v >= 0.0 { v } else { 0.0 }
                })
                .sum::<f32>()
        })
        .fold(0.0_f32, f32::max);
    let domain_max = nice_domain_max(max_total, layout.y_tick_count).max(1.0);

    let step = plot_w / n as f32;
    let bar_w = (step * layout.bar_width_ratio).max(0.0);
    let offset = (step - bar_w) / 2.0;
    let baseline_y = plot.origin.y.0 + plot_h;

    let mut out = Vec::with_capacity(n * series.len());
    for i in 0..n {
        let mut acc_h = 0.0_f32;
        for s in series.iter() {
            let raw = s[i];
            let value = if raw.is_finite() && raw >= 0.0 {
                raw
            } else {
                0.0
            };
            let h = (value / domain_max) * plot_h;
            let x = plot.origin.x.0 + (i as f32) * step + offset;
            let y = baseline_y - acc_h - h;
            acc_h += h;

            out.push(BarRect {
                rect: Rect::new(
                    fret_core::Point::new(Px(x), Px(y)),
                    fret_core::Size::new(Px(bar_w), Px(h)),
                ),
                value,
            });
        }
    }

    out
}

/// Computes expected rectangles for a horizontal (`layout=\"vertical\"`) single-series Recharts bar chart.
pub fn horizontal_bar_rects(
    plot: Rect,
    values: &[f32],
    layout: BarChartSeriesLayout,
) -> Vec<BarRect> {
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
    let step_y = plot_h / n;
    let bar_h = (step_y * layout.bar_width_ratio).max(0.0);
    let offset_y = (step_y - bar_h) / 2.0;

    values
        .iter()
        .enumerate()
        .map(|(i, raw)| {
            let value = if raw.is_finite() && *raw >= 0.0 {
                *raw
            } else {
                0.0
            };
            let w = (value / domain_max) * plot_w;
            let x = plot.origin.x.0;
            let y = plot.origin.y.0 + (i as f32) * step_y + offset_y;

            BarRect {
                rect: Rect::new(
                    fret_core::Point::new(Px(x), Px(y)),
                    fret_core::Size::new(Px(w), Px(bar_h)),
                ),
                value,
            }
        })
        .collect()
}

/// Computes expected rectangles for a vertical bar chart that includes negative values.
///
/// Recharts will usually place the `0` baseline around the midpoint and pick a symmetric domain.
pub fn symmetric_bar_rects(
    plot: Rect,
    values: &[f32],
    layout: BarChartSeriesLayout,
) -> Vec<BarRect> {
    if values.is_empty() {
        return Vec::new();
    }

    let plot_w = plot.size.width.0;
    let plot_h = plot.size.height.0;
    if !(plot_w.is_finite() && plot_h.is_finite()) || plot_w <= 0.0 || plot_h <= 0.0 {
        return Vec::new();
    }

    let max_abs = values
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .map(|v| v.abs())
        .fold(0.0_f32, f32::max);
    let domain_abs = nice_step(max_abs).max(1.0);

    let n = values.len() as f32;
    let step = plot_w / n;
    let bar_w = (step * layout.bar_width_ratio).max(0.0);
    let offset = (step - bar_w) / 2.0;
    let baseline_y = plot.origin.y.0 + plot_h / 2.0;
    let scale = plot_h / (2.0 * domain_abs);

    values
        .iter()
        .enumerate()
        .map(|(i, raw)| {
            let value = if raw.is_finite() { *raw } else { 0.0 };
            let h = value.abs() * scale;
            let x = plot.origin.x.0 + (i as f32) * step + offset;
            let y = if value >= 0.0 {
                baseline_y - h
            } else {
                baseline_y
            };

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

#[derive(Debug, Clone, Copy)]
pub struct PieLayout {
    pub margin_px: f32,
    pub outer_radius_ratio: f32,
}

impl Default for PieLayout {
    fn default() -> Self {
        Self {
            margin_px: 5.0,
            outer_radius_ratio: 0.8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PieSectorRect {
    pub rect: Rect,
    pub value: f32,
}

fn pie_default_outer_radius(svg_rect: Rect, layout: PieLayout) -> f32 {
    let w = svg_rect.size.width.0;
    let h = svg_rect.size.height.0;
    if !(w.is_finite() && h.is_finite()) || w <= 0.0 || h <= 0.0 {
        return 0.0;
    }

    let inner_w = (w - 2.0 * layout.margin_px).max(0.0);
    let inner_h = (h - 2.0 * layout.margin_px).max(0.0);
    let max_r = inner_w.min(inner_h) / 2.0;
    (layout.outer_radius_ratio * max_r).max(0.0)
}

fn pie_point(cx: f32, cy: f32, r: f32, deg: f32) -> Pt {
    let rad = -deg.to_radians();
    Pt {
        x: cx + r * rad.cos(),
        y: cy + r * rad.sin(),
    }
}

fn push_pie_bounds_points(points: &mut Vec<Pt>, cx: f32, cy: f32, r: f32, start: f32, end: f32) {
    points.push(pie_point(cx, cy, r, start));
    points.push(pie_point(cx, cy, r, end));

    for crit in [0.0_f32, 90.0, 180.0, 270.0, 360.0] {
        if start <= crit && crit <= end {
            points.push(pie_point(cx, cy, r, crit));
        }
    }
}

fn pie_sector_rect(
    svg_rect: Rect,
    layout: PieLayout,
    start: f32,
    end: f32,
    value: f32,
    inner_radius: f32,
    outer_radius: Option<f32>,
) -> Option<PieSectorRect> {
    if !is_valid_rect(svg_rect) || !start.is_finite() || !end.is_finite() || start >= end {
        return None;
    }

    let cx = svg_rect.origin.x.0 + svg_rect.size.width.0 / 2.0;
    let cy = svg_rect.origin.y.0 + svg_rect.size.height.0 / 2.0;

    let outer = outer_radius.unwrap_or_else(|| pie_default_outer_radius(svg_rect, layout));
    if !outer.is_finite() || outer <= 0.0 {
        return None;
    }

    let inner = inner_radius.max(0.0).min(outer);

    let mut points = Vec::new();
    push_pie_bounds_points(&mut points, cx, cy, outer, start, end);
    if inner > 0.0 {
        push_pie_bounds_points(&mut points, cx, cy, inner, start, end);
    } else {
        points.push(Pt { x: cx, y: cy });
    }

    let (min_x, min_y, max_x, max_y) = points_bounds(&points)?;
    Some(PieSectorRect {
        rect: Rect::new(
            fret_core::Point::new(Px(min_x), Px(min_y)),
            fret_core::Size::new(Px(max_x - min_x), Px(max_y - min_y)),
        ),
        value,
    })
}

pub fn pie_sectors(
    svg_rect: Rect,
    values: &[f32],
    inner_radius: f32,
    outer_radius: Option<f32>,
    layout: PieLayout,
) -> Vec<PieSectorRect> {
    let finite_values: Vec<f32> = values
        .iter()
        .copied()
        .map(|v| if v.is_finite() && v > 0.0 { v } else { 0.0 })
        .collect();
    let total: f32 = finite_values.iter().sum();
    if total <= 0.0 {
        return Vec::new();
    }

    let mut start = 0.0_f32;
    let mut out = Vec::with_capacity(values.len());
    for value in finite_values {
        let end = start + (value / total) * 360.0;
        if let Some(sector) = pie_sector_rect(
            svg_rect,
            layout,
            start,
            end,
            value,
            inner_radius,
            outer_radius,
        ) {
            out.push(sector);
        }
        start = end;
    }
    out
}

pub fn pie_sectors_with_outer_radius_overrides(
    svg_rect: Rect,
    values: &[f32],
    inner_radius: f32,
    outer_radius: Option<f32>,
    layout: PieLayout,
    outer_radius_overrides: &[(usize, f32)],
) -> Vec<PieSectorRect> {
    let mut out = pie_sectors(svg_rect, values, inner_radius, outer_radius, layout);
    if out.is_empty() || outer_radius_overrides.is_empty() {
        return out;
    }

    let total: f32 = out.iter().map(|s| s.value).sum();
    if total <= 0.0 {
        return out;
    }

    for (index, override_outer) in outer_radius_overrides {
        if *index >= out.len() {
            continue;
        }

        let mut start = 0.0_f32;
        for (i, s) in out.iter_mut().enumerate() {
            let end = start + (s.value / total) * 360.0;
            if i == *index {
                if let Some(sector) = pie_sector_rect(
                    svg_rect,
                    layout,
                    start,
                    end,
                    s.value,
                    inner_radius,
                    Some(*override_outer),
                ) {
                    s.rect = sector.rect;
                }
                break;
            }
            start = end;
        }
    }

    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveKind {
    Natural,
    Linear,
    Step,
    Monotone,
}

#[derive(Debug, Clone, Copy)]
struct Pt {
    x: f32,
    y: f32,
}

fn is_valid_rect(rect: Rect) -> bool {
    let w = rect.size.width.0;
    let h = rect.size.height.0;
    w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0
}

fn line_points(plot: Rect, values: &[f32], domain_max: f32) -> Vec<Pt> {
    if values.is_empty() || !is_valid_rect(plot) || !domain_max.is_finite() || domain_max <= 0.0 {
        return Vec::new();
    }

    let plot_w = plot.size.width.0;
    let plot_h = plot.size.height.0;
    let baseline_y = plot.origin.y.0 + plot_h;

    let n = values.len();
    let step_x = if n > 1 {
        plot_w / (n as f32 - 1.0)
    } else {
        0.0
    };

    values
        .iter()
        .enumerate()
        .map(|(i, raw)| {
            let value = if raw.is_finite() { *raw } else { 0.0 };
            let x = plot.origin.x.0 + (i as f32) * step_x;
            let y = baseline_y - (value / domain_max) * plot_h;
            Pt { x, y }
        })
        .collect()
}

fn points_bounds(points: &[Pt]) -> Option<(f32, f32, f32, f32)> {
    let mut it = points.iter();
    let first = it.next()?;
    let mut min_x = first.x;
    let mut max_x = first.x;
    let mut min_y = first.y;
    let mut max_y = first.y;

    for p in it {
        min_x = min_x.min(p.x);
        max_x = max_x.max(p.x);
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
    }

    Some((min_x, min_y, max_x, max_y))
}

fn natural_controls_1d(knots: &[f32]) -> Vec<(f32, f32)> {
    if knots.len() < 2 {
        return Vec::new();
    }

    let n = knots.len() - 1;
    if n == 1 {
        let p0 = knots[0];
        let p1 = knots[1];
        let c1 = (2.0 * p0 + p1) / 3.0;
        let c2 = (2.0 * p1 + p0) / 3.0;
        return vec![(c1, c2)];
    }

    let mut a = vec![0.0_f32; n];
    let mut b = vec![0.0_f32; n];
    let mut c = vec![0.0_f32; n];
    let mut r = vec![0.0_f32; n];

    b[0] = 2.0;
    c[0] = 1.0;
    r[0] = knots[0] + 2.0 * knots[1];

    for i in 1..n - 1 {
        a[i] = 1.0;
        b[i] = 4.0;
        c[i] = 1.0;
        r[i] = 4.0 * knots[i] + 2.0 * knots[i + 1];
    }

    a[n - 1] = 2.0;
    b[n - 1] = 7.0;
    r[n - 1] = 8.0 * knots[n - 1] + knots[n];

    for i in 1..n {
        let m = a[i] / b[i - 1];
        b[i] -= m * c[i - 1];
        r[i] -= m * r[i - 1];
    }

    let mut p1 = vec![0.0_f32; n];
    p1[n - 1] = r[n - 1] / b[n - 1];
    for i in (0..n - 1).rev() {
        p1[i] = (r[i] - c[i] * p1[i + 1]) / b[i];
    }

    let mut p2 = vec![0.0_f32; n];
    for i in 0..n - 1 {
        p2[i] = 2.0 * knots[i + 1] - p1[i + 1];
    }
    p2[n - 1] = (knots[n] + p1[n - 1]) / 2.0;

    p1.into_iter().zip(p2).collect()
}

fn cubic_eval(p0: f32, c1: f32, c2: f32, p1: f32, t: f32) -> f32 {
    let a = -p0 + 3.0 * c1 - 3.0 * c2 + p1;
    let b = 3.0 * p0 - 6.0 * c1 + 3.0 * c2;
    let c = -3.0 * p0 + 3.0 * c1;
    ((a * t + b) * t + c) * t + p0
}

fn cubic_extrema(p0: f32, c1: f32, c2: f32, p1: f32) -> [Option<f32>; 2] {
    let a = -p0 + 3.0 * c1 - 3.0 * c2 + p1;
    let b = 3.0 * p0 - 6.0 * c1 + 3.0 * c2;
    let c = -3.0 * p0 + 3.0 * c1;

    let qa = 3.0 * a;
    let qb = 2.0 * b;
    let qc = c;

    if qa.abs() < 1e-9 {
        if qb.abs() < 1e-9 {
            return [None, None];
        }
        let t = -qc / qb;
        return if (0.0..=1.0).contains(&t) {
            [Some(t), None]
        } else {
            [None, None]
        };
    }

    let disc = qb * qb - 4.0 * qa * qc;
    if disc < 0.0 {
        return [None, None];
    }
    let s = disc.sqrt();
    let t0 = (-qb - s) / (2.0 * qa);
    let t1 = (-qb + s) / (2.0 * qa);

    let mut out = [None, None];
    if (0.0..=1.0).contains(&t0) {
        out[0] = Some(t0);
    }
    if (0.0..=1.0).contains(&t1) {
        out[1] = Some(t1);
    }
    out
}

fn natural_curve_bounds(points: &[Pt]) -> Option<(f32, f32, f32, f32)> {
    if points.len() < 2 {
        return points_bounds(points);
    }

    let xs: Vec<f32> = points.iter().map(|p| p.x).collect();
    let ys: Vec<f32> = points.iter().map(|p| p.y).collect();
    let x_ctrl = natural_controls_1d(&xs);
    let y_ctrl = natural_controls_1d(&ys);
    if x_ctrl.len() != y_ctrl.len() {
        return None;
    }

    let mut min_x = xs[0];
    let mut max_x = xs[0];
    let mut min_y = ys[0];
    let mut max_y = ys[0];

    for i in 0..x_ctrl.len() {
        let p0 = points[i];
        let p1 = points[i + 1];
        let (x1, x2) = x_ctrl[i];
        let (y1, y2) = y_ctrl[i];

        min_x = min_x.min(p0.x).min(p1.x);
        max_x = max_x.max(p0.x).max(p1.x);
        min_y = min_y.min(p0.y).min(p1.y);
        max_y = max_y.max(p0.y).max(p1.y);

        for t in cubic_extrema(p0.x, x1, x2, p1.x).into_iter().flatten() {
            let x = cubic_eval(p0.x, x1, x2, p1.x, t);
            min_x = min_x.min(x);
            max_x = max_x.max(x);
        }
        for t in cubic_extrema(p0.y, y1, y2, p1.y).into_iter().flatten() {
            let y = cubic_eval(p0.y, y1, y2, p1.y, t);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }

    Some((min_x, min_y, max_x, max_y))
}

pub fn line_curve_bounds(
    plot: Rect,
    values: &[f32],
    kind: CurveKind,
    domain_max: f32,
) -> Option<Rect> {
    let points = line_points(plot, values, domain_max);
    let (min_x, min_y, max_x, max_y) = match kind {
        CurveKind::Natural => natural_curve_bounds(&points)?,
        CurveKind::Linear | CurveKind::Step | CurveKind::Monotone => points_bounds(&points)?,
    };

    Some(Rect::new(
        fret_core::Point::new(Px(min_x), Px(min_y)),
        fret_core::Size::new(Px(max_x - min_x), Px(max_y - min_y)),
    ))
}

pub fn nice_domain_max_for_values(values: &[f32], tick_count: usize) -> f32 {
    let max_value = values
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold(0.0_f32, f32::max);
    nice_domain_max(max_value, tick_count).max(1.0)
}
