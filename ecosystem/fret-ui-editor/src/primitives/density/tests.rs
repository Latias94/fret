use fret_core::Px;

use super::EditorDensity;

#[test]
fn affordance_extent_prefers_row_height_when_visual_hit_is_smaller() {
    let density = EditorDensity {
        row_height: Px(24.0),
        hit_thickness: Px(20.0),
        ..Default::default()
    };

    assert_eq!(density.affordance_extent(), Px(24.0));
}

#[test]
fn affordance_extent_preserves_larger_hit_targets() {
    let density = EditorDensity {
        row_height: Px(22.0),
        hit_thickness: Px(28.0),
        ..Default::default()
    };

    assert_eq!(density.affordance_extent(), Px(28.0));
}
