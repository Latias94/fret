pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let avatar = shadcn::Skeleton::new()
        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
        .refine_layout(
            LayoutRefinement::default()
                .w_px(Px(48.0))
                .h_px(Px(48.0))
                .flex_shrink_0(),
        )
        .into_element(cx);

    let text_lines = ui::v_stack(|cx| {
        vec![
            shadcn::Skeleton::new()
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .h_px(Px(16.0))
                        .min_w_0(),
                )
                .into_element(cx),
            shadcn::Skeleton::new()
                .refine_layout(LayoutRefinement::default().w_px(Px(200.0)).h_px(Px(16.0)))
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .layout(LayoutRefinement::default().w_px(Px(250.0)))
    .into_element(cx);

    ui::h_row(|_cx| vec![avatar, text_lines])
        .gap(Space::N4)
        .items_center()
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-demo"),
        )
}
// endregion: example
