// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let avatar_media = shadcn::Avatar::new([shadcn::AvatarFallback::new("JD").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_px(Px(48.0)).h_px(Px(48.0)))
        .into_element(cx);

    shadcn::Empty::new([
        shadcn::empty::EmptyHeader::new([
            shadcn::empty::EmptyMedia::new([avatar_media]).into_element(cx),
            shadcn::empty::EmptyTitle::new("User Offline").into_element(cx),
            shadcn::empty::EmptyDescription::new(
                "This user is currently offline. Leave a message and notify later.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::empty::EmptyContent::new([shadcn::Button::new("Leave Message")
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx)])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
    .into_element(cx)
    .test_id("ui-gallery-empty-avatar")
}
// endregion: example
