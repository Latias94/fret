use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MenubarDemoRecipe {
    ConstrainedOverlayPlacement,
    CheckboxIndicatorSlotInset,
    RadioIndicatorSlotInset,
    ConstrainedMenuItemHeight,
    ItemPaddingAndShortcut,
    ConstrainedMenuContentInsets,
    ConstrainedScrollState,
    WheelDoesNotMoveOverlay,
    SubmenuOverlayPlacement,
    SubmenuConstrainedMenuContentInsets,
    SubmenuFirstVisible,
    SubmenuMenuItemHeight,
}

#[derive(Debug, Clone, Deserialize)]
struct MenubarDemoCase {
    id: String,
    web_name: String,
    recipe: MenubarDemoRecipe,
    wheel_dy_px: Option<f32>,
}

#[test]
fn web_vs_fret_menubar_demo_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_menubar_demo_cases_v1.json"
    ));
    let suite: FixtureSuite<MenubarDemoCase> =
        serde_json::from_str(raw).expect("menubar-demo fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("menubar-demo case={}", case.id);
        match case.recipe {
            MenubarDemoRecipe::ConstrainedOverlayPlacement => {
                assert_menubar_demo_constrained_overlay_placement_matches(&case.web_name);
            }
            MenubarDemoRecipe::CheckboxIndicatorSlotInset => {
                assert_menubar_demo_checkbox_indicator_slot_inset_matches_web_impl(&case.web_name);
            }
            MenubarDemoRecipe::RadioIndicatorSlotInset => {
                assert_menubar_demo_radio_indicator_slot_inset_matches_web_impl(&case.web_name);
            }
            MenubarDemoRecipe::ConstrainedMenuItemHeight => {
                assert_menubar_demo_constrained_menu_item_height_matches(&case.web_name);
            }
            MenubarDemoRecipe::ItemPaddingAndShortcut => {
                assert_menubar_demo_item_padding_and_shortcut_match_impl(&case.web_name);
            }
            MenubarDemoRecipe::ConstrainedMenuContentInsets => {
                assert_menubar_demo_constrained_menu_content_insets_match(&case.web_name);
            }
            MenubarDemoRecipe::ConstrainedScrollState => {
                assert_menubar_demo_constrained_scroll_state_matches(&case.web_name);
            }
            MenubarDemoRecipe::WheelDoesNotMoveOverlay => {
                let wheel_dy_px = case
                    .wheel_dy_px
                    .expect("menubar-demo wheel_does_not_move_overlay requires wheel_dy_px");
                assert_menubar_demo_wheel_does_not_move_overlay(&case.web_name, wheel_dy_px);
            }
            MenubarDemoRecipe::SubmenuOverlayPlacement => {
                assert_menubar_demo_submenu_overlay_placement_matches(&case.web_name);
            }
            MenubarDemoRecipe::SubmenuConstrainedMenuContentInsets => {
                assert_menubar_demo_submenu_constrained_menu_content_insets_match(&case.web_name);
            }
            MenubarDemoRecipe::SubmenuFirstVisible => {
                assert_menubar_demo_submenu_first_visible_matches(&case.web_name);
            }
            MenubarDemoRecipe::SubmenuMenuItemHeight => {
                assert_menubar_demo_submenu_menu_item_height_matches(&case.web_name);
            }
        }
    }
}
