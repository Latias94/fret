use super::layout::sanitize_fractions;

pub(crate) fn fractions_from_sizes(sizes: &[f32], avail: f32) -> Vec<f32> {
    if avail <= 0.0 {
        return Vec::new();
    }
    let mut next: Vec<f32> = sizes.iter().map(|s| (*s / avail).clamp(0.0, 1.0)).collect();
    next = sanitize_fractions(next, sizes.len());
    next
}

pub(crate) fn apply_handle_delta(
    handle_ix: usize,
    mut delta: f32,
    sizes: &mut [f32],
    mins: &[f32],
) -> f32 {
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
