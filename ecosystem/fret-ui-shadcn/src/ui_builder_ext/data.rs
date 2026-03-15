use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::{IntoUiElement, UiBuilder};

use fret_ui_headless::table::{ColumnDef, RowKey, TableState};

use crate::data_grid_canvas::DataGridCanvas;
use crate::data_table::DataTable;
use crate::experimental::{DataGridElement, DataGridRowState};

pub trait DataGridCanvasUiBuilderExt {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        cell_text_at: impl Fn(u64, u64) -> Arc<str> + Send + Sync + 'static,
    ) -> AnyElement;
}

impl DataGridCanvasUiBuilderExt for UiBuilder<DataGridCanvas> {
    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        cell_text_at: impl Fn(u64, u64) -> Arc<str> + Send + Sync + 'static,
    ) -> AnyElement {
        self.build().into_element(cx, cell_text_at)
    }
}

pub trait DataGridElementUiBuilderExt {
    fn into_element<H: UiHost, FRowKey, FRowState, FCell, TCell>(
        self,
        cx: &mut ElementContext<'_, H>,
        rows_revision: u64,
        cols_revision: u64,
        row_key_at: FRowKey,
        row_state_at: FRowState,
        cell_at: FCell,
    ) -> AnyElement
    where
        FRowKey: FnMut(usize) -> u64,
        FRowState: FnMut(usize) -> DataGridRowState,
        FCell: FnMut(&mut ElementContext<'_, H>, usize, usize) -> TCell,
        TCell: IntoUiElement<H>;
}

impl DataGridElementUiBuilderExt for UiBuilder<DataGridElement> {
    fn into_element<H: UiHost, FRowKey, FRowState, FCell, TCell>(
        self,
        cx: &mut ElementContext<'_, H>,
        rows_revision: u64,
        cols_revision: u64,
        row_key_at: FRowKey,
        row_state_at: FRowState,
        cell_at: FCell,
    ) -> AnyElement
    where
        FRowKey: FnMut(usize) -> u64,
        FRowState: FnMut(usize) -> DataGridRowState,
        FCell: FnMut(&mut ElementContext<'_, H>, usize, usize) -> TCell,
        TCell: IntoUiElement<H>,
    {
        let mut cell_at = cell_at;
        self.build().into_element(
            cx,
            rows_revision,
            cols_revision,
            row_key_at,
            row_state_at,
            move |cx, row, col| cell_at(cx, row, col).into_element(cx),
        )
    }
}

pub trait DataTableUiBuilderExt {
    #[allow(clippy::too_many_arguments)]
    fn into_element<H: UiHost, TData, TCell>(
        self,
        cx: &mut ElementContext<'_, H>,
        data: Arc<[TData]>,
        data_revision: u64,
        state: Model<TableState>,
        columns: impl Into<Arc<[ColumnDef<TData>]>>,
        get_row_key: impl Fn(&TData, usize, Option<&RowKey>) -> RowKey + 'static,
        header_label: impl Fn(&ColumnDef<TData>) -> Arc<str> + 'static,
        cell_at: impl Fn(&mut ElementContext<'_, H>, &ColumnDef<TData>, &TData) -> TCell + 'static,
    ) -> AnyElement
    where
        TData: 'static,
        TCell: IntoUiElement<H>;
}

impl DataTableUiBuilderExt for UiBuilder<DataTable> {
    #[allow(clippy::too_many_arguments)]
    fn into_element<H: UiHost, TData, TCell>(
        self,
        cx: &mut ElementContext<'_, H>,
        data: Arc<[TData]>,
        data_revision: u64,
        state: Model<TableState>,
        columns: impl Into<Arc<[ColumnDef<TData>]>>,
        get_row_key: impl Fn(&TData, usize, Option<&RowKey>) -> RowKey + 'static,
        header_label: impl Fn(&ColumnDef<TData>) -> Arc<str> + 'static,
        cell_at: impl Fn(&mut ElementContext<'_, H>, &ColumnDef<TData>, &TData) -> TCell + 'static,
    ) -> AnyElement
    where
        TData: 'static,
        TCell: IntoUiElement<H>,
    {
        self.build().into_element(
            cx,
            data,
            data_revision,
            state,
            columns,
            get_row_key,
            header_label,
            move |cx, column, row| cell_at(cx, column, row).into_element(cx),
        )
    }
}
