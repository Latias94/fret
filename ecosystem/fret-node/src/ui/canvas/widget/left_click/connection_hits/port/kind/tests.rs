use super::*;

#[test]
fn wire_drag_kind_from_yanked_edges_falls_back_to_new_wire() {
    let port = PortId::new();
    assert!(matches!(
        wire_drag_kind_from_yanked_edges(port, None),
        WireDragKind::New { from, bundle } if from == port && bundle == vec![port]
    ));
}

#[test]
fn wire_drag_kind_from_yanked_edges_uses_single_and_many_reconnect_modes() {
    let port = PortId::new();
    let edge = EdgeId::new();
    let fixed = PortId::new();

    assert!(matches!(
        wire_drag_kind_from_yanked_edges(
            port,
            Some(vec![(edge, EdgeEndpoint::To, fixed)])
        ),
        WireDragKind::Reconnect { edge: got_edge, endpoint: EdgeEndpoint::To, fixed: got_fixed }
            if got_edge == edge && got_fixed == fixed
    ));

    assert!(matches!(
        wire_drag_kind_from_yanked_edges(
            port,
            Some(vec![
                (EdgeId::new(), EdgeEndpoint::From, PortId::new()),
                (EdgeId::new(), EdgeEndpoint::To, PortId::new()),
            ])
        ),
        WireDragKind::ReconnectMany { .. }
    ));
}
