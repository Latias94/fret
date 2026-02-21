use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_scroll_area(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = {
        let tags: Vec<Arc<str>> = (1..=50)
            .map(|idx| Arc::<str>::from(format!("v1.2.0-beta.{}", 51 - idx)))
            .collect();

        let content = {
            let content = ui::container(cx, move |cx| {
                let heading = ui::text(cx, "Tags")
                    .text_size_px(Px(12.0))
                    .line_height_px(Px(12.0))
                    .font_medium()
                    .into_element(cx);

                let tags_list = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N0)
                        .layout(LayoutRefinement::default().w_full()),
                    move |cx| {
                        let mut out: Vec<AnyElement> = Vec::with_capacity(tags.len() * 2);
                        for tag in tags {
                            out.push(
                                ui::text(cx, tag)
                                    .text_size_px(Px(12.0))
                                    .line_height_px(Px(16.0))
                                    .into_element(cx),
                            );
                            out.push(
                                shadcn::Separator::new()
                                    .refine_layout(
                                        LayoutRefinement::default().w_full().my(Space::N2),
                                    )
                                    .into_element(cx),
                            );
                        }
                        out
                    },
                );

                vec![stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N4)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full()),
                    |_cx| vec![heading, tags_list],
                )]
            })
            .p_4()
            .into_element(cx);

            content
        };

        let area = shadcn::ScrollArea::new([content])
            .axis(fret_ui::element::ScrollAxis::Y)
            .refine_layout(LayoutRefinement::default().w_full().h_full())
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-demo"),
            );

        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Md),
                LayoutRefinement::default().w_px(Px(192.0)).h_px(Px(288.0)),
            )
        });
        cx.container(props, move |_cx| [area])
    };

    let horizontal = {
        let rail = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_px(Px(760.0))),
            |cx| {
                let artists = ["Ornella Binni", "Tom Byrom", "Vladimir Malyavko"];
                artists
                    .iter()
                    .map(|artist| {
                        let art = shadcn::Skeleton::new()
                            .refine_style(ChromeRefinement::default().rounded(Radius::Md))
                            .refine_layout(
                                LayoutRefinement::default().w_px(Px(150.0)).h_px(Px(200.0)),
                            )
                            .into_element(cx);

                        let caption = shadcn::typography::muted(cx, format!("Photo by {artist}"));

                        stack::vstack(
                            cx,
                            stack::VStackProps::default()
                                .gap(Space::N2)
                                .items_start()
                                .layout(LayoutRefinement::default().flex_none()),
                            |_cx| vec![art, caption],
                        )
                    })
                    .collect::<Vec<_>>()
            },
        );

        let area = shadcn::ScrollArea::new([rail])
            .axis(fret_ui::element::ScrollAxis::X)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-scroll-area-horizontal"),
            );

        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(384.0)),
            )
        });
        cx.container(props, move |_cx| [area])
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
            .refine_layout(LayoutRefinement::default().w_full().h_full())
            .into_element(cx)
    })
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-scroll-area-rtl"),
    );

    let rtl = {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Md),
                LayoutRefinement::default().w_px(Px(192.0)).h_px(Px(288.0)),
            )
        });
        cx.container(props, move |_cx| [rtl])
    };

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
        Some("Preview follows shadcn ScrollArea demo: Vertical + Horizontal."),
        vec![
            DocSection::new("Demo", demo)
                .description("Vertical scroll region with tags and separators.")
                .code(
                    "rust",
                    r#"shadcn::ScrollArea::new([content])
    .axis(fret_ui::element::ScrollAxis::Y)
    .refine_layout(LayoutRefinement::default().w_full().h_full())
    .into_element(cx);"#,
                ),
            DocSection::new("Horizontal", horizontal)
                .description("Horizontal rail (fixed-size items) inside a scroll area.")
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
