use crate::embla::scroll_target::{ScrollTarget, Target};
use crate::embla::utils::{DIRECTION_NONE, factor_abs, math_sign};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerKind {
    Mouse,
    Touch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragReleaseConfig {
    pub drag_free: bool,
    pub skip_snaps: bool,
    pub view_size: f32,
    pub base_friction: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragReleaseResult {
    pub duration: f32,
    pub friction: f32,
    pub raw_force: f32,
    pub force: f32,
    pub force_factor: f32,
    pub target: Target,
}

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    v.max(min).min(max)
}

fn force_boost(kind: PointerKind, drag_free: bool) -> f32 {
    match (drag_free, kind) {
        (true, PointerKind::Mouse) => 500.0,
        (true, PointerKind::Touch) => 600.0,
        (false, PointerKind::Mouse) => 300.0,
        (false, PointerKind::Touch) => 400.0,
    }
}

fn base_duration(drag_free: bool) -> f32 {
    if drag_free { 43.0 } else { 25.0 }
}

fn next_index(current: usize, direction_sign: f32, max: usize, loop_enabled: bool) -> usize {
    let delta = -(math_sign(direction_sign) as i32);
    if delta == 0 {
        return current;
    }
    if loop_enabled {
        let len = (max + 1).max(1) as i32;
        let mut next = current as i32 + delta;
        next %= len;
        if next < 0 {
            next += len;
        }
        next as usize
    } else if delta < 0 {
        current.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        (current + (delta as usize)).min(max)
    }
}

fn allowed_force(
    cfg: DragReleaseConfig,
    scroll_target: &ScrollTarget,
    index_current: usize,
    force: f32,
) -> f32 {
    let go_to_next_threshold = clamp(cfg.view_size * 0.2, 50.0, 225.0);

    let base_force =
        |force: f32| -> f32 { scroll_target.by_distance(force, !cfg.drag_free).distance };

    if cfg.drag_free || force.abs() < go_to_next_threshold {
        return base_force(force);
    }

    let current_location = scroll_target.by_distance(0.0, false);
    let index_changed = current_location.index != index_current;
    if cfg.skip_snaps && index_changed {
        return base_force(force) * 0.5;
    }

    let max = scroll_target.max_index();
    let next = next_index(index_current, force, max, scroll_target.loop_enabled());
    scroll_target.by_index(next, DIRECTION_NONE).distance
}

/// Compute Embla-style release shaping.
///
/// Upstream: `repo-ref/embla-carousel/packages/embla-carousel/src/components/DragHandler.ts` (`up`).
pub fn compute_release(
    cfg: DragReleaseConfig,
    pointer_kind: PointerKind,
    scroll_target: &ScrollTarget,
    index_current: usize,
    pointer_delta: f32,
    direction: impl Fn(f32) -> f32,
) -> DragReleaseResult {
    let raw_force = pointer_delta * force_boost(pointer_kind, cfg.drag_free);
    let directed_raw = direction(raw_force);
    let force = allowed_force(cfg, scroll_target, index_current, directed_raw);
    let target = scroll_target.by_distance(force, !cfg.drag_free);
    let force_factor = factor_abs(directed_raw, force);
    let duration = base_duration(cfg.drag_free) - 10.0 * force_factor;
    let friction = cfg.base_friction + force_factor / 50.0;

    DragReleaseResult {
        target,
        duration,
        friction,
        raw_force: directed_raw,
        force,
        force_factor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embla::scroll_limit::scroll_limit;

    #[test]
    fn compute_release_returns_duration_and_friction() {
        let snaps = vec![0.0, 100.0, 200.0, 300.0];
        let content_size = 300.0;
        let limit = scroll_limit(content_size, &snaps, false);
        let scroll_target = ScrollTarget::new(false, snaps, content_size, limit, 0.0);

        let cfg = DragReleaseConfig {
            drag_free: false,
            skip_snaps: false,
            view_size: 320.0,
            base_friction: 0.68,
        };
        let out = compute_release(cfg, PointerKind::Mouse, &scroll_target, 0, 0.2, |v| v);
        assert!(out.duration <= 25.0);
        assert!(out.friction >= 0.68);
    }

    #[test]
    fn skip_snaps_halves_base_force_when_index_changed() {
        // Arrange: simulate a drag where the target vector is already closest to a later snap while
        // the selected index is still the start snap (Embla `indexChanged()`).
        let snaps = vec![0.0, -100.0, -200.0, -300.0];
        let content_size = 300.0;
        let limit = scroll_limit(content_size, &snaps, false);
        let scroll_target = ScrollTarget::new(false, snaps, content_size, limit, -200.0);

        let base_cfg = DragReleaseConfig {
            drag_free: false,
            skip_snaps: false,
            view_size: 320.0,
            base_friction: 0.68,
        };

        // `pointer_delta` is Embla dragTracker force (px/ms). Use a value that yields a large
        // boosted force and triggers the `goToNextThreshold` branch.
        let pointer_delta = -1.0;

        let out_no_skip = compute_release(
            base_cfg,
            PointerKind::Mouse,
            &scroll_target,
            0,
            pointer_delta,
            |v| v,
        );
        let out_skip = compute_release(
            DragReleaseConfig {
                skip_snaps: true,
                ..base_cfg
            },
            PointerKind::Mouse,
            &scroll_target,
            0,
            pointer_delta,
            |v| v,
        );

        // Without skipSnaps, Embla clamps to the neighbor index distance (from the current
        // `index_current`), which in this setup moves back toward snap 1.
        assert!(
            (out_no_skip.force - 100.0).abs() < 0.001,
            "out={out_no_skip:?}"
        );

        // With skipSnaps and `indexChanged()`, Embla uses `baseForce(force) * 0.5`.
        // Here, baseForce resolves to the raw boosted force because we are past bounds, so the
        // result is exactly half.
        assert!((out_skip.force - -150.0).abs() < 0.001, "out={out_skip:?}");
    }
}
