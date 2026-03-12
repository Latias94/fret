use super::*;
use crate::core::{EdgeId, PortId};
use crate::rules::EdgeEndpoint;

#[test]
fn should_prepare_sticky_wire_pointer_down_requires_left_sticky_and_active_drag() {
    assert!(should_prepare_sticky_wire_pointer_down(
        MouseButton::Left,
        true,
        true,
    ));
    assert!(!should_prepare_sticky_wire_pointer_down(
        MouseButton::Right,
        true,
        true,
    ));
    assert!(!should_prepare_sticky_wire_pointer_down(
        MouseButton::Left,
        false,
        true,
    ));
    assert!(!should_prepare_sticky_wire_pointer_down(
        MouseButton::Left,
        true,
        false,
    ));
}

#[test]
fn sticky_wire_from_port_only_accepts_new_wire_drags() {
    let from = PortId::new();
    assert_eq!(
        sticky_wire_from_port(&WireDragKind::New {
            from,
            bundle: vec![from],
        }),
        Some(from)
    );
    assert_eq!(
        sticky_wire_from_port(&WireDragKind::Reconnect {
            edge: EdgeId::new(),
            endpoint: EdgeEndpoint::From,
            fixed: PortId::new(),
        }),
        None
    );
}
