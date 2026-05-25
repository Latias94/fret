use super::*;

#[test]
fn debug_draw_stroke_style_uses_v1_for_default_and_v2_for_explicit_policy() {
    let default_style = DebugDrawStrokeStyle::new(Px(2.0));
    assert_eq!(
        default_style.path_style(),
        PathStyle::Stroke(StrokeStyle { width: Px(2.0) })
    );

    let styled = DebugDrawStrokeStyle::new(Px(3.0))
        .with_join(StrokeJoinV1::Round)
        .with_cap(StrokeCapV1::Round)
        .with_miter_limit(8.0)
        .with_dash(Px(6.0), Px(4.0), Px(1.0));

    let PathStyle::StrokeV2(stroke) = styled.path_style() else {
        panic!("explicit debug-draw stroke policy should use StrokeV2");
    };
    assert_eq!(stroke.width, Px(3.0));
    assert_eq!(stroke.join, StrokeJoinV1::Round);
    assert_eq!(stroke.cap, StrokeCapV1::Round);
    assert_eq!(stroke.miter_limit, 8.0);
    assert_eq!(
        stroke.dash,
        Some(DashPatternV1::new(Px(6.0), Px(4.0), Px(1.0)))
    );
}

#[test]
fn debug_draw_stroke_style_ignores_invalid_dash_and_miter_inputs() {
    let style = DebugDrawStrokeStyle::new(Px(2.0))
        .with_miter_limit(f32::NAN)
        .with_dash(Px(0.0), Px(4.0), Px(0.0))
        .with_dash_pattern(DashPatternV1::new(Px(4.0), Px(-1.0), Px(0.0)));

    assert_eq!(style.miter_limit, 4.0);
    assert_eq!(style.dash, None);
    assert_eq!(
        style.path_style(),
        PathStyle::Stroke(StrokeStyle { width: Px(2.0) })
    );
}
