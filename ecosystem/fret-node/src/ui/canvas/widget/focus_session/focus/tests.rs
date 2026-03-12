use super::*;

#[test]
fn focus_edge_clears_port_focus_and_hints() {
    let mut interaction = InteractionState {
        focused_node: Some(GraphNodeId::from_u128(1)),
        focused_port: Some(PortId::from_u128(2)),
        focused_port_valid: true,
        focused_port_convertible: true,
        ..Default::default()
    };

    let edge = EdgeId::from_u128(3);
    focus_edge(&mut interaction, edge);

    assert_eq!(interaction.focused_edge, Some(edge));
    assert_eq!(interaction.focused_node, None);
    assert_eq!(interaction.focused_port, None);
    assert!(!interaction.focused_port_valid);
    assert!(!interaction.focused_port_convertible);
}
