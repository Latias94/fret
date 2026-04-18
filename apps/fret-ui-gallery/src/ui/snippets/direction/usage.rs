pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl).into_element(cx, |cx| {
        ui::v_stack(|cx| {
            vec![
                shadcn::Badge::new("RTL subtree")
                    .variant(shadcn::BadgeVariant::Outline)
                    .into_element(cx)
                    .test_id("ui-gallery-direction-usage-badge"),
                ui::v_stack(|cx| {
                    vec![
                        cx.text("هذه المجموعة كلها تقرأ من اتجاه RTL.")
                            .test_id("ui-gallery-direction-usage-copy"),
                        ui::h_flex(|cx| {
                            vec![
                                shadcn::Button::new("التالي")
                                    .test_id("ui-gallery-direction-usage-next")
                                    .into_element(cx),
                                shadcn::Button::new("السابق")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .test_id("ui-gallery-direction-usage-back")
                                    .into_element(cx),
                            ]
                        })
                        .gap(Space::N2)
                        .wrap()
                        .items_center()
                        .layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    ]
                })
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx),
            ]
        })
        .gap(Space::N3)
        .items_start()
        .layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(480.0))
                .min_w_0(),
        )
        .into_element(cx)
        .test_id("ui-gallery-direction-usage")
    })
}
// endregion: example
