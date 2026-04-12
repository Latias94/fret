fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn navigation_menu_page_keeps_query_axis_teaching_explicit() {
    let source = include_str!("../src/ui/pages/navigation_menu.rs");

    for needle in [
        "Container Query Toggle remains an explicit Fret follow-up.",
        "Container Query Toggle is the explicit query-axis teaching surface for this page: keep the window wide, change only the local demo card, and compare viewport-vs-container `md` behavior without mixing the two.",
        "Keep the window wide, shrink only the local demo card, and compare viewport-driven versus container-driven md breakpoint behavior.",
    ] {
        assert!(
            source.contains(needle),
            "navigation menu page should keep the query-axis teaching explicit; missing `{needle}`",
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            demo,
            usage,
            link_component,
            rtl,
            api_reference,
            demo_with_toggle,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "navigation menu page should keep the upstream docs path first and the query-axis follow-up after API Reference",
    );
}

#[test]
fn navigation_menu_toggle_snippet_and_diag_gate_compare_viewport_vs_container_without_mixing_axes()
{
    let snippet = include_str!("../src/ui/snippets/navigation_menu/demo.rs");
    let diag_script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/navigation/ui-gallery-navigation-menu-md-breakpoint-query-source-toggle.json"
    );

    for needle in [
        "Use container query breakpoints (UI gallery)",
        "Query source: container width. The demo keeps the window wide but clamps only the local card below md.",
        "Query source: viewport width. The breakpoint follows the full window width for upstream web parity.",
        "Outcome: the Components panel drops to the single-column layout because only the local demo region shrinks.",
        "Outcome: the Components panel stays on the two-column desktop layout because the window itself remains above md.",
        "ui-gallery-navigation-menu-query-source-copy",
        "ui-gallery-navigation-menu-query-outcome-copy",
        "LayoutRefinement::default().w_px(Px(560.0)).min_w_0()",
        "NavigationMenuMdBreakpointQuery::Container",
        "NavigationMenuMdBreakpointQuery::Viewport",
        "ui-gallery-navigation-menu-demo-components-layout-two-col",
        "ui-gallery-navigation-menu-demo-components-layout-single-col",
    ] {
        assert!(
            snippet.contains(needle),
            "navigation menu toggle snippet should keep the axis split copy and layout seam explicit; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-navigation-menu-md-breakpoint-query-switch\"",
        "\"checked\": false",
        "\"checked\": true",
        "\"ui-gallery-navigation-menu-demo-components-layout-two-col\"",
        "\"ui-gallery-navigation-menu-demo-components-layout-single-col\"",
        "\"ui-gallery-navigation-menu-md-breakpoint-query-source-toggle-viewport\"",
        "\"ui-gallery-navigation-menu-md-breakpoint-query-source-toggle-container\"",
        "\"ui-gallery-navigation-menu-md-breakpoint-query-source-toggle\"",
    ] {
        assert!(
            diag_script.contains(needle),
            "navigation menu md-breakpoint-query diag gate should prove the viewport/container split explicitly; missing `{needle}`",
        );
    }
}
