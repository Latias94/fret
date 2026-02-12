use super::super::super::super::*;

pub(in crate::ui) fn preview_intro(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    let card = |cx: &mut ElementContext<'_, App>, title: &str, desc: &str| -> AnyElement {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                .into_element(cx),
            shadcn::CardContent::new(vec![ui::text_block(cx, desc).into_element(cx)])
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
        .into_element(cx)
    };

    let grid = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        |cx| {
            vec![
                card(
                    cx,
                    "Core",
                    "Window / event / UiTree / renderer contracts (mechanisms & boundaries)",
                ),
                card(
                    cx,
                    "UI Kit",
                    "Headless interaction policies: focus trap, dismiss, hover intent, etc.",
                ),
                card(
                    cx,
                    "Shadcn",
                    "Visual recipes: composed defaults built on the Kit layer",
                ),
            ]
        },
    );
    let grid = grid.attach_semantics(
        SemanticsDecoration::default()
            .label("Debug:ui-gallery:intro:preview-grid")
            .test_id("ui-gallery-intro-preview-grid"),
    );

    let note = {
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .rounded(Radius::Md)
                .p(Space::N4),
            LayoutRefinement::default().w_full().min_w_0(),
        );
        cx.container(props, |cx| {
            vec![ui::text_block(cx, "Phase 1: fixed two-pane layout + hardcoded docs strings (focus on validating component usability). Docking/multi-window views will come later.").into_element(cx)]
        })
    };
    let note = note.attach_semantics(
        SemanticsDecoration::default()
            .label("Debug:ui-gallery:intro:preview-note")
            .test_id("ui-gallery-intro-preview-note"),
    );

    vec![grid, note]
}
