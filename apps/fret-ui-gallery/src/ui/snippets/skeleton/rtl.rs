pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::element::SemanticsDecoration;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn round<H: UiHost>(size: f32) -> impl IntoUiElement<H> + use<H> {
    shadcn::Skeleton::new()
        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
        .refine_layout(
            LayoutRefinement::default()
                .w_px(Px(size))
                .h_px(Px(size))
                .flex_shrink_0(),
        )
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let text_lines = ui::v_stack(|cx| {
            vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(16.0)))
                    .into_element(cx),
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(200.0)).h_px(Px(16.0)))
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .layout(LayoutRefinement::default().w_px(Px(250.0)))
        .into_element(cx);

        ui::h_row(|cx| vec![round(48.0).into_element(cx), text_lines])
            .gap(Space::N4)
            .items_center()
            .into_element(cx)
    })
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-skeleton-rtl"),
    )
}
// endregion: example
