use super::*;

#[test]
fn web_vs_fret_dropdown_menu_demo_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "dropdown-menu-demo",
        Some("menu"),
        |cx, open| {
            use fret_ui_shadcn::{
                Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
                DropdownMenuLabel, DropdownMenuShortcut,
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
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Profile")
                                    .trailing(DropdownMenuShortcut::new("⇧⌘P").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Billing")
                                    .trailing(DropdownMenuShortcut::new("⌘B").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Settings")
                                    .trailing(DropdownMenuShortcut::new("⌘S").into_element(cx)),
                            ),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Keyboard shortcuts")
                                    .trailing(DropdownMenuShortcut::new("⌘K").into_element(cx)),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(
                                vec![
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                                    DropdownMenuEntry::Separator,
                                    DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                                ],
                            )),
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("New Team")
                                    .trailing(DropdownMenuShortcut::new("⌘+T").into_element(cx)),
                            ),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(
                                DropdownMenuItem::new("Log out")
                                    .trailing(DropdownMenuShortcut::new("⇧⌘Q").into_element(cx)),
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
#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_overlay_placement_matches() {
    assert_dropdown_menu_demo_constrained_overlay_placement_matches(
        "dropdown-menu-demo.vp1440x320",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_overlay_placement_matches() {
    assert_dropdown_menu_demo_constrained_overlay_placement_matches(
        "dropdown-menu-demo.vp1440x240",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_mobile_tiny_viewport_overlay_placement_matches() {
    assert_dropdown_menu_demo_constrained_overlay_placement_matches("dropdown-menu-demo.vp375x240");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_menu_item_height_matches() {
    assert_dropdown_menu_demo_constrained_menu_item_height_matches("dropdown-menu-demo.vp1440x320");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_menu_item_height_matches() {
    assert_dropdown_menu_demo_constrained_menu_item_height_matches("dropdown-menu-demo.vp1440x240");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_mobile_tiny_viewport_menu_item_height_matches() {
    assert_dropdown_menu_demo_constrained_menu_item_height_matches("dropdown-menu-demo.vp375x240");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_menu_item_height_matches() {
    assert_dropdown_menu_demo_constrained_menu_item_height_matches("dropdown-menu-demo");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_profile_item_padding_and_shortcut_match() {
    assert_dropdown_menu_demo_profile_item_padding_and_shortcut_match_impl("dropdown-menu-demo");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_menu_content_insets_match() {
    assert_dropdown_menu_demo_constrained_menu_content_insets_match(
        "dropdown-menu-demo.vp1440x320",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_menu_content_insets_match() {
    assert_dropdown_menu_demo_constrained_menu_content_insets_match(
        "dropdown-menu-demo.vp1440x240",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_mobile_tiny_viewport_menu_content_insets_match() {
    assert_dropdown_menu_demo_constrained_menu_content_insets_match("dropdown-menu-demo.vp375x240");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_small_viewport_scroll_state_matches() {
    assert_dropdown_menu_demo_constrained_scroll_state_matches("dropdown-menu-demo.vp1440x320");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_scroll_state_matches() {
    assert_dropdown_menu_demo_constrained_scroll_state_matches("dropdown-menu-demo.vp1440x240");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_mobile_tiny_viewport_scroll_state_matches() {
    assert_dropdown_menu_demo_constrained_scroll_state_matches("dropdown-menu-demo.vp375x240");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_mobile_tiny_viewport_wheel_scroll_matches_web_scrolled_80() {
    assert_dropdown_menu_demo_wheel_scroll_matches_web_scrolled(
        "dropdown-menu-demo.vp375x240-scrolled-80",
        -80.0,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_menu_content_insets_match() {
    assert_dropdown_menu_demo_constrained_menu_content_insets_match("dropdown-menu-demo");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches("dropdown-menu-demo.submenu-kbd");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_hover_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches("dropdown-menu-demo.submenu");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_small_viewport_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_tiny_viewport_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_mobile_tiny_viewport_overlay_placement_matches() {
    assert_dropdown_menu_demo_submenu_overlay_placement_matches(
        "dropdown-menu-demo.submenu-kbd-vp375x240",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_small_viewport_menu_content_insets_match() {
    assert_dropdown_menu_demo_submenu_constrained_menu_content_insets_match(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_tiny_viewport_menu_content_insets_match() {
    assert_dropdown_menu_demo_submenu_constrained_menu_content_insets_match(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_mobile_tiny_viewport_menu_content_insets_match() {
    assert_dropdown_menu_demo_submenu_constrained_menu_content_insets_match(
        "dropdown-menu-demo.submenu-kbd-vp375x240",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_menu_content_insets_match() {
    assert_dropdown_menu_demo_submenu_constrained_menu_content_insets_match(
        "dropdown-menu-demo.submenu-kbd",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_hover_menu_content_insets_match() {
    assert_dropdown_menu_demo_submenu_constrained_menu_content_insets_match(
        "dropdown-menu-demo.submenu",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_first_visible_matches() {
    assert_dropdown_menu_demo_submenu_first_visible_matches("dropdown-menu-demo.submenu-kbd");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_hover_first_visible_matches() {
    assert_dropdown_menu_demo_submenu_first_visible_matches("dropdown-menu-demo.submenu");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_small_viewport_first_visible_matches() {
    assert_dropdown_menu_demo_submenu_first_visible_matches(
        "dropdown-menu-demo.submenu-kbd-vp1440x320",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_tiny_viewport_first_visible_matches() {
    assert_dropdown_menu_demo_submenu_first_visible_matches(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_mobile_tiny_viewport_first_visible_matches() {
    assert_dropdown_menu_demo_submenu_first_visible_matches(
        "dropdown-menu-demo.submenu-kbd-vp375x240",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_menu_item_height_matches() {
    assert_dropdown_menu_demo_submenu_menu_item_height_matches("dropdown-menu-demo.submenu-kbd");
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_hover_menu_item_height_matches() {
    assert_dropdown_menu_demo_submenu_menu_item_height_matches("dropdown-menu-demo.submenu");
}
