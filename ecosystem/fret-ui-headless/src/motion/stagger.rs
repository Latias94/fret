use std::time::Duration;

/// Determines which item in a group should animate first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaggerFrom {
    First,
    Last,
}

fn order_index(index: usize, count: usize, from: StaggerFrom) -> usize {
    if count <= 1 {
        return 0;
    }
    let index = index.min(count - 1);
    match from {
        StaggerFrom::First => index,
        StaggerFrom::Last => (count - 1).saturating_sub(index),
    }
}

/// Maps a single shared progress value (`0..1`) into a per-item progress value with a staggered
/// start delay.
///
/// This is intentionally a pure, deterministic helper (no runtime state). A common pattern is to
/// drive **one** transition timeline and then compute item-local progress for a list/stack:
///
/// - toast stack shift / enter / exit
/// - list insert/remove choreography
/// - menu item cascade
///
/// `each_delay` is a normalized fraction of the overall timeline (`0..1`). For example, for a
/// 240ms overall duration with a 24ms stagger per item, pass `each_delay = 0.1`.
pub fn staggered_normalized_progress(
    global_progress: f32,
    index: usize,
    count: usize,
    each_delay: f32,
    from: StaggerFrom,
) -> f32 {
    let global_progress = global_progress.clamp(0.0, 1.0);
    let count = count.max(1);
    let each_delay = each_delay.max(0.0);

    let order = order_index(index, count, from);
    let max_delay = each_delay * (count.saturating_sub(1) as f32);
    let span = (1.0 - max_delay).max(1e-6);
    let delay = each_delay * (order as f32);
    ((global_progress - delay) / span).clamp(0.0, 1.0)
}

/// Duration-based convenience wrapper over [`staggered_normalized_progress`].
pub fn staggered_progress_for_duration(
    global_progress: f32,
    index: usize,
    count: usize,
    each_delay: Duration,
    total_duration: Duration,
    from: StaggerFrom,
) -> f32 {
    if total_duration == Duration::ZERO {
        return global_progress.clamp(0.0, 1.0);
    }

    let each_delay_frac =
        (each_delay.as_secs_f64() / total_duration.as_secs_f64()).clamp(0.0, 1.0) as f32;
    staggered_normalized_progress(global_progress, index, count, each_delay_frac, from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staggered_progress_is_clamped_and_settles() {
        for from in [StaggerFrom::First, StaggerFrom::Last] {
            assert_eq!(staggered_normalized_progress(-1.0, 0, 4, 0.1, from), 0.0);
            assert_eq!(staggered_normalized_progress(2.0, 0, 4, 0.1, from), 1.0);

            assert_eq!(staggered_normalized_progress(0.0, 0, 4, 0.1, from), 0.0);
            assert_eq!(staggered_normalized_progress(1.0, 3, 4, 0.1, from), 1.0);
        }
    }

    #[test]
    fn staggered_progress_orders_items() {
        let t = 0.45;
        let each = 0.1;

        let a0 = staggered_normalized_progress(t, 0, 3, each, StaggerFrom::First);
        let a1 = staggered_normalized_progress(t, 1, 3, each, StaggerFrom::First);
        let a2 = staggered_normalized_progress(t, 2, 3, each, StaggerFrom::First);
        assert!(a0 >= a1 && a1 >= a2);

        let b0 = staggered_normalized_progress(t, 0, 3, each, StaggerFrom::Last);
        let b1 = staggered_normalized_progress(t, 1, 3, each, StaggerFrom::Last);
        let b2 = staggered_normalized_progress(t, 2, 3, each, StaggerFrom::Last);
        assert!(b2 >= b1 && b1 >= b0);
    }

    #[test]
    fn duration_wrapper_matches_normalized_math() {
        let total = Duration::from_millis(240);
        let each = Duration::from_millis(24);
        let t = 0.6;

        let a = staggered_progress_for_duration(t, 2, 5, each, total, StaggerFrom::First);
        let b = staggered_normalized_progress(t, 2, 5, 0.1, StaggerFrom::First);
        assert!((a - b).abs() < 1e-6);
    }
}
