use crate::embla::limit::Limit;
use crate::embla::utils::{DIRECTION_NONE, Direction, math_abs, math_sign};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    pub distance: f32,
    pub index: usize,
}

/// Ported from Embla `ScrollTarget`.
///
/// Upstream: `repo-ref/embla-carousel/packages/embla-carousel/src/components/ScrollTarget.ts`
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollTarget {
    loop_enabled: bool,
    scroll_snaps: Vec<f32>,
    content_size: f32,
    limit: Limit,
    target_vector: f32,
}

impl ScrollTarget {
    pub fn new(
        loop_enabled: bool,
        scroll_snaps: Vec<f32>,
        content_size: f32,
        limit: Limit,
        target_vector: f32,
    ) -> Self {
        Self {
            loop_enabled,
            scroll_snaps,
            content_size,
            limit,
            target_vector,
        }
    }

    pub fn set_target_vector(&mut self, target_vector: f32) {
        self.target_vector = target_vector;
    }

    pub fn target_vector(&self) -> f32 {
        self.target_vector
    }

    pub fn loop_enabled(&self) -> bool {
        self.loop_enabled
    }

    pub fn max_index(&self) -> usize {
        self.scroll_snaps.len().saturating_sub(1)
    }

    fn min_distance(distances: &mut [f32]) -> f32 {
        distances.sort_by(|a, b| math_abs(*a).total_cmp(&math_abs(*b)));
        distances[0]
    }

    pub fn shortcut(&self, target: f32, direction: Direction) -> f32 {
        if !self.loop_enabled {
            return target;
        }

        let targets = [
            target,
            target + self.content_size,
            target - self.content_size,
        ];
        if direction == DIRECTION_NONE {
            let mut tmp = targets;
            return Self::min_distance(&mut tmp);
        }

        let mut valid = targets
            .into_iter()
            .filter(|t| math_sign(*t) == direction as f32)
            .collect::<Vec<_>>();
        if !valid.is_empty() {
            return Self::min_distance(&mut valid);
        }
        targets[2] - self.content_size
    }

    fn get_closest_snap(&self, target: f32) -> Target {
        let distance = if self.loop_enabled {
            self.limit.remove_offset(target)
        } else {
            self.limit.clamp(target)
        };

        let mut smallest = f32::INFINITY;
        let mut index = 0usize;
        for (snap_index, snap) in self.scroll_snaps.iter().copied().enumerate() {
            let displacement_abs = math_abs(self.shortcut(snap - distance, DIRECTION_NONE));
            if displacement_abs >= smallest {
                continue;
            }
            smallest = displacement_abs;
            index = snap_index;
        }

        Target { index, distance }
    }

    pub fn by_index(&self, index: usize, direction: Direction) -> Target {
        let index = index.min(self.scroll_snaps.len().saturating_sub(1));
        let diff_to_snap = self.scroll_snaps[index] - self.target_vector;
        let distance = self.shortcut(diff_to_snap, direction);
        Target { index, distance }
    }

    pub fn by_distance(&self, distance: f32, snap_to_closest: bool) -> Target {
        let target = self.target_vector + distance;
        let Target {
            index,
            distance: target_snap_distance,
        } = self.get_closest_snap(target);
        let is_past_any_bound = !self.loop_enabled && self.limit.past_any_bound(target);

        if !snap_to_closest || is_past_any_bound {
            return Target { index, distance };
        }

        let diff_to_snap = self.scroll_snaps[index] - target_snap_distance;
        let snap_distance = distance + self.shortcut(diff_to_snap, DIRECTION_NONE);
        Target {
            index,
            distance: snap_distance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embla::scroll_limit::scroll_limit;

    #[test]
    fn by_distance_snaps_to_closest_when_enabled() {
        // Embla scroll snaps are measured as non-increasing offsets (0, -x, -2x, ...).
        let snaps = vec![0.0, -100.0, -200.0, -300.0];
        let content_size = 300.0;
        let limit = scroll_limit(content_size, &snaps, false);
        let target = ScrollTarget::new(false, snaps, content_size, limit, 0.0);

        let out = target.by_distance(-130.0, true);
        assert_eq!(out.index, 1);
        assert!((out.distance - -100.0).abs() < 0.001);
    }

    #[test]
    fn by_distance_does_not_snap_when_flag_is_false() {
        let snaps = vec![0.0, -100.0, -200.0, -300.0];
        let content_size = 300.0;
        let limit = scroll_limit(content_size, &snaps, false);
        let target = ScrollTarget::new(false, snaps, content_size, limit, 0.0);

        let out = target.by_distance(-130.0, false);
        assert_eq!(out.index, 1);
        assert!((out.distance - -130.0).abs() < 0.001);
    }
}
