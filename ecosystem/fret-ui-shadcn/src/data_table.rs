use std::sync::Arc;

use fret_core::geometry::Edges;
use fret_core::{Axis, Color, FontId, FontWeight, Px, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, Overflow};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::table::{
    TableRowMeasureMode, TableViewOutput, TableViewProps, table_virtualized,
};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, ui};

use fret_ui_headless::table::{
    ColumnDef, ColumnId, ColumnPinPosition, RowKey, SortSpec, TableState, pin_column,
    set_column_visible,
};

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::dropdown_menu::{
    DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem, DropdownMenuSide,
};

fn border_color(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn table_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.table.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.table.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn mixed_revision(a: u64, b: u64) -> u64 {
    // Cheap, deterministic mixing to avoid obvious collisions.
    a ^ b.wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

const COLUMN_ACTION_PREFIX: &str = "fret_ui_shadcn.data_table.column_action/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColumnAction {
    SortAsc,
    SortDesc,
    SortClear,
    Hide,
    PinLeft,
    PinRight,
    PinClear,
}

fn column_action_command(action: ColumnAction, column_id: &str) -> CommandId {
    let action_key = match action {
        ColumnAction::SortAsc => "sort_asc",
        ColumnAction::SortDesc => "sort_desc",
        ColumnAction::SortClear => "sort_clear",
        ColumnAction::Hide => "hide",
        ColumnAction::PinLeft => "pin_left",
        ColumnAction::PinRight => "pin_right",
        ColumnAction::PinClear => "pin_clear",
    };
    CommandId::new(format!("{COLUMN_ACTION_PREFIX}{action_key}/{column_id}"))
}

fn parse_column_action(command: &str) -> Option<(ColumnAction, &str)> {
    let rest = command.strip_prefix(COLUMN_ACTION_PREFIX)?;
    let (action_key, column_id) = rest.split_once('/')?;
    let action = match action_key {
        "sort_asc" => ColumnAction::SortAsc,
        "sort_desc" => ColumnAction::SortDesc,
        "sort_clear" => ColumnAction::SortClear,
        "hide" => ColumnAction::Hide,
        "pin_left" => ColumnAction::PinLeft,
        "pin_right" => ColumnAction::PinRight,
        "pin_clear" => ColumnAction::PinClear,
        _ => return None,
    };
    Some((action, column_id))
}

fn wire_column_actions_command_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state: Model<TableState>,
) {
    cx.command_on_command_for(
        cx.root_id(),
        Arc::new(move |host, acx, command| {
            let Some((action, column_id)) = parse_column_action(command.as_str()) else {
                return false;
            };

            let column_id: ColumnId = Arc::from(column_id);
            let _ = host.models_mut().update(&state, |st| {
                match action {
                    ColumnAction::SortAsc => {
                        st.sorting = vec![SortSpec {
                            column: column_id.clone(),
                            desc: false,
                        }];
                    }
                    ColumnAction::SortDesc => {
                        st.sorting = vec![SortSpec {
                            column: column_id.clone(),
                            desc: true,
                        }];
                    }
                    ColumnAction::SortClear => {
                        st.sorting
                            .retain(|s| s.column.as_ref() != column_id.as_ref());
                    }
                    ColumnAction::Hide => {
                        set_column_visible(&mut st.column_visibility, &column_id, false);
                    }
                    ColumnAction::PinLeft => {
                        pin_column(
                            &mut st.column_pinning,
                            &column_id,
                            Some(ColumnPinPosition::Left),
                        );
                    }
                    ColumnAction::PinRight => {
                        pin_column(
                            &mut st.column_pinning,
                            &column_id,
                            Some(ColumnPinPosition::Right),
                        );
                    }
                    ColumnAction::PinClear => {
                        pin_column(&mut st.column_pinning, &column_id, None);
                    }
                }
                st.pagination.page_index = 0;
            });

            host.request_redraw(acx.window);
            true
        }),
    );
}

fn menu_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let open = cx.with_state(|| None::<Model<bool>>, |st| st.clone());
    match open {
        Some(open) => open,
        None => {
            let open = cx.app.models_mut().insert(false);
            cx.with_state(
                || None::<Model<bool>>,
                |st| {
                    if st.is_none() {
                        *st = Some(open.clone());
                    }
                },
            );
            open
        }
    }
}

fn render_column_actions_menu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state: Model<TableState>,
    column_id: Arc<str>,
    can_sort: bool,
    can_hide: bool,
    can_pin: bool,
) -> AnyElement {
    cx.keyed(("data-table-column-actions", column_id.as_ref()), |cx| {
        let open = menu_open_model(cx);

        let trigger_label: Arc<str> =
            Arc::from(format!("Column actions for {}", column_id.as_ref()));

        let column_id_for_items = column_id.clone();
        let state_for_items = state.clone();
        DropdownMenu::new(open)
            .align(DropdownMenuAlign::End)
            .side(DropdownMenuSide::Bottom)
            .into_element(
                cx,
                |cx| {
                    Button::new(trigger_label.clone())
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .children([cx.text("⋯")])
                        .into_element(cx)
                },
                move |cx| {
                    wire_column_actions_command_handler(cx, state_for_items.clone());

                    let id = column_id_for_items.as_ref();
                    let mut entries: Vec<DropdownMenuEntry> = Vec::new();

                    entries.push(DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Sort Asc")
                            .disabled(!can_sort)
                            .on_select(column_action_command(ColumnAction::SortAsc, id)),
                    ));
                    entries.push(DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Sort Desc")
                            .disabled(!can_sort)
                            .on_select(column_action_command(ColumnAction::SortDesc, id)),
                    ));
                    entries.push(DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Clear sort")
                            .disabled(!can_sort)
                            .on_select(column_action_command(ColumnAction::SortClear, id)),
                    ));

                    entries.push(DropdownMenuEntry::Separator);

                    entries.push(DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Hide")
                            .disabled(!can_hide)
                            .on_select(column_action_command(ColumnAction::Hide, id)),
                    ));

                    entries.push(DropdownMenuEntry::Separator);

                    entries.push(DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Pin Left")
                            .disabled(!can_pin)
                            .on_select(column_action_command(ColumnAction::PinLeft, id)),
                    ));
                    entries.push(DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Pin Right")
                            .disabled(!can_pin)
                            .on_select(column_action_command(ColumnAction::PinRight, id)),
                    ));
                    entries.push(DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Unpin")
                            .disabled(!can_pin)
                            .on_select(column_action_command(ColumnAction::PinClear, id)),
                    ));

                    entries
                },
            )
    })
}

/// shadcn/ui `DataTable` backed by the TanStack-aligned headless engine (ADR 0101).
///
/// This is an integration surface:
/// - headless row model: filtering/sorting/pagination/visibility (future: sizing/pinning)
/// - UI: `Table` primitives + fixed header + virtualized body
///
/// Prefer `DataTable` for "business table" use-cases (toolbars, filters, pagination, column
/// visibility). For spreadsheet-scale density, prefer the canvas-backed [`crate::DataGrid`].
///
/// Notes (v0):
/// - row activation toggles selection by `RowKey` (flat tables; sub-row selection is deferred)
/// - header activation toggles sorting (Shift-click appends/toggles multi-sort, TanStack-style)
#[derive(Debug, Clone)]
pub struct DataTable {
    overscan: usize,
    keep_alive: Option<usize>,
    row_height: Option<Px>,
    measure_rows: bool,
    column_actions_menu: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    output: Option<Model<TableViewOutput>>,
}

impl Default for DataTable {
    fn default() -> Self {
        Self {
            overscan: 4,
            keep_alive: None,
            row_height: None,
            measure_rows: false,
            column_actions_menu: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            output: None,
        }
    }
}

impl DataTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }

    pub fn keep_alive(mut self, keep_alive: usize) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }

    pub fn row_height(mut self, row_height: Px) -> Self {
        self.row_height = Some(row_height);
        self
    }

    /// Enables measured (variable-height) body rows.
    ///
    /// This is intended for content-driven row heights (e.g. wrapping Markdown).
    /// The default remains fixed-height rows for performance.
    pub fn measure_rows(mut self, enabled: bool) -> Self {
        self.measure_rows = enabled;
        self
    }

    /// Enables a TanStack-style per-column header actions menu (sort/hide/pin).
    pub fn column_actions_menu(mut self, enabled: bool) -> Self {
        self.column_actions_menu = enabled;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn output_model(mut self, output: Model<TableViewOutput>) -> Self {
        self.output = Some(output);
        self
    }

    /// A retained-host variant of [`Self::into_element`] that enables composable body rows under
    /// cache-root reuse (virt-003 / ADR 0192).
    ///
    /// Notes (v0):
    /// - supports fixed-height and measured (variable-height) rows
    /// - intended for perf/correctness harnesses; API stability is not guaranteed
    pub fn into_element_retained<H: UiHost + 'static, TData>(
        self,
        cx: &mut ElementContext<'_, H>,
        data: Arc<[TData]>,
        data_revision: u64,
        state: Model<TableState>,
        columns: impl Into<Arc<[ColumnDef<TData>]>>,
        get_row_key: impl Fn(&TData, usize, Option<&RowKey>) -> RowKey + 'static,
        header_label: impl Fn(&ColumnDef<TData>) -> Arc<str> + 'static,
        cell_at: impl Fn(&mut ElementContext<'_, H>, &ColumnDef<TData>, &TData) -> AnyElement + 'static,
        debug_header_cell_test_id_prefix: Option<Arc<str>>,
        debug_row_test_id_prefix: Option<Arc<str>>,
    ) -> AnyElement
    where
        TData: 'static,
    {
        let DataTable {
            overscan,
            keep_alive,
            row_height,
            measure_rows,
            column_actions_menu,
            chrome,
            layout,
            output: _output,
        } = self;

        let theme = Theme::global(&*cx.app).clone();
        let border = border_color(&theme);

        let state_revision = state.revision(&*cx.app).unwrap_or(0);
        let items_revision = mixed_revision(data_revision, state_revision);

        let columns: Arc<[ColumnDef<TData>]> = columns.into();

        let root_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(border))
            .merge(chrome);
        let mut root_props = decl_style::container_props(&theme, root_chrome, layout.w_full());
        root_props.layout.overflow = Overflow::Clip;

        cx.container(root_props, move |cx| {
            let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

            let get_row_key = Arc::new(get_row_key);
            let header_label = Arc::new(header_label);
            let cell_at = Arc::new(cell_at);

            let header_accessory_at: Option<
                Arc<dyn Fn(&mut ElementContext<'_, H>, &ColumnDef<TData>) -> AnyElement>,
            > = if column_actions_menu {
                let state_for_actions = state.clone();
                Some(Arc::new(
                    move |cx: &mut ElementContext<'_, H>, col: &ColumnDef<TData>| {
                        render_column_actions_menu(
                            cx,
                            state_for_actions.clone(),
                            Arc::<str>::from(col.id.as_ref()),
                            col.enable_sorting,
                            col.enable_hiding,
                            col.enable_pinning,
                        )
                    },
                ))
            } else {
                None
            };

            let mut view_props = TableViewProps::default();
            view_props.overscan = overscan;
            view_props.keep_alive = keep_alive;
            view_props.row_height = row_height;
            view_props.row_measure_mode = if measure_rows {
                TableRowMeasureMode::Measured
            } else {
                TableRowMeasureMode::Fixed
            };
            view_props.enable_column_grouping = false;
            view_props.enable_column_resizing = false;
            view_props.draw_frame = false;

            let row_key_at = Arc::new(move |d: &TData, index: usize| (get_row_key)(d, index, None));

            vec![
                fret_ui_kit::declarative::table::table_virtualized_retained_v0(
                    cx,
                    data.clone(),
                    columns.clone(),
                    state.clone(),
                    &scroll_handle,
                    items_revision,
                    row_key_at,
                    None,
                    view_props,
                    header_label,
                    header_accessory_at,
                    cell_at,
                    debug_header_cell_test_id_prefix,
                    debug_row_test_id_prefix,
                ),
            ]
        })
    }

    pub fn into_element<H: UiHost, TData>(
        self,
        cx: &mut ElementContext<'_, H>,
        data: Arc<[TData]>,
        data_revision: u64,
        state: Model<TableState>,
        columns: impl Into<Arc<[ColumnDef<TData>]>>,
        get_row_key: impl Fn(&TData, usize, Option<&RowKey>) -> RowKey + 'static,
        header_label: impl Fn(&ColumnDef<TData>) -> Arc<str> + 'static,
        cell_at: impl Fn(&mut ElementContext<'_, H>, &ColumnDef<TData>, &TData) -> AnyElement + 'static,
    ) -> AnyElement
    where
        TData: 'static,
    {
        let DataTable {
            overscan,
            keep_alive,
            row_height,
            measure_rows,
            column_actions_menu,
            chrome,
            layout,
            output,
        } = self;

        let theme = Theme::global(&*cx.app).clone();
        let border = border_color(&theme);

        let state_revision = state.revision(&*cx.app).unwrap_or(0);
        let items_revision = mixed_revision(data_revision, state_revision);

        let columns: Arc<[ColumnDef<TData>]> = columns.into();

        let root_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(border))
            .merge(chrome);
        let mut root_props = decl_style::container_props(&theme, root_chrome, layout.w_full());
        root_props.layout.overflow = Overflow::Clip;

        let root = cx.container(root_props, move |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

            let header_style = TextStyle {
                weight: FontWeight::MEDIUM,
                ..table_text_style(&theme)
            };
            let header_fg = theme.color_required("foreground");
            let sort_fg = theme.color_required("muted-foreground");

            let get_row_key = Arc::new(get_row_key);
            let header_label = Arc::new(header_label);
            let cell_at = Arc::new(cell_at);

            let mut view_props = TableViewProps::default();
            view_props.overscan = overscan;
            view_props.keep_alive = keep_alive;
            view_props.row_height = row_height;
            view_props.row_measure_mode = if measure_rows {
                TableRowMeasureMode::Measured
            } else {
                TableRowMeasureMode::Fixed
            };
            view_props.enable_column_grouping = false;
            view_props.enable_column_resizing = true;
            view_props.draw_frame = false;

            let row_key_at = move |d: &TData, index: usize| (get_row_key)(d, index, None);

            let state_for_header = state.clone();
            let state_for_column_actions_header = state.clone();
            let column_actions_menu_enabled = column_actions_menu;
            let columns = columns.clone();
            let state = state.clone();
            let data = data.clone();
            let table = table_virtualized(
                cx,
                data.as_ref(),
                columns.as_ref(),
                state,
                &scroll_handle,
                items_revision,
                &row_key_at,
                None,
                view_props,
                |_row| None,
                move |cx, col, sort_state| {
                    if !column_actions_menu_enabled {
                        let theme = Theme::global(&*cx.app).clone();
                        let label = (header_label)(col);
                        let style = header_style.clone();
                        let header_fg = header_fg;
                        let sort_fg = sort_fg;
                        let state_for_header = state_for_header.clone();
                        return vec![cx.flex(
                            FlexProps {
                                layout: decl_style::layout_style(
                                    &theme,
                                    LayoutRefinement::default().w_full().h_full(),
                                ),
                                direction: Axis::Horizontal,
                                gap: Px(2.0),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::Start,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            move |cx| {
                                let mut pieces: Vec<AnyElement> = Vec::new();
                                {
                                    let mut text = ui::label(cx, label.clone())
                                        .text_size_px(style.size)
                                        .font_weight(style.weight)
                                        .text_color(ColorRef::Color(header_fg))
                                        .nowrap();
                                    if let Some(line_height) = style.line_height {
                                        text = text.line_height_px(line_height);
                                    }
                                    if let Some(letter_spacing_em) = style.letter_spacing_em {
                                        text = text.letter_spacing_em(letter_spacing_em);
                                    }
                                    pieces.push(text.into_element(cx));
                                }

                                if let Some(desc) = sort_state {
                                    let sorting = cx
                                        .app
                                        .models()
                                        .read(&state_for_header, |st| st.sorting.clone())
                                        .ok()
                                        .unwrap_or_default();
                                    let order = if sorting.len() > 1 {
                                        sorting
                                            .iter()
                                            .position(|s| s.column.as_ref() == col.id.as_ref())
                                            .map(|idx| idx + 1)
                                    } else {
                                        None
                                    };
                                    let indicator: Arc<str> = match order {
                                        Some(order) => Arc::<str>::from(format!(
                                            "{}{order}",
                                            if desc { "▼" } else { "▲" }
                                        )),
                                        None => Arc::from(if desc { "▼" } else { "▲" }),
                                    };
                                    let mut text = ui::label(cx, indicator)
                                        .text_size_px(style.size)
                                        .font_weight(style.weight)
                                        .text_color(ColorRef::Color(sort_fg))
                                        .nowrap();
                                    if let Some(line_height) = style.line_height {
                                        text = text.line_height_px(line_height);
                                    }
                                    if let Some(letter_spacing_em) = style.letter_spacing_em {
                                        text = text.letter_spacing_em(letter_spacing_em);
                                    }
                                    pieces.push(text.into_element(cx));
                                }

                                pieces
                            },
                        )];
                    }

                    let theme = Theme::global(&*cx.app).clone();
                    let label = (header_label)(col);
                    let style = header_style.clone();
                    let header_fg = header_fg;
                    let sort_fg = sort_fg;
                    let state_for_header = state_for_header.clone();
                    let state_for_column_actions = state_for_column_actions_header.clone();
                    let col_id: Arc<str> = Arc::from(col.id.as_ref());
                    let can_hide = col.enable_hiding;
                    let can_pin = col.enable_pinning;
                    vec![cx.flex(
                        FlexProps {
                            layout: decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default().w_full().h_full(),
                            ),
                            direction: Axis::Horizontal,
                            gap: Px(2.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| {
                            let mut pieces: Vec<AnyElement> = Vec::new();
                            {
                                let mut text = ui::label(cx, label.clone())
                                    .text_size_px(style.size)
                                    .font_weight(style.weight)
                                    .text_color(ColorRef::Color(header_fg))
                                    .nowrap();
                                if let Some(line_height) = style.line_height {
                                    text = text.line_height_px(line_height);
                                }
                                if let Some(letter_spacing_em) = style.letter_spacing_em {
                                    text = text.letter_spacing_em(letter_spacing_em);
                                }
                                pieces.push(text.into_element(cx));
                            }

                            if let Some(desc) = sort_state {
                                let sorting = cx
                                    .app
                                    .models()
                                    .read(&state_for_header, |st| st.sorting.clone())
                                    .ok()
                                    .unwrap_or_default();
                                let order = if sorting.len() > 1 {
                                    sorting
                                        .iter()
                                        .position(|s| s.column.as_ref() == col.id.as_ref())
                                        .map(|idx| idx + 1)
                                } else {
                                    None
                                };
                                let indicator: Arc<str> = match order {
                                    Some(order) => Arc::<str>::from(format!(
                                        "{}{order}",
                                        if desc { "▼" } else { "▲" }
                                    )),
                                    None => Arc::from(if desc { "▼" } else { "▲" }),
                                };
                                let mut text = ui::label(cx, indicator)
                                    .text_size_px(style.size)
                                    .font_weight(style.weight)
                                    .text_color(ColorRef::Color(sort_fg))
                                    .nowrap();
                                if let Some(line_height) = style.line_height {
                                    text = text.line_height_px(line_height);
                                }
                                if let Some(letter_spacing_em) = style.letter_spacing_em {
                                    text = text.letter_spacing_em(letter_spacing_em);
                                }
                                pieces.push(text.into_element(cx));
                            }

                            pieces.push(cx.spacer(fret_ui::element::SpacerProps::default()));

                            pieces.push(render_column_actions_menu(
                                cx,
                                state_for_column_actions.clone(),
                                col_id.clone(),
                                col.enable_sorting,
                                can_hide,
                                can_pin,
                            ));

                            pieces
                        },
                    )]
                },
                move |cx, row, col| vec![(cell_at)(cx, col, row.original)],
                output,
            );

            vec![table]
        });

        root
    }
}
