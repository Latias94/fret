use crate::data::DataTable;
use crate::transform::RowSelection;

#[derive(Debug, Clone)]
pub struct DataTableView<'a> {
    table: &'a DataTable,
    selection: RowSelection,
}

impl<'a> DataTableView<'a> {
    pub fn new(table: &'a DataTable, selection: RowSelection) -> Self {
        Self { table, selection }
    }

    pub fn table(&self) -> &'a DataTable {
        self.table
    }

    pub fn selection(&self) -> &RowSelection {
        &self.selection
    }

    pub fn raw_len(&self) -> usize {
        self.table.row_count()
    }

    pub fn len(&self) -> usize {
        self.selection.view_len(self.table.row_count())
    }

    pub fn get_raw_index(&self, view_index: usize) -> Option<usize> {
        self.selection
            .get_raw_index(self.table.row_count(), view_index)
    }

    pub fn column_f64(&self, index: usize) -> Option<&'a [f64]> {
        self.table.column_f64(index)
    }
}
