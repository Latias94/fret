use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, Corners, CursorIcon, Edges, Px, SemanticsRole};
use fret_ui::action::ActivateReason;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, Overflow, PointerRegionProps, PressableA11y,
    PressableProps, PressableState, SemanticsProps,
};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};

use super::containers::build_imui_children_with_focus;
use super::label_identity::parse_label_identity;
use super::{
    ImUiFacade, ResponseExt, TableColumn, TableColumnWidth, TableHeaderResponse, TableOptions,
    TableResponse, TableRowOptions, TableSortDirection, UiWriterImUiFacadeExt,
};

use super::TableColumnResizeResponse;

const TABLE_RESIZE_HANDLE_HIT_WIDTH: Px = Px(12.0);
const TABLE_RESIZE_HANDLE_MIN_HEIGHT: Px = Px(24.0);
const TABLE_RESIZE_HANDLE_VISUAL_WIDTH: Px = Px(1.0);

struct BuiltTableRow {
    key: Arc<str>,
    test_id: Option<Arc<str>>,
    cells: Vec<BuiltTableCell>,
}

struct BuiltTableCell {
    test_id: Option<Arc<str>>,
    content: AnyElement,
}

pub struct ImUiTable<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    rows: &'cx mut Vec<BuiltTableRow>,
    root_test_id: Option<Arc<str>>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
}

pub struct ImUiTableRow<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    cells: &'cx mut Vec<BuiltTableCell>,
    row_test_id: Option<Arc<str>>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
}

pub(super) fn table_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    columns: &[TableColumn],
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: TableOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
) -> (AnyElement, TableResponse) {
    let columns = columns.to_vec();
    let mut rows = Vec::new();
    {
        let mut table = ImUiTable {
            cx,
            rows: &mut rows,
            root_test_id: options.test_id.clone(),
            build_focus,
        };
        f(&mut table);
    }

    render_table(cx, id, columns, rows, options)
}

impl<'cx, 'a, H: UiHost> ImUiTable<'cx, 'a, H> {
    pub fn row(
        &mut self,
        key: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTableRow<'cx2, 'a2, H>),
    ) {
        self.row_with_options(key, TableRowOptions::default(), f);
    }

    pub fn row_with_options(
        &mut self,
        key: impl Into<Arc<str>>,
        options: TableRowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTableRow<'cx2, 'a2, H>),
    ) {
        let key = key.into();
        let row_index = self.rows.len();
        let default_test_id = self
            .root_test_id
            .as_ref()
            .map(|base| Arc::from(format!("{base}.row.{row_index}")));
        let row_test_id = options.test_id.or(default_test_id);
        let mut cells = Vec::new();
        let build_focus = self.build_focus.clone();
        self.cx.keyed(key.clone(), |cx| {
            let mut row = ImUiTableRow {
                cx,
                cells: &mut cells,
                row_test_id: row_test_id.clone(),
                build_focus,
            };
            f(&mut row);
        });
        self.rows.push(BuiltTableRow {
            key,
            test_id: row_test_id,
            cells,
        });
    }
}

impl<'cx, 'a, H: UiHost> ImUiTableRow<'cx, 'a, H> {
    pub fn cell(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let cell_index = self.cells.len();
        let mut out = Vec::new();
        build_imui_children_with_focus(self.cx, &mut out, self.build_focus.clone(), f);
        let content = pack_cell_children(self.cx, out);
        let test_id = self
            .row_test_id
            .as_ref()
            .map(|base| Arc::from(format!("{base}.cell.{cell_index}")));
        self.cells.push(BuiltTableCell { test_id, content });
    }

    pub fn cell_text(&mut self, text: impl Into<Arc<str>>) {
        let text = text.into();
        self.cell(move |ui| {
            ui.text(text.clone());
        });
    }
}

fn render_table<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    columns: Vec<TableColumn>,
    rows: Vec<BuiltTableRow>,
    options: TableOptions,
) -> (AnyElement, TableResponse) {
    let palette = resolve_table_palette(Theme::global(&*cx.app));
    let root_test_id = options.test_id.clone();
    let show_header = options.show_header && columns.iter().any(|column| column.header.is_some());
    let column_test_id_suffixes = columns
        .iter()
        .enumerate()
        .map(|(index, column)| column_test_id_suffix(column, index))
        .collect::<Vec<_>>();
    let mut header_responses = Vec::new();
    let header = if show_header {
        let column_test_id_suffixes = column_test_id_suffixes.clone();
        Some(cx.keyed(format!("{id}.header"), |cx| {
            let cells = columns
                .iter()
                .enumerate()
                .map(|(index, column)| {
                    let visible_label = visible_header_label(column);
                    let test_id = root_test_id.as_ref().map(|base| {
                        Arc::from(format!(
                            "{base}.header.cell.{}",
                            column_test_id_suffixes[index]
                        ))
                    });
                    let sortable = column_is_sortable(column);
                    let resize_options = column.resize;
                    let mut resize = TableColumnResizeResponse {
                        column_index: index,
                        column_id: column.id.clone(),
                        enabled: resize_options.is_some(),
                        min_width: resize_options.and_then(|options| options.min_width),
                        max_width: resize_options.and_then(|options| options.max_width),
                        drag: Default::default(),
                    };
                    let built = if sortable {
                        wrap_sortable_header_cell(
                            cx,
                            column,
                            index,
                            visible_label.clone(),
                            test_id,
                            &options,
                            &mut resize,
                        )
                    } else {
                        BuiltHeaderCell {
                            element: wrap_plain_header_cell(
                                cx,
                                column,
                                index,
                                visible_label,
                                test_id,
                                &options,
                                &mut resize,
                            ),
                            trigger: ResponseExt::default(),
                        }
                    };
                    header_responses.push(TableHeaderResponse {
                        column_index: index,
                        column_id: column.id.clone(),
                        sortable,
                        sort_direction: column.sort_direction,
                        trigger: built.trigger,
                        resize,
                    });
                    built.element
                })
                .collect::<Vec<_>>();
            wrap_table_row(
                cx,
                cells,
                root_test_id
                    .as_ref()
                    .map(|base| Arc::from(format!("{base}.header"))),
                true,
                false,
                &palette,
                &options,
            )
        }))
    } else {
        None
    };

    let body_rows = rows
        .into_iter()
        .enumerate()
        .map(|(row_index, row)| {
            let striped = options.striped && row_index % 2 == 1;
            let column_test_id_suffixes = column_test_id_suffixes.clone();
            cx.keyed(row.key.clone(), |cx| {
                let mut iter = row.cells.into_iter();
                let mut cells = Vec::with_capacity(columns.len());
                for (column_index, column) in columns.iter().enumerate() {
                    let built = iter.next().unwrap_or_else(|| BuiltTableCell {
                        test_id: None,
                        content: empty_cell(cx),
                    });
                    let test_id = row
                        .test_id
                        .as_ref()
                        .map(|base| {
                            Arc::from(format!(
                                "{base}.cell.{}",
                                column_test_id_suffixes[column_index]
                            ))
                        })
                        .or(built.test_id);
                    cells.push(wrap_table_cell(
                        cx,
                        column,
                        built.content,
                        test_id,
                        false,
                        striped,
                        &options,
                    ));
                }
                debug_assert!(
                    iter.next().is_none(),
                    "imui table rows must emit exactly one cell per declared column"
                );
                wrap_table_row(cx, cells, row.test_id, false, striped, &palette, &options)
            })
        })
        .collect::<Vec<_>>();

    let mut children = Vec::new();
    if let Some(header) = header {
        children.push(header);
    }
    children.extend(body_rows);

    let mut root = ContainerProps::default();
    root.layout.size.width = Length::Fill;
    root.layout.size.height = Length::Auto;
    root.background = Some(palette.table_bg);
    root.border = Edges::all(Px(1.0));
    root.border_color = Some(palette.border);
    root.corner_radii = Corners::all(Px(6.0));

    let table = cx.container(root, move |cx| {
        vec![
            crate::ui::v_flex(move |_cx| children)
                .gap_metric(options.row_gap.clone())
                .justify(crate::Justify::Start)
                .items(crate::Items::Stretch)
                .no_wrap()
                .into_element(cx),
        ]
    });

    let element = if let Some(test_id) = options.test_id {
        let mut semantics = SemanticsProps::default();
        semantics.role = SemanticsRole::Group;
        semantics.test_id = Some(test_id);
        cx.semantics(semantics, move |_cx| vec![table])
    } else {
        table
    };

    (
        element,
        TableResponse {
            headers: header_responses,
        },
    )
}

struct BuiltHeaderCell {
    element: AnyElement,
    trigger: ResponseExt,
}

#[derive(Default)]
struct TableResizeHandleDragState {
    was_dragging: bool,
}

fn wrap_table_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    cells: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    header: bool,
    striped: bool,
    palette: &TablePalette,
    options: &TableOptions,
) -> AnyElement {
    let background = if header {
        Some(palette.header_bg)
    } else if striped {
        Some(palette.striped_bg)
    } else {
        None
    };

    let mut row = ContainerProps::default();
    row.layout.size.width = Length::Fill;
    row.layout.size.height = Length::Auto;
    row.background = background;

    let row = cx.container(row, move |cx| {
        vec![
            crate::ui::h_flex(move |_cx| cells)
                .gap_metric(options.column_gap.clone())
                .justify(crate::Justify::Start)
                .items(crate::Items::Stretch)
                .no_wrap()
                .into_element(cx),
        ]
    });

    if let Some(test_id) = test_id {
        let mut semantics = SemanticsProps::default();
        semantics.role = SemanticsRole::Group;
        semantics.test_id = Some(test_id);
        cx.semantics(semantics, move |_cx| vec![row])
    } else {
        row
    }
}

fn column_test_id_suffix(column: &TableColumn, index: usize) -> String {
    column
        .id
        .as_deref()
        .map(test_id_slug)
        .filter(|slug| !slug.is_empty())
        .unwrap_or_else(|| index.to_string())
}

fn test_id_slug(s: &str) -> String {
    let mut out = String::new();
    let mut last_was_separator = false;

    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !out.is_empty() && !last_was_separator {
            out.push('-');
            last_was_separator = true;
        }
    }

    if out.ends_with('-') {
        out.pop();
    }

    out
}

fn visible_header_label(column: &TableColumn) -> Option<Arc<str>> {
    column.header.as_ref().map(|label| {
        let parts = parse_label_identity(label.as_ref());
        Arc::<str>::from(parts.visible)
    })
}

fn column_is_sortable(column: &TableColumn) -> bool {
    column.sortable || column.sort_direction.is_some()
}

fn sort_direction_indicator(direction: TableSortDirection) -> &'static str {
    match direction {
        TableSortDirection::Ascending => "^",
        TableSortDirection::Descending => "v",
    }
}

fn sort_direction_a11y_label(direction: TableSortDirection) -> &'static str {
    match direction {
        TableSortDirection::Ascending => "ascending",
        TableSortDirection::Descending => "descending",
    }
}

fn sortable_header_a11y_label(
    column: &TableColumn,
    visible_label: Option<&Arc<str>>,
    column_index: usize,
) -> Arc<str> {
    let base = visible_label
        .cloned()
        .or_else(|| column.id.clone())
        .unwrap_or_else(|| Arc::from(format!("Column {}", column_index + 1)));
    match column.sort_direction {
        Some(direction) => Arc::from(format!(
            "{base}, sorted {}",
            sort_direction_a11y_label(direction)
        )),
        None => Arc::from(format!("{base}, sortable")),
    }
}

fn wrap_sortable_header_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    column: &TableColumn,
    column_index: usize,
    visible_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    options: &TableOptions,
    resize_response: &mut TableColumnResizeResponse,
) -> BuiltHeaderCell {
    let mut trigger = ResponseExt::default();
    let column_key = column
        .id
        .clone()
        .unwrap_or_else(|| Arc::from(column_index.to_string()));
    let sort_direction = column.sort_direction;
    let a11y_label = sortable_header_a11y_label(column, visible_label.as_ref(), column_index);

    let trigger_element = cx.keyed(("sortable-header-cell", column_key), |cx| {
        let trigger = &mut trigger;
        let enabled = !super::imui_is_disabled(cx);
        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled;
        props.layout.size.width = Length::Fill;
        props.layout.flex.grow = 1.0;
        props.layout.flex.shrink = 1.0;
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::Button),
            label: Some(a11y_label.clone()),
            ..Default::default()
        };

        cx.pressable_with_id(props, move |cx, state, element_id| {
            let behavior = super::active_trigger_behavior::install_active_trigger_behavior(
                cx,
                element_id,
                super::active_trigger_behavior::ActiveTriggerBehaviorOptions::default(),
            );
            let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

            if enabled {
                cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
                    if reason == ActivateReason::Keyboard {
                        super::mark_lifecycle_instant_if_inactive(
                            host,
                            acx,
                            &lifecycle_model_for_activate,
                            false,
                        );
                    }
                    host.record_transient_event(acx, super::KEY_CLICKED);
                    host.notify(acx);
                }));
            }

            let clicked = cx.take_transient_for(element_id, super::KEY_CLICKED);
            super::active_trigger_behavior::populate_active_trigger_response(
                cx,
                element_id,
                state,
                &behavior,
                super::active_trigger_behavior::ActiveTriggerResponseInput {
                    enabled,
                    clicked,
                    changed: false,
                    lifecycle_edited: false,
                },
                trigger,
            );

            vec![sortable_header_visual(
                cx,
                visible_label.clone(),
                sort_direction,
                enabled,
                state,
            )]
        })
    });

    let element = wrap_table_header_cell(
        cx,
        column,
        column_index,
        trigger_element,
        test_id,
        options,
        resize_response,
    );

    BuiltHeaderCell { element, trigger }
}

fn sortable_header_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    visible_label: Option<Arc<str>>,
    sort_direction: Option<TableSortDirection>,
    enabled: bool,
    state: PressableState,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let hover_bg = if enabled && (state.hovered || state.focused || state.pressed) {
        Some(
            theme
                .color_by_key("muted")
                .unwrap_or_else(|| theme.color_token("muted")),
        )
    } else {
        None
    };
    let mut cell = ContainerProps::default();
    cell.layout.size.width = Length::Fill;
    cell.layout.size.height = Length::Auto;
    cell.padding = table_cell_padding().into();
    cell.background = hover_bg;

    cx.container(cell, move |cx| {
        let mut children = Vec::new();
        if let Some(label) = visible_label.clone() {
            children.push(cx.text(label));
        }
        if let Some(direction) = sort_direction {
            children.push(cx.text(Arc::<str>::from(sort_direction_indicator(direction))));
        }
        if children.is_empty() {
            Vec::new()
        } else if children.len() == 1 {
            children
        } else {
            vec![
                crate::ui::h_flex(move |_cx| children)
                    .gap_metric(crate::MetricRef::space(crate::Space::N1))
                    .justify(crate::Justify::Start)
                    .items(crate::Items::Center)
                    .no_wrap()
                    .into_element(cx),
            ]
        }
    })
}

fn wrap_plain_header_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    column: &TableColumn,
    column_index: usize,
    visible_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    options: &TableOptions,
    resize_response: &mut TableColumnResizeResponse,
) -> AnyElement {
    let content = visible_label
        .map(|label| cx.text(label))
        .unwrap_or_else(|| empty_cell(cx));
    let content = table_header_content_box(cx, content);
    wrap_table_header_cell(
        cx,
        column,
        column_index,
        content,
        test_id,
        options,
        resize_response,
    )
}

fn table_header_content_box<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    content: AnyElement,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Auto;
    props.layout.flex.grow = 1.0;
    props.layout.flex.shrink = 1.0;
    props.padding = table_cell_padding().into();
    cx.container(props, move |_cx| vec![content])
}

fn wrap_table_header_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    column: &TableColumn,
    column_index: usize,
    content: AnyElement,
    test_id: Option<Arc<str>>,
    options: &TableOptions,
    resize_response: &mut TableColumnResizeResponse,
) -> AnyElement {
    let resize_handle = column.resize.map(|_| {
        let handle_test_id = test_id
            .as_ref()
            .map(|base| Arc::from(format!("{base}.resize")));
        table_resize_handle(cx, column, column_index, resize_response, handle_test_id)
    });

    let mut cell = ContainerProps::default();
    cell.layout = table_cell_layout(column.width, options.clip_cells);

    let cell = cx.container(cell, move |cx| {
        let mut children = vec![content];
        if let Some(handle) = resize_handle {
            children.push(handle);
        }
        vec![
            crate::ui::h_flex(move |_cx| children)
                .gap_metric(crate::MetricRef::space(crate::Space::N0))
                .justify(crate::Justify::Start)
                .items(crate::Items::Stretch)
                .no_wrap()
                .into_element(cx),
        ]
    });

    if let Some(test_id) = test_id {
        cell.test_id(test_id)
    } else {
        cell
    }
}

fn table_resize_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    column: &TableColumn,
    column_index: usize,
    response: &mut TableColumnResizeResponse,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let column_key = column
        .id
        .clone()
        .or_else(|| column.header.clone())
        .unwrap_or_else(|| Arc::from(format!("column-{column_index}")));
    let enabled = !super::imui_is_disabled(cx);
    response.enabled = enabled;

    let handle = cx.keyed(("table-column-resize", column_key, column_index), |cx| {
        let mut props = PointerRegionProps::default();
        props.enabled = enabled;
        props.layout.size.width = Length::Px(TABLE_RESIZE_HANDLE_HIT_WIDTH);
        props.layout.size.height = Length::Auto;
        props.layout.size.min_height = Some(Length::Px(TABLE_RESIZE_HANDLE_MIN_HEIGHT));
        props.layout.flex.shrink = 0.0;

        cx.pointer_region(props, move |cx| {
            let region_id = cx.root_id();
            let drag_kind = super::drag_kind_for_element(region_id);
            let drag_threshold = super::drag_threshold_for(cx);

            cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
                super::prepare_pointer_region_drag_on_left_down(
                    host,
                    acx,
                    down,
                    enabled.then_some(drag_kind),
                    Some(CursorIcon::ColResize),
                )
            }));
            cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
                if !enabled {
                    return false;
                }
                host.set_cursor_icon(CursorIcon::ColResize);
                super::handle_pointer_region_drag_move_with_threshold(
                    host,
                    acx,
                    mv,
                    drag_kind,
                    drag_threshold,
                )
            }));
            cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
                if !enabled {
                    return false;
                }
                super::finish_pointer_region_drag(host, acx, up.pointer_id, drag_kind)
            }));

            let mut drag_response = ResponseExt::default();
            super::populate_pressable_drag_response(cx, region_id, &mut drag_response);
            response.drag = drag_response.drag;
            let dragging = response.drag.dragging;
            let (started, stopped) =
                cx.state_for(region_id, TableResizeHandleDragState::default, |state| {
                    let started = dragging && !state.was_dragging;
                    let stopped = !dragging && state.was_dragging;
                    state.was_dragging = dragging;
                    (started, stopped)
                });
            response.drag.started |= started;
            response.drag.stopped |= stopped;

            vec![table_resize_handle_visual(cx, enabled)]
        })
    });

    if let Some(test_id) = test_id {
        handle.test_id(test_id)
    } else {
        handle
    }
}

fn table_resize_handle_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let mut color = theme
        .color_by_key("table.border")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("border"));
    if !enabled {
        color.a *= 0.45;
    }

    let mut grip = ContainerProps::default();
    grip.background = Some(color);
    grip.layout.size.width = Length::Px(TABLE_RESIZE_HANDLE_VISUAL_WIDTH);
    grip.layout.size.height = Length::Px(TABLE_RESIZE_HANDLE_MIN_HEIGHT);
    grip.layout.flex.shrink = 0.0;

    crate::ui::h_flex(move |cx| vec![cx.container(grip, |_cx| Vec::new())])
        .gap_metric(crate::MetricRef::space(crate::Space::N0))
        .justify(crate::Justify::Center)
        .items(crate::Items::Stretch)
        .no_wrap()
        .into_element(cx)
}

fn wrap_table_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    column: &TableColumn,
    content: AnyElement,
    test_id: Option<Arc<str>>,
    header: bool,
    striped: bool,
    options: &TableOptions,
) -> AnyElement {
    let mut cell = ContainerProps::default();
    cell.layout = table_cell_layout(column.width, options.clip_cells);
    cell.padding = table_cell_padding().into();
    if header || striped {
        cell.background = None;
    }
    let cell = cx.container(cell, move |_cx| vec![content]);
    if let Some(test_id) = test_id {
        let mut semantics = SemanticsProps::default();
        semantics.role = if header {
            SemanticsRole::Heading
        } else {
            SemanticsRole::Group
        };
        semantics.test_id = Some(test_id);
        cx.semantics(semantics, move |_cx| vec![cell])
    } else {
        cell
    }
}

fn table_cell_padding() -> Edges {
    Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(4.0),
        bottom: Px(4.0),
    }
}

fn pack_cell_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
) -> AnyElement {
    match children.len() {
        0 => empty_cell(cx),
        1 => children.into_iter().next().expect("single cell child"),
        _ => crate::ui::v_flex(move |_cx| children)
            .gap_metric(crate::MetricRef::space(crate::Space::N0))
            .justify(crate::Justify::Start)
            .items(crate::Items::Stretch)
            .no_wrap()
            .into_element(cx),
    }
}

fn empty_cell<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.container(ContainerProps::default(), |_cx| Vec::new())
}

fn table_cell_layout(width: TableColumnWidth, clip_cells: bool) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.height = Length::Auto;
    if clip_cells {
        layout.overflow = Overflow::Clip;
    }

    match width {
        TableColumnWidth::Px(width) => {
            layout.size.width = Length::Px(width);
            layout.size.min_width = Some(Length::Px(width));
            layout.size.max_width = Some(Length::Px(width));
            layout.flex.shrink = 0.0;
        }
        TableColumnWidth::Fill(weight) => {
            let grow = if weight.is_finite() && weight > 0.0 {
                weight
            } else {
                1.0
            };
            layout.size.width = Length::Px(Px(0.0));
            layout.flex.grow = grow;
            layout.flex.shrink = 1.0;
            layout.flex.basis = Length::Px(Px(0.0));
        }
    }

    layout
}

struct TablePalette {
    table_bg: Color,
    border: Color,
    header_bg: Color,
    striped_bg: Color,
}

fn resolve_table_palette(theme: &Theme) -> TablePalette {
    let table_bg = theme
        .color_by_key("table.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_token("card"));
    let border = theme
        .color_by_key("table.border")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("border"));
    let header_bg = theme
        .color_by_key("table.header.background")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or_else(|| theme.color_token("muted"));
    let mut striped_bg = theme
        .color_by_key("table.row.striped")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or_else(|| theme.color_token("muted"));
    striped_bg.a *= 0.35;

    TablePalette {
        table_bg,
        border,
        header_bg,
        striped_bg,
    }
}
