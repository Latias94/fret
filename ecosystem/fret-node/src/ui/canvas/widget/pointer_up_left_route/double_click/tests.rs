use super::*;

#[test]
fn should_open_edge_insert_picker_requires_plain_double_click() {
    assert!(should_open_edge_insert_picker(2, Modifiers::default()));
    assert!(!should_open_edge_insert_picker(1, Modifiers::default()));
    assert!(!should_open_edge_insert_picker(
        2,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        }
    ));
    assert!(!should_open_edge_insert_picker(
        2,
        Modifiers {
            alt_gr: true,
            ..Modifiers::default()
        }
    ));
}
