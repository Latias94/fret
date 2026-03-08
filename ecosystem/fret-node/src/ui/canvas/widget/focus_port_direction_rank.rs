use super::PortNavDir;
use crate::core::{CanvasPoint, PortId};

#[derive(Clone, Copy)]
pub(super) struct DirectionalPortRank {
    pub(super) angle: f32,
    pub(super) parallel: f32,
    pub(super) dist2: f32,
    pub(super) port: PortId,
}

pub(super) fn rank_directional_port_candidate(
    port: PortId,
    from_center: CanvasPoint,
    candidate_center: CanvasPoint,
    dir: PortNavDir,
) -> Option<DirectionalPortRank> {
    let dx = candidate_center.x - from_center.x;
    let dy = candidate_center.y - from_center.y;
    let (parallel, perp) = match dir {
        PortNavDir::Left => (-dx, dy.abs()),
        PortNavDir::Right => (dx, dy.abs()),
        PortNavDir::Up => (-dy, dx.abs()),
        PortNavDir::Down => (dy, dx.abs()),
    };
    if !parallel.is_finite() || !perp.is_finite() || parallel <= 1.0e-6 {
        return None;
    }

    let angle = (perp / parallel).abs();
    let dist2 = dx * dx + dy * dy;
    if !angle.is_finite() || !dist2.is_finite() {
        return None;
    }

    Some(DirectionalPortRank {
        angle,
        parallel,
        dist2,
        port,
    })
}

pub(super) fn is_better_directional_port_rank(
    candidate: DirectionalPortRank,
    best: Option<DirectionalPortRank>,
) -> bool {
    let Some(best) = best else {
        return true;
    };
    let by_angle = candidate.angle.total_cmp(&best.angle);
    if by_angle != std::cmp::Ordering::Equal {
        return by_angle == std::cmp::Ordering::Less;
    }
    let by_parallel = candidate.parallel.total_cmp(&best.parallel);
    if by_parallel != std::cmp::Ordering::Equal {
        return by_parallel == std::cmp::Ordering::Less;
    }
    let by_dist = candidate.dist2.total_cmp(&best.dist2);
    if by_dist != std::cmp::Ordering::Equal {
        return by_dist == std::cmp::Ordering::Less;
    }
    candidate.port < best.port
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f32, y: f32) -> CanvasPoint {
        CanvasPoint { x, y }
    }

    #[test]
    fn rank_directional_port_candidate_rejects_points_behind_direction() {
        let from = point(10.0, 10.0);
        let candidate = point(5.0, 10.0);
        assert!(
            rank_directional_port_candidate(PortId::new(), from, candidate, PortNavDir::Right)
                .is_none()
        );
    }

    #[test]
    fn better_directional_rank_prefers_smaller_angle_then_parallel_then_distance() {
        let base = DirectionalPortRank {
            angle: 0.5,
            parallel: 20.0,
            dist2: 500.0,
            port: PortId::new(),
        };
        let tighter_angle = DirectionalPortRank {
            angle: 0.25,
            ..base
        };
        assert!(is_better_directional_port_rank(tighter_angle, Some(base)));

        let shorter_parallel = DirectionalPortRank {
            angle: base.angle,
            parallel: 10.0,
            ..base
        };
        assert!(is_better_directional_port_rank(
            shorter_parallel,
            Some(base)
        ));

        let shorter_distance = DirectionalPortRank {
            angle: base.angle,
            parallel: base.parallel,
            dist2: 100.0,
            ..base
        };
        assert!(is_better_directional_port_rank(
            shorter_distance,
            Some(base)
        ));
    }
}
