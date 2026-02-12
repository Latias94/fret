use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NavigationMenuRecipe {
    VariantOverlayPlacement,
    HoverSwitchOverlayPlacement,
    MobileViewportGeometry,
    MobileViewportInsets,
    MobileViewportGeometryAfterHover,
}

#[derive(Debug, Clone, Deserialize)]
struct NavigationMenuCase {
    id: String,
    web_name: String,
    recipe: NavigationMenuRecipe,
    open_value: Option<String>,
    from_value: Option<String>,
    to_value: Option<String>,
}

#[test]
fn web_vs_fret_navigation_menu_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_navigation_menu_cases_v1.json"
    ));
    let suite: FixtureSuite<NavigationMenuCase> =
        serde_json::from_str(raw).expect("navigation-menu fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("navigation-menu case={}", case.id);
        match case.recipe {
            NavigationMenuRecipe::VariantOverlayPlacement => {
                let open_value = case.open_value.as_deref().unwrap_or_else(|| {
                    panic!(
                        "missing open_value for recipe variant_overlay_placement: {}",
                        case.id
                    )
                });
                web_vs_fret_navigation_menu_demo_variant_overlay_placement_matches(
                    &case.web_name,
                    open_value,
                );
            }
            NavigationMenuRecipe::HoverSwitchOverlayPlacement => {
                let from_value = case.from_value.as_deref().unwrap_or_else(|| {
                    panic!(
                        "missing from_value for recipe hover_switch_overlay_placement: {}",
                        case.id
                    )
                });
                let to_value = case.to_value.as_deref().unwrap_or_else(|| {
                    panic!(
                        "missing to_value for recipe hover_switch_overlay_placement: {}",
                        case.id
                    )
                });
                web_vs_fret_navigation_menu_demo_hover_switch_overlay_placement_matches(
                    &case.web_name,
                    from_value,
                    to_value,
                );
            }
            NavigationMenuRecipe::MobileViewportGeometry => {
                let open_value = case.open_value.as_deref().unwrap_or_else(|| {
                    panic!(
                        "missing open_value for recipe mobile_viewport_geometry: {}",
                        case.id
                    )
                });
                assert_navigation_menu_demo_mobile_viewport_geometry_matches(
                    &case.web_name,
                    open_value,
                );
            }
            NavigationMenuRecipe::MobileViewportInsets => {
                let open_value = case.open_value.as_deref().unwrap_or_else(|| {
                    panic!(
                        "missing open_value for recipe mobile_viewport_insets: {}",
                        case.id
                    )
                });
                assert_navigation_menu_demo_mobile_viewport_insets_match(
                    &case.web_name,
                    open_value,
                );
            }
            NavigationMenuRecipe::MobileViewportGeometryAfterHover => {
                let from_value = case.from_value.as_deref().unwrap_or_else(|| {
                    panic!(
                        "missing from_value for recipe mobile_viewport_geometry_after_hover: {}",
                        case.id
                    )
                });
                let to_value = case.to_value.as_deref().unwrap_or_else(|| {
                    panic!(
                        "missing to_value for recipe mobile_viewport_geometry_after_hover: {}",
                        case.id
                    )
                });
                assert_navigation_menu_demo_mobile_viewport_geometry_after_hover_matches(
                    &case.web_name,
                    from_value,
                    to_value,
                );
            }
        }
    }
}
