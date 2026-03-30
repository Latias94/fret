use fret_core::geometry::{Point, Px, Size};

use crate::cartesian::{AxisScale, DataRect};

fn mix_u64(mut state: u64, v: u64) -> u64 {
    state ^= v
        .wrapping_add(0x9e3779b97f4a7c15)
        .wrapping_add(state << 6)
        .wrapping_add(state >> 2);
    state
}

fn mix_f64_bits(state: u64, v: f64) -> u64 {
    mix_u64(state, v.to_bits())
}

pub fn data_rect_key(bounds: DataRect) -> u64 {
    let mut key = 0u64;
    key = mix_f64_bits(key, bounds.x_min);
    key = mix_f64_bits(key, bounds.x_max);
    key = mix_f64_bits(key, bounds.y_min);
    key = mix_f64_bits(key, bounds.y_max);
    key
}

pub fn data_rect_key_scaled(bounds: DataRect, x_scale: AxisScale, y_scale: AxisScale) -> u64 {
    let mut key = data_rect_key(bounds);
    key = mix_u64(key, x_scale.key());
    key = mix_u64(key, y_scale.key());
    key
}

pub fn sanitize_data_rect(bounds: DataRect) -> DataRect {
    let mut x0 = bounds.x_min;
    let mut x1 = bounds.x_max;
    let mut y0 = bounds.y_min;
    let mut y1 = bounds.y_max;

    if !x0.is_finite() || !x1.is_finite() || !y0.is_finite() || !y1.is_finite() {
        return DataRect {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
        };
    }

    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
    }
    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
    }

    let min_span = 1.0e-12_f64;

    let w = x1 - x0;
    if !w.is_finite() || w.abs() < min_span {
        let cx = (x0 + x1) * 0.5;
        x0 = cx - 0.5;
        x1 = cx + 0.5;
    }

    let h = y1 - y0;
    if !h.is_finite() || h.abs() < min_span {
        let cy = (y0 + y1) * 0.5;
        y0 = cy - 0.5;
        y1 = cy + 0.5;
    }

    DataRect {
        x_min: x0,
        x_max: x1,
        y_min: y0,
        y_max: y1,
    }
}

pub fn sanitize_data_rect_scaled(
    bounds: DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    let mut x0 = bounds.x_min;
    let mut x1 = bounds.x_max;
    let mut y0 = bounds.y_min;
    let mut y1 = bounds.y_max;

    if !x0.is_finite() || !x1.is_finite() || !y0.is_finite() || !y1.is_finite() {
        return DataRect {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
        };
    }

    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
    }
    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
    }

    let (x0, x1) = x_scale.sanitize_bounds(x0, x1);
    let (y0, y1) = y_scale.sanitize_bounds(y0, y1);

    match (x_scale, y_scale) {
        (AxisScale::Linear, AxisScale::Linear) => sanitize_data_rect(DataRect {
            x_min: x0,
            x_max: x1,
            y_min: y0,
            y_max: y1,
        }),
        _ => {
            // Ensure a minimum span in axis space. For log scales, this corresponds to a
            // minimum multiplicative ratio (>= 10^(min_span)).
            let min_span_axis = 1.0e-12_f64;

            let (mut x0, mut x1) = (x0, x1);
            let (mut y0, mut y1) = (y0, y1);

            if let (Some(ax0), Some(ax1)) = (x_scale.to_axis(x0), x_scale.to_axis(x1))
                && (ax1 - ax0).abs() < min_span_axis
            {
                if x_scale == AxisScale::Log10 {
                    x1 = x0 * 10.0;
                } else {
                    let cx = (x0 + x1) * 0.5;
                    x0 = cx - 0.5;
                    x1 = cx + 0.5;
                }
            }

            if let (Some(ay0), Some(ay1)) = (y_scale.to_axis(y0), y_scale.to_axis(y1))
                && (ay1 - ay0).abs() < min_span_axis
            {
                if y_scale == AxisScale::Log10 {
                    y1 = y0 * 10.0;
                } else {
                    let cy = (y0 + y1) * 0.5;
                    y0 = cy - 0.5;
                    y1 = cy + 0.5;
                }
            }

            DataRect {
                x_min: x0,
                x_max: x1,
                y_min: y0,
                y_max: y1,
            }
        }
    }
}

pub fn expand_data_bounds(data: DataRect, overscroll_fraction: f32) -> DataRect {
    let data = sanitize_data_rect(data);
    let frac = f64::from(overscroll_fraction.clamp(0.0, 1.0));
    let w = data.x_max - data.x_min;
    let h = data.y_max - data.y_min;
    if !w.is_finite() || !h.is_finite() {
        return data;
    }

    let dx = w * frac;
    let dy = h * frac;
    sanitize_data_rect(DataRect {
        x_min: data.x_min - dx,
        x_max: data.x_max + dx,
        y_min: data.y_min - dy,
        y_max: data.y_max + dy,
    })
}

pub fn expand_data_bounds_scaled(
    data: DataRect,
    overscroll_fraction: f32,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    let data = sanitize_data_rect_scaled(data, x_scale, y_scale);
    let frac = f64::from(overscroll_fraction.clamp(0.0, 1.0));

    let x0 = x_scale.to_axis(data.x_min);
    let x1 = x_scale.to_axis(data.x_max);
    let y0 = y_scale.to_axis(data.y_min);
    let y1 = y_scale.to_axis(data.y_max);

    let Some((x0, x1)) = x0.zip(x1) else {
        return data;
    };
    let Some((y0, y1)) = y0.zip(y1) else {
        return data;
    };

    let w = x1 - x0;
    let h = y1 - y0;
    if !w.is_finite() || !h.is_finite() {
        return data;
    }

    let dx = w * frac;
    let dy = h * frac;

    let x_min = x_scale.from_axis(x0 - dx).unwrap_or(data.x_min);
    let x_max = x_scale.from_axis(x1 + dx).unwrap_or(data.x_max);
    let y_min = y_scale.from_axis(y0 - dy).unwrap_or(data.y_min);
    let y_max = y_scale.from_axis(y1 + dy).unwrap_or(data.y_max);

    sanitize_data_rect_scaled(
        DataRect {
            x_min,
            x_max,
            y_min,
            y_max,
        },
        x_scale,
        y_scale,
    )
}

pub fn clamp_view_to_data(view: DataRect, data: DataRect, overscroll_fraction: f32) -> DataRect {
    let allowed = expand_data_bounds(data, overscroll_fraction);
    let allowed = sanitize_data_rect(allowed);
    let mut out = sanitize_data_rect(view);

    let allowed_w = allowed.x_max - allowed.x_min;
    let out_w = out.x_max - out.x_min;
    if out_w >= allowed_w {
        out.x_min = allowed.x_min;
        out.x_max = allowed.x_max;
    } else {
        if out.x_min < allowed.x_min {
            out.x_min = allowed.x_min;
            out.x_max = out.x_min + out_w;
        }
        if out.x_max > allowed.x_max {
            out.x_max = allowed.x_max;
            out.x_min = out.x_max - out_w;
        }
    }

    let allowed_h = allowed.y_max - allowed.y_min;
    let out_h = out.y_max - out.y_min;
    if out_h >= allowed_h {
        out.y_min = allowed.y_min;
        out.y_max = allowed.y_max;
    } else {
        if out.y_min < allowed.y_min {
            out.y_min = allowed.y_min;
            out.y_max = out.y_min + out_h;
        }
        if out.y_max > allowed.y_max {
            out.y_max = allowed.y_max;
            out.y_min = out.y_max - out_h;
        }
    }

    sanitize_data_rect(out)
}

pub fn clamp_view_to_data_scaled(
    view: DataRect,
    data: DataRect,
    overscroll_fraction: f32,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    let allowed = expand_data_bounds_scaled(data, overscroll_fraction, x_scale, y_scale);
    let allowed = sanitize_data_rect_scaled(allowed, x_scale, y_scale);
    let mut out = sanitize_data_rect_scaled(view, x_scale, y_scale);

    // Clamp in axis space for consistent behavior across scales.
    let Some((ax0, ax1)) = x_scale
        .to_axis(allowed.x_min)
        .zip(x_scale.to_axis(allowed.x_max))
    else {
        return out;
    };
    let Some((ox0, ox1)) = x_scale.to_axis(out.x_min).zip(x_scale.to_axis(out.x_max)) else {
        return out;
    };
    let allowed_w = ax1 - ax0;
    let out_w = ox1 - ox0;
    let mut nx0 = ox0;
    let mut nx1 = ox1;
    if out_w >= allowed_w {
        nx0 = ax0;
        nx1 = ax1;
    } else {
        if nx0 < ax0 {
            nx0 = ax0;
            nx1 = nx0 + out_w;
        }
        if nx1 > ax1 {
            nx1 = ax1;
            nx0 = nx1 - out_w;
        }
    }
    out.x_min = x_scale.from_axis(nx0).unwrap_or(out.x_min);
    out.x_max = x_scale.from_axis(nx1).unwrap_or(out.x_max);

    let Some((ay0, ay1)) = y_scale
        .to_axis(allowed.y_min)
        .zip(y_scale.to_axis(allowed.y_max))
    else {
        return out;
    };
    let Some((oy0, oy1)) = y_scale.to_axis(out.y_min).zip(y_scale.to_axis(out.y_max)) else {
        return out;
    };
    let allowed_h = ay1 - ay0;
    let out_h = oy1 - oy0;
    let mut ny0 = oy0;
    let mut ny1 = oy1;
    if out_h >= allowed_h {
        ny0 = ay0;
        ny1 = ay1;
    } else {
        if ny0 < ay0 {
            ny0 = ay0;
            ny1 = ny0 + out_h;
        }
        if ny1 > ay1 {
            ny1 = ay1;
            ny0 = ny1 - out_h;
        }
    }
    out.y_min = y_scale.from_axis(ny0).unwrap_or(out.y_min);
    out.y_max = y_scale.from_axis(ny1).unwrap_or(out.y_max);

    sanitize_data_rect_scaled(out, x_scale, y_scale)
}

pub fn pan_view_by_px(view: DataRect, viewport: Size, dx_px: f32, dy_px: f32) -> Option<DataRect> {
    let view = sanitize_data_rect(view);

    let viewport_w = f64::from(viewport.width.0);
    let viewport_h = f64::from(viewport.height.0);
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    let w = view.x_max - view.x_min;
    let h = view.y_max - view.y_min;
    if !w.is_finite() || !h.is_finite() || w == 0.0 || h == 0.0 {
        return None;
    }

    if !dx_px.is_finite() || !dy_px.is_finite() {
        return None;
    }

    let dx_data = (f64::from(dx_px) / viewport_w) * w;
    let dy_data = (f64::from(dy_px) / viewport_h) * h;

    Some(sanitize_data_rect(DataRect {
        x_min: view.x_min - dx_data,
        x_max: view.x_max - dx_data,
        y_min: view.y_min + dy_data,
        y_max: view.y_max + dy_data,
    }))
}

pub fn pan_view_by_px_scaled(
    view: DataRect,
    viewport: Size,
    dx_px: f32,
    dy_px: f32,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataRect> {
    let view = sanitize_data_rect_scaled(view, x_scale, y_scale);

    let viewport_w = f64::from(viewport.width.0);
    let viewport_h = f64::from(viewport.height.0);
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    if !dx_px.is_finite() || !dy_px.is_finite() {
        return None;
    }

    let (x0, x1) = x_scale
        .to_axis(view.x_min)
        .zip(x_scale.to_axis(view.x_max))?;
    let (y0, y1) = y_scale
        .to_axis(view.y_min)
        .zip(y_scale.to_axis(view.y_max))?;
    let w = x1 - x0;
    let h = y1 - y0;
    if !w.is_finite() || !h.is_finite() || w == 0.0 || h == 0.0 {
        return None;
    }

    let dx_axis = (f64::from(dx_px) / viewport_w) * w;
    let dy_axis = (f64::from(dy_px) / viewport_h) * h;

    let x_min = x_scale.from_axis(x0 - dx_axis)?;
    let x_max = x_scale.from_axis(x1 - dx_axis)?;
    let y_min = y_scale.from_axis(y0 + dy_axis)?;
    let y_max = y_scale.from_axis(y1 + dy_axis)?;

    Some(sanitize_data_rect_scaled(
        DataRect {
            x_min,
            x_max,
            y_min,
            y_max,
        },
        x_scale,
        y_scale,
    ))
}

pub fn zoom_view_at_px(
    view: DataRect,
    viewport: Size,
    local_px: Point,
    zoom_x: f32,
    zoom_y: f32,
) -> Option<DataRect> {
    let view = sanitize_data_rect(view);

    let viewport_w = f64::from(viewport.width.0);
    let viewport_h = f64::from(viewport.height.0);
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    let w = view.x_max - view.x_min;
    let h = view.y_max - view.y_min;
    if !w.is_finite() || !h.is_finite() || w == 0.0 || h == 0.0 {
        return None;
    }

    if !zoom_x.is_finite() || !zoom_y.is_finite() || zoom_x <= 0.0 || zoom_y <= 0.0 {
        return None;
    }

    let nx = f64::from(local_px.x.0) / viewport_w;
    let ny = f64::from(local_px.y.0) / viewport_h;
    if !nx.is_finite() || !ny.is_finite() {
        return None;
    }

    let anchor_x = view.x_min + nx * w;
    let anchor_y = view.y_min + (1.0 - ny) * h;
    if !anchor_x.is_finite() || !anchor_y.is_finite() {
        return None;
    }

    let new_w = w / f64::from(zoom_x);
    let new_h = h / f64::from(zoom_y);
    if !new_w.is_finite() || !new_h.is_finite() {
        return None;
    }

    let x_min = anchor_x - nx * new_w;
    let y_min = anchor_y - (1.0 - ny) * new_h;

    Some(sanitize_data_rect(DataRect {
        x_min,
        x_max: x_min + new_w,
        y_min,
        y_max: y_min + new_h,
    }))
}

pub fn zoom_view_at_px_scaled(
    view: DataRect,
    viewport: Size,
    local_px: Point,
    zoom_x: f32,
    zoom_y: f32,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataRect> {
    let view = sanitize_data_rect_scaled(view, x_scale, y_scale);

    let viewport_w = f64::from(viewport.width.0);
    let viewport_h = f64::from(viewport.height.0);
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    if !zoom_x.is_finite() || !zoom_y.is_finite() || zoom_x <= 0.0 || zoom_y <= 0.0 {
        return None;
    }

    let nx = f64::from(local_px.x.0) / viewport_w;
    let ny = f64::from(local_px.y.0) / viewport_h;
    if !nx.is_finite() || !ny.is_finite() {
        return None;
    }

    let (x0, x1) = x_scale
        .to_axis(view.x_min)
        .zip(x_scale.to_axis(view.x_max))?;
    let (y0, y1) = y_scale
        .to_axis(view.y_min)
        .zip(y_scale.to_axis(view.y_max))?;
    let w = x1 - x0;
    let h = y1 - y0;
    if !w.is_finite() || !h.is_finite() || w == 0.0 || h == 0.0 {
        return None;
    }

    let anchor_x = x0 + nx * w;
    let anchor_y = y0 + (1.0 - ny) * h;
    if !anchor_x.is_finite() || !anchor_y.is_finite() {
        return None;
    }

    let new_w = w / f64::from(zoom_x);
    let new_h = h / f64::from(zoom_y);
    if !new_w.is_finite() || !new_h.is_finite() {
        return None;
    }

    let ax_min = anchor_x - nx * new_w;
    let ay_min = anchor_y - (1.0 - ny) * new_h;

    let x_min = x_scale.from_axis(ax_min)?;
    let x_max = x_scale.from_axis(ax_min + new_w)?;
    let y_min = y_scale.from_axis(ay_min)?;
    let y_max = y_scale.from_axis(ay_min + new_h)?;

    Some(sanitize_data_rect_scaled(
        DataRect {
            x_min,
            x_max,
            y_min,
            y_max,
        },
        x_scale,
        y_scale,
    ))
}

pub fn data_rect_from_plot_points(
    view: DataRect,
    viewport: Size,
    a: Point,
    b: Point,
) -> Option<DataRect> {
    let view = sanitize_data_rect(view);

    let viewport_w = f64::from(viewport.width.0);
    let viewport_h = f64::from(viewport.height.0);
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    let w = view.x_max - view.x_min;
    let h = view.y_max - view.y_min;
    if !w.is_finite() || !h.is_finite() || w == 0.0 || h == 0.0 {
        return None;
    }

    let x0 = f64::from(a.x.0)
        .min(f64::from(b.x.0))
        .clamp(0.0, viewport_w);
    let x1 = f64::from(a.x.0)
        .max(f64::from(b.x.0))
        .clamp(0.0, viewport_w);
    let y0 = f64::from(a.y.0)
        .min(f64::from(b.y.0))
        .clamp(0.0, viewport_h);
    let y1 = f64::from(a.y.0)
        .max(f64::from(b.y.0))
        .clamp(0.0, viewport_h);

    let nx0 = x0 / viewport_w;
    let nx1 = x1 / viewport_w;
    let ny0 = y0 / viewport_h;
    let ny1 = y1 / viewport_h;

    if !nx0.is_finite() || !nx1.is_finite() || !ny0.is_finite() || !ny1.is_finite() {
        return None;
    }

    let dx0 = view.x_min + nx0 * w;
    let dx1 = view.x_min + nx1 * w;
    let dy0 = view.y_min + (1.0 - ny0) * h;
    let dy1 = view.y_min + (1.0 - ny1) * h;

    if !dx0.is_finite() || !dx1.is_finite() || !dy0.is_finite() || !dy1.is_finite() {
        return None;
    }

    Some(sanitize_data_rect(DataRect {
        x_min: dx0.min(dx1),
        x_max: dx0.max(dx1),
        y_min: dy0.min(dy1),
        y_max: dy0.max(dy1),
    }))
}

pub fn data_rect_from_plot_points_scaled(
    view: DataRect,
    viewport: Size,
    a: Point,
    b: Point,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataRect> {
    let view = sanitize_data_rect_scaled(view, x_scale, y_scale);

    let viewport_w = f64::from(viewport.width.0);
    let viewport_h = f64::from(viewport.height.0);
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    let (x0, x1) = x_scale
        .to_axis(view.x_min)
        .zip(x_scale.to_axis(view.x_max))?;
    let (y0, y1) = y_scale
        .to_axis(view.y_min)
        .zip(y_scale.to_axis(view.y_max))?;
    let w = x1 - x0;
    let h = y1 - y0;
    if !w.is_finite() || !h.is_finite() || w == 0.0 || h == 0.0 {
        return None;
    }

    let px0 = f64::from(a.x.0)
        .min(f64::from(b.x.0))
        .clamp(0.0, viewport_w);
    let px1 = f64::from(a.x.0)
        .max(f64::from(b.x.0))
        .clamp(0.0, viewport_w);
    let py0 = f64::from(a.y.0)
        .min(f64::from(b.y.0))
        .clamp(0.0, viewport_h);
    let py1 = f64::from(a.y.0)
        .max(f64::from(b.y.0))
        .clamp(0.0, viewport_h);

    let nx0 = px0 / viewport_w;
    let nx1 = px1 / viewport_w;
    let ny0 = py0 / viewport_h;
    let ny1 = py1 / viewport_h;
    if !nx0.is_finite() || !nx1.is_finite() || !ny0.is_finite() || !ny1.is_finite() {
        return None;
    }

    let dx0 = x0 + nx0 * w;
    let dx1 = x0 + nx1 * w;
    let dy0 = y0 + (1.0 - ny0) * h;
    let dy1 = y0 + (1.0 - ny1) * h;
    if !dx0.is_finite() || !dx1.is_finite() || !dy0.is_finite() || !dy1.is_finite() {
        return None;
    }

    let x_min = x_scale.from_axis(dx0.min(dx1))?;
    let x_max = x_scale.from_axis(dx0.max(dx1))?;
    let y_min = y_scale.from_axis(dy0.min(dy1))?;
    let y_max = y_scale.from_axis(dy0.max(dy1))?;

    Some(sanitize_data_rect_scaled(
        DataRect {
            x_min,
            x_max,
            y_min,
            y_max,
        },
        x_scale,
        y_scale,
    ))
}

pub fn clamp_zoom_factors(zoom: f32) -> f32 {
    zoom.clamp(0.05, 20.0)
}

pub fn local_from_absolute(plot_origin: Point, position: Point) -> Point {
    Point::new(
        Px(position.x.0 - plot_origin.x.0),
        Px(position.y.0 - plot_origin.y.0),
    )
}
