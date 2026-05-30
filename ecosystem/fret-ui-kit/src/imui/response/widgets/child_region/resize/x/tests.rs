use super::*;
use fret_core::Point;

#[test]
fn child_region_resize_x_width_from_start_clamps_to_min_and_max() {
    let mut response = ChildRegionResizeXResponse {
        min_width: Some(Px(80.0)),
        max_width: Some(Px(320.0)),
        ..Default::default()
    };

    response
        .drag
        .set_motion(Point::new(Px(0.0), Px(0.0)), Point::new(Px(24.0), Px(0.0)));
    assert_eq!(response.width_from_start(Px(160.0)), Px(184.0));

    response.drag.set_motion(
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(-120.0), Px(0.0)),
    );
    assert_eq!(response.width_from_start(Px(160.0)), Px(80.0));

    response
        .drag
        .set_motion(Point::new(Px(0.0), Px(0.0)), Point::new(Px(240.0), Px(0.0)));
    assert_eq!(response.width_from_start(Px(160.0)), Px(320.0));
}
