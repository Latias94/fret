pub const SOURCE: &str = include_str!("text.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
        vec![
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_full().h_px(Px(16.0)))
                .into_element(cx),
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_full().h_px(Px(16.0)))
                .into_element(cx),
            shadcn::Skeleton::new()
                .refine_layout(
                    LayoutRefinement::default()
                        .w_fraction(3.0 / 4.0)
                        .h_px(Px(16.0)),
                )
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-skeleton-text"),
    )
}
// endregion: example
