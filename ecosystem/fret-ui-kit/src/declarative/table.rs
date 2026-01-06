use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::CommandId;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps, ScrollAxis,
    ScrollProps,
};
use fret_ui::scroll::{ScrollHandle, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::action_hooks::ActionHooksExt;
use crate::declarative::collection_semantics::CollectionSemanticsExt as _;
use crate::declarative::stack;
use crate::{Items, Justify, MetricRef, Size, Space};

use crate::headless::table::{
    ColumnDef, ColumnId, Row, SortSpec, Table, TableState, is_row_selected,
};

fn stable_key64(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

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

#[allow(clippy::too_many_arguments)]
pub fn table_virtualized<H: UiHost, TData>(
    cx: &mut ElementContext<'_, H>,
    data: &[TData],
    columns: Vec<ColumnDef<TData>>,
    state: TableState,
    size: Size,
    row_height: Option<Px>,
    overscan: usize,
    vertical_scroll: &VirtualListScrollHandle,
    items_revision: u64,
    on_toggle_sort: impl Fn(&ColumnId, Option<bool>) -> Option<CommandId>,
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
    let theme = Theme::global(&*cx.app);
    let (table_bg, border, header_bg, row_hover, row_active) = resolve_table_colors(theme);
    let radius = theme.metrics.radius_md;

    let row_h = row_height.unwrap_or_else(|| resolve_row_height(theme, size));
    let cell_px = resolve_cell_padding_x(theme);
    let cell_py = resolve_cell_padding_y(theme);

    let scroll_x = cx.with_state(ScrollHandle::default, |h| h.clone());

    let table = Table::builder(data)
        .columns(columns)
        .state(state.clone())
        .build();

    let visible_columns = table.visible_columns();
    let row_model = table.row_model();
    let set_size = row_model.root_rows().len();

    let mut list_options = fret_ui::element::VirtualListOptions::new(row_h, overscan);
    list_options.items_revision = items_revision;

    cx.semantics(
        fret_ui::element::SemanticsProps {
            role: SemanticsRole::List,
            ..Default::default()
        },
        |cx| {
            vec![cx.container(
                ContainerProps {
                    background: Some(table_bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.scroll(
                        ScrollProps {
                            axis: ScrollAxis::X,
                            scroll_handle: Some(scroll_x),
                            ..Default::default()
                        },
                        |cx| {
                            vec![stack::vstack(
                                cx,
                                stack::VStackProps::default()
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
                                            padding: Edges::symmetric(cell_px, cell_py),
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
                                            vec![stack::hstack(
                                                cx,
                                                stack::HStackProps::default()
                                                    .gap_x(Space::N0)
                                                    .justify(Justify::Start)
                                                    .items(Items::Center),
                                                |cx| {
                                                    visible_columns
                                                        .iter()
                                                        .map(|col| {
                                                            let sort_state =
                                                                sort_for_column(&state.sorting, &col.id);
                                                            let next = next_sort_for_column(sort_state);
                                                            let cmd = if col.sort_cmp.is_some() {
                                                                on_toggle_sort(&col.id, next)
                                                            } else {
                                                                None
                                                            };
                                                            let enabled = cmd.is_some();

                                                            let col_w = table.column_size(col.id.as_ref());
                                                            let mut cell_props = ContainerProps {
                                                                padding: Edges::all(Px(0.0)),
                                                                ..Default::default()
                                                            };
                                                            if let Some(col_w) = col_w {
                                                                cell_props.layout.size.width =
                                                                    Length::Px(Px(col_w.max(0.0)));
                                                                cell_props.layout.flex.shrink = 0.0;
                                                            }

                                                            cx.container(cell_props, |cx| {
                                                                vec![cx.pressable(
                                                                    PressableProps {
                                                                        enabled,
                                                                        a11y: PressableA11y {
                                                                            role: Some(SemanticsRole::Button),
                                                                            ..Default::default()
                                                                        },
                                                                        ..Default::default()
                                                                    },
                                                                    |cx, _| {
                                                                        cx.pressable_dispatch_command_opt(cmd);
                                                                        let mut out =
                                                                            render_header_cell(cx, col, sort_state);

                                                                        if let Some(desc) = sort_state {
                                                                            out.push(cx.text(if desc { "↓" } else { "↑" }));
                                                                        }

                                                                        out
                                                                    },
                                                                )]
                                                            })
                                                        })
                                                        .collect()
                                                },
                                            )]
                                        },
                                    );

                                    let body = cx.virtual_list_keyed(
                                        set_size,
                                        list_options,
                                        vertical_scroll,
                                        |i| {
                                            let root = row_model.root_rows()[i];
                                            let row = row_model
                                                .row(root)
                                                .expect("root row exists");
                                            stable_key64(row.id.as_ref())
                                        },
                                        |cx, i| {
                                            let root = row_model.root_rows()[i];
                                            let row = row_model
                                                .row(root)
                                                .expect("root row exists");

                                            let cmd = on_row_activate(row);
                                            let enabled = cmd.is_some();
                                            let is_selected = is_row_selected(&row.id, &state.row_selection);

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
                                                    cx.pressable_dispatch_command_opt(cmd);

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
                                                            vec![stack::hstack(
                                                                cx,
                                                                stack::HStackProps::default()
                                                                    .gap_x(Space::N0)
                                                                    .justify(Justify::Start)
                                                                    .items(Items::Center),
                                                                |cx| {
                                                                    visible_columns
                                                                        .iter()
                                                                        .map(|col| {
                                                                            let col_w = table.column_size(col.id.as_ref());
                                                                            let mut cell_props = ContainerProps {
                                                                                padding: Edges::symmetric(cell_px, cell_py),
                                                                                ..Default::default()
                                                                            };
                                                                            if let Some(col_w) = col_w {
                                                                                cell_props.layout.size.width =
                                                                                    Length::Px(Px(col_w.max(0.0)));
                                                                                cell_props.layout.flex.shrink = 0.0;
                                                                            }
                                                                            cx.container(cell_props, |cx| {
                                                                                render_cell(cx, row, col)
                                                                            })
                                                                        })
                                                                        .collect()
                                                                },
                                                            )]
                                                        },
                                                    )]
                                                },
                                            )
                                        },
                                    );

                                    vec![header, body]
                                },
                            )]
                        },
                    )]
                },
            )]
        },
    )
}
