pub const SOURCE: &str = include_str!("form.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::element::SemanticsDecoration;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn row<H: UiHost>(label_w: Px) -> impl IntoUiElement<H> + use<H> {
    ui::v_flex(move |cx| {
        vec![
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_px(label_w).h_px(Px(16.0)))
                .into_element(cx),
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_full().h_px(Px(40.0)))
                .into_element(cx),
        ]
    })
    .gap(Space::N3)
    .layout(LayoutRefinement::default().w_full())
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![
            row(Px(80.0)).into_element(cx),
            row(Px(96.0)).into_element(cx),
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_px(Px(96.0)).h_px(Px(36.0)))
                .into_element(cx),
        ]
    })
    .gap(Space::N6)
    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-skeleton-form"),
    )
}
// endregion: example
