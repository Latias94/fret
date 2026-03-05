use super::super::super::super::*;
use crate::ui::doc_layout::{self, DocSection};

pub(in crate::ui) fn preview_intro(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    let card = |cx: &mut ElementContext<'_, App>, title: &str, desc: &str| -> AnyElement {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new(title).into_element(cx),
                shadcn::CardDescription::new(desc).into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
        .into_element(cx)
    };

    let grid = ui::h_flex(|cx| {
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
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N4)
    .items_stretch()
    .into_element(cx);
    let grid = grid.attach_semantics(
        SemanticsDecoration::default()
            .label("Debug:ui-gallery:intro:preview-grid")
            .test_id("ui-gallery-intro-preview-grid"),
    );

    let note = {
        let text_color = ColorRef::Color(theme.color_token("muted-foreground"));
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_token("muted")))
                .rounded(Radius::Md)
                .p(Space::N4),
            LayoutRefinement::default().w_full().min_w_0(),
        );
        cx.container(props, |cx| {
            vec![ui::text_block( "Phase 1: fixed two-pane layout + hardcoded docs strings (focus on validating component usability). Docking/multi-window views will come later.")
                .text_color(text_color)
                .into_element(cx)]
        })
    };
    let note = note.attach_semantics(
        SemanticsDecoration::default()
            .label("Debug:ui-gallery:intro:preview-note")
            .test_id("ui-gallery-intro-preview-note"),
    );

    let preview = ui::v_flex(|_cx| vec![grid, note])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N4)
        .items_start()
        .into_element(cx);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Pick a page from the sidebar to explore UI contracts and composed recipes."),
        vec![
            DocSection::new("Overview", preview)
                .no_shell()
                .max_w(Px(980.0)),
        ],
    );

    vec![page]
}
