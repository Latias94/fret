use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_avatar(
    cx: &mut ElementContext<'_, App>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let demo = {
        let a = {
            let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
            let fallback = shadcn::AvatarFallback::new("FR")
                .when_image_missing_model(avatar_image.clone())
                .delay_ms(120)
                .into_element(cx);
            shadcn::Avatar::new([image, fallback]).into_element(cx)
        };

        let b = shadcn::Avatar::new([shadcn::AvatarFallback::new("WK").into_element(cx)])
            .into_element(cx);

        let c = shadcn::Avatar::new([shadcn::AvatarFallback::new("?").into_element(cx)])
            .refine_layout(LayoutRefinement::default().w_px(Px(48.0)).h_px(Px(48.0)))
            .into_element(cx);

        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N3).items_center(),
            |_cx| [a, b, c],
        )
        .test_id("ui-gallery-avatar-demo")
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
        Some("Avatar composes image + fallback and keeps size/shape consistent."),
        vec![
            DocSection::new("Demo", demo)
                .description("Image-backed avatar with fallbacks.")
                .code(
                    "rust",
                    r#"let image = shadcn::AvatarImage::model(model).into_element(cx);
let fallback = shadcn::AvatarFallback::new("FR")
    .when_image_missing_model(model)
    .delay_ms(120)
    .into_element(cx);

shadcn::Avatar::new([image, fallback]).into_element(cx);"#,
                ),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-avatar")]
}
