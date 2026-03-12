use super::should_handle_click_connect_port_hit;

#[test]
fn should_handle_click_connect_port_hit_requires_all_flags() {
    assert!(should_handle_click_connect_port_hit(true, true, true));
    assert!(!should_handle_click_connect_port_hit(false, true, true));
    assert!(!should_handle_click_connect_port_hit(true, false, true));
    assert!(!should_handle_click_connect_port_hit(true, true, false));
}
