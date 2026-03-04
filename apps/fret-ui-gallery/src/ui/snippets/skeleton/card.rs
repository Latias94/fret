pub const SOURCE: &str = include_str!("card.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_fraction(2.0 / 3.0))
                .into_element(cx),
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_fraction(0.5))
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_full().aspect_ratio(1.0))
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-skeleton-card"),
    )
}
// endregion: example
