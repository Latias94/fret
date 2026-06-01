#[test]
fn material3_menu_search_view_composition_exposes_edge_and_full_screen_anchors() {
    let snippet = include_str!("../src/ui/snippets/material3/menu.rs");

    for needle in [
        "\"ui-gallery-material3-menu-search-bottom\"",
        "\"ui-gallery-material3-menu-search-bottom-panel\"",
        "\"ui-gallery-material3-menu-search-bottom-probe\"",
        "\"ui-gallery-material3-menu-search-full-screen\"",
        "\"ui-gallery-material3-menu-search-full-screen-panel\"",
        "\"ui-gallery-material3-menu-search-full-screen-actions\"",
        "\"ui-gallery-material3-menu-search-full-screen-actions-trigger\"",
        ".presentation(material3::SearchViewPresentation::FullScreen)",
    ] {
        assert!(
            snippet.contains(needle),
            "Material3 Menu SearchView composition should keep stable anchors; missing `{needle}`",
        );
    }
}

#[test]
fn material3_search_view_edge_full_screen_diag_uses_stable_contract_checks() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-search-view-edge-fullscreen-composition.json"
    );

    for needle in [
        "\"ui-gallery-material3-menu-search-bottom\"",
        "\"ui-gallery-material3-menu-search-bottom-panel\"",
        "\"kind\": \"bounds_within_window\"",
        "\"ui-gallery-material3-menu-search-full-screen\"",
        "\"ui-gallery-material3-menu-search-full-screen-panel\"",
        "\"role\": \"dialog\"",
        "\"ui-gallery-material3-menu-search-full-screen-actions\"",
        "\"kind\": \"not_exists\"",
        "\"kind\": \"focused_descendant_is\"",
    ] {
        assert!(
            script.contains(needle),
            "Material3 SearchView diag should keep edge/full-screen checks explicit; missing `{needle}`",
        );
    }
}
