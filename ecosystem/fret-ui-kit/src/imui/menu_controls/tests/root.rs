use super::*;

#[test]
fn menu_item_root_pressable_owns_visible_row_children() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        let mut response = ResponseExt::default();
        let options = MenuItemOptions {
            test_id: Some(Arc::from("menu.item")),
            ..Default::default()
        };

        menu_item_element_with_pressable_hook_inner(
            cx,
            Arc::from("Open"),
            options,
            SemanticsRole::MenuItem,
            None,
            None,
            noop_test_menu_item_pressable_hook::<App>,
            &mut response,
        )
    });

    let ElementKind::Pressable(props) = &el.kind else {
        panic!("expected menu item root to be the pressable row");
    };

    assert_eq!(props.a11y.test_id.as_deref(), Some("menu.item"));
    assert_ne!(
        props.layout.position,
        fret_ui::element::PositionStyle::Absolute
    );
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.size.height, Length::Auto);
    assert_eq!(el.children.len(), 1);
    assert!(matches!(el.children[0].kind, ElementKind::Container(_)));
}
