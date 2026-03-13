use super::*;

#[test]
fn advance_active_item_skips_disabled_entries() {
    let mut menu = super::test_support::menu(
        vec![
            super::test_support::item("first", false),
            super::test_support::item("second", true),
            super::test_support::item("third", false),
        ],
        0,
    );

    active_item::advance_context_menu_active_item(&mut menu, false);

    assert_eq!(menu.active_item, 1);
}

#[test]
fn advance_active_item_wraps_backwards() {
    let mut menu = super::test_support::menu(
        vec![
            super::test_support::item("first", true),
            super::test_support::item("second", false),
            super::test_support::item("third", true),
        ],
        0,
    );

    active_item::advance_context_menu_active_item(&mut menu, true);

    assert_eq!(menu.active_item, 2);
}

#[test]
fn typeahead_falls_back_to_single_character_match() {
    let mut menu = super::test_support::menu(
        vec![
            super::test_support::item("Alpha", true),
            super::test_support::item("Beta", true),
        ],
        0,
    );
    menu.typeahead.push('a');

    typeahead::apply_context_menu_typeahead(&mut menu, 'b');

    assert_eq!(menu.typeahead, "b");
    assert_eq!(menu.active_item, 1);
}

#[test]
fn pop_typeahead_reports_whether_anything_changed() {
    let mut menu = super::test_support::menu(vec![super::test_support::item("Alpha", true)], 0);
    assert!(!typeahead::pop_context_menu_typeahead(&mut menu));

    menu.typeahead.push('a');
    assert!(typeahead::pop_context_menu_typeahead(&mut menu));
    assert!(menu.typeahead.is_empty());
}

#[test]
fn sync_hovered_item_promotes_enabled_item_and_clears_typeahead() {
    let mut menu = super::test_support::menu(
        vec![
            super::test_support::item("Alpha", true),
            super::test_support::item("Beta", true),
        ],
        0,
    );
    menu.typeahead.push('a');

    assert!(hover::sync_context_menu_hovered_item(&mut menu, Some(1)));
    assert_eq!(menu.hovered_item, Some(1));
    assert_eq!(menu.active_item, 1);
    assert!(menu.typeahead.is_empty());
}

#[test]
fn sync_hovered_item_keeps_active_for_disabled_item() {
    let mut menu = super::test_support::menu(
        vec![
            super::test_support::item("Alpha", true),
            super::test_support::item("Beta", false),
        ],
        0,
    );

    assert!(hover::sync_context_menu_hovered_item(&mut menu, Some(1)));
    assert_eq!(menu.hovered_item, Some(1));
    assert_eq!(menu.active_item, 0);
}
