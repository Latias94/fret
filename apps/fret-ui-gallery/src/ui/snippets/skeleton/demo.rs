// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{self as shadcn, prelude::*};

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

fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(cx, children)
        .gap(Space::N4)
        .wrap()
        .w_full()
        .items_start()
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let avatar_row = {
        let text_lines = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_px(Px(150.0))),
            |cx| {
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
                        .refine_layout(LayoutRefinement::default().w_px(Px(100.0)).h_px(Px(16.0)))
                        .into_element(cx),
                ]
            },
        );

        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| vec![round(cx, 40.0), text_lines],
        )
    };

    let cards = {
        let card = |cx: &mut ElementContext<'_, H>, idx: usize| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(180.0)).h_px(Px(16.0)))
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(136.0)).h_px(Px(16.0)))
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::CardContent::new(vec![shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_full().aspect_ratio(1.0))
                    .into_element(cx)])
                .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)).min_w_0())
            .into_element(cx)
            .test_id(format!("ui-gallery-skeleton-demo-card-{idx}"))
        };

        wrap_row(cx, |cx| (1..=3).map(|idx| card(cx, idx)).collect())
    };

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |_cx| vec![avatar_row, cards],
    )
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-skeleton-demo"),
    )
}
// endregion: example

