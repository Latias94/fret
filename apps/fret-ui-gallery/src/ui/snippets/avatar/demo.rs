pub const SOURCE: &str = include_str!("demo.rs");

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

fn avatar_with_image_rounded<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
    size: shadcn::AvatarSize,
    fallback_text: &'static str,
    chrome: ChromeRefinement,
) -> AnyElement {
    let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
    let fallback = shadcn::AvatarFallback::new(fallback_text)
        .when_image_missing_model(avatar_image)
        .delay_ms(120)
        .into_element(cx);

    shadcn::Avatar::new([image, fallback])
        .size(size)
        .refine_style(chrome)
        .into_element(cx)
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
) -> AnyElement {
    let rounded = avatar_with_image_rounded(
        cx,
        avatar_image.clone(),
        shadcn::AvatarSize::Default,
        "ER",
        ChromeRefinement::default().rounded(Radius::Lg),
    )
    .test_id("ui-gallery-avatar-demo-rounded");

    let group = fret_ui_kit::ui::h_flex(|cx| {
        (0..3)
            .map(|idx| {
                let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
                let fallback = shadcn::AvatarFallback::new(["CN", "ML", "ER"][idx])
                    .when_image_missing_model(avatar_image.clone())
                    .delay_ms(120)
                    .into_element(cx);
                let mut avatar =
                    shadcn::Avatar::new([image, fallback]).size(shadcn::AvatarSize::Default);
                if idx != 0 {
                    avatar = avatar.refine_layout(LayoutRefinement::default().ml_neg(Space::N2));
                }
                avatar
                    .into_element(cx)
                    .test_id(format!("ui-gallery-avatar-demo-group-item-{idx}"))
            })
            .collect::<Vec<_>>()
    })
    .gap(Space::N0)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-avatar-demo-group");

    fret_ui_kit::ui::h_flex(|cx| {
        vec![
            avatar_with_image(cx, avatar_image, shadcn::AvatarSize::Default, "CN")
                .test_id("ui-gallery-avatar-demo-round"),
            rounded,
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
