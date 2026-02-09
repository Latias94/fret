use super::*;

#[test]
fn web_vs_fret_dropdown_menu_dialog_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "dropdown-menu-dialog",
        Some("menu"),
        |cx, open| {
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{
                Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign,
                DropdownMenuEntry, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel,
            };

            use fret_ui_kit::declarative::icon as decl_icon;

            let button = Button::new("")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::IconSm)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(32.0)))
                        .h_px(MetricRef::Px(Px(32.0))),
                )
                .children([decl_icon::icon(cx, fret_icons::ids::ui::MORE_HORIZONTAL)]);

            DropdownMenu::new(open.clone())
                // new-york-v4 dropdown-menu-dialog: `DropdownMenuContent className="w-40"`.
                .min_width(Px(160.0))
                .align(DropdownMenuAlign::End)
                .into_element(
                    cx,
                    |cx| button.into_element(cx),
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("File Actions")),
                            DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("New File...")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Share...")),
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Download").disabled(true),
                                ),
                            ])),
                        ]
                    },
                )
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_dialog_overlay_placement_matches_mobile_tiny_viewport() {
    assert_overlay_placement_matches(
        "dropdown-menu-dialog.vp375x240",
        Some("menu"),
        |cx, open| {
            use fret_ui_kit::{LayoutRefinement, MetricRef};
            use fret_ui_shadcn::{
                Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign,
                DropdownMenuEntry, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel,
            };

            use fret_ui_kit::declarative::icon as decl_icon;

            let button = Button::new("")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::IconSm)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(Px(32.0)))
                        .h_px(MetricRef::Px(Px(32.0))),
                )
                .children([decl_icon::icon(cx, fret_icons::ids::ui::MORE_HORIZONTAL)]);

            DropdownMenu::new(open.clone())
                // new-york-v4 dropdown-menu-dialog: `DropdownMenuContent className="w-40"`.
                .min_width(Px(160.0))
                .align(DropdownMenuAlign::End)
                .into_element(
                    cx,
                    |cx| button.into_element(cx),
                    |_cx| {
                        vec![
                            DropdownMenuEntry::Label(DropdownMenuLabel::new("File Actions")),
                            DropdownMenuEntry::Group(DropdownMenuGroup::new(vec![
                                DropdownMenuEntry::Item(DropdownMenuItem::new("New File...")),
                                DropdownMenuEntry::Item(DropdownMenuItem::new("Share...")),
                                DropdownMenuEntry::Item(
                                    DropdownMenuItem::new("Download").disabled(true),
                                ),
                            ])),
                        ]
                    },
                )
        },
        SemanticsRole::Button,
        None,
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_dialog_menu_item_height_matches() {
    let web_name = "dropdown-menu-dialog";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));

    let snap = build_dropdown_menu_dialog_open_snapshot(theme);
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}
#[test]
fn web_vs_fret_dropdown_menu_dialog_menu_item_height_matches_mobile_tiny_viewport() {
    let web_name = "dropdown-menu-dialog.vp375x240";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));

    let snap = build_dropdown_menu_dialog_open_snapshot(theme);
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}
#[test]
fn web_vs_fret_dropdown_menu_dialog_menu_content_insets_match() {
    let web_name = "dropdown-menu-dialog";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);

    let snap = build_dropdown_menu_dialog_open_snapshot(theme);
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
}
#[test]
fn web_vs_fret_dropdown_menu_dialog_menu_content_insets_match_mobile_tiny_viewport() {
    let web_name = "dropdown-menu-dialog.vp375x240";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);

    let snap = build_dropdown_menu_dialog_open_snapshot(theme);
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
}
