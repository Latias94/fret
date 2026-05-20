use fret_core::{
    Axis, Color, Corners, DrawOrder, Edges, Paint, Point, Px, Rect, Scene, SceneOp, Size,
};

#[derive(Debug, Clone)]
pub(super) struct SplitGeometryLayout {
    pub(super) panel_rects: Vec<Rect>,
    pub(super) handle_hit_rects: Vec<Rect>,
    pub(super) handle_centers: Vec<f32>,
    pub(super) sizes: Vec<f32>,
    mins: Vec<f32>,
    avail: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SplitHandle {
    pub(super) axis: Axis,
    pub(super) paint_device_px: f32,
}

impl SplitHandle {
    pub(super) fn paint(
        self,
        scene: &mut Scene,
        order: DrawOrder,
        bounds: Rect,
        center: f32,
        scale_factor: f32,
        color: Color,
    ) {
        let rect = self.paint_rect(bounds, center, scale_factor);
        scene.push(SceneOp::Quad {
            order,
            rect,
            background: Paint::Solid(color).into(),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT).into(),
            corner_radii: Corners::all(Px(0.0)),
        });
    }

    fn paint_rect(self, bounds: Rect, center: f32, scale_factor: f32) -> Rect {
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        let thickness = Px(self.paint_device_px / scale_factor);

        match self.axis {
            Axis::Horizontal => Rect {
                origin: Point::new(Px(center - thickness.0 * 0.5), bounds.origin.y),
                size: Size::new(thickness, bounds.size.height),
            },
            Axis::Vertical => Rect {
                origin: Point::new(bounds.origin.x, Px(center - thickness.0 * 0.5)),
                size: Size::new(bounds.size.width, thickness),
            },
        }
    }
}

pub(super) fn compute_layout(
    axis: Axis,
    bounds: Rect,
    children_len: usize,
    fractions: &[f32],
    gap: Px,
    hit_thickness: Px,
    min_px: &[Px],
) -> SplitGeometryLayout {
    compute_split_geometry_layout(
        axis,
        bounds,
        children_len,
        fractions.to_vec(),
        gap,
        hit_thickness,
        min_px,
    )
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(super) fn drag_update_fractions(
    axis: Axis,
    bounds: Rect,
    children_len: usize,
    fractions: &[f32],
    handle_ix: usize,
    gap: Px,
    hit_thickness: Px,
    min_px: &[Px],
    grab_offset: f32,
    position: Point,
) -> Option<Vec<f32>> {
    if children_len < 2 || handle_ix + 1 >= children_len {
        return None;
    }

    let layout = compute_layout(
        axis,
        bounds,
        children_len,
        fractions,
        gap,
        hit_thickness,
        min_px,
    );
    let old_center = *layout.handle_centers.get(handle_ix)?;

    let desired_center = axis_pos(position, axis) - grab_offset;
    let desired_delta = desired_center - old_center;
    if !desired_delta.is_finite() {
        return None;
    }

    let mut sizes = layout.sizes.clone();
    let actual = apply_handle_delta(handle_ix, desired_delta, &mut sizes, &layout.mins);
    if actual.abs() <= 1.0e-6 {
        return None;
    }
    Some(fractions_from_sizes(&sizes, layout.avail))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn drag_update_adjacent_fractions(
    axis: Axis,
    bounds: Rect,
    children_len: usize,
    fractions: &[f32],
    handle_ix: usize,
    gap: Px,
    hit_thickness: Px,
    min_px: &[Px],
    grab_offset: f32,
    position: Point,
) -> Option<Vec<f32>> {
    if children_len < 2 || handle_ix + 1 >= children_len {
        return None;
    }

    let layout = compute_layout(
        axis,
        bounds,
        children_len,
        fractions,
        gap,
        hit_thickness,
        min_px,
    );
    let old_center = *layout.handle_centers.get(handle_ix)?;

    let desired_center = axis_pos(position, axis) - grab_offset;
    let desired_delta = desired_center - old_center;
    if !desired_delta.is_finite() {
        return None;
    }

    let i = handle_ix;
    let j = handle_ix + 1;
    if layout.sizes.len() != children_len || layout.mins.len() != children_len {
        return None;
    }

    let pair_sum = layout.sizes[i] + layout.sizes[j];
    if !pair_sum.is_finite() || pair_sum <= 0.0 {
        return None;
    }

    let min_i = layout.mins[i].max(0.0);
    let min_j = layout.mins[j].max(0.0);
    let max_i = (pair_sum - min_j).clamp(0.0, pair_sum);
    let min_i = min_i.clamp(0.0, max_i);

    let mut next_i = (layout.sizes[i] + desired_delta).clamp(min_i, max_i);
    if !next_i.is_finite() {
        return None;
    }
    let mut next_j = (pair_sum - next_i).max(0.0);
    if next_j < min_j {
        next_j = min_j.clamp(0.0, pair_sum);
        next_i = (pair_sum - next_j).clamp(min_i, max_i);
    }

    let actual = next_i - layout.sizes[i];
    if actual.abs() <= 1.0e-6 {
        return None;
    }

    let mut sizes = layout.sizes.clone();
    sizes[i] = next_i;
    sizes[j] = next_j;
    Some(fractions_from_sizes(&sizes, layout.avail))
}

fn compute_split_geometry_layout(
    axis: Axis,
    bounds: Rect,
    children_len: usize,
    fractions: Vec<f32>,
    gap: Px,
    hit_thickness: Px,
    min_px: &[Px],
) -> SplitGeometryLayout {
    let gap = gap.0.max(0.0);
    let hit = hit_thickness.0.max(0.0).max(gap);

    let axis_len = axis_len(bounds, axis).max(0.0);
    let total_gap = gap * (children_len.saturating_sub(1) as f32);
    let avail = (axis_len - total_gap).max(0.0);

    let mins = effective_min_px(children_len, avail, min_px);
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

    SplitGeometryLayout {
        panel_rects,
        handle_hit_rects,
        handle_centers,
        sizes,
        mins,
        avail,
    }
}

fn handle_hit_rect(axis: Axis, bounds: Rect, center: f32, thickness: f32) -> Rect {
    if thickness <= 0.0 || !thickness.is_finite() {
        return Rect::default();
    }

    let axis_origin = axis_origin(bounds, axis);
    let axis_len = axis_len(bounds, axis).max(0.0);
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

fn axis_len(bounds: Rect, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => bounds.size.width.0,
        Axis::Vertical => bounds.size.height.0,
    }
}

fn axis_origin(bounds: Rect, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => bounds.origin.x.0,
        Axis::Vertical => bounds.origin.y.0,
    }
}

fn axis_pos(pos: Point, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => pos.x.0,
        Axis::Vertical => pos.y.0,
    }
}

fn effective_min_px(count: usize, avail: f32, min_px: &[Px]) -> Vec<f32> {
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

fn sanitize_fractions(mut v: Vec<f32>, count: usize) -> Vec<f32> {
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

fn fractions_from_sizes(sizes: &[f32], avail: f32) -> Vec<f32> {
    if avail <= 0.0 {
        return Vec::new();
    }
    let mut next: Vec<f32> = sizes.iter().map(|s| (*s / avail).clamp(0.0, 1.0)).collect();
    next = sanitize_fractions(next, sizes.len());
    next
}

#[cfg(test)]
fn apply_handle_delta(handle_ix: usize, mut delta: f32, sizes: &mut [f32], mins: &[f32]) -> f32 {
    if sizes.len() < 2 || handle_ix + 1 >= sizes.len() {
        return 0.0;
    }
    if mins.len() != sizes.len() {
        return 0.0;
    }

    if delta > 0.0 {
        let mut reducible = 0.0;
        for k in (handle_ix + 1)..sizes.len() {
            reducible += (sizes[k] - mins[k]).max(0.0);
        }
        if reducible <= 1.0e-6 {
            return 0.0;
        }
        delta = delta.min(reducible);
        sizes[handle_ix] += delta;

        let mut remaining = delta;
        for k in (handle_ix + 1)..sizes.len() {
            if remaining <= 1.0e-6 {
                break;
            }
            let available = (sizes[k] - mins[k]).max(0.0);
            let take = remaining.min(available);
            sizes[k] -= take;
            remaining -= take;
        }
        delta - remaining
    } else if delta < 0.0 {
        let shrinkable = (sizes[handle_ix] - mins[handle_ix]).max(0.0);
        if shrinkable <= 1.0e-6 {
            return 0.0;
        }
        delta = delta.max(-shrinkable);
        sizes[handle_ix] += delta;
        sizes[handle_ix + 1] -= delta;
        delta
    } else {
        0.0
    }
}
