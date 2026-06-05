use super::*;

fn center_of_rect(rect: Rect) -> Point {
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    )
}

fn baseline_bounds(baseline: &[(&str, Rect)], test_id: &str) -> Rect {
    baseline
        .iter()
        .find_map(|(id, rect)| (*id == test_id).then_some(*rect))
        .unwrap_or_else(|| panic!("missing baseline bounds for {test_id}"))
}

fn control_bounds_for_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
    test_id: &str,
) -> Rect {
    let node = node_for_test_id(ui, app, services, bounds, test_id);
    ui.debug_node_bounds(node)
        .unwrap_or_else(|| panic!("missing layout bounds for {test_id}"))
}

fn assert_same_rect(test_id: &str, before: Rect, after: Rect, state: &str) {
    assert_eq!(
        after.origin, before.origin,
        "{test_id} origin changed during {state}"
    );
    assert_eq!(
        after.size, before.size,
        "{test_id} size changed during {state}"
    );
}

mod base_controls;
mod disabled;
mod menu_tabs;
mod variants;
