pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::ImageId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

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

fn avatar_with_badge<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
    fallback_text: &'static str,
) -> AnyElement {
    let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
    let fallback = shadcn::AvatarFallback::new(fallback_text)
        .when_image_missing_model(avatar_image)
        .delay_ms(120)
        .into_element(cx);
    let badge = shadcn::AvatarBadge::new().into_element(cx);

    shadcn::Avatar::new([image, fallback, badge])
        .size(shadcn::AvatarSize::Default)
        .into_element(cx)
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
) -> AnyElement {
    let group = {
        let avatars = ["CN", "ML", "ER"]
            .into_iter()
            .map(|fallback| {
                avatar_with_image(
                    cx,
                    avatar_image.clone(),
                    shadcn::AvatarSize::Default,
                    fallback,
                )
            })
            .collect::<Vec<_>>();
        let count =
            shadcn::AvatarGroupCount::new([ui::text("+3").font_medium().nowrap().into_element(cx)])
                .into_element(cx);

        shadcn::AvatarGroup::new(avatars.into_iter().chain([count]).collect::<Vec<_>>())
            .size(shadcn::AvatarSize::Default)
            .into_element(cx)
            .test_id("ui-gallery-avatar-demo-group")
    };

    fret_ui_kit::ui::h_flex(|cx| {
        vec![
            avatar_with_image(cx, avatar_image.clone(), shadcn::AvatarSize::Default, "CN")
                .test_id("ui-gallery-avatar-demo-basic"),
            avatar_with_badge(cx, avatar_image, "ER").test_id("ui-gallery-avatar-demo-badge"),
            group,
        ]
    })
    .gap(Space::N12)
    .wrap()
    .w_full()
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-avatar-demo")
}
// endregion: example
