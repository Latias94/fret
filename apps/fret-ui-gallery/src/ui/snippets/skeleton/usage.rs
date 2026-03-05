pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::h_row(|cx| {
        vec![
            shadcn::Skeleton::new()
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .refine_layout(
                    LayoutRefinement::default()
                        .h_px(Px(20.0))
                        .w_px(Px(100.0))
                        .flex_shrink_0(),
                )
                .into_element(cx),
        ]
    })
    .items_center()
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-skeleton-usage"),
    )
}
// endregion: example
