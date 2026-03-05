pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let header = ui::v_stack(|cx| {
        vec![
            shadcn::typography::small(cx, "Tailwind CSS"),
            shadcn::typography::muted(cx, "A utility-first CSS framework."),
        ]
    })
    .gap(Space::N1)
    .items_start()
    .into_element(cx);

    let links = ui::h_flex(|cx| {
        vec![
            cx.text("Blog"),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .flex_stretch_cross_axis(true)
                .into_element(cx),
            cx.text("Docs"),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .flex_stretch_cross_axis(true)
                .into_element(cx),
            cx.text("Source"),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full().h_px(Px(20.0)))
    .into_element(cx)
    .test_id("ui-gallery-separator-links");

    let separator = shadcn::Separator::new()
        .refine_layout(LayoutRefinement::default().w_full().my(Space::N4))
        .into_element(cx);

    ui::v_flex(|_cx| vec![header, separator, links])
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
        .into_element(cx)
        .test_id("ui-gallery-separator-demo")
}
// endregion: example
