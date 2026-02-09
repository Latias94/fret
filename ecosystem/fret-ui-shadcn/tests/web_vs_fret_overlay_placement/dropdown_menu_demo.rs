use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DropdownMenuDemoRecipe {
    OverlayPlacement,
    ConstrainedOverlayPlacement,
    ConstrainedMenuItemHeight,
    ProfileItemPaddingAndShortcut,
    ConstrainedMenuContentInsets,
    ConstrainedScrollState,
    WheelScroll,
    SubmenuOverlayPlacement,
    SubmenuConstrainedMenuContentInsets,
    SubmenuFirstVisible,
    SubmenuMenuItemHeight,
}

#[derive(Debug, Clone, Deserialize)]
struct DropdownMenuDemoCase {
    id: String,
    web_name: String,
    recipe: DropdownMenuDemoRecipe,
    delta_y: Option<f32>,
}

#[test]
fn web_vs_fret_dropdown_menu_demo_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_dropdown_menu_demo_cases_v1.json"
    ));
    let suite: FixtureSuite<DropdownMenuDemoCase> =
        serde_json::from_str(raw).expect("dropdown-menu-demo fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("dropdown-menu-demo case={}", case.id);
        match case.recipe {
            DropdownMenuDemoRecipe::OverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("menu"),
                    |cx, open| {
                        use fret_ui_shadcn::{
                            Button, ButtonVariant, DropdownMenu, DropdownMenuEntry,
                            DropdownMenuItem, DropdownMenuLabel, DropdownMenuShortcut,
                        };

                        DropdownMenu::new(open.clone())
                            // new-york-v4 dropdown-menu-demo: `DropdownMenuContent className="w-56"`.
                            .min_width(Px(224.0))
                            .into_element(
                                cx,
                                |cx| {
                                    Button::new("Open")
                                        .variant(ButtonVariant::Outline)
                                        .into_element(cx)
                                },
                                |cx| {
                                    vec![
                                        DropdownMenuEntry::Label(DropdownMenuLabel::new(
                                            "My Account",
                                        )),
                                        DropdownMenuEntry::Item(
                                            DropdownMenuItem::new("Profile").trailing(
                                                DropdownMenuShortcut::new("⇧⌘P").into_element(cx),
                                            ),
                                        ),
                                        DropdownMenuEntry::Item(
                                            DropdownMenuItem::new("Billing").trailing(
                                                DropdownMenuShortcut::new("⌘B").into_element(cx),
                                            ),
                                        ),
                                        DropdownMenuEntry::Item(
                                            DropdownMenuItem::new("Settings").trailing(
                                                DropdownMenuShortcut::new("⌘S").into_element(cx),
                                            ),
                                        ),
                                        DropdownMenuEntry::Item(
                                            DropdownMenuItem::new("Keyboard shortcuts").trailing(
                                                DropdownMenuShortcut::new("⌘K").into_element(cx),
                                            ),
                                        ),
                                        DropdownMenuEntry::Separator,
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                                        DropdownMenuEntry::Item(
                                            DropdownMenuItem::new("Invite users").submenu(vec![
                                                DropdownMenuEntry::Item(DropdownMenuItem::new(
                                                    "Email",
                                                )),
                                                DropdownMenuEntry::Item(DropdownMenuItem::new(
                                                    "Message",
                                                )),
                                                DropdownMenuEntry::Separator,
                                                DropdownMenuEntry::Item(DropdownMenuItem::new(
                                                    "More...",
                                                )),
                                            ]),
                                        ),
                                        DropdownMenuEntry::Item(
                                            DropdownMenuItem::new("New Team").trailing(
                                                DropdownMenuShortcut::new("⌘+T").into_element(cx),
                                            ),
                                        ),
                                        DropdownMenuEntry::Separator,
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                                        DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                                        DropdownMenuEntry::Item(
                                            DropdownMenuItem::new("API").disabled(true),
                                        ),
                                        DropdownMenuEntry::Separator,
                                        DropdownMenuEntry::Item(
                                            DropdownMenuItem::new("Log out").trailing(
                                                DropdownMenuShortcut::new("⇧⌘Q").into_element(cx),
                                            ),
                                        ),
                                    ]
                                },
                            )
                    },
                    SemanticsRole::Button,
                    Some("Open"),
                    SemanticsRole::Menu,
                );
            }
            DropdownMenuDemoRecipe::ConstrainedOverlayPlacement => {
                assert_dropdown_menu_demo_constrained_overlay_placement_matches(&case.web_name);
            }
            DropdownMenuDemoRecipe::ConstrainedMenuItemHeight => {
                assert_dropdown_menu_demo_constrained_menu_item_height_matches(&case.web_name);
            }
            DropdownMenuDemoRecipe::ProfileItemPaddingAndShortcut => {
                assert_dropdown_menu_demo_profile_item_padding_and_shortcut_match_impl(
                    &case.web_name,
                );
            }
            DropdownMenuDemoRecipe::ConstrainedMenuContentInsets => {
                assert_dropdown_menu_demo_constrained_menu_content_insets_match(&case.web_name);
            }
            DropdownMenuDemoRecipe::ConstrainedScrollState => {
                assert_dropdown_menu_demo_constrained_scroll_state_matches(&case.web_name);
            }
            DropdownMenuDemoRecipe::WheelScroll => {
                let delta_y = case
                    .delta_y
                    .expect("dropdown-menu-demo wheel_scroll requires delta_y");
                assert_dropdown_menu_demo_wheel_scroll_matches_web_scrolled(
                    &case.web_name,
                    delta_y,
                );
            }
            DropdownMenuDemoRecipe::SubmenuOverlayPlacement => {
                assert_dropdown_menu_demo_submenu_overlay_placement_matches(&case.web_name);
            }
            DropdownMenuDemoRecipe::SubmenuConstrainedMenuContentInsets => {
                assert_dropdown_menu_demo_submenu_constrained_menu_content_insets_match(
                    &case.web_name,
                );
            }
            DropdownMenuDemoRecipe::SubmenuFirstVisible => {
                assert_dropdown_menu_demo_submenu_first_visible_matches(&case.web_name);
            }
            DropdownMenuDemoRecipe::SubmenuMenuItemHeight => {
                assert_dropdown_menu_demo_submenu_menu_item_height_matches(&case.web_name);
            }
        }
    }
}
