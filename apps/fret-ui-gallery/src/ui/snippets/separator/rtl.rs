pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let header = ui::v_stack(|cx| {
            vec![
                shadcn::typography::small(cx, "shadcn/ui"),
                shadcn::typography::muted(cx, "الأساس لنظام التصميم الخاص بك"),
            ]
        })
        .gap(Space::N1p5)
        .items_start()
        .into_element(cx);

        let separator = shadcn::Separator::new()
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        let description = shadcn::typography::small(
            cx,
            "مجموعة من المكونات المصممة بشكل جميل يمكنك تخصيصها وتوسيعها والبناء عليها.",
        );

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
