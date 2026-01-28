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

/// Returns a "nice" tick step for polar charts.
///
/// Recharts polar charts in the shadcn/ui v4 examples commonly round the rough step up to the next
/// half power-of-ten increment (e.g. 51.75 => 55, 71.25 => 75).
fn nice_step_polar(raw_step: f32) -> f32 {
    if !raw_step.is_finite() || raw_step <= 0.0 {
        return 0.0;
    }

    let exponent = raw_step.abs().log10().floor();
    let base = 10.0_f32.powf(exponent) / 2.0;
    if base <= 0.0 {
        return raw_step;
    }
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

/// Computes an upper-bounded "nice" max for polar chart domains.
pub fn nice_polar_domain_max(max_value: f32, tick_count: usize) -> f32 {
    if !max_value.is_finite() || max_value <= 0.0 {
        return 0.0;
    }
    let tick_count = tick_count.max(2);
    let slots = (tick_count - 1) as f32;
    let step = nice_step_polar(max_value / slots);
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

fn push_sector_bounds_points(points: &mut Vec<Pt>, cx: f32, cy: f32, r: f32, start: f32, end: f32) {
    points.push(pie_point(cx, cy, r, start));
    points.push(pie_point(cx, cy, r, end));

    let start_k = (start / 90.0).floor() as i32;
    let end_k = (end / 90.0).ceil() as i32;
    for k in start_k..=end_k {
        let crit = (k as f32) * 90.0;
        if start <= crit && crit <= end {
            points.push(pie_point(cx, cy, r, crit));
        }
    }
}

/// Returns the bounding rect of an annular sector (donut slice) in Recharts polar coordinates.
///
/// Angles are in degrees and follow the same convention as `pie_sectors` (0° on the +X axis).
pub fn annular_sector_rect(
    svg_rect: Rect,
    start_deg: f32,
    end_deg: f32,
    inner_radius: f32,
    outer_radius: f32,
) -> Option<Rect> {
    if !is_valid_rect(svg_rect)
        || !start_deg.is_finite()
        || !end_deg.is_finite()
        || !inner_radius.is_finite()
        || !outer_radius.is_finite()
    {
        return None;
    }

    // Keep angles in their original (possibly negative / > 360°) space.
    // Recharts (via d3-shape) treats `startAngle`/`endAngle` as raw numbers; callers that want
    // to wrap across the 0° boundary pass values outside the 0..360 range (e.g. `end=380`).
    let mut start = start_deg;
    let mut end = end_deg;
    if end < start {
        std::mem::swap(&mut start, &mut end);
    }

    let span = end - start;
    if span <= 0.0 {
        return None;
    }

    let outer = outer_radius.max(0.0);
    if outer <= 0.0 {
        return None;
    }
    let inner = inner_radius.max(0.0).min(outer);

    let cx = svg_rect.origin.x.0 + svg_rect.size.width.0 / 2.0;
    let cy = svg_rect.origin.y.0 + svg_rect.size.height.0 / 2.0;

    if span >= 360.0 {
        return Some(Rect::new(
            fret_core::Point::new(Px(cx - outer), Px(cy - outer)),
            fret_core::Size::new(Px(2.0 * outer), Px(2.0 * outer)),
        ));
    }

    let mut points = Vec::new();
    push_sector_bounds_points(&mut points, cx, cy, outer, start, end);
    if inner > 0.0 {
        push_sector_bounds_points(&mut points, cx, cy, inner, start, end);
    } else {
        points.push(Pt { x: cx, y: cy });
    }

    let (min_x, min_y, max_x, max_y) = points_bounds(&points)?;
    Some(Rect::new(
        fret_core::Point::new(Px(min_x), Px(min_y)),
        fret_core::Size::new(Px(max_x - min_x), Px(max_y - min_y)),
    ))
}

#[derive(Debug, Clone, Copy)]
struct Bounds {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl Bounds {
    fn new(x: f32, y: f32) -> Self {
        Self {
            min_x: x,
            min_y: y,
            max_x: x,
            max_y: y,
        }
    }

    fn include(&mut self, x: f32, y: f32) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }
}

fn norm_radians(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut out = a % tau;
    if out < 0.0 {
        out += tau;
    }
    out
}

fn arc_sweep_contains(start: f32, end: f32, ccw: bool, angle: f32) -> bool {
    let tau = std::f32::consts::TAU;
    let eps = 1e-6_f32;
    let start = norm_radians(start);
    let end = norm_radians(end);
    let angle = norm_radians(angle);

    if !ccw {
        let sweep = norm_radians(end - start);
        let t = norm_radians(angle - start);
        t <= sweep + eps || (sweep >= tau - eps)
    } else {
        let sweep = norm_radians(start - end);
        let t = norm_radians(start - angle);
        t <= sweep + eps || (sweep >= tau - eps)
    }
}

fn include_arc_bounds(
    bounds: &mut Bounds,
    cx: f32,
    cy: f32,
    r: f32,
    start: f32,
    end: f32,
    ccw: bool,
) {
    let eps = 1e-6_f32;
    if !r.is_finite() || r <= eps {
        return;
    }

    let (sx, sy) = (cx + r * start.cos(), cy + r * start.sin());
    let (ex, ey) = (cx + r * end.cos(), cy + r * end.sin());
    bounds.include(sx, sy);
    bounds.include(ex, ey);

    for crit in [
        0.0_f32,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        3.0 * std::f32::consts::FRAC_PI_2,
    ] {
        if arc_sweep_contains(start, end, ccw, crit) {
            bounds.include(cx + r * crit.cos(), cy + r * crit.sin());
        }
    }
}

fn bounds_include_point(bounds: &mut Option<Bounds>, x: f32, y: f32) {
    match bounds.as_mut() {
        Some(b) => b.include(x, y),
        None => *bounds = Some(Bounds::new(x, y)),
    }
}

fn bounds_include_arc(
    bounds: &mut Option<Bounds>,
    cx: f32,
    cy: f32,
    r: f32,
    start: f32,
    end: f32,
    ccw: bool,
) {
    let eps = 1e-6_f32;
    if !r.is_finite() || r <= eps {
        return;
    }

    match bounds.as_mut() {
        Some(b) => include_arc_bounds(b, cx, cy, r, start, end, ccw),
        None => {
            let (sx, sy) = (cx + r * start.cos(), cy + r * start.sin());
            let mut b = Bounds::new(sx, sy);
            include_arc_bounds(&mut b, cx, cy, r, start, end, ccw);
            *bounds = Some(b);
        }
    }
}

fn recharts_sign(v: f32) -> f32 {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}

fn recharts_delta_angle(start_deg: f32, end_deg: f32) -> f32 {
    let sign = recharts_sign(end_deg - start_deg);
    let delta = (end_deg - start_deg).abs().min(359.999);
    sign * delta
}

#[derive(Debug, Clone, Copy)]
struct TangentCircle {
    center: Pt,
    circle_tangency: Pt,
    line_tangency: Pt,
    theta_deg: f32,
}

fn recharts_tangent_circle(
    cx: f32,
    cy: f32,
    radius: f32,
    angle_deg: f32,
    sign: f32,
    is_external: bool,
    corner_radius: f32,
    corner_is_external: bool,
) -> Option<TangentCircle> {
    if !radius.is_finite()
        || !angle_deg.is_finite()
        || !sign.is_finite()
        || !corner_radius.is_finite()
    {
        return None;
    }

    let center_radius = radius + corner_radius * (if is_external { 1.0 } else { -1.0 });
    if !center_radius.is_finite() || center_radius == 0.0 {
        return None;
    }

    let ratio = (corner_radius / center_radius).clamp(-1.0, 1.0);
    let theta_deg = ratio.asin().to_degrees();

    let center_angle = if corner_is_external {
        angle_deg
    } else {
        angle_deg + sign * theta_deg
    };
    let center = pie_point(cx, cy, center_radius, center_angle);
    let circle_tangency = pie_point(cx, cy, radius, center_angle);

    let line_tangency_angle = if corner_is_external {
        angle_deg - sign * theta_deg
    } else {
        angle_deg
    };
    let line_tangency_radius = center_radius * (theta_deg.to_radians()).cos();
    let line_tangency = pie_point(cx, cy, line_tangency_radius, line_tangency_angle);

    Some(TangentCircle {
        center,
        circle_tangency,
        line_tangency,
        theta_deg: theta_deg.abs(),
    })
}

/// Returns the bounding rect of an annular sector with Recharts-style rounded corners.
///
/// Recharts' `Sector` shape (`recharts@2.15.1`) implements its own corner-radius math (not d3),
/// and RadialBar uses that `Sector` implementation. This helper mirrors the geometry well
/// enough for bounding-box golden tests.
///
/// Angles are in degrees and follow Recharts' convention (0° on the +X axis).
pub fn annular_sector_rect_with_corner_radius(
    svg_rect: Rect,
    start_deg: f32,
    end_deg: f32,
    inner_radius: f32,
    outer_radius: f32,
    corner_radius: f32,
) -> Option<Rect> {
    if !is_valid_rect(svg_rect)
        || !start_deg.is_finite()
        || !end_deg.is_finite()
        || !inner_radius.is_finite()
        || !outer_radius.is_finite()
    {
        return None;
    }

    let mut r0 = inner_radius.max(0.0);
    let mut r1 = outer_radius.max(0.0);
    if r1 <= 0.0 {
        return None;
    }
    if r1 < r0 {
        std::mem::swap(&mut r0, &mut r1);
    }

    let cx = svg_rect.origin.x.0 + svg_rect.size.width.0 / 2.0;
    let cy = svg_rect.origin.y.0 + svg_rect.size.height.0 / 2.0;

    let eps = 1e-6_f32;

    let delta_radius = r1 - r0;
    if delta_radius <= eps {
        return None;
    }

    let cr = if corner_radius.is_finite() && corner_radius > 0.0 {
        corner_radius.min(delta_radius / 2.0)
    } else {
        0.0
    };

    // 360° sectors are (almost) full circles in Recharts; bbox matches the full outer circle.
    if (start_deg - end_deg).abs() >= 360.0 - 1e-3 {
        return Some(Rect::new(
            fret_core::Point::new(Px(cx - r1), Px(cy - r1)),
            fret_core::Size::new(Px(2.0 * r1), Px(2.0 * r1)),
        ));
    }

    let sign = recharts_sign(end_deg - start_deg);
    if sign == 0.0 {
        return None;
    }

    // If we can't apply rounded corners cleanly, fall back to the non-cornered sector path.
    let fallback = || {
        let delta = recharts_delta_angle(start_deg, end_deg);
        annular_sector_rect(svg_rect, start_deg, start_deg + delta, r0, r1)
    };

    if cr <= eps {
        return fallback();
    }

    let abs_angle = (start_deg - end_deg).abs();

    let outer_start = recharts_tangent_circle(cx, cy, r1, start_deg, sign, false, cr, false)?;
    let outer_end = recharts_tangent_circle(cx, cy, r1, end_deg, -sign, false, cr, false)?;
    let outer_arc_angle = abs_angle - outer_start.theta_deg - outer_end.theta_deg;
    if outer_arc_angle < 0.0 {
        return fallback();
    }

    let sweep_flag_corner = sign < 0.0;
    let corner_ccw = !sweep_flag_corner;

    let mut bounds: Option<Bounds> = None;
    let point_angle = |center: Pt, p: Pt| (p.y - center.y).atan2(p.x - center.x);

    // Outer start corner: solt -> soct
    bounds_include_point(
        &mut bounds,
        outer_start.line_tangency.x,
        outer_start.line_tangency.y,
    );
    bounds_include_point(
        &mut bounds,
        outer_start.circle_tangency.x,
        outer_start.circle_tangency.y,
    );
    bounds_include_arc(
        &mut bounds,
        outer_start.center.x,
        outer_start.center.y,
        cr,
        point_angle(outer_start.center, outer_start.line_tangency),
        point_angle(outer_start.center, outer_start.circle_tangency),
        corner_ccw,
    );

    // Outer main arc: soct -> eoct
    let outer_center = Pt { x: cx, y: cy };
    let outer_start_rad = point_angle(outer_center, outer_start.circle_tangency);
    let outer_sweep_rad = outer_arc_angle.to_radians();
    let outer_end_rad = if corner_ccw {
        outer_start_rad - outer_sweep_rad
    } else {
        outer_start_rad + outer_sweep_rad
    };
    bounds_include_arc(
        &mut bounds,
        cx,
        cy,
        r1,
        outer_start_rad,
        outer_end_rad,
        corner_ccw,
    );

    // Outer end corner: eoct -> eolt
    bounds_include_point(
        &mut bounds,
        outer_end.circle_tangency.x,
        outer_end.circle_tangency.y,
    );
    bounds_include_point(
        &mut bounds,
        outer_end.line_tangency.x,
        outer_end.line_tangency.y,
    );
    bounds_include_arc(
        &mut bounds,
        outer_end.center.x,
        outer_end.center.y,
        cr,
        point_angle(outer_end.center, outer_end.circle_tangency),
        point_angle(outer_end.center, outer_end.line_tangency),
        corner_ccw,
    );

    if r0 <= eps {
        bounds_include_point(&mut bounds, cx, cy);
    } else {
        let inner_start = recharts_tangent_circle(cx, cy, r0, start_deg, sign, true, cr, false)?;
        let inner_end = recharts_tangent_circle(cx, cy, r0, end_deg, -sign, true, cr, false)?;
        let inner_arc_angle = abs_angle - inner_start.theta_deg - inner_end.theta_deg;
        if inner_arc_angle < 0.0 {
            return fallback();
        }

        // Inner end corner: eilt -> eict
        bounds_include_point(
            &mut bounds,
            inner_end.line_tangency.x,
            inner_end.line_tangency.y,
        );
        bounds_include_point(
            &mut bounds,
            inner_end.circle_tangency.x,
            inner_end.circle_tangency.y,
        );
        bounds_include_arc(
            &mut bounds,
            inner_end.center.x,
            inner_end.center.y,
            cr,
            point_angle(inner_end.center, inner_end.line_tangency),
            point_angle(inner_end.center, inner_end.circle_tangency),
            corner_ccw,
        );

        // Inner main arc: eict -> sict (note the opposite sweep flag in Recharts)
        let inner_sweep_flag = sign > 0.0;
        let inner_ccw = !inner_sweep_flag;
        let inner_start_rad = point_angle(outer_center, inner_end.circle_tangency);
        let inner_sweep_rad = inner_arc_angle.to_radians();
        let inner_end_rad = if inner_ccw {
            inner_start_rad - inner_sweep_rad
        } else {
            inner_start_rad + inner_sweep_rad
        };
        bounds_include_arc(
            &mut bounds,
            cx,
            cy,
            r0,
            inner_start_rad,
            inner_end_rad,
            inner_ccw,
        );

        // Inner start corner: sict -> silt
        bounds_include_point(
            &mut bounds,
            inner_start.circle_tangency.x,
            inner_start.circle_tangency.y,
        );
        bounds_include_point(
            &mut bounds,
            inner_start.line_tangency.x,
            inner_start.line_tangency.y,
        );
        bounds_include_arc(
            &mut bounds,
            inner_start.center.x,
            inner_start.center.y,
            cr,
            point_angle(inner_start.center, inner_start.circle_tangency),
            point_angle(inner_start.center, inner_start.line_tangency),
            corner_ccw,
        );
    }

    let b = bounds?;
    Some(Rect::new(
        fret_core::Point::new(Px(b.min_x), Px(b.min_y)),
        fret_core::Size::new(Px(b.max_x - b.min_x), Px(b.max_y - b.min_y)),
    ))
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

#[derive(Debug, Clone, Copy)]
pub struct PolarChartLayout {
    pub margin_top_px: f32,
    pub margin_right_px: f32,
    pub margin_bottom_px: f32,
    pub margin_left_px: f32,
    pub outer_radius_ratio: f32,
}

impl Default for PolarChartLayout {
    fn default() -> Self {
        Self {
            margin_top_px: 5.0,
            margin_right_px: 5.0,
            margin_bottom_px: 5.0,
            margin_left_px: 5.0,
            outer_radius_ratio: 0.8,
        }
    }
}

fn polar_default_outer_radius(svg_rect: Rect, layout: PolarChartLayout) -> f32 {
    let w = svg_rect.size.width.0;
    let h = svg_rect.size.height.0;
    if !(w.is_finite() && h.is_finite()) || w <= 0.0 || h <= 0.0 {
        return 0.0;
    }

    let inner_w = (w - layout.margin_left_px - layout.margin_right_px).max(0.0);
    let inner_h = (h - layout.margin_top_px - layout.margin_bottom_px).max(0.0);
    let max_r = inner_w.min(inner_h) / 2.0;
    (layout.outer_radius_ratio * max_r).max(0.0)
}

fn polar_regular_polygon_points(cx: f32, cy: f32, r: f32, sides: usize) -> Vec<Pt> {
    if sides < 3 || !r.is_finite() || r <= 0.0 {
        return Vec::new();
    }

    let step = 360.0 / sides as f32;
    (0..sides)
        .map(|i| {
            let deg = 90.0 - (i as f32) * step;
            pie_point(cx, cy, r, deg)
        })
        .collect()
}

pub fn radar_grid_polygon_rects(
    svg_rect: Rect,
    sides: usize,
    layout: PolarChartLayout,
) -> Vec<Rect> {
    if !is_valid_rect(svg_rect) || sides < 3 {
        return Vec::new();
    }

    let outer = polar_default_outer_radius(svg_rect, layout);
    if !(outer.is_finite() && outer > 0.0) {
        return Vec::new();
    }

    // Recharts polar charts default to 5 ticks (including 0), and includes a degenerate
    // center polygon (0x0 rect) as the smallest concentric element.
    let tick_count = 5_usize;
    let denom = (tick_count - 1) as f32;

    let cx = svg_rect.origin.x.0 + svg_rect.size.width.0 / 2.0;
    let cy = svg_rect.origin.y.0 + svg_rect.size.height.0 / 2.0;

    (0..tick_count)
        .filter_map(|i| {
            let r = outer * (i as f32 / denom);
            if r == 0.0 {
                return Some(Rect::new(
                    fret_core::Point::new(Px(cx), Px(cy)),
                    fret_core::Size::new(Px(0.0), Px(0.0)),
                ));
            }

            let points = polar_regular_polygon_points(cx, cy, r, sides);
            let (min_x, min_y, max_x, max_y) = points_bounds(&points)?;
            Some(Rect::new(
                fret_core::Point::new(Px(min_x), Px(min_y)),
                fret_core::Size::new(Px(max_x - min_x), Px(max_y - min_y)),
            ))
        })
        .collect()
}

pub fn radar_grid_circle_rects(svg_rect: Rect, layout: PolarChartLayout) -> Vec<Rect> {
    if !is_valid_rect(svg_rect) {
        return Vec::new();
    }

    let outer = polar_default_outer_radius(svg_rect, layout);
    if !(outer.is_finite() && outer > 0.0) {
        return Vec::new();
    }

    let tick_count = 5_usize;
    let denom = (tick_count - 1) as f32;

    let cx = svg_rect.origin.x.0 + svg_rect.size.width.0 / 2.0;
    let cy = svg_rect.origin.y.0 + svg_rect.size.height.0 / 2.0;

    (0..tick_count)
        .map(|i| {
            let r = outer * (i as f32 / denom);
            Rect::new(
                fret_core::Point::new(Px(cx - r), Px(cy - r)),
                fret_core::Size::new(Px(2.0 * r), Px(2.0 * r)),
            )
        })
        .collect()
}

pub fn radar_grid_polygon_rects_with_radii(
    svg_rect: Rect,
    sides: usize,
    radii_px: &[f32],
) -> Vec<Rect> {
    if !is_valid_rect(svg_rect) || sides < 3 {
        return Vec::new();
    }

    let cx = svg_rect.origin.x.0 + svg_rect.size.width.0 / 2.0;
    let cy = svg_rect.origin.y.0 + svg_rect.size.height.0 / 2.0;

    radii_px
        .iter()
        .copied()
        .filter(|r| r.is_finite() && *r > 0.0)
        .filter_map(|r| {
            let points = polar_regular_polygon_points(cx, cy, r, sides);
            let (min_x, min_y, max_x, max_y) = points_bounds(&points)?;
            Some(Rect::new(
                fret_core::Point::new(Px(min_x), Px(min_y)),
                fret_core::Size::new(Px(max_x - min_x), Px(max_y - min_y)),
            ))
        })
        .collect()
}

pub fn radar_polygon_rect(
    svg_rect: Rect,
    values: &[f32],
    domain_max: f32,
    layout: PolarChartLayout,
) -> Option<Rect> {
    if !is_valid_rect(svg_rect) || values.len() < 3 || !(domain_max.is_finite() && domain_max > 0.0)
    {
        return None;
    }

    let outer = polar_default_outer_radius(svg_rect, layout);
    if !(outer.is_finite() && outer > 0.0) {
        return None;
    }

    let cx = svg_rect.origin.x.0 + svg_rect.size.width.0 / 2.0;
    let cy = svg_rect.origin.y.0 + svg_rect.size.height.0 / 2.0;
    let step = 360.0 / values.len() as f32;

    let points: Vec<Pt> = values
        .iter()
        .copied()
        .enumerate()
        .map(|(i, v)| {
            let clamped = if v.is_finite() && v > 0.0 { v } else { 0.0 };
            let r = outer * (clamped / domain_max);
            let deg = 90.0 - (i as f32) * step;
            pie_point(cx, cy, r, deg)
        })
        .collect();

    let (min_x, min_y, max_x, max_y) = points_bounds(&points)?;
    Some(Rect::new(
        fret_core::Point::new(Px(min_x), Px(min_y)),
        fret_core::Size::new(Px(max_x - min_x), Px(max_y - min_y)),
    ))
}

pub fn radar_dot_rects(
    svg_rect: Rect,
    values: &[f32],
    domain_max: f32,
    dot_radius_px: f32,
    layout: PolarChartLayout,
) -> Vec<Rect> {
    if dot_radius_px <= 0.0 || !dot_radius_px.is_finite() {
        return Vec::new();
    }

    if !is_valid_rect(svg_rect) || values.len() < 3 || !(domain_max.is_finite() && domain_max > 0.0)
    {
        return Vec::new();
    }

    let outer = polar_default_outer_radius(svg_rect, layout);
    if !(outer.is_finite() && outer > 0.0) {
        return Vec::new();
    }

    let cx = svg_rect.origin.x.0 + svg_rect.size.width.0 / 2.0;
    let cy = svg_rect.origin.y.0 + svg_rect.size.height.0 / 2.0;
    let step = 360.0 / values.len() as f32;

    values
        .iter()
        .copied()
        .enumerate()
        .map(|(i, v)| {
            let clamped = if v.is_finite() && v > 0.0 { v } else { 0.0 };
            let r = outer * (clamped / domain_max);
            let deg = 90.0 - (i as f32) * step;
            let p = pie_point(cx, cy, r, deg);
            Rect::new(
                fret_core::Point::new(Px(p.x - dot_radius_px), Px(p.y - dot_radius_px)),
                fret_core::Size::new(Px(2.0 * dot_radius_px), Px(2.0 * dot_radius_px)),
            )
        })
        .collect()
}

/// Computes concentric circle rects for a RadialBarChart `PolarGrid gridType="circle"`.
///
/// In the shadcn/ui v4 examples, Recharts emits one circle per category (excluding the outer
/// boundary), with radii linearly spaced between `inner_radius` and `outer_radius`.
pub fn radial_grid_circle_rects(
    svg_rect: Rect,
    inner_radius: f32,
    outer_radius: f32,
    category_count: usize,
) -> Vec<Rect> {
    if !is_valid_rect(svg_rect) || category_count == 0 {
        return Vec::new();
    }

    let inner = inner_radius.max(0.0);
    let outer = outer_radius.max(inner);

    let step = (outer - inner) / category_count as f32;
    let cx = svg_rect.origin.x.0 + svg_rect.size.width.0 / 2.0;
    let cy = svg_rect.origin.y.0 + svg_rect.size.height.0 / 2.0;

    (0..category_count)
        .map(|i| {
            let r = inner + (i as f32) * step;
            Rect::new(
                fret_core::Point::new(Px(cx - r), Px(cy - r)),
                fret_core::Size::new(Px(2.0 * r), Px(2.0 * r)),
            )
        })
        .collect()
}

pub fn polar_circle_rects(svg_rect: Rect, radii: &[f32]) -> Vec<Rect> {
    if !is_valid_rect(svg_rect) || radii.is_empty() {
        return Vec::new();
    }

    let cx = svg_rect.origin.x.0 + svg_rect.size.width.0 / 2.0;
    let cy = svg_rect.origin.y.0 + svg_rect.size.height.0 / 2.0;

    radii
        .iter()
        .copied()
        .filter(|r| r.is_finite() && *r >= 0.0)
        .map(|r| {
            Rect::new(
                fret_core::Point::new(Px(cx - r), Px(cy - r)),
                fret_core::Size::new(Px(2.0 * r), Px(2.0 * r)),
            )
        })
        .collect()
}

fn radial_band_radii(
    inner_radius: f32,
    outer_radius: f32,
    category_count: usize,
    half_thickness: f32,
) -> Vec<(f32, f32)> {
    if category_count == 0 || !half_thickness.is_finite() || half_thickness <= 0.0 {
        return Vec::new();
    }

    let inner = inner_radius.max(0.0);
    let outer = outer_radius.max(inner);
    let step = (outer - inner) / category_count as f32;

    (0..category_count)
        .map(|i| {
            let center = inner + (i as f32) * step;
            (center - half_thickness, center + half_thickness)
        })
        .collect()
}

pub fn radial_bar_sector_rects(
    svg_rect: Rect,
    values: &[f32],
    domain_max: f32,
    start_angle: f32,
    end_angle: f32,
    inner_radius: f32,
    outer_radius: f32,
    half_thickness: f32,
) -> Vec<Rect> {
    if values.is_empty()
        || !is_valid_rect(svg_rect)
        || !domain_max.is_finite()
        || domain_max <= 0.0
        || !start_angle.is_finite()
        || !end_angle.is_finite()
    {
        return Vec::new();
    }

    let span = end_angle - start_angle;
    let bands = radial_band_radii(inner_radius, outer_radius, values.len(), half_thickness);
    if bands.len() != values.len() {
        return Vec::new();
    }

    values
        .iter()
        .copied()
        .zip(bands)
        .filter_map(|(raw, (rin, rout))| {
            let value = if raw.is_finite() && raw > 0.0 {
                raw
            } else {
                0.0
            };
            let end = start_angle + (value / domain_max) * span;
            annular_sector_rect(svg_rect, start_angle, end, rin, rout)
        })
        .collect()
}

pub fn radial_bar_background_rects(
    svg_rect: Rect,
    category_count: usize,
    start_angle: f32,
    end_angle: f32,
    inner_radius: f32,
    outer_radius: f32,
    half_thickness: f32,
) -> Vec<Rect> {
    if category_count == 0
        || !is_valid_rect(svg_rect)
        || !start_angle.is_finite()
        || !end_angle.is_finite()
    {
        return Vec::new();
    }

    radial_band_radii(inner_radius, outer_radius, category_count, half_thickness)
        .into_iter()
        .filter_map(|(rin, rout)| annular_sector_rect(svg_rect, start_angle, end_angle, rin, rout))
        .collect()
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

pub fn nice_polar_domain_max_for_values(values: &[f32], tick_count: usize) -> f32 {
    let max_value = values
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold(0.0_f32, f32::max);
    nice_polar_domain_max(max_value, tick_count).max(1.0)
}
