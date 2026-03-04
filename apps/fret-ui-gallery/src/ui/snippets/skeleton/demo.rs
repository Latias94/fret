pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let avatar = shadcn::Skeleton::new()
        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
        .refine_layout(
            LayoutRefinement::default()
                .w_px(Px(48.0))
                .h_px(Px(48.0))
                .flex_shrink_0(),
        )
        .into_element(cx);

    let text_lines = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .layout(LayoutRefinement::default().w_px(Px(250.0))),
        |cx| {
            vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(200.0)))
                    .into_element(cx),
            ]
        },
    );

    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        |_cx| vec![avatar, text_lines],
    )
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-skeleton-demo"),
    )
}
// endregion: example
