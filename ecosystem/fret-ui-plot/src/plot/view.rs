use fret_core::geometry::{Point, Px, Size};

use crate::cartesian::DataRect;

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

pub fn clamp_zoom_factors(zoom: f32) -> f32 {
    zoom.clamp(0.05, 20.0)
}

pub fn local_from_absolute(plot_origin: Point, position: Point) -> Point {
    Point::new(
        Px(position.x.0 - plot_origin.x.0),
        Px(position.y.0 - plot_origin.y.0),
    )
}
