use super::super::super::super::*;

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
            |cx| [ui::label(cx, label).w_full().into_element(cx)],
        )
    };

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_stretch(),
        |cx| {
            vec![
                boxy(cx, "Left (fill)", theme.color_required("accent")),
                boxy(cx, "Center (fill)", theme.color_required("muted")),
                boxy(cx, "Right (fill)", theme.color_required("card")),
            ]
        },
    );

    vec![
        ui::text_block(
            cx,
            "Layout mental model: LayoutRefinement (constraints) + stack (composition) + Theme tokens (color/spacing).",
        )
        .into_element(cx),
        row,
    ]
}
