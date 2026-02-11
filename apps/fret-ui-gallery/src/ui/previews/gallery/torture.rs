use super::super::super::*;

pub(in crate::ui) fn preview_inspector_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    let len: usize = std::env::var("FRET_UI_GALLERY_INSPECTOR_LEN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(50_000)
        .clamp(16, 200_000);
    let row_height = Px(28.0);
    let overscan = 12;
    let keep_alive: usize = std::env::var("FRET_UI_GALLERY_INSPECTOR_KEEP_ALIVE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
        .clamp(0, 4096);

    let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

    let list_layout = fret_ui::element::LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: fret_ui::element::Length::Fill,
            height: fret_ui::element::Length::Px(Px(460.0)),
            ..Default::default()
        },
        overflow: fret_ui::element::Overflow::Clip,
        ..Default::default()
    };

    let options =
        fret_ui::element::VirtualListOptions::known(row_height, overscan, move |_index| row_height)
            .keep_alive(keep_alive);

    let theme = theme.clone();
    let row = move |cx: &mut ElementContext<'_, App>, index: usize| {
        let zebra = (index % 2) == 0;
        let background = if zebra {
            theme.color_required("muted")
        } else {
            theme.color_required("background")
        };

        let depth = (index % 8) as f32;
        let indent_px = Px(depth * 12.0);

        let name = cx.text(format!("prop_{index}"));
        let value = cx.text(format!("value {index}"));

        let spacer = cx.container(
            fret_ui::element::ContainerProps {
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Px(indent_px),
                        height: fret_ui::element::Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        let mut row_props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(background))
                .p(Space::N2),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(row_height)),
        );
        row_props.layout.overflow = fret_ui::element::Overflow::Clip;

        let row = cx.container(row_props, |cx| {
            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .gap(Space::N2)
                    .items_center(),
                |_cx| vec![spacer, name, value],
            )]
        });

        row.test_id(format!("ui-gallery-inspector-row-{index}-label"))
    };

    let list = cx.virtual_list_keyed_retained_with_layout_fn(
        list_layout,
        len,
        options,
        &scroll_handle,
        |i| i as fret_ui::ItemKey,
        row,
    );

    let list = list.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::List)
            .test_id("ui-gallery-inspector-root"),
    );

    vec![cx.cached_subtree_with(
        CachedSubtreeProps::default().contained_layout(true),
        |_cx| vec![list],
    )]
}

pub(in crate::ui) fn preview_file_tree_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    let _ = theme;
    use std::collections::HashSet;

    let row_height = Px(26.0);
    let overscan = 12;

    let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

    let list_layout = fret_ui::element::LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: fret_ui::element::Length::Fill,
            height: fret_ui::element::Length::Px(Px(460.0)),
            ..Default::default()
        },
        overflow: fret_ui::element::Overflow::Clip,
        ..Default::default()
    };

    use fret_ui_kit::{TreeItem, TreeItemId, TreeState};

    #[derive(Default)]
    struct FileTreeTortureModels {
        items: Option<Model<Vec<TreeItem>>>,
        state: Option<Model<TreeState>>,
    }

    let (items, state) = cx.with_state(FileTreeTortureModels::default, |st| {
        (st.items.clone(), st.state.clone())
    });
    let (items, state) = match (items, state) {
        (Some(items), Some(state)) => (items, state),
        _ => {
            let (items_value, state_value) = {
                let root_count: u64 = std::env::var("FRET_UI_GALLERY_FILE_TREE_ROOTS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(200);
                let folders_per_root = 10u64;
                let leaves_per_folder = 25u64;

                let mut expanded: HashSet<TreeItemId> = HashSet::new();
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
                            leaves.push(TreeItem::new(
                                leaf_id,
                                Arc::<str>::from(format!("file_{r}_{f}_{l}.rs")),
                            ));
                        }

                        folders.push(
                            TreeItem::new(folder_id, Arc::<str>::from(format!("dir_{r}_{f}")))
                                .children(leaves),
                        );
                    }

                    roots.push(
                        TreeItem::new(root_id, Arc::<str>::from(format!("root_{r}")))
                            .children(folders),
                    );
                }

                (
                    roots,
                    TreeState {
                        expanded,
                        selected: None,
                    },
                )
            };

            let items = cx.app.models_mut().insert(items_value);
            let state = cx.app.models_mut().insert(state_value);
            cx.with_state(FileTreeTortureModels::default, |st| {
                st.items = Some(items.clone());
                st.state = Some(state.clone());
            });
            (items, state)
        }
    };

    let mut props = fret_ui_kit::declarative::file_tree::FileTreeViewProps::default();
    props.layout = list_layout;
    props.row_height = row_height;
    props.overscan = overscan;
    props.debug_root_test_id = Some(Arc::<str>::from("ui-gallery-file-tree-root"));
    props.debug_row_test_id_prefix = Some(Arc::<str>::from("ui-gallery-file-tree-node"));

    vec![
        fret_ui_kit::declarative::file_tree::file_tree_view_retained_v0(
            cx,
            items,
            state,
            &scroll_handle,
            props,
        ),
    ]
}

pub(in crate::ui) fn preview_table_retained_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_kit::headless::table::{
        ColumnDef, RowKey, RowPinPosition, TableState, pagination_bounds, pin_rows,
    };
    let variable_height = std::env::var_os("FRET_UI_GALLERY_TABLE_VARIABLE_HEIGHT")
        .filter(|v| !v.is_empty())
        .is_some();
    let keep_alive: usize = std::env::var("FRET_UI_GALLERY_TABLE_KEEP_ALIVE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    #[derive(Clone)]
    struct TableRow {
        id: u64,
        name: Arc<str>,
        status: Arc<str>,
        cpu: u32,
        mem_mb: u32,
    }

    #[derive(Default)]
    struct TableTortureModels {
        data: Option<Arc<[TableRow]>>,
        columns: Option<Arc<[ColumnDef<TableRow>]>>,
        state: Option<Model<TableState>>,
        keep_pinned_rows: Option<Model<bool>>,
    }

    let (data, columns, state, keep_pinned_rows) =
        cx.with_state(TableTortureModels::default, |st| {
            (
                st.data.clone(),
                st.columns.clone(),
                st.state.clone(),
                st.keep_pinned_rows.clone(),
            )
        });

    let (data, columns, state, keep_pinned_rows) = match (data, columns, state, keep_pinned_rows) {
        (Some(data), Some(columns), Some(state), Some(keep_pinned_rows)) => {
            (data, columns, state, keep_pinned_rows)
        }
        _ => {
            let mut rows: Vec<TableRow> = Vec::with_capacity(50_000);
            for i in 0..50_000u64 {
                rows.push(TableRow {
                    id: i,
                    name: Arc::from(format!("Row {i}")),
                    status: Arc::from(if i % 3 == 0 {
                        "idle"
                    } else if i % 3 == 1 {
                        "busy"
                    } else {
                        "offline"
                    }),
                    cpu: ((i * 7) % 100) as u32,
                    mem_mb: (128 + (i % 4096)) as u32,
                });
            }
            let data: Arc<[TableRow]> = Arc::from(rows);

            let cols: Vec<ColumnDef<TableRow>> = vec![
                ColumnDef::new("name").sort_by(|a: &TableRow, b: &TableRow| a.name.cmp(&b.name)),
                ColumnDef::new("status")
                    .sort_by(|a: &TableRow, b: &TableRow| a.status.cmp(&b.status)),
                ColumnDef::new("cpu%").sort_by(|a: &TableRow, b: &TableRow| a.cpu.cmp(&b.cpu)),
                ColumnDef::new("mem_mb")
                    .sort_by(|a: &TableRow, b: &TableRow| a.mem_mb.cmp(&b.mem_mb)),
            ];
            let columns: Arc<[ColumnDef<TableRow>]> = Arc::from(cols);

            let state = cx.app.models_mut().insert(TableState::default());
            let keep_pinned_rows = cx.app.models_mut().insert(true);

            cx.with_state(TableTortureModels::default, |st| {
                st.data = Some(data.clone());
                st.columns = Some(columns.clone());
                st.state = Some(state.clone());
                st.keep_pinned_rows = Some(keep_pinned_rows.clone());
            });

            (data, columns, state, keep_pinned_rows)
        }
    };

    let sorting: Vec<fret_ui_kit::headless::table::SortSpec> = cx
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

    let row_pinning_text: Arc<str> = {
        let pinning = cx
            .app
            .models()
            .read(&state, |st| st.row_pinning.clone())
            .ok()
            .unwrap_or_default();
        let top = pinning
            .top
            .iter()
            .map(|k| k.0.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let bottom = pinning
            .bottom
            .iter()
            .map(|k| k.0.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Arc::<str>::from(format!("RowPinning: top=[{top}] bottom=[{bottom}]"))
    };

    let keep_pinned_rows_value = cx
        .get_model_copied(&keep_pinned_rows, Invalidation::Paint)
        .unwrap_or(true);
    let keep_pinned_rows_text: Arc<str> =
        Arc::<str>::from(format!("KeepPinnedRows: {keep_pinned_rows_value}"));

    let page_text: Arc<str> = {
        let pagination = cx
            .app
            .models()
            .read(&state, |st| st.pagination)
            .ok()
            .unwrap_or_default();
        let bounds = pagination_bounds(data.len(), pagination);
        if bounds.page_count == 0 {
            Arc::<str>::from("Page: 0/0")
        } else {
            Arc::<str>::from(format!(
                "Page: {}/{}",
                bounds.page_index + 1,
                bounds.page_count
            ))
        }
    };

    let state_for_pin_top = state.clone();
    let on_pin_top: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&state_for_pin_top, |st| {
            let Some(&row_key) = st.row_selection.iter().next() else {
                return;
            };
            pin_rows(&mut st.row_pinning, Some(RowPinPosition::Top), [row_key]);
        });
        host.request_redraw(action_cx.window);
    });

    let state_for_pin_bottom = state.clone();
    let on_pin_bottom: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&state_for_pin_bottom, |st| {
            let Some(&row_key) = st.row_selection.iter().next() else {
                return;
            };
            pin_rows(&mut st.row_pinning, Some(RowPinPosition::Bottom), [row_key]);
        });
        host.request_redraw(action_cx.window);
    });

    let state_for_unpin = state.clone();
    let on_unpin: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&state_for_unpin, |st| {
            let Some(&row_key) = st.row_selection.iter().next() else {
                return;
            };
            pin_rows(&mut st.row_pinning, None, [row_key]);
        });
        host.request_redraw(action_cx.window);
    });

    let state_for_prev_page = state.clone();
    let on_prev_page: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&state_for_prev_page, |st| {
            st.pagination.page_index = st.pagination.page_index.saturating_sub(1);
        });
        host.request_redraw(action_cx.window);
    });

    let state_for_next_page = state.clone();
    let on_next_page: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&state_for_next_page, |st| {
            st.pagination.page_index = st.pagination.page_index.saturating_add(1);
        });
        host.request_redraw(action_cx.window);
    });

    let actions = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Prev page")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-table-retained-prev-page")
                    .on_activate(on_prev_page)
                    .into_element(cx),
                shadcn::Button::new("Next page")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-table-retained-next-page")
                    .on_activate(on_next_page)
                    .into_element(cx),
                shadcn::Button::new("Pin top")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-table-retained-pin-top")
                    .on_activate(on_pin_top)
                    .into_element(cx),
                shadcn::Button::new("Pin bottom")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-table-retained-pin-bottom")
                    .on_activate(on_pin_bottom)
                    .into_element(cx),
                shadcn::Button::new("Unpin")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-table-retained-unpin")
                    .on_activate(on_unpin)
                    .into_element(cx),
                shadcn::Switch::new(keep_pinned_rows.clone())
                    .a11y_label("Keep pinned rows")
                    .test_id("ui-gallery-table-retained-keep-pinned-rows")
                    .into_element(cx),
                cx.text("Keep pinned rows"),
            ]
        },
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text(
                    "Goal: baseline harness for `fret-ui-kit::declarative::table` running on the virt-003 retained host path.",
                ),
                cx.text(
                    "Use scripted sort/selection + scroll to validate reconcile deltas under view-cache reuse (no notify-based dirty views).",
                ),
                cx.text(sorting_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(sorting_text.clone())
                        .test_id("ui-gallery-table-retained-sorting"),
                ),
                cx.text(row_pinning_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(row_pinning_text.clone())
                        .test_id("ui-gallery-table-retained-row-pinning"),
                ),
                cx.text(keep_pinned_rows_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(keep_pinned_rows_text.clone())
                        .test_id("ui-gallery-table-retained-keep-pinned-rows-text"),
                ),
                cx.text(page_text.as_ref()).attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .label(page_text.clone())
                        .test_id("ui-gallery-table-retained-pagination"),
                ),
                actions,
            ]
        },
    );

    let table =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

            let state_revision = cx.app.models().revision(&state).unwrap_or(0);
            let items_revision = 1 ^ state_revision.rotate_left(17);

            let mut props = fret_ui_kit::declarative::table::TableViewProps::default();
            props.overscan = 10;
            props.row_height = Some(Px(28.0));
            if keep_alive > 0 {
                props.keep_alive = Some(keep_alive);
            }
            props.row_measure_mode = if variable_height {
                fret_ui_kit::declarative::table::TableRowMeasureMode::Measured
            } else {
                fret_ui_kit::declarative::table::TableRowMeasureMode::Fixed
            };
            props.enable_column_grouping = false;
            props.enable_column_resizing = false;
            props.keep_pinned_rows = cx
                .get_model_copied(&keep_pinned_rows, Invalidation::Layout)
                .unwrap_or(true);

            let header_label =
                Arc::new(|col: &ColumnDef<TableRow>| Arc::<str>::from(col.id.as_ref()));
            let row_key_at = Arc::new(|row: &TableRow, _index: usize| RowKey(row.id));
            let cell_at = Arc::new(
                move |cx: &mut ElementContext<'_, App>,
                      col: &ColumnDef<TableRow>,
                      row: &TableRow| {
                    match col.id.as_ref() {
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
                    }
                },
            );

            let table = fret_ui_kit::declarative::table::table_virtualized_retained_v0(
                cx,
                data.clone(),
                columns.clone(),
                state.clone(),
                &scroll_handle,
                items_revision,
                row_key_at,
                Some(Arc::new(|row: &TableRow, _index: usize| {
                    Arc::from(row.id.to_string())
                })),
                props,
                header_label,
                None,
                cell_at,
                Some(Arc::<str>::from("ui-gallery-table-retained-header-")),
                Some(Arc::<str>::from("ui-gallery-table-retained-row-")),
            );

            vec![
                table.attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Group)
                        .test_id("ui-gallery-table-retained-torture-root"),
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
