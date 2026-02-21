use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_scroll_area(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = {
        let versions: Vec<Arc<str>> = (1..=50)
            .map(|idx| Arc::<str>::from(format!("v1.2.0-beta.{:02}", 51 - idx)))
            .collect();

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                let mut rows: Vec<AnyElement> = Vec::with_capacity(versions.len() * 2 + 1);
                rows.push(shadcn::typography::small(cx, "Tags"));
                for tag in versions {
                    rows.push(cx.text(tag));
                    rows.push(
                        shadcn::Separator::new()
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                    );
                }
                rows
            },
        );

        shadcn::ScrollArea::new([content])
            .axis(fret_ui::element::ScrollAxis::Y)
            .refine_layout(LayoutRefinement::default().w_px(Px(192.0)).h_px(Px(288.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-demo"),
            )
    };

    let horizontal = {
        let rail = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(760.0))),
            |cx| {
                let artists = [
                    "Ornella Binni",
                    "Tom Byrom",
                    "Vladimir Malyavko",
                    "Silvia Serra",
                ];
                artists
                    .iter()
                    .map(|artist| {
                        shadcn::Card::new(vec![
                            shadcn::CardContent::new(vec![
                                shadcn::Skeleton::new()
                                    .refine_style(ChromeRefinement::default().rounded(Radius::Md))
                                    .refine_layout(
                                        LayoutRefinement::default().w_px(Px(140.0)).h_px(Px(180.0)),
                                    )
                                    .into_element(cx),
                                shadcn::typography::muted(cx, format!("Photo by {artist}")),
                            ])
                            .into_element(cx),
                        ])
                        .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                        .into_element(cx)
                    })
                    .collect::<Vec<_>>()
            },
        );

        shadcn::ScrollArea::new([rail])
            .axis(fret_ui::element::ScrollAxis::X)
            .refine_layout(LayoutRefinement::default().w_px(Px(384.0)).h_px(Px(280.0)))
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-horizontal"),
            )
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                let mut rows: Vec<AnyElement> = vec![shadcn::typography::small(cx, "العلامات")];
                for idx in 1..=40 {
                    rows.push(cx.text(format!("v1.2.0-beta.{:02}", 41 - idx)));
                    rows.push(
                        shadcn::Separator::new()
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                    );
                }
                rows
            },
        );

        shadcn::ScrollArea::new([content])
            .axis(fret_ui::element::ScrollAxis::Y)
            .refine_layout(LayoutRefinement::default().w_px(Px(192.0)).h_px(Px(288.0)))
            .into_element(cx)
    })
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-scroll-area-rtl"),
    );

    let notes = doc_layout::notes(
        cx,
        [
            "ScrollArea is for custom scrollbars + consistent styling; use native scrolling when you don't need custom chrome.",
            "Keep scroll region sizes explicit in docs to avoid layout drift.",
            "Horizontal rails are easiest to reason about when the child has a fixed width.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Scrollable region with custom scrollbars and nested content."),
        vec![
            DocSection::new("Demo", demo)
                .description("Vertical scroll region with separators.")
                .code(
                    "rust",
                    r#"shadcn::ScrollArea::new([content])
    .axis(fret_ui::element::ScrollAxis::Y)
    .refine_layout(LayoutRefinement::default().w_px(Px(192.0)).h_px(Px(288.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("Horizontal", horizontal)
                .description("Horizontal rail inside a scroll area.")
                .code(
                    "rust",
                    r#"shadcn::ScrollArea::new([rail])
    .axis(fret_ui::element::ScrollAxis::X)
    .into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("ScrollArea behavior under an RTL direction provider.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| shadcn::ScrollArea::new([content]).into_element(cx),
);"#,
                ),
            DocSection::new("Notes", notes).description("Usage notes and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-scroll-area")]
}
