pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::element::{CrossAlign, MainAlign};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let item = shadcn::Item::new([
        shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)]).into_element(cx),
        shadcn::ItemContent::new(
            [shadcn::ItemTitle::new("Processing payment...").into_element(cx)],
        )
        .into_element(cx),
        shadcn::ItemContent::new([decl_text::text_control_readout(cx, "$100.00")])
            .refine_layout(LayoutRefinement::default().flex_none())
            .justify(MainAlign::End)
            .align(CrossAlign::End)
            .into_element(cx),
    ])
    .variant(shadcn::ItemVariant::Muted)
    .into_element(cx);

    ui::v_flex(|_cx| vec![item])
        .gap(Space::N4)
        .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-demo")
}
// endregion: example
