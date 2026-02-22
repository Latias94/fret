use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_avatar(
    cx: &mut ElementContext<'_, App>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).clone();

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

    let notes = doc_layout::notes(
        cx,
        [
            "Use `AvatarImage` when you already have an `ImageId` (cached/decoded).",
            "Use `AvatarFallback` to cover missing images and slow network loads.",
            "If you customize sizes, set both width and height to keep the avatar circular.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Avatar demo order: Sizes, Fallback, With Badge, Avatar Group, Avatar Group Count.",
        ),
        vec![
            DocSection::new("Sizes", sizes)
                .description("Upstream: `size=\"sm\" | \"default\" | \"lg\"`.")
                .code(
                    "rust",
                    r#"shadcn::Avatar::new([image, fallback])
    .size(shadcn::AvatarSize::Lg)
    .into_element(cx);"#,
                ),
            DocSection::new("Fallback", fallback)
                .description("Fallback-only avatars at each size.")
                .code(
                    "rust",
                    r#"shadcn::Avatar::new([shadcn::AvatarFallback::new("CN").into_element(cx)])
    .size(shadcn::AvatarSize::Sm)
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
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-avatar")]
}
