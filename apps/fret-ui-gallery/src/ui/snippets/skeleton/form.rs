pub const SOURCE: &str = include_str!("form.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, label_w: Px) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .layout(LayoutRefinement::default().w_full()),
        move |cx| {
            vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(label_w).h_px(Px(16.0)))
                    .into_element(cx),
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(40.0)))
                    .into_element(cx),
            ]
        },
    )
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
        |cx| {
            vec![
                row(cx, Px(80.0)),
                row(cx, Px(96.0)),
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(96.0)).h_px(Px(36.0)))
                    .into_element(cx),
            ]
        },
    )
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-skeleton-form"),
    )
}
// endregion: example
