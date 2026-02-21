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

    let (with_badge_title, with_badge) = doc_layout::gap_card(
        cx,
        "With Badge",
        "Upstream shadcn `AvatarBadge` overlays a small status dot/plus icon. Fret shadcn `AvatarBadge` is not implemented yet; this section is a parity placeholder.",
        "ui-gallery-avatar-badge-gap",
    );

    let avatar_group = {
        let group = |cx: &mut ElementContext<'_, App>, size: shadcn::AvatarSize| {
            let avatars = (0..3)
                .map(|idx| {
                    let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
                    let fallback = shadcn::AvatarFallback::new(["CN", "ML", "ER"][idx])
                        .when_image_missing_model(avatar_image.clone())
                        .delay_ms(120)
                        .into_element(cx);

                    let mut avatar = shadcn::Avatar::new([image, fallback]).size(size);
                    if idx > 0 {
                        avatar =
                            avatar.refine_layout(LayoutRefinement::default().ml_neg(Space::N2));
                    }

                    avatar
                        .into_element(cx)
                        .test_id(format!("ui-gallery-avatar-group-item-{idx}"))
                })
                .collect::<Vec<_>>();

            cx.flex(
                fret_ui::element::FlexProps {
                    layout: fret_ui::element::LayoutStyle::default(),
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
                    justify: fret_ui::element::MainAlign::Start,
                    align: fret_ui::element::CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| avatars,
            )
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |cx| {
                vec![
                    group(cx, shadcn::AvatarSize::Sm).test_id("ui-gallery-avatar-group-sm"),
                    group(cx, shadcn::AvatarSize::Default)
                        .test_id("ui-gallery-avatar-group-default"),
                    group(cx, shadcn::AvatarSize::Lg).test_id("ui-gallery-avatar-group-lg"),
                ]
            },
        )
        .test_id("ui-gallery-avatar-group")
    };

    let (group_count_title, group_count) = doc_layout::gap_card(
        cx,
        "Avatar Group Count",
        "Upstream shadcn `AvatarGroupCount` renders a trailing count bubble (e.g. `+3`) that matches the group's size. Fret does not expose an equivalent shadcn builder yet.",
        "ui-gallery-avatar-group-count-gap",
    );

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
            DocSection::new(with_badge_title, with_badge)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"// Not yet implemented: upstream Avatar demo composes an `AvatarBadge` overlay.
// Track as a dedicated shadcn API surface (badge positioning + sizing + theming) before adding it."#,
                ),
            DocSection::new("Avatar Group", avatar_group)
                .description(
                    "Fret composes groups via negative margins (no `AvatarGroup` builder yet).",
                )
                .code(
                    "rust",
                    r#"let avatars = (0..3).map(|idx| {
    let mut avatar = shadcn::Avatar::new([/* ... */]).size(size).into_element(cx);
    if idx > 0 {
        avatar = avatar.refine_layout(LayoutRefinement::default().ml_neg(Space::N2));
    }
    avatar
});

cx.flex(FlexProps::default(), |_cx| avatars.collect::<Vec<_>>());"#,
                ),
            DocSection::new(group_count_title, group_count)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"// Not yet implemented: upstream shadcn exposes `AvatarGroupCount` (e.g. `+3`).
// Current Fret demo uses manual composition; add a proper shadcn builder once sizing tokens are settled."#,
                ),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-avatar")]
}
