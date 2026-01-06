use fret_core::{Color, Corners, CursorIcon, Edges, Px, SemanticsRole};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, Overflow, PointerRegionProps, PressableA11y,
    PressableProps, ScrollAxis, ScrollProps,
};
use fret_ui::scroll::{ScrollHandle, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};

use std::cell::Cell;
use std::sync::Arc;
use std::time::Instant;

use crate::declarative::action_hooks::ActionHooksExt;
use crate::declarative::collection_semantics::CollectionSemanticsExt as _;
use crate::declarative::model_watch::ModelWatchExt as _;
use crate::declarative::stack;
use crate::{Items, Justify, LayoutRefinement, MetricRef, Size, Space};

use crate::headless::table::{
    ColumnDef, ColumnId, Row, RowKey, SortCmpFn, SortSpec, TableState, column_size,
    is_column_visible, is_row_selected, order_columns, split_pinned_columns,
};

fn resolve_table_colors(theme: &Theme) -> (Color, Color, Color, Color, Color) {
    let table_bg = theme
        .color_by_key("table.background")
        .or_else(|| theme.color_by_key("list.background"))
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or(theme.colors.panel_background);
    let border = theme
        .color_by_key("table.border")
        .or_else(|| theme.color_by_key("border"))
        .or_else(|| theme.color_by_key("list.border"))
        .unwrap_or(theme.colors.panel_border);
    let header_bg = theme
        .color_by_key("table.header.background")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or(table_bg);
    let row_hover = theme
        .color_by_key("table.row.hover")
        .or_else(|| theme.color_by_key("list.hover.background"))
        .or_else(|| theme.color_by_key("list.row.hover"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or(theme.colors.list_row_hover);
    let row_active = theme
        .color_by_key("table.row.active")
        .or_else(|| theme.color_by_key("list.active.background"))
        .or_else(|| theme.color_by_key("list.row.active"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or(theme.colors.list_row_selected);
    (table_bg, border, header_bg, row_hover, row_active)
}

fn resolve_row_height(theme: &Theme, size: Size) -> Px {
    let base = theme
        .metric_by_key("component.table.row_height")
        .or_else(|| theme.metric_by_key("component.list.row_height"))
        .unwrap_or_else(|| size.list_row_h(theme));
    Px(base.0.max(0.0))
}

fn resolve_cell_padding_x(theme: &Theme) -> Px {
    MetricRef::space(Space::N2p5).resolve(theme)
}

fn resolve_cell_padding_y(theme: &Theme) -> Px {
    MetricRef::space(Space::N1p5).resolve(theme)
}

fn sort_for_column(sorting: &[SortSpec], id: &ColumnId) -> Option<bool> {
    sorting
        .iter()
        .find(|s| s.column.as_ref() == id.as_ref())
        .map(|s| s.desc)
}

fn next_sort_for_column(current: Option<bool>) -> Option<bool> {
    match current {
        None => Some(false),
        Some(false) => Some(true),
        Some(true) => None,
    }
}

#[derive(Debug, Clone)]
pub struct TableViewProps {
    pub size: Size,
    pub row_height: Option<Px>,
    pub overscan: usize,
    pub default_column_width: Px,
    pub min_column_width: Px,
    pub enable_column_resizing: bool,
    pub enable_row_selection: bool,
    pub single_row_selection: bool,
}

impl Default for TableViewProps {
    fn default() -> Self {
        Self {
            size: Size::Medium,
            row_height: None,
            overscan: 2,
            default_column_width: Px(160.0),
            min_column_width: Px(40.0),
            enable_column_resizing: true,
            enable_row_selection: true,
            single_row_selection: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SortKey {
    column: ColumnId,
    desc: bool,
}

#[derive(Default)]
struct Memo<TDeps, TValue> {
    deps: Option<TDeps>,
    value: Option<TValue>,
}

impl<TDeps: PartialEq, TValue> Memo<TDeps, TValue> {
    fn get_or_compute(&mut self, deps: TDeps, f: impl FnOnce() -> TValue) -> (&TValue, bool) {
        let should_recompute = self.deps.as_ref().map_or(true, |d| d != &deps);
        if should_recompute {
            self.deps = Some(deps);
            self.value = Some(f());
        }
        (self.value.as_ref().expect("memo value"), should_recompute)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct FlatRowOrderDeps {
    items_revision: u64,
    data_len: usize,
    sorting: Vec<SortKey>,
}

#[derive(Default)]
struct FlatRowOrderCache {
    memo: Memo<FlatRowOrderDeps, Arc<[usize]>>,
}

fn compute_flat_row_order<TData>(
    data: &[TData],
    columns: &[ColumnDef<TData>],
    sorting: &[SortKey],
) -> Arc<[usize]> {
    let sorters: Vec<(SortCmpFn<TData>, bool)> = sorting
        .iter()
        .filter_map(|spec| {
            let cmp = columns
                .iter()
                .find(|c| c.id.as_ref() == spec.column.as_ref())?
                .sort_cmp
                .clone()?;
            Some((cmp, spec.desc))
        })
        .collect();

    let mut order: Vec<usize> = (0..data.len()).collect();
    if !sorters.is_empty() {
        order.sort_by(|&a, &b| {
            let a_row = &data[a];
            let b_row = &data[b];
            for (cmp, desc) in &sorters {
                let mut ord = cmp(a_row, b_row);
                if *desc {
                    ord = ord.reverse();
                }
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            a.cmp(&b)
        });
    }

    Arc::from(order.into_boxed_slice())
}

#[allow(clippy::too_many_arguments)]
pub fn table_virtualized<H: UiHost, TData>(
    cx: &mut ElementContext<'_, H>,
    data: &[TData],
    columns: Vec<ColumnDef<TData>>,
    state: Model<TableState>,
    vertical_scroll: &VirtualListScrollHandle,
    items_revision: u64,
    row_key_at: &dyn Fn(&TData, usize) -> RowKey,
    props: TableViewProps,
    on_row_activate: impl Fn(&Row<'_, TData>) -> Option<CommandId>,
    mut render_header_cell: impl FnMut(
        &mut ElementContext<'_, H>,
        &ColumnDef<TData>,
        Option<bool>,
    ) -> Vec<AnyElement>,
    mut render_cell: impl FnMut(
        &mut ElementContext<'_, H>,
        &Row<'_, TData>,
        &ColumnDef<TData>,
    ) -> Vec<AnyElement>,
) -> AnyElement {
    let profile = std::env::var_os("FRET_TABLE_PROFILE").is_some();
    let state_value = cx.watch_model(&state).layout().cloned().unwrap_or_default();

    let theme = Theme::global(&*cx.app);
    let (table_bg, border, header_bg, row_hover, row_active) = resolve_table_colors(theme);
    let radius = theme.metrics.radius_md;

    let row_h = props
        .row_height
        .unwrap_or_else(|| resolve_row_height(theme, props.size));
    let cell_px = resolve_cell_padding_x(theme);
    let cell_py = resolve_cell_padding_y(theme);

    let scroll_x = cx.with_state(ScrollHandle::default, |h| h.clone());

    let ordered_columns = order_columns(&columns, &state_value.column_order);
    let visible_columns = ordered_columns
        .into_iter()
        .filter(|c| is_column_visible(&state_value.column_visibility, &c.id))
        .collect::<Vec<_>>();
    let (left_cols, center_cols, right_cols) =
        split_pinned_columns(visible_columns.as_slice(), &state_value.column_pinning);

    let sorting_key = state_value
        .sorting
        .iter()
        .map(|s| SortKey {
            column: s.column.clone(),
            desc: s.desc,
        })
        .collect::<Vec<_>>();

    let row_order = cx.with_state(FlatRowOrderCache::default, |cache| {
        let deps = FlatRowOrderDeps {
            items_revision,
            data_len: data.len(),
            sorting: sorting_key.clone(),
        };

        let started = Instant::now();
        let (order, recomputed) = cache.memo.get_or_compute(deps, || {
            compute_flat_row_order(data, &columns, &sorting_key)
        });
        let elapsed = started.elapsed();

        if profile && recomputed {
            tracing::info!(
                "table_virtualized: recompute row_order len={} sorting={} took {:.2}ms",
                data.len(),
                sorting_key.len(),
                elapsed.as_secs_f64() * 1000.0
            );
        }

        order.clone()
    });

    let page_size = state_value.pagination.page_size;
    let page_start = state_value.pagination.page_index.saturating_mul(page_size);
    let page_end = page_start.saturating_add(page_size);
    let page_rows: &[usize] = if page_size == 0 {
        &[]
    } else {
        row_order.get(page_start..page_end).unwrap_or_default()
    };

    let set_size = page_rows.len();

    let mut list_options = fret_ui::element::VirtualListOptions::new(row_h, props.overscan);
    list_options.items_revision = items_revision;
    list_options.measure_mode = fret_ui::element::VirtualListMeasureMode::Fixed;

    let rendered_rows = Cell::new(0usize);

    cx.semantics(
        fret_ui::element::SemanticsProps {
            role: SemanticsRole::List,
            ..Default::default()
        },
        |cx| {
            vec![cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout.flex.grow = 1.0;
                        layout.flex.basis = Length::Px(Px(0.0));
                        layout.overflow = Overflow::Clip;
                        layout
                    },
                    background: Some(table_bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                |cx| {
                    vec![stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().size_full())
                            .gap_y(Space::N0)
                            .justify(Justify::Start)
                            .items(Items::Stretch),
                        |cx| {
                                    let header = cx.container(
                                        ContainerProps {
                                            background: Some(header_bg),
                                            border: Edges {
                                                bottom: Px(1.0),
                                                ..Default::default()
                                            },
                                            border_color: Some(border),
                                            layout: LayoutStyle {
                                                size: fret_ui::element::SizeStyle {
                                                    height: Length::Px(row_h),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx| {
                                            let mut render_header_group =
                                                |cx: &mut ElementContext<'_, H>,
                                                 cols: &[&ColumnDef<TData>],
                                                 scroll_x: Option<ScrollHandle>| {
                                                    let row = stack::hstack(
                                                        cx,
                                                        stack::HStackProps::default()
                                                            .gap_x(Space::N0)
                                                            .justify(Justify::Start)
                                                            .items(Items::Center),
                                                        |cx| {
                                                            cols.iter()
                                                                .map(|col| {
                                                                    let sort_state = sort_for_column(
                                                                        &state_value.sorting,
                                                                        &col.id,
                                                                    );

                                                                    let col_w = column_size(
                                                                        &state_value.column_sizing,
                                                                        &col.id,
                                                                    )
                                                                    .map(|w| Px(w.max(0.0)))
                                                                    .unwrap_or(props.default_column_width);

                                                                    let cell_props = ContainerProps {
                                                                        padding: Edges::all(Px(0.0)),
                                                                        layout: LayoutStyle {
                                                                            size: fret_ui::element::SizeStyle {
                                                                                width: Length::Px(col_w),
                                                                                ..Default::default()
                                                                            },
                                                                            flex: fret_ui::element::FlexItemStyle {
                                                                                shrink: 0.0,
                                                                                ..Default::default()
                                                                            },
                                                                            ..Default::default()
                                                                        },
                                                                        ..Default::default()
                                                                    };

                                                                    cx.container(cell_props, |cx| {
                                                                        let mut out = Vec::new();

                                                                        out.push(stack::hstack(
                                                                            cx,
                                                                            stack::HStackProps::default()
                                                                                .gap_x(Space::N0)
                                                                                .justify(Justify::Start)
                                                                                .items(Items::Center),
                                                                            |cx| {
                                                                                let mut pieces = Vec::new();

                                                                                let enabled =
                                                                                    col.sort_cmp.is_some();
                                                                                let col_id = col.id.clone();
                                                                                let state_model =
                                                                                    state.clone();

                                                                                pieces.push(cx.pressable(
                                                                                    PressableProps {
                                                                                        enabled,
                                                                                        a11y: PressableA11y {
                                                                                            role: Some(
                                                                                                SemanticsRole::Button,
                                                                                            ),
                                                                                            ..Default::default()
                                                                                        },
                                                                                        ..Default::default()
                                                                                    },
                                                                                    |cx, _| {
                                                                                        if enabled {
                                                                                            cx.pressable_update_model(
                                                                                                &state_model,
                                                                                                move |st| {
                                                                                                    let current = sort_for_column(&st.sorting, &col_id);
                                                                                                    let next = next_sort_for_column(current);
                                                                                                    st.sorting.clear();
                                                                                                    if let Some(desc) = next {
                                                                                                        st.sorting.push(SortSpec { column: col_id.clone(), desc });
                                                                                                    }
                                                                                                    st.pagination.page_index = 0;
                                                                                                },
                                                                                            );
                                                                                        }

                                                                                        let mut cell =
                                                                                            render_header_cell(cx, col, sort_state);
                                                                                        if let Some(desc) = sort_state {
                                                                                            cell.push(cx.text(if desc { "↓" } else { "↑" }));
                                                                                        }
                                                                                        cell
                                                                                    },
                                                                                ));

                                                                                if props.enable_column_resizing {
                                                                                    let col_id = col.id.clone();
                                                                                    let state_model = state.clone();
                                                                                    let min_w = props.min_column_width;
                                                                                    let default_w = props.default_column_width;

                                                                                    pieces.push(cx.pointer_region(
                                                                                        PointerRegionProps {
                                                                                            layout: LayoutStyle {
                                                                                                size: fret_ui::element::SizeStyle {
                                                                                                    width: Length::Px(Px(6.0)),
                                                                                                    height: Length::Fill,
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                flex: fret_ui::element::FlexItemStyle {
                                                                                                    shrink: 0.0,
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                ..Default::default()
                                                                                            },
                                                                                            enabled: true,
                                                                                        },
                                                                                        |cx| {
                                                                                            let state_model_down = state_model.clone();
                                                                                            let state_model_move = state_model.clone();
                                                                                            let state_model_up = state_model.clone();
                                                                                            let col_id_down = col_id.clone();
                                                                                            let col_id_move = col_id.clone();
                                                                                            let col_id_up = col_id.clone();

                                                                                            cx.pointer_region_on_pointer_down(
                                                                                                std::sync::Arc::new(move |host, _acx, down| {
                                                                                                    if down.button != fret_core::MouseButton::Left {
                                                                                                        return false;
                                                                                                    }
                                                                                                    host.capture_pointer();
                                                                                                    host.set_cursor_icon(CursorIcon::ColResize);
                                                                                                    let _ = host.models_mut().update(&state_model_down, |st| {
                                                                                                        let start = st
                                                                                                            .column_sizing
                                                                                                            .get(&col_id_down)
                                                                                                            .copied()
                                                                                                            .unwrap_or(default_w.0);
                                                                                                        st.column_sizing.insert(col_id_down.clone(), start);
                                                                                                        st.column_sizing_info.is_resizing_column = Some(col_id_down.clone());
                                                                                                        st.column_sizing_info.start_pointer_x = down.position.x.0;
                                                                                                        st.column_sizing_info.start_size = start;
                                                                                                    });
                                                                                                    true
                                                                                                }),
                                                                                            );
                                                                                            cx.pointer_region_on_pointer_move(
                                                                                                std::sync::Arc::new(move |host, _acx, mv| {
                                                                                                    host.set_cursor_icon(CursorIcon::ColResize);
                                                                                                    if !mv.buttons.left {
                                                                                                        return false;
                                                                                                    }
                                                                                                    let _ = host.models_mut().update(&state_model_move, |st| {
                                                                                                        let Some(active) = &st.column_sizing_info.is_resizing_column else { return; };
                                                                                                        if active.as_ref() != col_id_move.as_ref() { return; }
                                                                                                        let dx = mv.position.x.0 - st.column_sizing_info.start_pointer_x;
                                                                                                        let next = (st.column_sizing_info.start_size + dx).max(min_w.0);
                                                                                                        st.column_sizing.insert(col_id_move.clone(), next);
                                                                                                    });
                                                                                                    true
                                                                                                }),
                                                                                            );
                                                                                            cx.pointer_region_on_pointer_up(
                                                                                                std::sync::Arc::new(move |host, _acx, up| {
                                                                                                    if up.button != fret_core::MouseButton::Left {
                                                                                                        return false;
                                                                                                    }
                                                                                                    host.release_pointer_capture();
                                                                                                    let _ = host.models_mut().update(&state_model_up, |st| {
                                                                                                        if st
                                                                                                            .column_sizing_info
                                                                                                            .is_resizing_column
                                                                                                            .as_ref()
                                                                                                            .is_some_and(|a| a.as_ref() == col_id_up.as_ref())
                                                                                                        {
                                                                                                            st.column_sizing_info.is_resizing_column = None;
                                                                                                        }
                                                                                                    });
                                                                                                    true
                                                                                                }),
                                                                                            );
                                                                                            Vec::new()
                                                                                        },
                                                                                    ));
                                                                                }

                                                                                pieces
                                                                            },
                                                                        ));

                                                                        out
                                                                    })
                                                                })
                                                                .collect()
                                                        },
                                                    );

                                                            if let Some(scroll_x) = scroll_x {
                                                                cx.scroll(
                                                                    ScrollProps {
                                                                        axis: ScrollAxis::X,
                                                                        scroll_handle: Some(scroll_x),
                                                                        ..Default::default()
                                                                    },
                                                                    |_| vec![row],
                                                                )
                                                            } else {
                                                                row
                                                            }
                                                };

                                            vec![stack::hstack(
                                                cx,
                                                stack::HStackProps::default()
                                                    .gap_x(Space::N0)
                                                    .justify(Justify::Start)
                                                    .items(Items::Stretch),
                                                |cx| {
                                                    vec![
                                                        render_header_group(cx, &left_cols, None),
                                                        render_header_group(
                                                            cx,
                                                            &center_cols,
                                                            Some(scroll_x.clone()),
                                                        ),
                                                        render_header_group(cx, &right_cols, None),
                                                    ]
                                                },
                                            )]
                                        },
                                    );

                                    let body = cx.virtual_list_keyed_with_layout(
                                        {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Fill;
                                            layout.size.height = Length::Fill;
                                            layout.flex.grow = 1.0;
                                            layout.flex.basis = Length::Px(Px(0.0));
                                            layout
                                        },
                                        set_size,
                                        list_options,
                                        vertical_scroll,
                                        |i| {
                                            row_key_at(&data[page_rows[i]], page_rows[i]).0
                                        },
                                        |cx, i| {
                                            rendered_rows.set(rendered_rows.get().saturating_add(1));
                                            let data_index = page_rows[i];
                                            let row_key = row_key_at(&data[data_index], data_index);
                                            let data_row = Row {
                                                key: row_key,
                                                original: &data[data_index],
                                                index: data_index,
                                                depth: 0,
                                                parent: None,
                                                parent_key: None,
                                                sub_rows: Vec::new(),
                                            };

                                            let cmd = on_row_activate(&data_row);
                                            let enabled = cmd.is_some() || props.enable_row_selection;
                                            let is_selected =
                                                is_row_selected(data_row.key, &state_value.row_selection);

                                            cx.pressable(
                                                PressableProps {
                                                    enabled,
                                                    a11y: PressableA11y {
                                                        role: Some(SemanticsRole::ListItem),
                                                        selected: is_selected,
                                                        ..Default::default()
                                                    }
                                                    .with_collection_position(i, set_size),
                                                    ..Default::default()
                                                },
                                                |cx, st| {
                                                    cx.pressable_dispatch_command_opt(cmd.clone());
                                                    if props.enable_row_selection {
                                                        let state_model = state.clone();
                                                        let row_key = data_row.key;
                                                        let single = props.single_row_selection;
                                                        cx.pressable_update_model(&state_model, move |st| {
                                                            let selected = st.row_selection.contains(&row_key);
                                                            if single {
                                                                st.row_selection.clear();
                                                            }
                                                            if selected {
                                                                st.row_selection.remove(&row_key);
                                                            } else {
                                                                st.row_selection.insert(row_key);
                                                            }
                                                        });
                                                    }

                                                    let bg = if is_selected || (enabled && st.pressed) {
                                                        Some(row_active)
                                                    } else if enabled && st.hovered {
                                                        Some(row_hover)
                                                    } else {
                                                        None
                                                    };

                                                    vec![cx.container(
                                                        ContainerProps {
                                                            background: bg,
                                                            layout: LayoutStyle {
                                                                size: fret_ui::element::SizeStyle {
                                                                    height: Length::Px(row_h),
                                                                    ..Default::default()
                                                                },
                                                                ..Default::default()
                                                            },
                                                            ..Default::default()
                                                        },
                                                        |cx| {
                                                            let mut render_row_group =
                                                                |cx: &mut ElementContext<'_, H>,
                                                                 cols: &[&ColumnDef<TData>],
                                                                 scroll_x: Option<ScrollHandle>| {
                                                                    let row = stack::hstack(
                                                                        cx,
                                                                        stack::HStackProps::default()
                                                                            .gap_x(Space::N0)
                                                                            .justify(Justify::Start)
                                                                            .items(Items::Center),
                                                                        |cx| {
                                                                            cols.iter()
                                                                                .map(|col| {
                                                                                    let col_w = column_size(
                                                                                        &state_value.column_sizing,
                                                                                        &col.id,
                                                                                    )
                                                                                    .map(|w| Px(w.max(0.0)))
                                                                                    .unwrap_or(
                                                                                        props.default_column_width,
                                                                                    );
                                                                                    cx.container(
                                                                                        ContainerProps {
                                                                                            padding: Edges::symmetric(
                                                                                                cell_px, cell_py,
                                                                                            ),
                                                                                            layout: LayoutStyle {
                                                                                                size: fret_ui::element::SizeStyle {
                                                                                                    width: Length::Px(col_w),
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                flex: fret_ui::element::FlexItemStyle {
                                                                                                    shrink: 0.0,
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                ..Default::default()
                                                                                            },
                                                                                            ..Default::default()
                                                                                        },
                                                                                        |cx| {
                                                                                            render_cell(cx, &data_row, col)
                                                                                        },
                                                                                    )
                                                                                })
                                                                                .collect()
                                                                        },
                                                                    );

                                                                    if let Some(scroll_x) = scroll_x {
                                                                        cx.scroll(
                                                                            ScrollProps {
                                                                                axis: ScrollAxis::X,
                                                                                scroll_handle: Some(scroll_x),
                                                                                ..Default::default()
                                                                            },
                                                                            |_| vec![row],
                                                                        )
                                                                    } else {
                                                                        row
                                                                    }
                                                                };

                                                            vec![stack::hstack(
                                                                cx,
                                                                stack::HStackProps::default()
                                                                    .gap_x(Space::N0)
                                                                    .justify(Justify::Start)
                                                                    .items(Items::Stretch),
                                                                |cx| {
                                                                    vec![
                                                                        render_row_group(cx, &left_cols, None),
                                                                        render_row_group(
                                                                            cx,
                                                                            &center_cols,
                                                                            Some(scroll_x.clone()),
                                                                        ),
                                                                        render_row_group(cx, &right_cols, None),
                                                                    ]
                                                                },
                                                            )]
                                                        },
                                                    )]
                                                },
                                            )
                                        },
                                    );

                                    if profile {
                                        tracing::info!(
                                            "table_virtualized: list len={} page_rows={} rendered_rows={} row_h={:.1}px",
                                            data.len(),
                                            set_size,
                                            rendered_rows.get(),
                                            row_h.0
                                        );
                                    }

                                    vec![header, body]
                        },
                    )]
                },
            )]
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{SortKey, compute_flat_row_order};
    use crate::headless::table::{ColumnDef, ColumnId};
    use std::cmp::Ordering;

    #[derive(Debug)]
    struct Row {
        id: u32,
        score: i32,
        name: &'static str,
    }

    fn col<T: 'static>(id: &str, cmp: fn(&T, &T) -> Ordering) -> ColumnDef<T> {
        ColumnDef::new(id).sort_by(cmp)
    }

    #[test]
    fn flat_row_order_is_stable_without_sorting() {
        let data = [
            Row {
                id: 1,
                score: 10,
                name: "b",
            },
            Row {
                id: 2,
                score: 9,
                name: "a",
            },
        ];

        let columns = vec![col::<Row>("score", |a, b| a.score.cmp(&b.score))];
        let order = compute_flat_row_order(&data, &columns, &[]);
        assert_eq!(&*order, &[0, 1]);
    }

    #[test]
    fn flat_row_order_sorts_by_single_column() {
        let data = [
            Row {
                id: 1,
                score: 10,
                name: "b",
            },
            Row {
                id: 2,
                score: 9,
                name: "a",
            },
        ];

        let columns = vec![col::<Row>("score", |a, b| a.score.cmp(&b.score))];
        let order = compute_flat_row_order(
            &data,
            &columns,
            &[SortKey {
                column: ColumnId::from("score"),
                desc: false,
            }],
        );
        assert_eq!(&*order, &[1, 0]);
    }

    #[test]
    fn flat_row_order_sorts_descending() {
        let data = [
            Row {
                id: 1,
                score: 10,
                name: "a",
            },
            Row {
                id: 2,
                score: 10,
                name: "b",
            },
        ];

        let columns = vec![col::<Row>("name", |a, b| a.name.cmp(b.name))];
        let order = compute_flat_row_order(
            &data,
            &columns,
            &[SortKey {
                column: ColumnId::from("name"),
                desc: true,
            }],
        );
        assert_eq!(&*order, &[1, 0]);
    }

    #[test]
    fn flat_row_order_uses_index_tiebreaker() {
        let data = [
            Row {
                id: 1,
                score: 10,
                name: "x",
            },
            Row {
                id: 2,
                score: 10,
                name: "x",
            },
            Row {
                id: 3,
                score: 10,
                name: "x",
            },
        ];

        let columns = vec![col::<Row>("score", |a, b| a.score.cmp(&b.score))];
        let order = compute_flat_row_order(
            &data,
            &columns,
            &[SortKey {
                column: ColumnId::from("score"),
                desc: false,
            }],
        );
        assert_eq!(&*order, &[0, 1, 2]);
    }

    #[test]
    fn flat_row_order_supports_multi_sort() {
        let data = [
            Row {
                id: 1,
                score: 10,
                name: "b",
            },
            Row {
                id: 2,
                score: 10,
                name: "a",
            },
            Row {
                id: 3,
                score: 9,
                name: "z",
            },
        ];

        let columns = vec![
            col::<Row>("score", |a, b| a.score.cmp(&b.score)),
            col::<Row>("name", |a, b| a.name.cmp(b.name)),
        ];
        let order = compute_flat_row_order(
            &data,
            &columns,
            &[
                SortKey {
                    column: ColumnId::from("score"),
                    desc: false,
                },
                SortKey {
                    column: ColumnId::from("name"),
                    desc: false,
                },
            ],
        );
        assert_eq!(&*order, &[2, 1, 0]);
    }
}
