pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let horizontal = ui::v_flex(|cx| {
        vec![
            ui::v_stack(|cx| {
                vec![
                    shadcn::raw::typography::small("Radix Primitives").into_element(cx),
                    shadcn::raw::typography::muted("An open-source UI component library.")
                        .into_element(cx),
                ]
            })
            .gap(Space::N1)
            .items_start()
            .into_element(cx),
            shadcn::Separator::new().into_element(cx),
            shadcn::raw::typography::small(
                "A set of low-level UI primitives that power higher-level component systems.",
            )
            .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let vertical = ui::h_flex(|cx| {
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
    .layout(LayoutRefinement::default().h_px(Px(20.0)).min_w_0())
    .into_element(cx);

    ui::v_flex(|_cx| vec![horizontal, vertical])
        .gap(Space::N6)
        .items_start()
        .layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(384.0))
                .min_w_0(),
        )
        .into_element(cx)
        .test_id("ui-gallery-separator-demo")
}
// endregion: example
