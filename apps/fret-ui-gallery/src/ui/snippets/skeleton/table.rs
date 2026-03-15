pub const SOURCE: &str = include_str!("table.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::element::SemanticsDecoration;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn row<H: UiHost>() -> impl IntoUiElement<H> + use<H> {
    ui::h_flex(|cx| {
        vec![
            shadcn::Skeleton::new()
                .refine_layout(
                    LayoutRefinement::default()
                        .flex_1()
                        .h_px(Px(16.0))
                        .min_w_0(),
                )
                .into_element(cx),
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_px(Px(96.0)).h_px(Px(16.0)))
                .into_element(cx),
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_px(Px(80.0)).h_px(Px(16.0)))
                .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full())
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| (0..3).map(|_| row().into_element(cx)).collect::<Vec<_>>())
        .gap(Space::N2)
        .layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-table"),
        )
}
// endregion: example
