use super::super::*;

pub(super) fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).clone();

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
        cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(900.0)),
            ),
            move |_cx| [body],
        )
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let gap_card = |cx: &mut ElementContext<'_, App>,
                    title: &'static str,
                    details: &'static str,
                    test_id: &'static str| {
        let alert_content = shadcn::Alert::new([
            shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.info")),
            shadcn::AlertTitle::new("Guide-aligned placeholder").into_element(cx),
            shadcn::AlertDescription::new(details).into_element(cx),
        ])
        .variant(shadcn::AlertVariant::Default)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(760.0)))
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id(test_id));
        section_card(cx, title, alert_content)
    };

    let live_table = {
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
            "Basic Table",
            live_stack.attach_semantics(
                SemanticsDecoration::default().test_id("ui-gallery-data-table-basic"),
            ),
        )
    };

    let row_actions = gap_card(
        cx,
        "Row Actions",
        "Current gallery shows sortable/selectable process rows. Dedicated row-action menus are tracked as follow-up for strict docs parity.",
        "ui-gallery-data-table-row-actions-gap",
    );
    let pagination = gap_card(
        cx,
        "Pagination",
        "Guide expects pageable datasets and page-size controls. Current demo focuses on dense viewport rendering and row selection semantics.",
        "ui-gallery-data-table-pagination-gap",
    );
    let sorting = gap_card(
        cx,
        "Sorting",
        "Sorting is active in the live table section. Next iteration will expose explicit sortable header affordances matching docs screenshots.",
        "ui-gallery-data-table-sorting-gap",
    );
    let filtering = gap_card(
        cx,
        "Filtering",
        "Column/global filtering controls are not yet surfaced on this page. Keep this section explicit to avoid hidden parity drift.",
        "ui-gallery-data-table-filtering-gap",
    );
    let visibility = gap_card(
        cx,
        "Visibility",
        "Column visibility toggles (dropdown checkboxes) are documented upstream and planned for the next pass in this page module.",
        "ui-gallery-data-table-visibility-gap",
    );
    let row_selection = gap_card(
        cx,
        "Row Selection",
        "Row selection is active in the current demo state model; docs-style header checkbox UX will be aligned in follow-up.",
        "ui-gallery-data-table-row-selection-gap",
    );
    let reusable = gap_card(
        cx,
        "Reusable Components",
        "Upstream recommends extracting reusable column-header, pagination, and view-options components. We keep this reminder as architecture guidance.",
        "ui-gallery-data-table-reusable-gap",
    );
    let rtl = gap_card(
        cx,
        "RTL",
        "RTL coverage for data-table interactions is tracked; add an explicit RTL matrix once column visibility/filtering controls land.",
        "ui-gallery-data-table-rtl-gap",
    );

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Data Table guide order: Basic Table, Row Actions, Pagination, Sorting, Filtering, Visibility, Row Selection, Reusable Components, RTL.",
    );

    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                preview_hint,
                live_table,
                row_actions,
                pagination,
                sorting,
                filtering,
                visibility,
                row_selection,
                reusable,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).attach_semantics(
        SemanticsDecoration::default().test_id("ui-gallery-data-table-component"),
    );

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
                    "Data Table in shadcn is a guide recipe, not a single fixed widget; keep page structure aligned to guide milestones.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Preserve visible gap markers for missing milestones (pagination/filtering/visibility) instead of silently omitting them.",
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
