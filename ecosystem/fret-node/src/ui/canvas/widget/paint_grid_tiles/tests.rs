use super::grid_tile_ops;
use crate::ui::style::NodeGraphBackgroundPattern;
use fret_core::{Color, DrawOrder, Edges, Px};

#[test]
fn dots_pattern_emits_rounded_quads() {
    let white = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let ops = grid_tile_ops(
        NodeGraphBackgroundPattern::Dots,
        fret_core::Point::new(Px(0.0), Px(0.0)),
        100.0,
        20.0,
        4,
        white,
        white,
        Px(1.0),
        2.0,
        6.0,
    );

    assert!(!ops.is_empty());
    let any_rounded = ops.iter().any(|op| match op {
        fret_core::SceneOp::Quad { corner_radii, .. } => {
            corner_radii.top_left.0 > 0.0
                || corner_radii.top_right.0 > 0.0
                || corner_radii.bottom_left.0 > 0.0
                || corner_radii.bottom_right.0 > 0.0
        }
        _ => false,
    });
    assert!(any_rounded);
}

#[test]
fn cross_pattern_emits_axis_aligned_segments() {
    let white = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let ops = grid_tile_ops(
        NodeGraphBackgroundPattern::Cross,
        fret_core::Point::new(Px(0.0), Px(0.0)),
        40.0,
        20.0,
        4,
        white,
        white,
        Px(1.0),
        1.0,
        6.0,
    );

    assert!(!ops.is_empty());
    assert!(ops.iter().all(|op| matches!(
        op,
        fret_core::SceneOp::Quad {
            order: DrawOrder(1),
            border: Edges {
                top: Px(0.0),
                right: Px(0.0),
                bottom: Px(0.0),
                left: Px(0.0)
            },
            ..
        }
    )));
}
