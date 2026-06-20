use super::super::super::super::*;
use fret::AppComponentCx;
use fret::app::AppRenderActionsExt;
use fret_ui_kit::declarative::ModelWatchExt as _;

fn data_table_torture_cell_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    fret_ui_kit::declarative::text::text_table_cell(cx, text)
}

pub(in crate::ui) fn preview_data_table_torture(
    cx: &mut AppComponentCx<'_>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_headless::table::{ColumnDef, RowKey, SortSpec};

    let variable_height = std::env::var_os("FRET_UI_GALLERY_DATA_TABLE_VARIABLE_HEIGHT")
        .filter(|v| !v.is_empty())
        .is_some();
    let keep_alive: usize = std::env::var("FRET_UI_GALLERY_DATA_TABLE_KEEP_ALIVE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let overscan: usize = std::env::var("FRET_UI_GALLERY_DATA_TABLE_OVERSCAN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    #[derive(Debug, Clone)]
    struct Row {
        id: u64,
        name: Arc<str>,
        status: Arc<str>,
        cpu: u64,
        mem_mb: u64,
    }

    let (data, columns) = cx.slot_state(
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

    let reset_epoch = cx.local_model_keyed("reset_epoch", || 0_u64);
    let reset_epoch_value = cx.watch_model(&reset_epoch).layout().copied().unwrap_or(0);

    cx.keyed(("data_table_torture_reset_epoch", reset_epoch_value), |cx| {
        let state = cx.local_model_keyed("state", || {
            let mut state_value = fret_ui_headless::table::TableState::default();
            state_value.pagination.page_size = data.len();
            state_value.pagination.page_index = 0;
            state_value
        });
        let reset_state = state.clone();
        let reset_epoch = reset_epoch.clone();
        let reset_page_size = data.len();

    let state_snapshot = cx.watch_model(&state).layout().cloned_or_default();

    let sorting: Vec<SortSpec> = state_snapshot.sorting.clone();
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
        let pinning = state_snapshot.column_pinning.clone();
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
        let global_filter = state_snapshot.global_filter.clone();
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
        let value = state_snapshot
            .column_filters
            .iter()
            .find(|f| f.column.as_ref() == "name")
            .map(|f| f.value.clone());
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
        let value = state_snapshot
            .column_filters
            .iter()
            .find(|f| f.column.as_ref() == "status")
            .map(|f| f.value.clone());
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
        .test_id_prefix("ui-gallery-data-table-torture-toolbar")
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

    let header = ui::v_flex(move |cx| {
            vec![
                doc_layout::paragraph_text(cx, "Goal: baseline perf harness for a virtualized business table (TanStack-aligned headless engine + VirtualList)."),
                doc_layout::paragraph_text(cx, "Use scripted scroll + bundle stats to validate cache-root reuse and prepaint-driven windowing refactors."),
                doc_layout::control_readout_text(cx, sorting_text.clone()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(sorting_text.clone())
                        .test_id("ui-gallery-data-table-torture-sorting"),
                ),
                doc_layout::control_readout_text(cx, pinning_text.clone()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(pinning_text.clone())
                        .test_id("ui-gallery-data-table-torture-pinning"),
                ),
                doc_layout::control_readout_text(cx, global_filter_text.clone()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(global_filter_text.clone())
                        .test_id("ui-gallery-data-table-torture-global-filter"),
                ),
                doc_layout::control_readout_text(cx, name_filter_text.clone()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(name_filter_text.clone())
                        .test_id("ui-gallery-data-table-torture-name-filter"),
                ),
                doc_layout::control_readout_text(cx, status_filter_text.clone()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(status_filter_text.clone())
                        .test_id("ui-gallery-data-table-torture-status-filter"),
                ),
                shadcn::Button::new("Reset harness")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-data-table-torture-reset-state")
                    .on_activate(cx.actions().listen({
                        let reset_state = reset_state.clone();
                        let reset_epoch = reset_epoch.clone();
                        move |host, _action_cx| {
                            let _ = host.models_mut().update(&reset_state, |st| {
                                *st = fret_ui_headless::table::TableState::default();
                                st.pagination.page_size = reset_page_size;
                                st.pagination.page_index = 0;
                            });
                            let _ = host
                                .models_mut()
                                .update(&reset_epoch, |epoch| *epoch = epoch.wrapping_add(1));
                        }
                    }))
                    .into_element(cx),
                toolbar.into_element(cx),
            ]
        })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2).into_element(cx);

    let state_for_table = state.clone();
    let table = cx.cached_subtree_with(
        CachedSubtreeProps::default().contain_layout_when_bounds_known(true),
        |cx| {
            let retained = std::env::var_os("FRET_UI_GALLERY_DATA_TABLE_RETAINED").is_some();
            let data_table = if retained {
                let mut t = shadcn::DataTable::new();
                if keep_alive > 0 {
                    t = t.keep_alive(keep_alive);
                }
                t.overscan(overscan)
                    .row_height(Px(32.0))
                    .measure_rows(variable_height)
                    .column_actions_menu(true)
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(420.0)))
                    .debug_ids(fret_ui_kit::declarative::table::TableDebugIds {
                        header_row_test_id: Some(Arc::<str>::from(
                            "ui-gallery-data-table-header-row",
                        )),
                        header_cell_test_id_prefix: Some(Arc::<str>::from(
                            "ui-gallery-data-table-header-",
                        )),
                        row_test_id_prefix: Some(Arc::<str>::from("ui-gallery-data-table-row-")),
                        row_cell_test_ids: false,
                        ..Default::default()
                    })
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
                                    ui::v_stack(|cx| {
                                        vec![
                                            data_table_torture_cell_text(cx, row.name.clone()),
                                            doc_layout::control_readout_text(cx, format!(
                                                "Details: id={} cpu={} mem={}",
                                                row.id, row.cpu, row.mem_mb
                                            )),
                                        ]
                                    })
                                    .gap(Space::N0)
                                    .into_element(cx)
                                } else {
                                    data_table_torture_cell_text(cx, row.name.clone())
                                }
                            }
                            "status" => data_table_torture_cell_text(cx, row.status.clone()),
                            "cpu%" => data_table_torture_cell_text(cx, format!("{}%", row.cpu)),
                            "mem_mb" => {
                                data_table_torture_cell_text(cx, format!("{} MB", row.mem_mb))
                            }
                            _ => data_table_torture_cell_text(cx, "?"),
                        },
                    )
            } else {
                let mut t = shadcn::DataTable::new();
                if keep_alive > 0 {
                    t = t.keep_alive(keep_alive);
                }
                t.overscan(overscan)
                    .row_height(Px(32.0))
                    .measure_rows(variable_height)
                    .column_actions_menu(true)
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(420.0)))
                    .debug_ids(fret_ui_kit::declarative::table::TableDebugIds {
                        header_row_test_id: Some(Arc::<str>::from(
                            "ui-gallery-data-table-header-row",
                        )),
                        header_cell_test_id_prefix: Some(Arc::<str>::from(
                            "ui-gallery-data-table-header-",
                        )),
                        row_test_id_prefix: Some(Arc::<str>::from("ui-gallery-data-table-row-")),
                        row_cell_test_ids: false,
                        ..Default::default()
                    })
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
                                    ui::v_stack(|cx| {
                                        vec![
                                            data_table_torture_cell_text(cx, row.name.clone()),
                                            doc_layout::control_readout_text(cx, format!(
                                                "Details: id={} cpu={} mem={}",
                                                row.id, row.cpu, row.mem_mb
                                            )),
                                        ]
                                    })
                                    .gap(Space::N0)
                                    .into_element(cx)
                                } else {
                                    data_table_torture_cell_text(cx, row.name.clone())
                                }
                            }
                            "status" => data_table_torture_cell_text(cx, row.status.clone()),
                            "cpu%" => data_table_torture_cell_text(cx, format!("{}%", row.cpu)),
                            "mem_mb" => {
                                data_table_torture_cell_text(cx, format!("{} MB", row.mem_mb))
                            }
                            _ => data_table_torture_cell_text(cx, "?"),
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
        },
    );

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full().h_px(Px(460.0)),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

        vec![header, cx.container(container_props, |_cx| vec![table])]
    })
}
