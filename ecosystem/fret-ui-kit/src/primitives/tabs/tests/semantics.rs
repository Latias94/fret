use super::*;

#[test]
fn tab_panel_semantics_props_sets_role_and_labelled_by() {
    let props =
        tab_panel_semantics_props(LayoutStyle::default(), Some(Arc::from("Panel")), Some(123));
    assert_eq!(props.role, SemanticsRole::TabPanel);
    assert_eq!(props.label.as_deref(), Some("Panel"));
    assert_eq!(props.labelled_by_element, Some(123));
    assert!(
        props.focusable,
        "tabpanel should be focusable like Radix tabIndex=0"
    );
}

#[test]
fn tab_list_semantics_props_sets_role_and_orientation() {
    let props = tab_list_semantics_props(LayoutStyle::default(), TabsOrientation::Vertical);
    assert_eq!(props.role, SemanticsRole::TabList);
    assert_eq!(
        props.orientation,
        Some(fret_core::SemanticsOrientation::Vertical)
    );
}
