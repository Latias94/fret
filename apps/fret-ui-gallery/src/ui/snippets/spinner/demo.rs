pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui::element::{CrossAlign, MainAlign};
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let item = shadcn::Item::new([
        shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)]).into_element(cx),
        shadcn::ItemContent::new(
            [shadcn::ItemTitle::new("Processing payment...").into_element(cx)],
        )
        .into_element(cx),
        shadcn::ItemContent::new([ui::text("$100.00")
            .text_sm()
            .tabular_nums()
            .into_element(cx)])
        .refine_layout(LayoutRefinement::default().flex_none())
        .justify(MainAlign::End)
        .align(CrossAlign::End)
        .into_element(cx),
    ])
    .variant(shadcn::ItemVariant::Muted)
    .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        |_cx| vec![item],
    )
    .test_id("ui-gallery-spinner-demo")
}
// endregion: example
