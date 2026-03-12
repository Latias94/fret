use super::*;

fn white() -> Color {
    Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    }
}

#[test]
fn grid_tile_spec_projects_tile_indices_from_bounds() {
    let spec = grid_tile_spec(
        Point::new(Px(5.0), Px(15.0)),
        40.0,
        20.0,
        4,
        white(),
        white(),
        Px(1.0),
        2.0,
        6.0,
    );

    assert_eq!(spec.x0, 0);
    assert_eq!(spec.x1, 3);
    assert_eq!(spec.y0, 0);
    assert_eq!(spec.y1, 3);
}

#[test]
fn approx_ops_matches_pattern_density() {
    let spec = grid_tile_spec(
        Point::new(Px(0.0), Px(0.0)),
        40.0,
        20.0,
        4,
        white(),
        white(),
        Px(1.0),
        2.0,
        6.0,
    );

    assert_eq!(approx_ops(&spec, NodeGraphBackgroundPattern::Lines), 6);
    assert_eq!(approx_ops(&spec, NodeGraphBackgroundPattern::Dots), 9);
    assert_eq!(approx_ops(&spec, NodeGraphBackgroundPattern::Cross), 18);
}
