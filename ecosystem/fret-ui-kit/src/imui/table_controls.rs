use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, Overflow, SemanticsProps};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};

use super::containers::build_imui_children_with_focus;
use super::{
    ImUiFacade, TableColumn, TableColumnWidth, TableOptions, TableRowOptions, UiWriterImUiFacadeExt,
};

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
) -> AnyElement {
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
) -> AnyElement {
    let palette = resolve_table_palette(Theme::global(&*cx.app));
    let root_test_id = options.test_id.clone();
    let show_header = options.show_header && columns.iter().any(|column| column.header.is_some());
    let header = show_header.then(|| {
        cx.keyed(format!("{id}.header"), |cx| {
            let cells = columns
                .iter()
                .enumerate()
                .map(|(index, column)| {
                    let content = match column.header.as_ref() {
                        Some(label) => cx.text(label.clone()),
                        None => empty_cell(cx),
                    };
                    let test_id = root_test_id
                        .as_ref()
                        .map(|base| Arc::from(format!("{base}.header.cell.{index}")));
                    wrap_table_cell(cx, column, content, test_id, true, false, &options)
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
        })
    });

    let body_rows = rows
        .into_iter()
        .enumerate()
        .map(|(row_index, row)| {
            let striped = options.striped && row_index % 2 == 1;
            cx.keyed(row.key.clone(), |cx| {
                let mut iter = row.cells.into_iter();
                let mut cells = Vec::with_capacity(columns.len());
                for column in &columns {
                    let built = iter.next().unwrap_or_else(|| BuiltTableCell {
                        test_id: None,
                        content: empty_cell(cx),
                    });
                    cells.push(wrap_table_cell(
                        cx,
                        column,
                        built.content,
                        built.test_id,
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

    if let Some(test_id) = options.test_id {
        let mut semantics = SemanticsProps::default();
        semantics.role = SemanticsRole::Group;
        semantics.test_id = Some(test_id);
        cx.semantics(semantics, move |_cx| vec![table])
    } else {
        table
    }
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
    cell.padding = Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(4.0),
        bottom: Px(4.0),
    }
    .into();
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
