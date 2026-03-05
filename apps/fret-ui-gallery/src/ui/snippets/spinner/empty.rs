pub const SOURCE: &str = include_str!("empty.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Empty::new([
        shadcn::empty::EmptyHeader::new([
            shadcn::empty::EmptyMedia::new([shadcn::Spinner::new().into_element(cx)])
                .variant(shadcn::empty::EmptyMediaVariant::Icon)
                .into_element(cx),
            shadcn::empty::EmptyTitle::new("Processing your request").into_element(cx),
            shadcn::empty::EmptyDescription::new(
                "Please wait while we process your request. Do not refresh the page.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::empty::EmptyContent::new([shadcn::Button::new("Cancel")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-spinner-empty")
}

// endregion: example
