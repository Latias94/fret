use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ItemDropdownRecipe {
    OverlayPlacement,
}

#[derive(Debug, Clone, Deserialize)]
struct ItemDropdownCase {
    id: String,
    web_name: String,
    recipe: ItemDropdownRecipe,
}

fn web_open_trigger_rect(web_name: &str, web: &WebGolden) -> WebRect {
    let is_open_trigger = |n: &WebNode| {
        n.tag == "button"
            && (n
                .attrs
                .get("data-state")
                .is_some_and(|v| v.as_str() == "open")
                || n.attrs
                    .get("aria-expanded")
                    .is_some_and(|v| v.as_str() == "true"))
    };

    let web_trigger = find_first(&web.themes["light"].root, &is_open_trigger)
        .or_else(|| find_first(&web.themes["dark"].root, &is_open_trigger))
        .unwrap_or_else(|| panic!("web trigger (button) for {web_name}"));

    web_trigger.rect
}

fn web_dropdown_menu_item_height(web_name: &str, theme: &WebGoldenTheme) -> f32 {
    let expected_item_hs = web_portal_slot_heights(theme, &["dropdown-menu-item"]);
    let expected_item_h = expected_item_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));
    expected_item_h.round()
}

fn build_item_dropdown_overlay(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
    trigger_rect: WebRect,
    item_h: f32,
) -> AnyElement {
    use fret_ui::element::LayoutStyle;
    use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
    use fret_ui_shadcn::{
        Avatar, AvatarFallback, Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuAlign,
        DropdownMenuEntry, DropdownMenuItem, Item, ItemContent, ItemDescription, ItemMedia,
        ItemSize, ItemTitle,
    };

    use fret_ui_kit::declarative::icon as decl_icon;

    let button = Button::new("Select")
        .variant(ButtonVariant::Outline)
        .size(ButtonSize::Sm)
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(trigger_rect.w)))
                .h_px(MetricRef::Px(Px(trigger_rect.h))),
        )
        .children([decl_icon::icon(cx, fret_icons::ids::ui::CHEVRON_DOWN)]);

    let item_h = Px(item_h);

    let people = vec![
        ("shadcn", "shadcn@vercel.com"),
        ("maxleiter", "maxleiter@vercel.com"),
        ("evilrabbit", "evilrabbit@vercel.com"),
    ];

    let entries: Vec<DropdownMenuEntry> = people
        .into_iter()
        .map(|(username, email)| {
            let content = Item::new(vec![
                ItemMedia::new(vec![
                    Avatar::new(vec![
                        AvatarFallback::new(
                            username
                                .chars()
                                .next()
                                .map(|ch| ch.to_string())
                                .unwrap_or_else(|| "?".to_owned()),
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                ItemContent::new(vec![
                    ItemTitle::new(username).into_element(cx),
                    ItemDescription::new(email).into_element(cx),
                ])
                .gap(Px(2.0))
                .into_element(cx),
            ])
            .size(ItemSize::Sm)
            .refine_style(
                ChromeRefinement::default()
                    .p(Space::N2)
                    .rounded(fret_ui_kit::Radius::Md),
            )
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .h_px(MetricRef::Px(item_h)),
            )
            .into_element(cx);

            DropdownMenuEntry::Item(
                DropdownMenuItem::new(username)
                    .padding(Edges::all(Px(0.0)))
                    .estimated_height(item_h)
                    .content(content),
            )
        })
        .collect();

    let dropdown = DropdownMenu::new(open.clone())
        // new-york-v4 item-dropdown: `DropdownMenuContent className="w-72"`.
        .min_width(Px(288.0))
        .align(DropdownMenuAlign::End)
        .into_element(cx, |cx| button.into_element(cx), |_cx| entries);

    cx.container(
        ContainerProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout
            },
            padding: Edges {
                left: Px(trigger_rect.x),
                ..Default::default()
            },
            ..Default::default()
        },
        |_cx| vec![dropdown],
    )
}

#[test]
fn web_vs_fret_item_dropdown_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_item_dropdown_cases_v1.json"
    ));
    let suite: FixtureSuite<ItemDropdownCase> =
        serde_json::from_str(raw).expect("item-dropdown fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("item-dropdown case={}", case.id);
        match case.recipe {
            ItemDropdownRecipe::OverlayPlacement => {
                let web = read_web_golden_open(&case.web_name);
                let theme = web_theme(&web);
                let trigger_rect = web_open_trigger_rect(&case.web_name, &web);
                let item_h = web_dropdown_menu_item_height(&case.web_name, &theme);

                assert_overlay_placement_matches(
                    &case.web_name,
                    Some("menu"),
                    move |cx, open| build_item_dropdown_overlay(cx, open, trigger_rect, item_h),
                    SemanticsRole::Button,
                    Some("Select"),
                    SemanticsRole::Menu,
                );
            }
        }
    }
}
