use super::*;
use fret_core::Point;

#[test]
fn child_region_resize_y_height_from_start_clamps_to_min_and_max() {
    let mut response = ChildRegionResizeYResponse {
        min_height: Some(Px(48.0)),
        max_height: Some(Px(160.0)),
        ..Default::default()
    };

    response
        .drag
        .set_motion(Point::new(Px(0.0), Px(0.0)), Point::new(Px(0.0), Px(24.0)));
    assert_eq!(response.height_from_start(Px(100.0)), Px(124.0));

    response.drag.set_motion(
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(0.0), Px(-120.0)),
    );
    assert_eq!(response.height_from_start(Px(100.0)), Px(48.0));

    response
        .drag
        .set_motion(Point::new(Px(0.0), Px(0.0)), Point::new(Px(0.0), Px(120.0)));
    assert_eq!(response.height_from_start(Px(100.0)), Px(160.0));
}
