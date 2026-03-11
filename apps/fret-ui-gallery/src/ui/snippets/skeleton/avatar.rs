pub const SOURCE: &str = include_str!("avatar.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn round<H: UiHost>(cx: &mut ElementContext<'_, H>, size: f32) -> AnyElement {
    shadcn::Skeleton::new()
        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
        .refine_layout(
            LayoutRefinement::default()
                .w_px(Px(size))
                .h_px(Px(size))
                .flex_shrink_0(),
        )
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let text_lines = ui::v_stack(|cx| {
        vec![
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_full().h_px(Px(16.0)))
                .into_element(cx),
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_px(Px(100.0)).h_px(Px(16.0)))
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .layout(LayoutRefinement::default().w_px(Px(150.0)))
    .into_element(cx);

    ui::h_row(|cx| vec![round(cx, 40.0), text_lines])
        .gap(Space::N4)
        .items_center()
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-avatar"),
        )
}
// endregion: example
