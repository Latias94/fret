use fret_core::geometry::{Point, Px, Size};

use crate::cartesian::DataRect;

fn mix_u64(mut state: u64, v: u64) -> u64 {
    state ^= v
        .wrapping_add(0x9e3779b97f4a7c15)
        .wrapping_add(state << 6)
        .wrapping_add(state >> 2);
    state
}

fn mix_f32_bits(state: u64, v: f32) -> u64 {
    mix_u64(state, u64::from(v.to_bits()))
}

pub fn data_rect_key(bounds: DataRect) -> u64 {
    let mut key = 0u64;
    key = mix_f32_bits(key, bounds.x_min);
    key = mix_f32_bits(key, bounds.x_max);
    key = mix_f32_bits(key, bounds.y_min);
    key = mix_f32_bits(key, bounds.y_max);
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

    let min_span = 1.0e-6_f32;

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

pub fn pan_view_by_px(view: DataRect, viewport: Size, dx_px: f32, dy_px: f32) -> Option<DataRect> {
    let view = sanitize_data_rect(view);

    let viewport_w = viewport.width.0;
    let viewport_h = viewport.height.0;
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

    let dx_data = (dx_px / viewport_w) * w;
    let dy_data = (dy_px / viewport_h) * h;

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

    let viewport_w = viewport.width.0;
    let viewport_h = viewport.height.0;
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

    let nx = local_px.x.0 / viewport_w;
    let ny = local_px.y.0 / viewport_h;
    if !nx.is_finite() || !ny.is_finite() {
        return None;
    }

    let anchor_x = view.x_min + nx * w;
    let anchor_y = view.y_min + (1.0 - ny) * h;
    if !anchor_x.is_finite() || !anchor_y.is_finite() {
        return None;
    }

    let new_w = w / zoom_x;
    let new_h = h / zoom_y;
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

pub fn clamp_zoom_factors(zoom: f32) -> f32 {
    zoom.clamp(0.05, 20.0)
}

pub fn local_from_absolute(plot_origin: Point, position: Point) -> Point {
    Point::new(
        Px(position.x.0 - plot_origin.x.0),
        Px(position.y.0 - plot_origin.y.0),
    )
}
