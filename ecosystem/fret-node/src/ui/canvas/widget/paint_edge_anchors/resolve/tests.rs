use super::edge_anchor_endpoint_allowed;
use crate::rules::EdgeEndpoint;

#[test]
fn edge_anchor_endpoint_allowed_maps_endpoint_to_matching_flag() {
    assert!(edge_anchor_endpoint_allowed(
        EdgeEndpoint::From,
        (true, false)
    ));
    assert!(!edge_anchor_endpoint_allowed(
        EdgeEndpoint::To,
        (true, false)
    ));
    assert!(edge_anchor_endpoint_allowed(
        EdgeEndpoint::To,
        (false, true)
    ));
}
