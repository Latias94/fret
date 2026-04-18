pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let header = ui::v_stack(|cx| {
            vec![
                shadcn::raw::typography::small("shadcn/ui").into_element(cx),
                shadcn::raw::typography::muted("الأساس لنظام التصميم الخاص بك").into_element(cx),
            ]
        })
        .gap(Space::N1p5)
        .items_start()
        .into_element(cx);

        let separator = shadcn::Separator::new()
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        let description = shadcn::raw::typography::small(
            "مجموعة من المكونات المصممة بشكل جميل يمكنك تخصيصها وتوسيعها والبناء عليها.",
        )
        .into_element(cx);

        ui::v_flex(|_cx| vec![header, separator, description])
            .gap(Space::N4)
            .items_start()
            .layout(
                LayoutRefinement::default()
                    .w_full()
                    .max_w(Px(384.0))
                    .min_w_0(),
            )
            .into_element(cx)
            .test_id("ui-gallery-separator-rtl")
    })
}
// endregion: example
