use super::*;

#[test]
fn tail_pointer_down_route_maps_buttons_to_expected_lane() {
    assert_eq!(
        tail_pointer_down_route(MouseButton::Left),
        PointerDownTailRoute::LeftClick
    );
    assert_eq!(
        tail_pointer_down_route(MouseButton::Right),
        PointerDownTailRoute::RightClick
    );
    assert_eq!(
        tail_pointer_down_route(MouseButton::Middle),
        PointerDownTailRoute::Ignore
    );
}
