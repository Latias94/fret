use super::super::super::super::*;

pub(in crate::ui) fn preview_data_table_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    _state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    use fret_ui_headless::table::{ColumnDef, RowKey, SortSpec};

    let variable_height = std::env::var_os("FRET_UI_GALLERY_DATA_TABLE_VARIABLE_HEIGHT")
        .filter(|v| !v.is_empty())
        .is_some();
    let keep_alive: usize = std::env::var("FRET_UI_GALLERY_DATA_TABLE_KEEP_ALIVE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    #[derive(Debug, Clone)]
    struct Row {
        id: u64,
        name: Arc<str>,
        status: Arc<str>,
        cpu: u64,
        mem_mb: u64,
    }

    let (data, columns) = cx.with_state(
        || {
            let mut rows: Vec<Row> = Vec::with_capacity(50_000);
            for i in 0..50_000u64 {
                let status = match i % 4 {
                    0 => "Running",
                    1 => "Idle",
                    2 => "Sleeping",
                    _ => "Blocked",
                };
                rows.push(Row {
                    id: i,
                    name: Arc::from(format!("Process {i}")),
                    status: Arc::from(status),
                    cpu: (i * 7) % 100,
                    mem_mb: 32 + ((i * 13) % 4096),
                });
            }

            let columns: Arc<[ColumnDef<Row>]> = Arc::from(vec![
                ColumnDef::new("name")
                    .sort_by(|a: &Row, b: &Row| a.name.cmp(&b.name))
                    .filter_by(|row: &Row, q| row.name.as_ref().contains(q))
                    .size(220.0),
                ColumnDef::new("status")
                    .sort_by(|a: &Row, b: &Row| a.status.cmp(&b.status))
                    .filter_by_with_meta(|row: &Row, value: &serde_json::Value, _add_meta| {
                        match value {
                            serde_json::Value::String(s) => row.status.as_ref() == s,
                            serde_json::Value::Array(items) => items
                                .iter()
                                .filter_map(|it| it.as_str())
                                .any(|s| row.status.as_ref() == s),
                            _ => false,
                        }
                    })
                    .facet_key_by(|row: &Row| match row.status.as_ref() {
                        "Running" => 1,
                        "Idle" => 2,
                        "Sleeping" => 3,
                        "Blocked" => 4,
                        _ => 0,
                    })
                    .facet_str_by(|row: &Row| row.status.as_ref())
                    .size(140.0),
                ColumnDef::new("cpu%")
                    .sort_by(|a: &Row, b: &Row| a.cpu.cmp(&b.cpu))
                    .size(90.0),
                ColumnDef::new("mem_mb")
                    .sort_by(|a: &Row, b: &Row| a.mem_mb.cmp(&b.mem_mb))
                    .size(110.0),
            ]);

            (Arc::<[Row]>::from(rows), columns)
        },
        |(data, columns)| (data.clone(), columns.clone()),
    );

    #[derive(Default)]
    struct DataTableTortureModels {
        state: Option<Model<fret_ui_headless::table::TableState>>,
    }

    let state = cx.with_state(DataTableTortureModels::default, |st| st.state.clone());
    let state = match state {
        Some(state) => state,
        None => {
            let mut state_value = fret_ui_headless::table::TableState::default();
            state_value.pagination.page_size = data.len();
            state_value.pagination.page_index = 0;
            let state = cx.app.models_mut().insert(state_value);
            cx.with_state(DataTableTortureModels::default, |st| {
                st.state = Some(state.clone());
            });
            state
        }
    };

    let sorting: Vec<SortSpec> = cx
        .app
        .models()
        .read(&state, |st| st.sorting.clone())
        .ok()
        .unwrap_or_default();
    let sorting_text: Arc<str> = if sorting.is_empty() {
        Arc::<str>::from("Sorting: <none>")
    } else {
        let parts: Vec<String> = sorting
            .iter()
            .map(|s| format!("{} {}", s.column, if s.desc { "desc" } else { "asc" }))
            .collect();
        Arc::<str>::from(format!("Sorting: {}", parts.join(", ")))
    };

    let pinning_text: Arc<str> = {
        let pinning = cx
            .app
            .models()
            .read(&state, |st| st.column_pinning.clone())
            .ok()
            .unwrap_or_default();
        if pinning.left.is_empty() && pinning.right.is_empty() {
            Arc::<str>::from("Pinning: <none>")
        } else {
            let left = pinning
                .left
                .iter()
                .map(|v| v.as_ref().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let right = pinning
                .right
                .iter()
                .map(|v| v.as_ref().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Arc::<str>::from(format!("Pinning: left=[{left}] right=[{right}]"))
        }
    };

    let global_filter_text: Arc<str> = {
        let global_filter = cx
            .app
            .models()
            .read(&state, |st| st.global_filter.clone())
            .ok()
            .flatten();
        match global_filter {
            None => Arc::<str>::from("GlobalFilter: <none>"),
            Some(v) => {
                if let Some(s) = v.as_str() {
                    Arc::<str>::from(format!("GlobalFilter: {s}"))
                } else {
                    Arc::<str>::from(format!("GlobalFilter: {v}"))
                }
            }
        }
    };

    let name_filter_text: Arc<str> = {
        let value = cx
            .app
            .models()
            .read(&state, |st| {
                st.column_filters
                    .iter()
                    .find(|f| f.column.as_ref() == "name")
                    .map(|f| f.value.clone())
            })
            .ok()
            .flatten();
        match value {
            None => Arc::<str>::from("NameFilter: <none>"),
            Some(v) => {
                if let Some(s) = v.as_str() {
                    Arc::<str>::from(format!("NameFilter: {s}"))
                } else {
                    Arc::<str>::from(format!("NameFilter: {v}"))
                }
            }
        }
    };

    let status_filter_text: Arc<str> = {
        let value = cx
            .app
            .models()
            .read(&state, |st| {
                st.column_filters
                    .iter()
                    .find(|f| f.column.as_ref() == "status")
                    .map(|f| f.value.clone())
            })
            .ok()
            .flatten();
        match value {
            None => Arc::<str>::from("StatusFilter: <none>"),
            Some(serde_json::Value::String(s)) => Arc::<str>::from(format!("StatusFilter: {s}")),
            Some(serde_json::Value::Array(items)) => {
                let parts: Vec<&str> = items.iter().filter_map(|it| it.as_str()).collect();
                if parts.is_empty() {
                    Arc::<str>::from("StatusFilter: <none>")
                } else {
                    Arc::<str>::from(format!("StatusFilter: {}", parts.join(", ")))
                }
            }
            Some(v) => Arc::<str>::from(format!("StatusFilter: {v}")),
        }
    };

    let toolbar_columns = columns.clone();
    let toolbar =
        shadcn::DataTableToolbar::new(state.clone(), toolbar_columns, |col: &ColumnDef<Row>| {
            Arc::<str>::from(col.id.as_ref())
        })
        .column_filter("name")
        .column_filter_placeholder("Filter name...")
        .column_filter_a11y_label("Name filter")
        .faceted_filter(
            "status",
            "Status",
            Arc::<[Arc<str>]>::from(vec![
                Arc::<str>::from("Running"),
                Arc::<str>::from("Idle"),
                Arc::<str>::from("Sleeping"),
                Arc::<str>::from("Blocked"),
            ]),
        );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline perf harness for a virtualized business table (TanStack-aligned headless engine + VirtualList)."),
                cx.text("Use scripted scroll + bundle stats to validate cache-root reuse and prepaint-driven windowing refactors."),
                cx.text(sorting_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(sorting_text.clone())
                        .test_id("ui-gallery-data-table-torture-sorting"),
                ),
                cx.text(pinning_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(pinning_text.clone())
                        .test_id("ui-gallery-data-table-torture-pinning"),
                ),
                cx.text(global_filter_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(global_filter_text.clone())
                        .test_id("ui-gallery-data-table-torture-global-filter"),
                ),
                cx.text(name_filter_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(name_filter_text.clone())
                        .test_id("ui-gallery-data-table-torture-name-filter"),
                ),
                cx.text(status_filter_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(status_filter_text.clone())
                        .test_id("ui-gallery-data-table-torture-status-filter"),
                ),
                toolbar.clone().into_element(cx),
            ]
        },
    );

    let state_for_table = state.clone();
    let table =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let retained = std::env::var_os("FRET_UI_GALLERY_DATA_TABLE_RETAINED").is_some();
            let data_table = if retained {
                let mut t = shadcn::DataTable::new();
                if keep_alive > 0 {
                    t = t.keep_alive(keep_alive);
                }
                t.overscan(10)
                    .row_height(Px(28.0))
                    .measure_rows(variable_height)
                    .column_actions_menu(true)
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(420.0)))
                    .into_element_retained(
                        cx,
                        data.clone(),
                        1,
                        state_for_table.clone(),
                        columns.clone(),
                        |row, _index, _parent| RowKey(row.id),
                        |col| Arc::<str>::from(col.id.as_ref()),
                        move |cx, col, row| match col.id.as_ref() {
                            "name" => {
                                if variable_height && row.id % 15 == 0 {
                                    stack::vstack(
                                        cx,
                                        stack::VStackProps::default().gap(Space::N0),
                                        |cx| {
                                            vec![
                                                cx.text(row.name.as_ref()),
                                                cx.text(format!(
                                                    "Details: id={} cpu={} mem={}",
                                                    row.id, row.cpu, row.mem_mb
                                                )),
                                            ]
                                        },
                                    )
                                } else {
                                    cx.text(row.name.as_ref())
                                }
                            }
                            "status" => cx.text(row.status.as_ref()),
                            "cpu%" => cx.text(format!("{}%", row.cpu)),
                            "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                            _ => cx.text("?"),
                        },
                        Some(Arc::<str>::from("ui-gallery-data-table-header-")),
                        Some(Arc::<str>::from("ui-gallery-data-table-row-")),
                    )
            } else {
                let mut t = shadcn::DataTable::new();
                if keep_alive > 0 {
                    t = t.keep_alive(keep_alive);
                }
                t.overscan(10)
                    .row_height(Px(28.0))
                    .measure_rows(variable_height)
                    .column_actions_menu(true)
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(420.0)))
                    .into_element(
                        cx,
                        data.clone(),
                        1,
                        state,
                        columns.clone(),
                        |row, _index, _parent| RowKey(row.id),
                        |col| Arc::<str>::from(col.id.as_ref()),
                        move |cx, col, row| match col.id.as_ref() {
                            "name" => {
                                if variable_height && row.id % 15 == 0 {
                                    stack::vstack(
                                        cx,
                                        stack::VStackProps::default().gap(Space::N0),
                                        |cx| {
                                            vec![
                                                cx.text(row.name.as_ref()),
                                                cx.text(format!(
                                                    "Details: id={} cpu={} mem={}",
                                                    row.id, row.cpu, row.mem_mb
                                                )),
                                            ]
                                        },
                                    )
                                } else {
                                    cx.text(row.name.as_ref())
                                }
                            }
                            "status" => cx.text(row.status.as_ref()),
                            "cpu%" => cx.text(format!("{}%", row.cpu)),
                            "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                            _ => cx.text("?"),
                        },
                    )
            };

            vec![
                data_table.attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Group)
                        .test_id("ui-gallery-data-table-torture-root"),
                ),
            ]
        });

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full().h_px(Px(460.0)),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

    vec![header, cx.container(container_props, |_cx| vec![table])]
}
