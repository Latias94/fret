// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let header = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N1).items_start(),
        |cx| {
            vec![
                shadcn::typography::small(cx, "Tailwind CSS"),
                shadcn::typography::muted(cx, "A utility-first CSS framework."),
            ]
        },
    );

    let links = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .items_center()
            .layout(LayoutRefinement::default().w_full().h_px(Px(20.0))),
        |cx| {
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
        },
    )
    .test_id("ui-gallery-separator-links");

    let separator = shadcn::Separator::new()
        .refine_layout(LayoutRefinement::default().w_full().my(Space::N4))
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(520.0))),
        |_cx| vec![header, separator, links],
    )
    .test_id("ui-gallery-separator-demo")
}
// endregion: example
