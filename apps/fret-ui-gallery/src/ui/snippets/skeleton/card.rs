pub const SOURCE: &str = include_str!("card.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::Skeleton::new().refine_layout(
                        LayoutRefinement::default()
                            .w_fraction(2.0 / 3.0)
                            .h_px(Px(16.0)),
                    ),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_fraction(0.5).h_px(Px(16.0))),
                ]
            }),
            shadcn::card_content(|cx| {
                ui::children![
                    cx;
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full().aspect_ratio(1.0)),
                ]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-skeleton-card"),
    )
}
// endregion: example
