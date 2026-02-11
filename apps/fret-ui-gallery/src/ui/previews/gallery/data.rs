use super::super::super::*;

#[derive(Debug, Clone)]
struct DemoProcessRow {
    id: u64,
    name: Arc<str>,
    status: Arc<str>,
    cpu: u64,
    mem_mb: u64,
}

#[derive(Debug, Clone)]
struct DemoProcessTableAssets {
    data: Arc<[DemoProcessRow]>,
    columns: Arc<[fret_ui_headless::table::ColumnDef<DemoProcessRow>]>,
}

pub(in crate::ui) fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    pages::preview_data_table(cx, state)
}

pub(in crate::ui) fn preview_data_table_legacy(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    let assets = cx.with_state(
        || {
            let data: Arc<[DemoProcessRow]> = Arc::from(vec![
                DemoProcessRow {
                    id: 1,
                    name: Arc::from("Renderer"),
                    status: Arc::from("Running"),
                    cpu: 12,
                    mem_mb: 420,
                },
                DemoProcessRow {
                    id: 2,
                    name: Arc::from("Asset Cache"),
                    status: Arc::from("Idle"),
                    cpu: 0,
                    mem_mb: 128,
                },
                DemoProcessRow {
                    id: 3,
                    name: Arc::from("Indexer"),
                    status: Arc::from("Running"),
                    cpu: 38,
                    mem_mb: 860,
                },
                DemoProcessRow {
                    id: 4,
                    name: Arc::from("Spellcheck"),
                    status: Arc::from("Disabled"),
                    cpu: 0,
                    mem_mb: 0,
                },
                DemoProcessRow {
                    id: 5,
                    name: Arc::from("Language Server"),
                    status: Arc::from("Running"),
                    cpu: 7,
                    mem_mb: 512,
                },
            ]);

            let columns: Arc<[fret_ui_headless::table::ColumnDef<DemoProcessRow>]> =
                Arc::from(vec![
                    fret_ui_headless::table::ColumnDef::new("name")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.name.cmp(&b.name))
                        .size(220.0),
                    fret_ui_headless::table::ColumnDef::new("status")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.status.cmp(&b.status))
                        .size(140.0),
                    fret_ui_headless::table::ColumnDef::new("cpu%")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.cpu.cmp(&b.cpu))
                        .size(90.0),
                    fret_ui_headless::table::ColumnDef::new("mem_mb")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.mem_mb.cmp(&b.mem_mb))
                        .size(110.0),
                ]);

            DemoProcessTableAssets { data, columns }
        },
        |st| st.clone(),
    );

    let selected_count = cx
        .app
        .models()
        .read(&state, |st| st.row_selection.len())
        .ok()
        .unwrap_or(0);
    let sorting = cx
        .app
        .models()
        .read(&state, |st| {
            st.sorting.first().map(|s| (s.column.clone(), s.desc))
        })
        .ok()
        .flatten();

    let sorting_text: Arc<str> = sorting
        .map(|(col, desc)| {
            Arc::<str>::from(format!(
                "Sorting: {} {}",
                col,
                if desc { "desc" } else { "asc" }
            ))
        })
        .unwrap_or_else(|| Arc::<str>::from("Sorting: <none>"));

    let normalize_col_id =
        |id: &str| -> Arc<str> { Arc::<str>::from(id.replace('%', "pct").replace('_', "-")) };

    let toolbar = shadcn::DataTableToolbar::new(
        state.clone(),
        assets.columns.clone(),
        |col: &fret_ui_headless::table::ColumnDef<DemoProcessRow>| col.id.clone(),
    )
    .into_element(cx);

    let table = shadcn::DataTable::new()
        .row_height(Px(36.0))
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(280.0)))
        .into_element(
            cx,
            assets.data.clone(),
            1,
            state.clone(),
            assets.columns.clone(),
            |row, _index, _parent| fret_ui_headless::table::RowKey(row.id),
            |col| col.id.clone(),
            move |cx, col, row| {
                let col_id = normalize_col_id(col.id.as_ref());
                let cell = match col.id.as_ref() {
                    "name" => cx.text(row.name.as_ref()),
                    "status" => cx.text(row.status.as_ref()),
                    "cpu%" => cx.text(format!("{}%", row.cpu)),
                    "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                    _ => cx.text("?"),
                };

                cell.test_id(Arc::<str>::from(format!(
                    "ui-gallery-data-table-cell-{}-{}",
                    row.id, col_id
                )))
            },
        );

    let table = table.test_id("ui-gallery-data-table-root");

    vec![
        cx.text("Click header to sort; click row to toggle selection."),
        cx.text(format!("Selected rows: {selected_count}")),
        cx.text(sorting_text.as_ref()),
        toolbar,
        table,
    ]
}

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

pub(in crate::ui) fn preview_tree_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use std::collections::HashSet;

    use fret_ui_kit::TreeItem;
    use fret_ui_kit::TreeState;

    let variable_height = std::env::var_os("FRET_UI_GALLERY_TREE_VARIABLE_HEIGHT")
        .filter(|v| !v.is_empty())
        .is_some();

    #[derive(Default)]
    struct TreeTortureModels {
        items: Option<Model<Vec<TreeItem>>>,
        state: Option<Model<TreeState>>,
    }

    let (items, state) = cx.with_state(TreeTortureModels::default, |st| {
        (st.items.clone(), st.state.clone())
    });
    let (items, state) = match (items, state) {
        (Some(items), Some(state)) => (items, state),
        _ => {
            let (items_value, state_value) = {
                let root_count = 200u64;
                let folders_per_root = 10u64;
                let leaves_per_folder = 25u64;

                let mut expanded: HashSet<u64> = HashSet::new();
                let mut roots: Vec<TreeItem> = Vec::with_capacity(root_count as usize);

                for r in 0..root_count {
                    let root_id = r;
                    expanded.insert(root_id);

                    let mut folders: Vec<TreeItem> = Vec::with_capacity(folders_per_root as usize);
                    for f in 0..folders_per_root {
                        let folder_id = 1_000_000 + r * 100 + f;
                        expanded.insert(folder_id);

                        let mut leaves: Vec<TreeItem> =
                            Vec::with_capacity(leaves_per_folder as usize);
                        for l in 0..leaves_per_folder {
                            let leaf_id = 2_000_000 + r * 10_000 + f * 100 + l;
                            let label = if variable_height && leaf_id % 15 == 0 {
                                format!(
                                    "Leaf {r}/{f}/{l} (id={leaf_id})\nDetails: id={} seed={}",
                                    leaf_id,
                                    leaf_id.wrapping_mul(2654435761)
                                )
                            } else {
                                format!("Leaf {r}/{f}/{l} (id={leaf_id})")
                            };
                            leaves.push(TreeItem::new(leaf_id, label).disabled(leaf_id % 97 == 0));
                        }

                        folders.push(
                            TreeItem::new(folder_id, format!("Folder {r}/{f}")).children(leaves),
                        );
                    }

                    roots.push(TreeItem::new(root_id, format!("Root {r}")).children(folders));
                }

                (
                    roots,
                    TreeState {
                        selected: None,
                        expanded,
                    },
                )
            };

            let items = cx.app.models_mut().insert(items_value);
            let state = cx.app.models_mut().insert(state_value);
            cx.with_state(TreeTortureModels::default, |st| {
                st.items = Some(items.clone());
                st.state = Some(state.clone());
            });
            (items, state)
        }
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline perf harness for a virtualized tree (expand/collapse + selection + scroll)."),
                cx.text("Use scripted scroll + bundle stats to validate cache-root reuse and prepaint-driven windowing refactors."),
            ]
        },
    );

    let tree = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        let retained = std::env::var_os("FRET_UI_GALLERY_TREE_RETAINED")
            .filter(|v| !v.is_empty())
            .is_some();

        let tree = if retained {
            if variable_height {
                fret_ui_kit::declarative::tree::tree_view_retained_with_measure_mode(
                    cx,
                    items,
                    state,
                    fret_ui_kit::Size::Medium,
                    fret_ui::element::VirtualListMeasureMode::Measured,
                    Some(Arc::<str>::from("ui-gallery-tree-row")),
                )
            } else {
                fret_ui_kit::declarative::tree::tree_view_retained(
                    cx,
                    items,
                    state,
                    fret_ui_kit::Size::Medium,
                    Some(Arc::<str>::from("ui-gallery-tree-row")),
                )
            }
        } else {
            fret_ui_kit::declarative::tree::tree_view(cx, items, state, fret_ui_kit::Size::Medium)
        };

        vec![
            tree.attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-tree-torture-root"),
            ),
        ]
    });

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full().h_px(Px(460.0)),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

    vec![header, cx.container(container_props, |_cx| vec![tree])]
}

pub(in crate::ui) fn preview_data_grid(
    cx: &mut ElementContext<'_, App>,
    selected_row: Model<Option<u64>>,
) -> Vec<AnyElement> {
    let selected = cx
        .get_model_copied(&selected_row, Invalidation::Paint)
        .flatten();

    let selected_text: Arc<str> = selected
        .map(|v| Arc::<str>::from(v.to_string()))
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let grid = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        let selected = cx
            .get_model_copied(&selected_row, Invalidation::Layout)
            .flatten();

        let grid = shadcn::experimental::DataGridElement::new(
            ["PID", "Name", "State", "CPU%"],
            DATA_GRID_ROWS,
        )
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(320.0)))
        .into_element(
            cx,
            1,
            1,
            |row| row as u64,
            move |row| {
                let is_selected = selected == Some(row as u64);
                let cmd = data_grid_row_command(row).unwrap_or_else(|| {
                    // Fallback for out-of-range row IDs.
                    CommandId::new(format!("{CMD_DATA_GRID_ROW_PREFIX}{row}"))
                });
                shadcn::experimental::DataGridRowState {
                    selected: is_selected,
                    enabled: row % 17 != 0,
                    on_click: Some(cmd),
                }
            },
            |cx, row, col| {
                let pid = 1000 + row as u64;
                match col {
                    0 => cx.text(pid.to_string()),
                    1 => cx.text(format!("Process {row}")),
                    2 => cx.text(if row % 3 == 0 { "Running" } else { "Idle" }),
                    _ => cx.text(((row * 7) % 100).to_string()),
                }
            },
        );

        vec![grid]
    });

    vec![
        cx.text("Virtualized rows/cols viewport; click a row to select (disabled every 17th row)."),
        cx.text(format!("Selected row: {selected_text}")),
        grid,
    ]
}
