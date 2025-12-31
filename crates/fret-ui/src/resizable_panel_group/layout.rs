use fret_core::{Axis, Point, Px, Rect, Size};

#[derive(Debug, Clone)]
pub struct ResizablePanelGroupLayout {
    pub panel_rects: Vec<Rect>,
    pub handle_hit_rects: Vec<Rect>,
    pub handle_centers: Vec<f32>,
    pub sizes: Vec<f32>,
    pub mins: Vec<f32>,
    pub avail: f32,
}

fn handle_hit_rect(axis: Axis, bounds: Rect, center: f32, thickness: f32) -> Rect {
    if thickness <= 0.0 || !thickness.is_finite() {
        return Rect::default();
    }

    let axis_origin = match axis {
        Axis::Horizontal => bounds.origin.x.0,
        Axis::Vertical => bounds.origin.y.0,
    };
    let axis_len = match axis {
        Axis::Horizontal => bounds.size.width.0,
        Axis::Vertical => bounds.size.height.0,
    }
    .max(0.0);

    let t = thickness.min(axis_len);
    let max_origin = (axis_origin + axis_len - t).max(axis_origin);
    let origin_axis = (center - t * 0.5).clamp(axis_origin, max_origin);

    match axis {
        Axis::Horizontal => Rect::new(
            Point::new(Px(origin_axis), bounds.origin.y),
            Size::new(Px(t), bounds.size.height),
        ),
        Axis::Vertical => Rect::new(
            Point::new(bounds.origin.x, Px(origin_axis)),
            Size::new(bounds.size.width, Px(t)),
        ),
    }
}

pub(super) fn axis_len(bounds: Rect, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => bounds.size.width.0,
        Axis::Vertical => bounds.size.height.0,
    }
}

pub(super) fn axis_origin(bounds: Rect, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => bounds.origin.x.0,
        Axis::Vertical => bounds.origin.y.0,
    }
}

pub(super) fn axis_pos(pos: Point, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => pos.x.0,
        Axis::Vertical => pos.y.0,
    }
}

pub(super) fn effective_min_px_static(count: usize, avail: f32, min_px: &[Px]) -> Vec<f32> {
    let default = Px(120.0);
    if count == 0 {
        return Vec::new();
    }

    let mut mins: Vec<f32> = if min_px.is_empty() {
        vec![default.0; count]
    } else if min_px.len() == 1 {
        vec![min_px[0].0.max(0.0); count]
    } else if min_px.len() == count {
        min_px.iter().map(|p| p.0.max(0.0)).collect()
    } else {
        vec![min_px[0].0.max(0.0); count]
    };

    let sum: f32 = mins.iter().copied().sum();
    if !sum.is_finite() || sum <= 0.0 {
        return mins;
    }
    if avail > 0.0 && avail < sum {
        let scale = (avail / sum).clamp(0.0, 1.0);
        for m in &mut mins {
            *m = (*m * scale).max(0.0);
        }
    }
    mins
}

pub(super) fn sanitize_fractions(mut v: Vec<f32>, count: usize) -> Vec<f32> {
    if count == 0 {
        return Vec::new();
    }
    if v.len() != count {
        return vec![1.0 / count as f32; count];
    }
    for x in &mut v {
        if !x.is_finite() {
            *x = 0.0;
        }
        *x = (*x).max(0.0);
    }
    let sum: f32 = v.iter().sum();
    if !sum.is_finite() || sum <= f32::EPSILON {
        return vec![1.0 / count as f32; count];
    }
    for x in &mut v {
        *x /= sum;
    }
    v
}

fn sizes_from_fractions(fractions: &[f32], avail: f32) -> Vec<f32> {
    let mut sizes: Vec<f32> = fractions
        .iter()
        .copied()
        .map(|f| (f.clamp(0.0, 1.0) * avail).max(0.0))
        .collect();
    let sum: f32 = sizes.iter().sum();
    let diff = avail - sum;
    if sizes.is_empty() {
        return sizes;
    }
    let last = sizes.len() - 1;
    sizes[last] = (sizes[last] + diff).max(0.0);
    sizes
}

fn apply_min_constraints(mut sizes: Vec<f32>, mins: &[f32], avail: f32) -> Vec<f32> {
    if sizes.is_empty() {
        return sizes;
    }
    if mins.len() != sizes.len() {
        return sizes;
    }

    let sum_min: f32 = mins.iter().copied().sum();
    if avail <= 0.0 {
        return vec![0.0; sizes.len()];
    }
    if sum_min.is_finite() && sum_min > 0.0 && avail < sum_min {
        let scale = (avail / sum_min).clamp(0.0, 1.0);
        for (s, m) in sizes.iter_mut().zip(mins.iter().copied()) {
            *s = (m * scale).max(0.0);
        }
        return sizes;
    }

    for (s, m) in sizes.iter_mut().zip(mins.iter().copied()) {
        if *s < m {
            *s = m;
        }
    }

    let mut sum: f32 = sizes.iter().sum();
    if sum <= avail + 1.0e-3 {
        let last = sizes.len() - 1;
        sizes[last] = (sizes[last] + (avail - sum)).max(mins[last]);
        return sizes;
    }

    let mut excess = sum - avail;
    for _ in 0..4 {
        if excess <= 1.0e-3 {
            break;
        }
        let mut adjustable_total = 0.0;
        for (s, m) in sizes.iter().zip(mins.iter().copied()) {
            adjustable_total += (*s - m).max(0.0);
        }
        if adjustable_total <= 1.0e-6 {
            break;
        }
        for (s, m) in sizes.iter_mut().zip(mins.iter().copied()) {
            let room = (*s - m).max(0.0);
            if room <= 0.0 {
                continue;
            }
            let take = (excess * (room / adjustable_total)).min(room);
            *s -= take;
            excess -= take;
            if excess <= 1.0e-3 {
                break;
            }
        }
    }

    sum = sizes.iter().sum();
    let last = sizes.len() - 1;
    sizes[last] = (sizes[last] + (avail - sum)).max(mins[last]);
    sizes
}

pub(crate) fn compute_resizable_panel_group_layout(
    axis: Axis,
    bounds: Rect,
    children_len: usize,
    fractions: Vec<f32>,
    gap: Px,
    hit_thickness: Px,
    min_px: &[Px],
) -> ResizablePanelGroupLayout {
    let gap = gap.0.max(0.0);
    let hit = hit_thickness.0.max(0.0).max(gap);

    let axis_len = axis_len(bounds, axis).max(0.0);
    let total_gap = gap * (children_len.saturating_sub(1) as f32);
    let avail = (axis_len - total_gap).max(0.0);

    let mins = effective_min_px_static(children_len, avail, min_px);
    let fractions = sanitize_fractions(fractions, children_len);
    let sizes = apply_min_constraints(sizes_from_fractions(&fractions, avail), &mins, avail);

    let mut panel_rects = Vec::with_capacity(children_len);
    let mut handle_hit_rects = Vec::with_capacity(children_len.saturating_sub(1));
    let mut handle_centers = Vec::with_capacity(children_len.saturating_sub(1));

    let mut cursor = axis_origin(bounds, axis);
    for i in 0..children_len {
        let len = sizes.get(i).copied().unwrap_or(0.0).max(0.0);
        match axis {
            Axis::Horizontal => {
                panel_rects.push(Rect::new(
                    Point::new(Px(cursor), bounds.origin.y),
                    Size::new(Px(len), bounds.size.height),
                ));
            }
            Axis::Vertical => {
                panel_rects.push(Rect::new(
                    Point::new(bounds.origin.x, Px(cursor)),
                    Size::new(bounds.size.width, Px(len)),
                ));
            }
        }
        cursor += len;

        if i + 1 < children_len {
            let center = cursor + gap * 0.5;
            handle_centers.push(center);
            handle_hit_rects.push(handle_hit_rect(axis, bounds, center, hit));
            cursor += gap;
        }
    }

    ResizablePanelGroupLayout {
        panel_rects,
        handle_hit_rects,
        handle_centers,
        sizes,
        mins,
        avail,
    }
}
