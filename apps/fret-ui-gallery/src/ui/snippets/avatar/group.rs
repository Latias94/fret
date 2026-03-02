pub const SOURCE: &str = include_str!("group.rs");

// region: example
use fret_core::ImageId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn avatar_with_image<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
    size: shadcn::AvatarSize,
    fallback_text: &'static str,
) -> AnyElement {
    let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
    let fallback = shadcn::AvatarFallback::new(fallback_text)
        .when_image_missing_model(avatar_image)
        .delay_ms(120)
        .into_element(cx);
    shadcn::Avatar::new([image, fallback])
        .size(size)
        .into_element(cx)
}

fn group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
    size: shadcn::AvatarSize,
    test_id: &'static str,
) -> AnyElement {
    let avatars = (0..3)
        .map(|idx| {
            avatar_with_image(cx, avatar_image.clone(), size, ["CN", "ML", "ER"][idx])
                .test_id(format!("ui-gallery-avatar-group-item-{test_id}-{idx}"))
        })
        .collect::<Vec<_>>();

    shadcn::AvatarGroup::new(avatars)
        .size(size)
        .into_element(cx)
        .test_id(test_id)
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            vec![
                group(
                    cx,
                    avatar_image.clone(),
                    shadcn::AvatarSize::Sm,
                    "ui-gallery-avatar-group-sm",
                ),
                group(
                    cx,
                    avatar_image.clone(),
                    shadcn::AvatarSize::Default,
                    "ui-gallery-avatar-group-default",
                ),
                group(
                    cx,
                    avatar_image.clone(),
                    shadcn::AvatarSize::Lg,
                    "ui-gallery-avatar-group-lg",
                ),
            ]
        },
    )
    .test_id("ui-gallery-avatar-group")
}
// endregion: example
