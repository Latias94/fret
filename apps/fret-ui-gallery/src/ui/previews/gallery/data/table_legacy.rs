use super::super::super::super::*;

use std::collections::HashMap;
use std::sync::Arc;

use fret_runtime::{CommandId, Model};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_headless::table::{ColumnDef, RowKey, Table, TableState};

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

#[derive(Default)]
struct DemoProcessTableFacetsState {
    status_facets: Option<Model<HashMap<Arc<str>, usize>>>,
}

#[derive(Default)]
struct DemoProcessTableToolbarResponsiveState {
    use_container_query: Option<Model<bool>>,
}

const CMD_SELECT_ALL_PAGE: &str = "ui_gallery.data_table.select_all_page";
const TOGGLE_ROW_SELECTED_ROUTE_PREFIX: &str = "ui_gallery.data_table.toggle_row_selected.";

fn toggle_row_selected_command(row_id: u64) -> CommandId {
    CommandId::new(format!("{TOGGLE_ROW_SELECTED_ROUTE_PREFIX}{row_id}"))
}

fn wire_selection_commands<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    state: Model<TableState>,
    data: Arc<[DemoProcessRow]>,
    columns: Arc<[ColumnDef<DemoProcessRow>]>,
) {
    cx.command_on_command_for(
        cx.root_id(),
        Arc::new(move |host, acx, command| {
            let cmd = command.as_str();

            let current = host
                .models_mut()
                .read(&state, |st| st.clone())
                .ok()
                .unwrap_or_default();

            let table = Table::builder(data.as_ref())
                .columns(columns.as_ref().to_vec())
                .get_row_key(|row, _index, _parent| RowKey(row.id))
                .state(current)
                .build();

            let next = if cmd == CMD_SELECT_ALL_PAGE {
                table.toggled_all_page_rows_selected(None)
            } else if let Some(row_id) = cmd
                .strip_prefix(TOGGLE_ROW_SELECTED_ROUTE_PREFIX)
                .and_then(|suffix| suffix.parse::<u64>().ok())
            {
                table.toggled_row_selected(RowKey(row_id), None, true)
            } else {
                return false;
            };

            let _ = host.models_mut().update(&state, |st| {
                st.row_selection = next;
            });

            host.request_redraw(acx.window);
            true
        }),
    );
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
                    fret_ui_headless::table::ColumnDef::new("select")
                        .enable_sorting(false)
                        .enable_multi_sort(false)
                        .enable_column_filter(false)
                        .enable_global_filter(false)
                        .enable_hiding(false)
                        .enable_ordering(false)
                        .enable_pinning(false)
                        .enable_resizing(false)
                        .size(44.0)
                        .min_size(44.0)
                        .max_size(44.0),
                    fret_ui_headless::table::ColumnDef::new("name")
                        .filter_by(|row: &DemoProcessRow, q| row.name.as_ref().contains(q))
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.name.cmp(&b.name))
                        .size(220.0),
                    fret_ui_headless::table::ColumnDef::new("status")
                        .filter_by_with_meta(
                            |row: &DemoProcessRow, value: &serde_json::Value, _| match value {
                                serde_json::Value::String(s) => row.status.as_ref() == s,
                                serde_json::Value::Array(items) => items
                                    .iter()
                                    .filter_map(|it| it.as_str())
                                    .any(|s| row.status.as_ref() == s),
                                _ => false,
                            },
                        )
                        .facet_key_by(|row: &DemoProcessRow| match row.status.as_ref() {
                            "Running" => 1,
                            "Idle" => 2,
                            "Disabled" => 3,
                            _ => 0,
                        })
                        .facet_str_by(|row: &DemoProcessRow| row.status.as_ref())
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.status.cmp(&b.status))
                        .size(140.0),
                    fret_ui_headless::table::ColumnDef::new("cpu%")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.cpu.cmp(&b.cpu))
                        .size(90.0),
                    fret_ui_headless::table::ColumnDef::new("mem_mb")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.mem_mb.cmp(&b.mem_mb))
                        .size(110.0),
                    fret_ui_headless::table::ColumnDef::new("actions")
                        .enable_sorting(false)
                        .enable_multi_sort(false)
                        .enable_column_filter(false)
                        .enable_global_filter(false)
                        .enable_hiding(false)
                        .enable_ordering(false)
                        .enable_pinning(false)
                        .enable_resizing(false)
                        .size(60.0)
                        .min_size(60.0)
                        .max_size(60.0),
                ]);

            DemoProcessTableAssets { data, columns }
        },
        |st| st.clone(),
    );

    let output = cx.with_state(
        || None::<Model<fret_ui_kit::declarative::table::TableViewOutput>>,
        |st| st.clone(),
    );
    let output = match output {
        Some(m) => m,
        None => {
            let m = cx
                .app
                .models_mut()
                .insert(fret_ui_kit::declarative::table::TableViewOutput::default());
            let m_for_state = m.clone();
            cx.with_state(
                || None::<Model<fret_ui_kit::declarative::table::TableViewOutput>>,
                move |st| {
                    if st.is_none() {
                        *st = Some(m_for_state);
                    }
                },
            );
            m
        }
    };

    let normalize_col_id =
        |id: &str| -> Arc<str> { Arc::<str>::from(id.replace('%', "pct").replace('_', "-")) };

    let state_value = cx.watch_model(&state).layout().cloned().unwrap_or_default();

    wire_selection_commands(
        cx,
        state.clone(),
        assets.data.clone(),
        assets.columns.clone(),
    );

    let add_task = shadcn::Button::new("Add Task")
        .size(shadcn::ButtonSize::Sm)
        .test_id("ui-gallery-data-table-add-task")
        .into_element(cx);

    let status_facets = cx.with_state(DemoProcessTableFacetsState::default, |st| {
        st.status_facets.clone()
    });
    let status_facets = match status_facets {
        Some(m) => m,
        None => {
            let mut facets: HashMap<Arc<str>, usize> = HashMap::new();
            for row in assets.data.iter() {
                *facets.entry(row.status.clone()).or_insert(0) += 1;
            }
            let m = cx.app.models_mut().insert(facets);
            let m_for_state = m.clone();
            cx.with_state(DemoProcessTableFacetsState::default, move |st| {
                st.status_facets = Some(m_for_state);
            });
            m
        }
    };

    let use_container_query = cx
        .with_state(DemoProcessTableToolbarResponsiveState::default, |st| {
            st.use_container_query.clone()
        });
    let use_container_query = match use_container_query {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(false);
            let m_for_state = m.clone();
            cx.with_state(DemoProcessTableToolbarResponsiveState::default, move |st| {
                st.use_container_query = Some(m_for_state);
            });
            m
        }
    };

    let faceted_badges_query = if cx
        .watch_model(&use_container_query)
        .cloned()
        .unwrap_or(false)
    {
        shadcn::DataTableToolbarResponsiveQuery::Container
    } else {
        shadcn::DataTableToolbarResponsiveQuery::Viewport
    };

    let responsive_toggle = shadcn::FieldGroup::new([shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Toolbar responsive query")
                .for_control("ui-gallery-data-table-toolbar-responsive-query-switch")
                .into_element(cx),
            shadcn::FieldDescription::new(
                "Toggle to drive the faceted-filter badges by the toolbar container width (editor-first) instead of the window viewport width (web parity).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Switch::new(use_container_query.clone())
            .control_id("ui-gallery-data-table-toolbar-responsive-query-switch")
            .test_id("ui-gallery-data-table-toolbar-responsive-query-switch")
            .a11y_label("Toolbar responsive query uses container width")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .into_element(cx)])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(720.0)))
    .into_element(cx)
    .test_id("ui-gallery-data-table-toolbar-responsive-query");

    let toolbar = shadcn::DataTableToolbar::new(
        state.clone(),
        assets.columns.clone(),
        |col: &fret_ui_headless::table::ColumnDef<DemoProcessRow>| col.id.clone(),
    )
    .show_global_filter(false)
    .filter_layout(LayoutRefinement::default().h_px(Px(32.0)).w_px(Px(250.0)))
    .column_filter("name")
    .column_filter_placeholder("Filter processes...")
    .column_filter_a11y_label("Name filter")
    .faceted_filter_options(
        "status",
        "Status",
        Arc::<[shadcn::DataTableFacetedFilterOption]>::from(vec![
            shadcn::DataTableFacetedFilterOption::new("Running", "Running")
                .icon(fret_icons::IconId::new_static("lucide.timer")),
            shadcn::DataTableFacetedFilterOption::new("Idle", "Idle")
                .icon(fret_icons::IconId::new_static("lucide.circle")),
            shadcn::DataTableFacetedFilterOption::new("Disabled", "Disabled")
                .icon(fret_icons::IconId::new_static("lucide.circle-off")),
        ]),
    )
    .faceted_filter_counts(status_facets)
    .faceted_selected_badges_query(faceted_badges_query)
    .columns_button_label("View")
    .show_pinning_menu(false)
    .show_selected_text(false)
    .trailing([add_task])
    .into_element(cx);

    let state_for_header_checkbox = state.clone();
    let assets_for_header_checkbox = assets.clone();
    let table = shadcn::DataTable::new()
        .row_click_selection(false)
        .row_height(Px(40.0))
        .header_height(Px(40.0))
        .column_actions_menu(false)
        .output_model(output.clone())
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(280.0)))
        .into_element_with_header_cell(
            cx,
            assets.data.clone(),
            1,
            state.clone(),
            assets.columns.clone(),
            |row, _index, _parent| fret_ui_headless::table::RowKey(row.id),
            |col| col.id.clone(),
            move |cx, col, _sort_state| {
                if col.id.as_ref() != "select" {
                    return None;
                }

                let state_value = cx
                    .app
                    .models()
                    .read(&state_for_header_checkbox, |st| st.clone())
                    .ok()
                    .unwrap_or_default();
                let table = Table::builder(assets_for_header_checkbox.data.as_ref())
                    .columns(assets_for_header_checkbox.columns.as_ref().to_vec())
                    .get_row_key(|row, _index, _parent| RowKey(row.id))
                    .state(state_value)
                    .build();

                let checked = if table.is_all_page_rows_selected() {
                    Some(true)
                } else if table.is_some_page_rows_selected() {
                    None
                } else {
                    Some(false)
                };

                let model = cx.with_state(|| None::<Model<Option<bool>>>, |st| st.clone());
                let model = match model {
                    Some(m) => m,
                    None => {
                        let m = cx.app.models_mut().insert(checked);
                        let m_for_state = m.clone();
                        cx.with_state(
                            || None::<Model<Option<bool>>>,
                            move |st| {
                                if st.is_none() {
                                    *st = Some(m_for_state);
                                }
                            },
                        );
                        m
                    }
                };
                let _ = cx.app.models_mut().update(&model, |v| *v = checked);

                Some(vec![
                    shadcn::Checkbox::new_optional(model)
                        .a11y_label("Select all")
                        .test_id("ui-gallery-data-table-select-all")
                        .on_click(CommandId::new(CMD_SELECT_ALL_PAGE))
                        .into_element(cx),
                ])
            },
            move |cx, col, row| {
                let col_id = normalize_col_id(col.id.as_ref());
                let cell = match col.id.as_ref() {
                    "select" => {
                        let checked = state_value.row_selection.contains(&RowKey(row.id));
                        cx.keyed(("ui-gallery-data-table-select-row", row.id), |cx| {
                            let model = cx.with_state(|| None::<Model<bool>>, |st| st.clone());
                            let model = match model {
                                Some(m) => m,
                                None => {
                                    let m = cx.app.models_mut().insert(checked);
                                    let m_for_state = m.clone();
                                    cx.with_state(
                                        || None::<Model<bool>>,
                                        move |st| {
                                            if st.is_none() {
                                                *st = Some(m_for_state);
                                            }
                                        },
                                    );
                                    m
                                }
                            };
                            let _ = cx.app.models_mut().update(&model, |v| *v = checked);

                            shadcn::Checkbox::new(model)
                                .a11y_label("Select row")
                                .test_id(Arc::<str>::from(format!(
                                    "ui-gallery-data-table-select-row-{}",
                                    row.id
                                )))
                                .on_click(toggle_row_selected_command(row.id))
                                .into_element(cx)
                        })
                    }
                    "name" => cx.text(row.name.as_ref()),
                    "status" => cx.text(row.status.as_ref()),
                    "cpu%" => cx.text(format!("{}%", row.cpu)),
                    "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                    "actions" => cx.keyed(("ui-gallery-data-table-row-actions", row.id), |cx| {
                        let open = cx.with_state(|| None::<Model<bool>>, |st| st.clone());
                        let open = match open {
                            Some(m) => m,
                            None => {
                                let m = cx.app.models_mut().insert(false);
                                let m_for_state = m.clone();
                                cx.with_state(
                                    || None::<Model<bool>>,
                                    move |st| {
                                        if st.is_none() {
                                            *st = Some(m_for_state);
                                        }
                                    },
                                );
                                m
                            }
                        };

                        let trigger = shadcn::Button::new("")
                            .a11y_label("Open menu")
                            .variant(shadcn::ButtonVariant::Ghost)
                            .size(shadcn::ButtonSize::IconXs)
                            .test_id(Arc::<str>::from(format!(
                                "ui-gallery-data-table-row-actions-open-{}",
                                row.id
                            )))
                            .icon(fret_icons::IconId::new_static("lucide.ellipsis"))
                            .into_element(cx);

                            shadcn::DropdownMenu::new(open)
                                .align(shadcn::DropdownMenuAlign::End)
                                .side(shadcn::DropdownMenuSide::Bottom)
                                .into_element(
                                    cx,
                                    move |_cx| trigger,
                                    move |_cx| {
                                        vec![
                                            shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("Edit")
                                                .test_id(Arc::<str>::from(format!(
                                                    "ui-gallery-data-table-row-actions-item-edit-{}",
                                                    row.id
                                                )))
                                                .on_select(CommandId::new(
                                                    "ui_gallery.data_table.row_actions.edit",
                                                )),
                                        ),
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("Make a copy")
                                                .test_id(Arc::<str>::from(format!(
                                                    "ui-gallery-data-table-row-actions-item-copy-{}",
                                                    row.id
                                                )))
                                                .on_select(CommandId::new(
                                                    "ui_gallery.data_table.row_actions.copy",
                                                )),
                                        ),
                                        shadcn::DropdownMenuEntry::Separator,
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("Delete")
                                                .test_id(Arc::<str>::from(format!(
                                                    "ui-gallery-data-table-row-actions-item-delete-{}",
                                                    row.id
                                                )))
                                                .on_select(CommandId::new(
                                                    "ui_gallery.data_table.row_actions.delete",
                                                )),
                                        ),
                                    ]
                                },
                            )
                    }),
                    _ => cx.text("?"),
                };

                if col_id.as_ref() == "actions" {
                    cell
                } else {
                    cell.test_id(Arc::<str>::from(format!(
                        "ui-gallery-data-table-cell-{}-{}",
                        row.id, col_id
                    )))
                }
            },
        );

    let table = table.test_id("ui-gallery-data-table-root");

    #[derive(Default)]
    struct ListLikeDataTableModels {
        state: Option<Model<TableState>>,
    }

    let list_like_state = cx.with_state(ListLikeDataTableModels::default, |st| st.state.clone());
    let list_like_state = match list_like_state {
        Some(state) => state,
        None => {
            let mut state_value = TableState::default();
            state_value.pagination.page_size = assets.data.len();
            state_value.pagination.page_index = 0;
            let state = cx.app.models_mut().insert(state_value);
            cx.with_state(ListLikeDataTableModels::default, |st| {
                st.state = Some(state.clone());
            });
            state
        }
    };

    let list_like_columns: Arc<[ColumnDef<DemoProcessRow>]> = Arc::from(
        assets
            .columns
            .iter()
            .filter(|c| c.id.as_ref() != "select" && c.id.as_ref() != "actions")
            .cloned()
            .collect::<Vec<_>>(),
    );

    let list_like_table = shadcn::DataTable::new()
        .row_click_selection(true)
        .row_click_selection_policy(
            fret_ui_kit::declarative::table::PointerRowSelectionPolicy::ListLike,
        )
        .single_row_selection(false)
        .row_height(Px(40.0))
        .header_height(Px(40.0))
        .column_actions_menu(false)
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(240.0)))
        .into_element_retained(
            cx,
            assets.data.clone(),
            1,
            list_like_state,
            list_like_columns,
            |row, _index, _parent| fret_ui_headless::table::RowKey(row.id),
            |col| col.id.clone(),
            move |cx, col, row| match col.id.as_ref() {
                "name" => cx.text(row.name.as_ref()),
                "status" => cx.text(row.status.as_ref()),
                "cpu%" => cx.text(format!("{}%", row.cpu)),
                "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                _ => cx.text("?"),
            },
            Some(Arc::<str>::from("ui-gallery-data-table-listlike-header-")),
            Some(Arc::<str>::from("ui-gallery-data-table-listlike-row-")),
        )
        .test_id("ui-gallery-data-table-listlike-root");

    vec![
        responsive_toggle,
        toolbar,
        table,
        list_like_table,
        shadcn::DataTablePagination::new(state, output).into_element(cx),
    ]
}
