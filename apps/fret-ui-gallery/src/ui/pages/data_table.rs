use super::super::*;

pub(super) fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(900.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let preview_hint = shadcn::typography::muted(
        cx,
        "shadcn Data Table is a guide recipe (TanStack + Table primitives). This page renders a guide-aligned demo backed by Fret's headless engine.",
    );

    let demo = {
        let legacy_content = super::super::preview_data_table_legacy(cx, state);
        let live_stack = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| legacy_content,
        );
        section_card(
            cx,
            "Guide Demo",
            live_stack.test_id("ui-gallery-data-table-guide-demo"),
        )
    };

    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![preview_hint, demo],
    );
    let component_panel = shell(cx, component_stack).test_id("ui-gallery-data-table-component");

    let code_block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, snippet: &'static str| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                    .into_element(cx),
                shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        };

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                code_block(
                    cx,
                    "Basic Table",
                    r#"let table = shadcn::DataTable::new()
    .row_height(Px(36.0))
    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(280.0)))
    .into_element(cx, data, 1, state, columns, row_key, col_key, render_cell, render_header);"#,
                ),
                code_block(
                    cx,
                    "State + Sorting",
                    r#"let selected_count = models.read(&state, |st| st.row_selection.len())?;
let sorting = models.read(&state, |st| st.sorting.first().cloned())?;

// show selection/sorting summaries in a deterministic status row"#,
                ),
                code_block(
                    cx,
                    "Docs Gap Markers",
                    r#"section_card("Filtering", Alert::new([...]))
section_card("Visibility", Alert::new([...]))
// keep unsupported guide sections explicit so parity work is traceable"#,
                ),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

    let notes_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "Data Table in shadcn is a guide recipe, not a single fixed widget; treat this page as a living parity surface.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Prefer small, explicit recipe surfaces (toolbar/pagination/column header) that can be reused by apps and gated by diag scripts.",
                ),
                shadcn::typography::muted(
                    cx,
                    "When extending this page, prefer deterministic state rows and stable test IDs so diag scripts can gate regressions.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Future refactor can split column/header/view-options into reusable subcomponents mirroring upstream guide chapters.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-data-table",
        component_panel,
        code_panel,
        notes_panel,
    )
}
