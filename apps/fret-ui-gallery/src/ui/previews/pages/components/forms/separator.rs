use super::super::super::super::super::*;

pub(in crate::ui) fn preview_separator(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    let header = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N1).items_start(),
        |cx| {
            vec![
                shadcn::typography::small(cx, "Tailwind CSS"),
                shadcn::typography::muted(cx, "A utility-first CSS framework."),
            ]
        },
    );

    let links = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .items_center()
            .layout(LayoutRefinement::default().w_full().h_px(Px(20.0))),
        |cx| {
            vec![
                cx.text("Blog"),
                shadcn::Separator::new()
                    .orientation(shadcn::SeparatorOrientation::Vertical)
                    .flex_stretch_cross_axis(true)
                    .into_element(cx),
                cx.text("Docs"),
                shadcn::Separator::new()
                    .orientation(shadcn::SeparatorOrientation::Vertical)
                    .flex_stretch_cross_axis(true)
                    .into_element(cx),
                cx.text("Source"),
            ]
        },
    )
    .test_id("ui-gallery-separator-links");

    let separator = shadcn::Separator::new()
        .refine_layout(LayoutRefinement::default().w_full().my(Space::N4))
        .into_element(cx);

    let demo = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(520.0))),
        |_cx| vec![header, separator, links],
    )
    .test_id("ui-gallery-separator-demo");

    let notes = doc_layout::notes(cx, ["Preview follows shadcn Separator demo (new-york-v4)."]);

    let body = doc_layout::render_doc_page(
        cx,
        Some("Visually or semantically separates content."),
        vec![
            DocSection::new("Demo", demo).code(
                "rust",
                r#"let header = stack::vstack(cx, props, |cx| vec![
    shadcn::typography::small(cx, "Tailwind CSS"),
    shadcn::typography::muted(cx, "A utility-first CSS framework."),
]);

let links = stack::hstack(cx, props, |cx| vec![
    cx.text("Blog"),
    shadcn::Separator::new()
        .orientation(shadcn::SeparatorOrientation::Vertical)
        .into_element(cx),
    cx.text("Docs"),
    shadcn::Separator::new()
        .orientation(shadcn::SeparatorOrientation::Vertical)
        .into_element(cx),
    cx.text("Source"),
]);

stack::vstack(cx, props, |cx| vec![
    header,
    shadcn::Separator::new().into_element(cx),
    links,
]);"#,
            ),
            DocSection::new("Notes", notes).no_shell(),
        ],
    );

    vec![body]
}
