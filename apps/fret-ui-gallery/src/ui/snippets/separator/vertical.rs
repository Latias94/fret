pub const SOURCE: &str = include_str!("vertical.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::h_flex(|cx| {
        vec![
            shadcn::raw::typography::small("Blog").into_element(cx),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .into_element(cx),
            shadcn::raw::typography::small("Docs").into_element(cx),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .into_element(cx),
            shadcn::raw::typography::small("Source").into_element(cx),
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
