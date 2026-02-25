use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_avatar(
    cx: &mut ElementContext<'_, App>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).clone();

    #[derive(Default)]
    struct AvatarDropdownOpenState {
        model: Option<Model<bool>>,
    }

    let dropdown_open = cx.with_state(AvatarDropdownOpenState::default, |st| st.model.clone());
    let dropdown_open = if let Some(model) = dropdown_open {
        model
    } else {
        let model = cx.app.models_mut().insert(false);
        cx.with_state(AvatarDropdownOpenState::default, |st| {
            st.model = Some(model.clone());
        });
        model
    };

    let avatar_with_image = |cx: &mut ElementContext<'_, App>,
                             size: shadcn::AvatarSize,
                             fallback_text: &'static str,
                             test_id: &'static str| {
        let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
        let fallback = shadcn::AvatarFallback::new(fallback_text)
            .when_image_missing_model(avatar_image.clone())
            .delay_ms(120)
            .into_element(cx);
        shadcn::Avatar::new([image, fallback])
            .size(size)
            .into_element(cx)
            .test_id(test_id)
    };

    let avatar_fallback_only =
        |cx: &mut ElementContext<'_, App>, size: shadcn::AvatarSize, test_id: &'static str| {
            shadcn::Avatar::new([shadcn::AvatarFallback::new("CN").into_element(cx)])
                .size(size)
                .into_element(cx)
                .test_id(test_id)
        };

    let sizes = {
        doc_layout::wrap_controls_row(cx, &theme, Space::N4, |cx| {
            vec![
                avatar_with_image(
                    cx,
                    shadcn::AvatarSize::Sm,
                    "CN",
                    "ui-gallery-avatar-sizes-sm",
                ),
                avatar_with_image(
                    cx,
                    shadcn::AvatarSize::Default,
                    "CN",
                    "ui-gallery-avatar-sizes-default",
                ),
                avatar_with_image(
                    cx,
                    shadcn::AvatarSize::Lg,
                    "CN",
                    "ui-gallery-avatar-sizes-lg",
                ),
            ]
        })
        .test_id("ui-gallery-avatar-sizes")
    };

    let fallback = {
        doc_layout::wrap_controls_row(cx, &theme, Space::N4, |cx| {
            vec![
                avatar_fallback_only(cx, shadcn::AvatarSize::Sm, "ui-gallery-avatar-fallback-sm"),
                avatar_fallback_only(
                    cx,
                    shadcn::AvatarSize::Default,
                    "ui-gallery-avatar-fallback-default",
                ),
                avatar_fallback_only(cx, shadcn::AvatarSize::Lg, "ui-gallery-avatar-fallback-lg"),
            ]
        })
        .test_id("ui-gallery-avatar-fallback")
    };

    let icon = |cx: &mut ElementContext<'_, App>, name: &'static str, size: Px, fg: ColorRef| {
        shadcn::icon::icon_with(
            cx,
            fret_icons::IconId::new_static(name),
            Some(size),
            Some(fg),
        )
    };

    let with_badge = {
        let dot_row = doc_layout::wrap_controls_row(cx, &theme, Space::N4, |cx| {
            let avatar = |cx: &mut ElementContext<'_, App>,
                          size: shadcn::AvatarSize,
                          badge: shadcn::AvatarBadge,
                          test_id: &'static str| {
                let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
                let fallback = shadcn::AvatarFallback::new("CN")
                    .when_image_missing_model(avatar_image.clone())
                    .delay_ms(120)
                    .into_element(cx);
                let badge = badge.into_element(cx);
                shadcn::Avatar::new([image, fallback, badge])
                    .size(size)
                    .into_element(cx)
                    .test_id(test_id)
            };

            let custom_badge = shadcn::AvatarBadge::new().refine_style(
                ChromeRefinement::default().bg(ColorRef::Color(theme.color_token("destructive"))),
            );

            vec![
                avatar(
                    cx,
                    shadcn::AvatarSize::Sm,
                    shadcn::AvatarBadge::new(),
                    "ui-gallery-avatar-badge-sm",
                ),
                avatar(
                    cx,
                    shadcn::AvatarSize::Default,
                    custom_badge,
                    "ui-gallery-avatar-badge-default",
                ),
                avatar(
                    cx,
                    shadcn::AvatarSize::Lg,
                    shadcn::AvatarBadge::new(),
                    "ui-gallery-avatar-badge-lg",
                ),
            ]
        })
        .test_id("ui-gallery-avatar-badge-dot-row");

        let icon_row = doc_layout::wrap_controls_row(cx, &theme, Space::N4, |cx| {
            let fg = ColorRef::Color(theme.color_token("primary-foreground"));
            let badge =
                shadcn::AvatarBadge::new().children([icon(cx, "lucide.plus", Px(10.0), fg)]);
            let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
            let fallback = shadcn::AvatarFallback::new("CN")
                .when_image_missing_model(avatar_image.clone())
                .delay_ms(120)
                .into_element(cx);
            vec![
                shadcn::Avatar::new([image, fallback, badge.into_element(cx)])
                    .size(shadcn::AvatarSize::Default)
                    .into_element(cx)
                    .test_id("ui-gallery-avatar-badge-icon"),
            ]
        })
        .test_id("ui-gallery-avatar-badge-icon-row");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| vec![dot_row, icon_row],
        )
        .test_id("ui-gallery-avatar-badge")
    };

    let avatar_group = {
        let group = |cx: &mut ElementContext<'_, App>,
                     size: shadcn::AvatarSize,
                     test_id: &'static str| {
            let avatars = (0..3)
                .map(|idx| {
                    let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
                    let fallback = shadcn::AvatarFallback::new(["CN", "ML", "ER"][idx])
                        .when_image_missing_model(avatar_image.clone())
                        .delay_ms(120)
                        .into_element(cx);
                    shadcn::Avatar::new([image, fallback])
                        .size(size)
                        .into_element(cx)
                        .test_id(format!("ui-gallery-avatar-group-item-{test_id}-{idx}"))
                })
                .collect::<Vec<_>>();

            shadcn::AvatarGroup::new(avatars)
                .size(size)
                .into_element(cx)
                .test_id(test_id)
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |cx| {
                vec![
                    group(cx, shadcn::AvatarSize::Sm, "ui-gallery-avatar-group-sm"),
                    group(
                        cx,
                        shadcn::AvatarSize::Default,
                        "ui-gallery-avatar-group-default",
                    ),
                    group(cx, shadcn::AvatarSize::Lg, "ui-gallery-avatar-group-lg"),
                ]
            },
        )
        .test_id("ui-gallery-avatar-group")
    };

    let group_count = {
        let group_with_count = |cx: &mut ElementContext<'_, App>,
                                size: shadcn::AvatarSize,
                                test_id: &'static str| {
            let avatars = (0..3)
                .map(|idx| {
                    let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
                    let fallback = shadcn::AvatarFallback::new(["CN", "ML", "ER"][idx])
                        .when_image_missing_model(avatar_image.clone())
                        .delay_ms(120)
                        .into_element(cx);
                    shadcn::Avatar::new([image, fallback])
                        .size(size)
                        .into_element(cx)
                })
                .collect::<Vec<_>>();

            let count = shadcn::AvatarGroupCount::new([ui::text(cx, "+3")
                .font_medium()
                .nowrap()
                .into_element(cx)])
            .into_element(cx);

            shadcn::AvatarGroup::new(avatars.into_iter().chain([count]).collect::<Vec<_>>())
                .size(size)
                .into_element(cx)
                .test_id(test_id)
        };

        let group_with_icon_count = |cx: &mut ElementContext<'_, App>,
                                     size: shadcn::AvatarSize,
                                     test_id: &'static str| {
            let avatars = (0..2)
                .map(|idx| {
                    let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
                    let fallback = shadcn::AvatarFallback::new(["CN", "ML"][idx])
                        .when_image_missing_model(avatar_image.clone())
                        .delay_ms(120)
                        .into_element(cx);
                    shadcn::Avatar::new([image, fallback])
                        .size(size)
                        .into_element(cx)
                })
                .collect::<Vec<_>>();

            let fg = ColorRef::Color(theme.color_token("muted-foreground"));
            let count = shadcn::AvatarGroupCount::new([icon(cx, "lucide.plus", Px(18.0), fg)])
                .into_element(cx);

            shadcn::AvatarGroup::new(avatars.into_iter().chain([count]).collect::<Vec<_>>())
                .size(size)
                .into_element(cx)
                .test_id(test_id)
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |cx| {
                vec![
                    group_with_count(
                        cx,
                        shadcn::AvatarSize::Sm,
                        "ui-gallery-avatar-group-count-sm",
                    ),
                    group_with_count(
                        cx,
                        shadcn::AvatarSize::Default,
                        "ui-gallery-avatar-group-count-default",
                    ),
                    group_with_count(
                        cx,
                        shadcn::AvatarSize::Lg,
                        "ui-gallery-avatar-group-count-lg",
                    ),
                    group_with_icon_count(
                        cx,
                        shadcn::AvatarSize::Default,
                        "ui-gallery-avatar-group-count-icon",
                    ),
                ]
            },
        )
        .test_id("ui-gallery-avatar-group-count")
    };

    let dropdown = {
        doc_layout::wrap_controls_row(cx, &theme, Space::N4, |cx| {
            let avatar_image_for_trigger = avatar_image.clone();
            let trigger = move |cx: &mut ElementContext<'_, App>| {
                let image =
                    shadcn::AvatarImage::model(avatar_image_for_trigger.clone()).into_element(cx);
                let fallback = shadcn::AvatarFallback::new("CN")
                    .when_image_missing_model(avatar_image_for_trigger.clone())
                    .delay_ms(120)
                    .into_element(cx);

            let avatar = shadcn::Avatar::new([image, fallback])
                .size(shadcn::AvatarSize::Default)
                .into_element(cx);

                // Match shadcn docs: Avatar is composed inside a ghost icon button used as the
                // dropdown trigger (`asChild`-style).
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Icon)
                    .a11y_label("Open user menu")
                    .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                    .children([avatar])
                    .test_id("ui-gallery-avatar-dropdown-trigger")
                    .into_element(cx)
            };

            let entries = |_cx: &mut ElementContext<'_, App>| {
                vec![
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Profile")
                            .test_id("ui-gallery-avatar-dropdown-item-profile"),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Billing")
                            .test_id("ui-gallery-avatar-dropdown-item-billing"),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Settings")
                            .test_id("ui-gallery-avatar-dropdown-item-settings"),
                    ),
                    shadcn::DropdownMenuEntry::Separator,
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Log out")
                            .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                            .test_id("ui-gallery-avatar-dropdown-item-logout"),
                    ),
                ]
            };

            vec![
                shadcn::DropdownMenu::new(dropdown_open.clone()).into_element(cx, trigger, entries),
            ]
        })
        .test_id("ui-gallery-avatar-dropdown-row")
    };

    let rtl = {
        doc_layout::wrap_controls_row(cx, &theme, Space::N4, |cx| {
            vec![fret_ui_kit::primitives::direction::with_direction_provider(
                cx,
                fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
                |cx| {
                    avatar_with_image(
                        cx,
                        shadcn::AvatarSize::Default,
                        "CN",
                        "ui-gallery-avatar-rtl",
                    )
                },
            )]
        })
        .test_id("ui-gallery-avatar-rtl-row")
    };

    let notes = doc_layout::notes(
        cx,
        [
            "Use `AvatarImage` when you already have an `ImageId` (cached/decoded).",
            "Use `AvatarFallback` to cover missing images and slow network loads.",
            "If you customize sizes, set both width and height to keep the avatar circular.",
        ],
    );

    let basic = avatar_with_image(
        cx,
        shadcn::AvatarSize::Default,
        "CN",
        "ui-gallery-avatar-basic",
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview aims to match shadcn Avatar docs: Basic, Badge, Group, Sizes, Dropdown, RTL (plus a small Fallback-only extra).",
        ),
        vec![
            DocSection::new("Basic", basic)
            .description("A basic avatar with an image + fallback.")
            .code(
                "rust",
                r#"let image = shadcn::AvatarImage::model(avatar_image).into_element(cx);
let fallback = shadcn::AvatarFallback::new("CN").into_element(cx);

shadcn::Avatar::new([image, fallback])
    .size(shadcn::AvatarSize::Default)
    .into_element(cx);"#,
            ),
            DocSection::new("With Badge", with_badge)
                .description(
                    "`AvatarBadge` overlays a status dot or icon at the avatar's bottom-right.",
                )
                .code(
                    "rust",
                    r#"let image = shadcn::AvatarImage::model(avatar_image).into_element(cx);
let fallback = shadcn::AvatarFallback::new("CN").into_element(cx);
let badge = shadcn::AvatarBadge::new().into_element(cx);

shadcn::Avatar::new([image, fallback, badge])
    .size(shadcn::AvatarSize::Default)
    .into_element(cx);"#,
                ),
            DocSection::new("Avatar Group", avatar_group)
                .description("Overlapping avatar group (`-space-x-2`).")
                .code(
                    "rust",
                    r#"let avatars = (0..3)
    .map(|_idx| shadcn::Avatar::new([/* ... */]).size(shadcn::AvatarSize::Default).into_element(cx))
    .collect::<Vec<_>>();

shadcn::AvatarGroup::new(avatars)
    .size(shadcn::AvatarSize::Default)
    .into_element(cx);"#,
                ),
            DocSection::new("Avatar Group Count", group_count)
                .description("Trailing count bubble that matches the group's size.")
                .code(
                    "rust",
                    r#"let avatars = vec![/* ... */];
let count = shadcn::AvatarGroupCount::new([ui::text(cx, "+3").into_element(cx)]).into_element(cx);

shadcn::AvatarGroup::new(avatars.into_iter().chain([count]).collect::<Vec<_>>())
    .size(shadcn::AvatarSize::Default)
    .into_element(cx);"#,
                ),
            DocSection::new("Sizes", sizes)
                .description("Upstream: `size=\"sm\" | \"default\" | \"lg\"`.")
                .code(
                    "rust",
                    r#"shadcn::Avatar::new([image, fallback])
    .size(shadcn::AvatarSize::Lg)
    .into_element(cx);"#,
                ),
            DocSection::new("Dropdown", dropdown)
                .description("Use Avatar as a DropdownMenu trigger (shadcn `asChild`-style composition).")
                .code(
                    "rust",
                    r#"let open: Model<bool> = cx.app.models_mut().insert(false);

shadcn::DropdownMenu::new(open).into_element(
    cx,
    |cx| {
        let avatar = shadcn::Avatar::new([/* image + fallback */]).into_element(cx);
        shadcn::Button::new("")
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::ButtonSize::Icon)
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .children([avatar])
            .into_element(cx)
    },
    |_cx| vec![
        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Profile")),
        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Billing")),
        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Settings")),
        shadcn::DropdownMenuEntry::Separator,
    ],
);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Avatar should behave under an RTL direction provider.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::Avatar::new([/* ... */]).into_element(cx)
})"#,
                ),
            DocSection::new("Extras: Fallback only", fallback)
                .description("Fallback-only avatars at each size.")
                .code(
                    "rust",
                    r#"shadcn::Avatar::new([shadcn::AvatarFallback::new("CN").into_element(cx)])
    .size(shadcn::AvatarSize::Sm)
    .into_element(cx);"#,
                ),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-avatar")]
}
