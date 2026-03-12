use super::should_arm_pending_edge_insert_drag;
use fret_core::Modifiers;

#[test]
fn should_arm_pending_edge_insert_drag_requires_policy_and_alt_modifier() {
    assert!(should_arm_pending_edge_insert_drag(
        true,
        Modifiers {
            alt: true,
            ..Modifiers::default()
        }
    ));
    assert!(should_arm_pending_edge_insert_drag(
        true,
        Modifiers {
            alt_gr: true,
            ..Modifiers::default()
        }
    ));
    assert!(!should_arm_pending_edge_insert_drag(
        true,
        Modifiers::default()
    ));
    assert!(!should_arm_pending_edge_insert_drag(
        false,
        Modifiers {
            alt: true,
            ..Modifiers::default()
        }
    ));
}
