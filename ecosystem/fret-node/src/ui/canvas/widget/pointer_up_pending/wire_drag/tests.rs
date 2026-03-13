use super::checks::should_promote_pending_wire_drag;
use crate::core::{EdgeId, PortId};
use crate::rules::EdgeEndpoint;
use crate::ui::canvas::state::WireDragKind;

fn new_wire_drag() -> WireDragKind {
    WireDragKind::New {
        from: PortId::new(),
        bundle: vec![PortId::new()],
    }
}

#[test]
fn should_promote_pending_wire_drag_requires_click_connect_and_new_drag() {
    assert!(should_promote_pending_wire_drag(true, &new_wire_drag()));
    assert!(!should_promote_pending_wire_drag(false, &new_wire_drag()));
    assert!(!should_promote_pending_wire_drag(
        true,
        &WireDragKind::Reconnect {
            edge: EdgeId::new(),
            endpoint: EdgeEndpoint::From,
            fixed: PortId::new(),
        }
    ));
}
