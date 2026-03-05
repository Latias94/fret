use super::super::super::super::*;
use crate::ui::doc_layout::{self, DocSection};

pub(in crate::ui) fn preview_layout(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    let boxy = |cx: &mut ElementContext<'_, App>, label: &str, color: fret_core::Color| {
        cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(color))
                    .rounded(Radius::Md)
                    .p(Space::N3),
                // In a horizontal flex row, we want "equal columns" semantics (`flex-1`), not
                // `w-full` (percent sizing). Percent sizing is fragile under intrinsic sizing
                // probes and can cause transient wrap widths (0px) to leak into final layout.
                LayoutRefinement::default().flex_1().min_w_0(),
            ),
            |cx| vec![ui::label(label).into_element(cx)],
        )
    };

    let row = ui::h_flex(|cx| {
        vec![
            boxy(cx, "Left (fill)", theme.color_token("accent")),
            boxy(cx, "Center (fill)", theme.color_token("muted")),
            boxy(cx, "Right (fill)", theme.color_token("card")),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N3)
    .items_stretch()
    .into_element(cx);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Layout mental model: LayoutRefinement (constraints) + stack (composition) + Theme tokens (color/spacing)."),
        vec![DocSection::new("Demo", row)
            .description("In a horizontal flex row, prefer `flex-1 + min-w-0` (equal columns) over percent widths (`w-full`).")],
    );

    vec![page]
}
