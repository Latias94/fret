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
}
