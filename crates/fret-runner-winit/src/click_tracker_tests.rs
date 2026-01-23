use super::*;

#[test]
fn click_tracker_reports_is_click_false_when_moved_beyond_slop() {
    let mut tracker = ClickTracker::default();
    let button = MouseButton::Left;

    let start = Point::new(Px(0.0), Px(0.0));
    let end = Point::new(Px(ClickTracker::CLICK_SLOP_PX + 1.0), Px(0.0));

    tracker.begin_press(button, start);
    tracker.update_move(end);
    let (_count, is_click) = tracker.end_press(button, end);
    assert!(!is_click);
}
