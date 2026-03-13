use super::*;

#[test]
fn opposite_port_direction_flips_io() {
    assert_eq!(
        opposite_port_direction(PortDirection::In),
        PortDirection::Out
    );
    assert_eq!(
        opposite_port_direction(PortDirection::Out),
        PortDirection::In
    );
}
