use super::super::*;

pub(super) fn preview_item(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(920.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let icon_media = |cx: &mut ElementContext<'_, App>, icon: &'static str| {
        shadcn::ItemMedia::new([shadcn::icon::icon(cx, fret_icons::IconId::new_static(icon))])
            .variant(shadcn::ItemMediaVariant::Icon)
            .into_element(cx)
    };

    let avatar_media = |cx: &mut ElementContext<'_, App>, initials: &'static str| {
        shadcn::ItemMedia::new([shadcn::Avatar::new([
            shadcn::AvatarFallback::new(initials).into_element(cx)
        ])
        .refine_layout(LayoutRefinement::default().w_px(Px(28.0)).h_px(Px(28.0)))
        .into_element(cx)])
        .into_element(cx)
    };

    let image_media = |cx: &mut ElementContext<'_, App>, label: &'static str| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(theme.color_required("muted")))
                    .rounded(Radius::Sm),
                LayoutRefinement::default().size_full(),
            )
        });
        shadcn::ItemMedia::new([
            cx.container(props, move |cx| vec![shadcn::typography::muted(cx, label)])
        ])
        .variant(shadcn::ItemMediaVariant::Image)
        .into_element(cx)
    };

    let item_row = |cx: &mut ElementContext<'_, App>,
                    title: &'static str,
                    description: &'static str,
                    media: AnyElement,
                    variant: shadcn::ItemVariant,
                    size: shadcn::ItemSize,
                    with_action: bool,
                    test_id: &'static str| {
        let mut children = vec![
            media,
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new(title).into_element(cx),
                shadcn::ItemDescription::new(description).into_element(cx),
            ])
            .into_element(cx),
        ];

        if with_action {
            children.push(
                shadcn::ItemActions::new([shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx)])
                .into_element(cx),
            );
        }

        shadcn::Item::new(children)
            .variant(variant)
            .size(size)
            .on_click(CMD_APP_OPEN)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id(test_id)
    };

    let item_row_icon = |cx: &mut ElementContext<'_, App>,
                         title: &'static str,
                         description: &'static str,
                         icon: &'static str,
                         variant: shadcn::ItemVariant,
                         size: shadcn::ItemSize,
                         with_action: bool,
                         test_id: &'static str| {
        let media = icon_media(cx, icon);
        item_row(
            cx,
            title,
            description,
            media,
            variant,
            size,
            with_action,
            test_id,
        )
    };

    let item_row_avatar = |cx: &mut ElementContext<'_, App>,
                           title: &'static str,
                           description: &'static str,
                           initials: &'static str,
                           variant: shadcn::ItemVariant,
                           size: shadcn::ItemSize,
                           with_action: bool,
                           test_id: &'static str| {
        let media = avatar_media(cx, initials);
        item_row(
            cx,
            title,
            description,
            media,
            variant,
            size,
            with_action,
            test_id,
        )
    };

    let item_row_image = |cx: &mut ElementContext<'_, App>,
                          title: &'static str,
                          description: &'static str,
                          label: &'static str,
                          variant: shadcn::ItemVariant,
                          size: shadcn::ItemSize,
                          with_action: bool,
                          test_id: &'static str| {
        let media = image_media(cx, label);
        item_row(
            cx,
            title,
            description,
            media,
            variant,
            size,
            with_action,
            test_id,
        )
    };

    let demo = {
        let content = item_row_icon(
            cx,
            "Invoice.pdf",
            "Updated 2 days ago",
            "lucide.file-text",
            shadcn::ItemVariant::Default,
            shadcn::ItemSize::Default,
            true,
            "ui-gallery-item-demo",
        );
        section_card(cx, "Demo", content)
    };

    let variant = {
        let content = shadcn::ItemGroup::new([
            item_row_icon(
                cx,
                "Default",
                "Neutral style with hover/press states.",
                "lucide.layout-dashboard",
                shadcn::ItemVariant::Default,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-variant-default",
            ),
            item_row_icon(
                cx,
                "Outline",
                "Visible border emphasis for dense lists.",
                "lucide.panel-top",
                shadcn::ItemVariant::Outline,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-variant-outline",
            ),
            item_row_icon(
                cx,
                "Muted",
                "Low-contrast background for secondary groups.",
                "lucide.inbox",
                shadcn::ItemVariant::Muted,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-variant-muted",
            ),
        ])
        .gap(Px(8.0))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(720.0)))
        .into_element(cx);
        section_card(cx, "Variant", content)
    };

    let size = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(720.0))),
            |cx| {
                vec![
                    item_row_icon(
                        cx,
                        "Default Size",
                        "Use for regular settings and list rows.",
                        "lucide.settings",
                        shadcn::ItemVariant::Default,
                        shadcn::ItemSize::Default,
                        false,
                        "ui-gallery-item-size-default",
                    ),
                    item_row_icon(
                        cx,
                        "Small Size",
                        "Compact row density.",
                        "lucide.layers",
                        shadcn::ItemVariant::Default,
                        shadcn::ItemSize::Sm,
                        false,
                        "ui-gallery-item-size-sm",
                    ),
                    shadcn::typography::muted(
                        cx,
                        "Current Fret API supports default/sm; docs `xs` is not exposed yet.",
                    ),
                ]
            },
        );
        section_card(cx, "Size", content)
    };

    let icon = {
        let content = item_row_icon(
            cx,
            "Analytics",
            "Open dashboard metrics and trends.",
            "lucide.chart-column-big",
            shadcn::ItemVariant::Default,
            shadcn::ItemSize::Default,
            true,
            "ui-gallery-item-icon",
        );
        section_card(cx, "Icon", content)
    };

    let avatar = {
        let content = item_row_avatar(
            cx,
            "Dana Chen",
            "Design review owner",
            "DC",
            shadcn::ItemVariant::Default,
            shadcn::ItemSize::Default,
            true,
            "ui-gallery-item-avatar",
        );
        section_card(cx, "Avatar", content)
    };

    let image = {
        let content = item_row_image(
            cx,
            "Cover Image",
            "Media-style item with image slot",
            "IMG",
            shadcn::ItemVariant::Default,
            shadcn::ItemSize::Default,
            true,
            "ui-gallery-item-image",
        );
        section_card(cx, "Image", content)
    };

    let group = {
        let content = shadcn::ItemGroup::new([
            item_row_icon(
                cx,
                "README.md",
                "Updated now",
                "lucide.file-text",
                shadcn::ItemVariant::Default,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-group-readme",
            ),
            shadcn::ItemSeparator::new().into_element(cx),
            item_row_icon(
                cx,
                "Roadmap.md",
                "Updated yesterday",
                "lucide.map",
                shadcn::ItemVariant::Default,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-group-roadmap",
            ),
            shadcn::ItemSeparator::new().into_element(cx),
            item_row_icon(
                cx,
                "Changelog.md",
                "Updated 3 days ago",
                "lucide.history",
                shadcn::ItemVariant::Default,
                shadcn::ItemSize::Default,
                false,
                "ui-gallery-item-group-changelog",
            ),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(720.0)))
        .into_element(cx)
        .test_id("ui-gallery-item-group");
        section_card(cx, "Group", content)
    };

    let header = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(720.0))),
            |cx| {
                vec![
                    shadcn::ItemHeader::new([
                        shadcn::ItemTitle::new("Recent Files").into_element(cx),
                        shadcn::Button::new("View all")
                            .variant(shadcn::ButtonVariant::Ghost)
                            .size(shadcn::ButtonSize::Sm)
                            .on_click(CMD_APP_OPEN)
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    item_row_icon(
                        cx,
                        "Draft proposal",
                        "Edited by Alex",
                        "lucide.file-pen-line",
                        shadcn::ItemVariant::Outline,
                        shadcn::ItemSize::Default,
                        false,
                        "ui-gallery-item-header-row",
                    ),
                ]
            },
        )
        .test_id("ui-gallery-item-header");
        section_card(cx, "Header", content)
    };

    let link = {
        let content = item_row_icon(
            cx,
            "Dashboard",
            "Overview of your account and activity.",
            "lucide.house",
            shadcn::ItemVariant::Outline,
            shadcn::ItemSize::Default,
            false,
            "ui-gallery-item-link",
        );
        section_card(cx, "Link", content)
    };

    let dropdown = {
        let dropdown_media = icon_media(cx, "lucide.folder");
        let content = shadcn::Item::new([
            dropdown_media,
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new("Team Drive").into_element(cx),
                shadcn::ItemDescription::new("Shared files and permissions").into_element(cx),
            ])
            .into_element(cx),
            shadcn::ItemActions::new([shadcn::Button::new("Actions")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::ButtonSize::Sm)
                .on_click(CMD_APP_OPEN)
                .into_element(cx)])
            .into_element(cx),
        ])
        .variant(shadcn::ItemVariant::Default)
        .on_click(CMD_APP_OPEN)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(720.0)))
        .into_element(cx)
        .test_id("ui-gallery-item-dropdown");
        section_card(cx, "Dropdown", content)
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let props = cx.with_theme(|theme| {
                    decl_style::container_props(
                        theme,
                        ChromeRefinement::default(),
                        LayoutRefinement::default().w_full().max_w(Px(720.0)),
                    )
                });
                cx.container(props, |cx| {
                    vec![item_row_icon(
                        cx,
                        "???? ??????",
                        "???? ???? ??? ?????",
                        "lucide.layout-dashboard",
                        shadcn::ItemVariant::Default,
                        shadcn::ItemSize::Default,
                        true,
                        "ui-gallery-item-rtl-row",
                    )]
                })
            },
        )
        .test_id("ui-gallery-item-rtl");

        section_card(cx, "RTL", rtl_content)
    };

    let component_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Preview follows shadcn Item docs order: Demo, Variant, Size, Icon, Avatar, Image, Group, Header, Link, Dropdown, RTL.",
                ),
                demo,
                variant,
                size,
                icon,
                avatar,
                image,
                group,
                header,
                link,
                dropdown,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_panel_body).test_id("ui-gallery-item-component");

    let code_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Basic Composition").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"Item::new([ItemMedia::new([...]), ItemContent::new([...]), ItemActions::new([...])])"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Variant and Size").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"Item::new([...]).variant(ItemVariant::Outline).size(ItemSize::Sm)"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Grouping").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"ItemGroup::new([item_a, ItemSeparator::new().into_element(cx), item_b])"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
            ]
        },
    );
    let code_panel = shell(cx, code_panel_body);

    let notes_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "Current API variants: default/outline/muted; sizes: default/sm.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Docs `asChild` link and avatar-specific media variant are approximated with `on_click` and composed `Avatar`.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Each section has stable test_id for future diag scripts.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_panel_body);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-item",
        component_panel,
        code_panel,
        notes_panel,
    )
}
