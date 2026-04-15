const INSPECTOR_PANEL_RS: &str = include_str!("../src/composites/inspector_panel.rs");
const PROPERTY_GROUP_RS: &str = include_str!("../src/composites/property_group.rs");
const PROPERTY_GRID_RS: &str = include_str!("../src/composites/property_grid.rs");

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn editor_composites_expose_explicit_context_access_overloads() {
    let inspector = normalize_ws(INSPECTOR_PANEL_RS);
    let property_group = normalize_ws(PROPERTY_GROUP_RS);
    let property_grid = normalize_ws(PROPERTY_GRID_RS);

    for (label, source, marker) in [
        (
            "inspector_panel.rs",
            &inspector,
            "pub fn into_element_in<'a, H: UiHost + 'a, Cx>( self, cx: &mut Cx, toolbar: impl FnOnce(&mut ElementContext<'_, H>, &InspectorPanelCx) -> Vec<AnyElement>, contents: impl FnOnce(&mut ElementContext<'_, H>, &InspectorPanelCx) -> Vec<AnyElement>, ) -> AnyElement where Cx: ElementContextAccess<'a, H>,",
        ),
        (
            "property_group.rs",
            &property_group,
            "pub fn into_element_in<'a, H: UiHost + 'a, Cx>( self, cx: &mut Cx, header_actions: impl FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>, contents: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>, ) -> AnyElement where Cx: ElementContextAccess<'a, H>,",
        ),
        (
            "property_grid.rs",
            &property_grid,
            "pub fn into_element_in<'a, H: UiHost + 'a, Cx>( self, cx: &mut Cx, rows: impl FnOnce(&mut ElementContext<'_, H>, PropertyGridRowCx) -> Vec<AnyElement>, ) -> AnyElement where Cx: ElementContextAccess<'a, H>,",
        ),
    ] {
        let marker = normalize_ws(marker);
        assert!(
            source.contains(&marker),
            "{label} should keep the explicit context-access overload for helper-heavy app/editor authoring"
        );
    }
}
