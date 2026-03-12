pub const SOURCE: &str = include_str!("vertical.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::h_flex(|cx| {
        vec![
            shadcn::raw::typography::small(cx, "Blog"),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .flex_stretch_cross_axis(true)
                .into_element(cx),
            shadcn::raw::typography::small(cx, "Docs"),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .flex_stretch_cross_axis(true)
                .into_element(cx),
            shadcn::raw::typography::small(cx, "Source"),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(
        LayoutRefinement::default()
            .w_full()
            .h_px(Px(20.0))
            .min_w_0(),
    )
    .into_element(cx)
    .test_id("ui-gallery-separator-vertical")
}
// endregion: example
