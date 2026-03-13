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
