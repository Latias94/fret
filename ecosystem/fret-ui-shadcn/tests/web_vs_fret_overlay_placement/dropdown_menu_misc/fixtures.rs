use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DropdownMenuMiscRecipe {
    ItemDropdownMenuItemHeight,
    ItemDropdownMenuContentInsets,
    ButtonGroupDemoOverlayPlacement,
    ModeToggleOverlayPlacement,
    ComboboxDropdownMenuOverlayPlacement,
    ComboboxDropdownMenuMenuItemHeight,
    ComboboxDropdownMenuMenuContentInsets,
    BreadcrumbDropdownMenuItemHeight,
    BreadcrumbDropdownMenuContentInsets,
}

#[derive(Debug, Clone, Deserialize)]
struct DropdownMenuMiscCase {
    id: String,
    web_name: String,
    recipe: DropdownMenuMiscRecipe,
}

fn assert_item_dropdown_menu_item_height_matches(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_h = expected_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));

    let snap = build_item_dropdown_open_snapshot(theme, expected_h.round());
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}

fn assert_item_dropdown_menu_content_insets_match(web_name: &str) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_item_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_item_h = expected_item_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let snap = build_item_dropdown_open_snapshot(theme, expected_item_h.round());
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);

    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}

fn build_mode_toggle_dropdown_menu(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry,
        DropdownMenuItem,
    };

    fn icon_stub<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(16.0));
                    layout.size.height = Length::Px(Px(16.0));
                    layout
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }

    DropdownMenu::new(open.clone())
        .align(DropdownMenuAlign::End)
        .into_element(
            cx,
            |cx| {
                Button::new("Toggle theme")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Icon)
                    .children([icon_stub(cx)])
                    .into_element(cx)
            },
            |_cx| {
                vec![
                    DropdownMenuEntry::Item(DropdownMenuItem::new("Light")),
                    DropdownMenuEntry::Item(DropdownMenuItem::new("Dark")),
                    DropdownMenuEntry::Item(DropdownMenuItem::new("System")),
                ]
            },
        )
}

#[test]
fn web_vs_fret_dropdown_menu_misc_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_dropdown_menu_misc_cases_v1.json"
    ));
    let suite: FixtureSuite<DropdownMenuMiscCase> =
        serde_json::from_str(raw).expect("dropdown-menu misc fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("dropdown-menu-misc case={}", case.id);
        match case.recipe {
            DropdownMenuMiscRecipe::ItemDropdownMenuItemHeight => {
                assert_item_dropdown_menu_item_height_matches(&case.web_name);
            }
            DropdownMenuMiscRecipe::ItemDropdownMenuContentInsets => {
                assert_item_dropdown_menu_content_insets_match(&case.web_name);
            }
            DropdownMenuMiscRecipe::ButtonGroupDemoOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("menu"),
                    |cx, open| render_button_group_demo_dropdown_menu(cx, open.clone()),
                    SemanticsRole::Button,
                    Some("More Options"),
                    SemanticsRole::Menu,
                );
            }
            DropdownMenuMiscRecipe::ModeToggleOverlayPlacement => {
                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("menu"),
                    |cx, open| build_mode_toggle_dropdown_menu(cx, open),
                    SemanticsRole::Button,
                    Some("Toggle theme"),
                    SemanticsRole::Menu,
                );
            }
            DropdownMenuMiscRecipe::ComboboxDropdownMenuOverlayPlacement => {
                assert_combobox_dropdown_menu_overlay_placement_matches(&case.web_name);
            }
            DropdownMenuMiscRecipe::ComboboxDropdownMenuMenuItemHeight => {
                assert_combobox_dropdown_menu_constrained_menu_item_height_matches(&case.web_name);
            }
            DropdownMenuMiscRecipe::ComboboxDropdownMenuMenuContentInsets => {
                assert_combobox_dropdown_menu_constrained_menu_content_insets_match(&case.web_name);
            }
            DropdownMenuMiscRecipe::BreadcrumbDropdownMenuItemHeight => {
                assert_breadcrumb_dropdown_menu_item_height_matches(&case.web_name);
            }
            DropdownMenuMiscRecipe::BreadcrumbDropdownMenuContentInsets => {
                assert_breadcrumb_dropdown_menu_content_insets_match(&case.web_name);
            }
        }
    }
}
