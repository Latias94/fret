use fret_core::Px;

/// Returns the index of the closest point (in absolute `Px` distance) to `target`.
///
/// - Returns `None` when `points` is empty, `target` is non-finite, or all points are non-finite.
/// - Tie-break: the first point wins (stable/deterministic).
pub fn closest_index_px(points: &[Px], target: Px) -> Option<usize> {
    if points.is_empty() || !target.0.is_finite() {
        return None;
    }

    let mut best = None::<(usize, f32)>;
    for (index, point) in points.iter().copied().enumerate() {
        if !point.0.is_finite() {
            continue;
        }
        let distance = (point.0 - target.0).abs();
        if !distance.is_finite() {
            continue;
        }
        match best {
            None => best = Some((index, distance)),
            Some((_, best_distance)) if distance < best_distance => best = Some((index, distance)),
            _ => {}
        }
    }

    best.map(|(index, _)| index)
}

/// Returns the closest point to `target` (see `closest_index_px`).
pub fn closest_value_px(points: &[Px], target: Px) -> Option<Px> {
    let ix = closest_index_px(points, target)?;
    points.get(ix).copied()
}

/// Steps an index by `delta` and clamps to `[0, len - 1]`.
///
/// - Returns `None` when `len == 0`.
/// - If `current` is out of range, it is clamped first.
pub fn step_index_clamped(len: usize, current: usize, delta: i32) -> Option<usize> {
    if len == 0 {
        return None;
    }

    let current = current.min(len.saturating_sub(1)) as i64;
    let len_i = len as i64;
    let delta = delta as i64;
    let next = (current + delta).clamp(0, len_i.saturating_sub(1));
    Some(next as usize)
}

/// Steps an index by `delta` and wraps within `[0, len - 1]`.
///
/// - Returns `None` when `len == 0`.
/// - If `current` is out of range, it is wrapped first.
pub fn step_index_wrapped(len: usize, current: usize, delta: i32) -> Option<usize> {
    if len == 0 {
        return None;
    }

    let len_i = len as i64;
    let current = (current as i64).rem_euclid(len_i);
    let delta = delta as i64;
    let next = (current + delta).rem_euclid(len_i);
    Some(next as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_index_px_returns_none_for_empty() {
        assert_eq!(closest_index_px(&[], Px(0.0)), None);
    }

    #[test]
    fn closest_index_px_picks_nearest_point() {
        let points = [Px(0.0), Px(10.0), Px(30.0)];
        assert_eq!(closest_index_px(&points, Px(12.0)), Some(1));
        assert_eq!(closest_value_px(&points, Px(12.0)), Some(Px(10.0)));
    }

    #[test]
    fn closest_index_px_is_stable_on_ties() {
        let points = [Px(0.0), Px(10.0)];
        // Equidistant from 0 and 10.
        assert_eq!(closest_index_px(&points, Px(5.0)), Some(0));
    }

    #[test]
    fn closest_index_px_ignores_non_finite_inputs() {
        let points = [Px(f32::NAN), Px(10.0)];
        assert_eq!(closest_index_px(&points, Px(9.0)), Some(1));
        assert_eq!(closest_index_px(&points, Px(f32::NAN)), None);
    }

    #[test]
    fn step_index_clamped_returns_none_for_empty() {
        assert_eq!(step_index_clamped(0, 0, 1), None);
    }

    #[test]
    fn step_index_clamped_saturates_at_bounds() {
        assert_eq!(step_index_clamped(3, 0, -1), Some(0));
        assert_eq!(step_index_clamped(3, 2, 1), Some(2));
        assert_eq!(step_index_clamped(3, 1, -1), Some(0));
        assert_eq!(step_index_clamped(3, 1, 1), Some(2));
    }

    #[test]
    fn step_index_wrapped_wraps_at_bounds() {
        assert_eq!(step_index_wrapped(3, 0, -1), Some(2));
        assert_eq!(step_index_wrapped(3, 2, 1), Some(0));
        assert_eq!(step_index_wrapped(3, 1, 2), Some(0));
    }

    #[test]
    fn step_index_wrapped_returns_none_for_empty() {
        assert_eq!(step_index_wrapped(0, 0, 1), None);
    }
}
