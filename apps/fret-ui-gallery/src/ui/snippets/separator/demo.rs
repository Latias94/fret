pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![
            ui::v_stack(|cx| {
                vec![
                    shadcn::raw::typography::small("shadcn/ui").into_element(cx),
                    shadcn::raw::typography::muted("The Foundation for your Design System")
                        .into_element(cx),
                ]
            })
            .gap(Space::N1p5)
            .items_start()
            .into_element(cx),
            shadcn::Separator::new().into_element(cx),
            shadcn::raw::typography::small(
                "A set of beautifully designed components that you can customize, extend, and build on.",
            )
            .into_element(cx),
        ]
    })
    .gap(Space::N4)
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
