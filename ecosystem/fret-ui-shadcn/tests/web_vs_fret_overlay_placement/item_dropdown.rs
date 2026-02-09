use super::*;

#[test]
fn web_vs_fret_item_dropdown_overlay_placement_matches() {
    let web_name = "item-dropdown";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

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
        .expect("web trigger (button)");
    let trigger_rect = web_trigger.rect;

    let expected_item_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_item_h = expected_item_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));
    let item_h = expected_item_h.round();

    assert_overlay_placement_matches(
        web_name,
        Some("menu"),
        move |cx, open| {
            use fret_ui::element::LayoutStyle;
            use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
            use fret_ui_shadcn::{
                Avatar, AvatarFallback, Button, ButtonSize, ButtonVariant, DropdownMenu,
                DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem, Item, ItemContent,
                ItemDescription, ItemMedia, ItemSize, ItemTitle,
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
                            .h_px(MetricRef::Px(Px(item_h))),
                    )
                    .into_element(cx);

                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new(username)
                            .padding(Edges::all(Px(0.0)))
                            .estimated_height(Px(item_h))
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
        },
        SemanticsRole::Button,
        Some("Select"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_item_dropdown_overlay_placement_matches_mobile_tiny_viewport() {
    let web_name = "item-dropdown.vp375x240";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

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
        .expect("web trigger (button)");
    let trigger_rect = web_trigger.rect;

    let expected_item_hs = web_portal_slot_heights(&theme, &["dropdown-menu-item"]);
    let expected_item_h = expected_item_hs
        .iter()
        .copied()
        .next()
        .unwrap_or_else(|| panic!("missing web dropdown-menu-item height for {web_name}"));
    let item_h = expected_item_h.round();

    assert_overlay_placement_matches(
        web_name,
        Some("menu"),
        move |cx, open| {
            use fret_ui::element::LayoutStyle;
            use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};
            use fret_ui_shadcn::{
                Avatar, AvatarFallback, Button, ButtonSize, ButtonVariant, DropdownMenu,
                DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem, Item, ItemContent,
                ItemDescription, ItemMedia, ItemSize, ItemTitle,
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
                            .h_px(MetricRef::Px(Px(item_h))),
                    )
                    .into_element(cx);

                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new(username)
                            .padding(Edges::all(Px(0.0)))
                            .estimated_height(Px(item_h))
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
        },
        SemanticsRole::Button,
        Some("Select"),
        SemanticsRole::Menu,
    );
}
