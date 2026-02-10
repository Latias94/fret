#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::ids::{DatasetId, Revision, StringId};

mod table_view;

pub use table_view::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataTableAppendError {
    ColumnCountMismatch {
        expected: usize,
        actual: usize,
    },
    ColumnLenMismatch {
        expected: usize,
        actual: usize,
        column: usize,
    },
    NonF64Column {
        column: usize,
    },
}

impl core::fmt::Display for DataTableAppendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ColumnCountMismatch { expected, actual } => write!(
                f,
                "column count mismatch: expected {expected} values, got {actual}"
            ),
            Self::ColumnLenMismatch {
                expected,
                actual,
                column,
            } => write!(
                f,
                "column length mismatch at column {column}: expected {expected} rows, got {actual}"
            ),
            Self::NonF64Column { column } => write!(f, "column {column} is not an f64 column"),
        }
    }
}

impl std::error::Error for DataTableAppendError {}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Column {
    F64(Vec<f64>),
    I64(Vec<i64>),
    U64(Vec<u64>),
    Bool(Vec<bool>),
    String(Vec<StringId>),
}

impl Column {
    pub fn len(&self) -> usize {
        match self {
            Self::F64(v) => v.len(),
            Self::I64(v) => v.len(),
            Self::U64(v) => v.len(),
            Self::Bool(v) => v.len(),
            Self::String(v) => v.len(),
        }
    }

    pub fn as_f64_slice(&self) -> Option<&[f64]> {
        match self {
            Self::F64(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataTable {
    revision: Revision,
    columns: Vec<Column>,
    row_count: usize,
}

impl DataTable {
    pub fn revision(&self) -> Revision {
        self.revision
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn clear(&mut self) {
        self.columns.clear();
        self.row_count = 0;
        self.revision.bump();
    }

    pub fn push_column(&mut self, column: Column) {
        let len = column.len();
        if self.row_count == 0 {
            self.row_count = len;
        } else {
            self.row_count = self.row_count.min(len);
        }
        self.columns.push(column);
        self.revision.bump();
    }

    pub fn append_column(&mut self, column: Column) {
        self.push_column(column);
    }

    /// Appends a single row to a table that contains only f64 columns.
    ///
    /// This is the recommended mutation API for streaming/real-time datasets because it:
    /// - keeps `row_count` consistent,
    /// - bumps `revision` so dependent caches can invalidate,
    /// - enforces a single, deterministic append path.
    pub fn append_row_f64(&mut self, row: &[f64]) -> Result<(), DataTableAppendError> {
        if row.len() != self.columns.len() {
            return Err(DataTableAppendError::ColumnCountMismatch {
                expected: self.columns.len(),
                actual: row.len(),
            });
        }

        for (i, col) in self.columns.iter_mut().enumerate() {
            let Column::F64(v) = col else {
                return Err(DataTableAppendError::NonF64Column { column: i });
            };
            v.push(row[i]);
        }

        self.row_count = self.columns.iter().map(|c| c.len()).min().unwrap_or(0);
        self.revision.bump();
        Ok(())
    }

    /// Appends multiple rows to a table that contains only f64 columns.
    ///
    /// `columns` is column-major: each slice corresponds to one f64 column and all slices must have
    /// the same length.
    ///
    /// This is the preferred ingestion path for append-only/streaming charts because it avoids
    /// per-row overhead while keeping the contract identical to `append_row_f64`:
    /// - bumps `revision` exactly once,
    /// - keeps `row_count` consistent,
    /// - provides deterministic invalidation for dependent caches.
    pub fn append_columns_f64(&mut self, columns: &[&[f64]]) -> Result<(), DataTableAppendError> {
        if columns.len() != self.columns.len() {
            return Err(DataTableAppendError::ColumnCountMismatch {
                expected: self.columns.len(),
                actual: columns.len(),
            });
        }

        let expected_len = columns.first().map(|c| c.len()).unwrap_or(0);
        for (i, col) in columns.iter().enumerate() {
            if col.len() != expected_len {
                return Err(DataTableAppendError::ColumnLenMismatch {
                    expected: expected_len,
                    actual: col.len(),
                    column: i,
                });
            }
        }

        for (i, col) in self.columns.iter_mut().enumerate() {
            let Column::F64(v) = col else {
                return Err(DataTableAppendError::NonF64Column { column: i });
            };
            v.extend_from_slice(columns[i]);
        }

        self.row_count = self.columns.iter().map(|c| c.len()).min().unwrap_or(0);
        self.revision.bump();
        Ok(())
    }

    /// Updates a single row in a table that contains only f64 columns.
    ///
    /// v1 semantics:
    /// - row count is unchanged (in-place value updates only),
    /// - `revision` bumps once on success,
    /// - callers must use this explicit API (no implicit column mutation contract).
    pub fn update_row_f64(
        &mut self,
        row_index: usize,
        row: &[f64],
    ) -> Result<(), DataTableUpdateError> {
        if row_index >= self.row_count {
            return Err(DataTableUpdateError::RowOutOfBounds {
                row: row_index,
                row_count: self.row_count,
            });
        }
        if row.len() != self.columns.len() {
            return Err(DataTableUpdateError::ColumnCountMismatch {
                expected: self.columns.len(),
                actual: row.len(),
            });
        }

        for (i, col) in self.columns.iter_mut().enumerate() {
            let Column::F64(v) = col else {
                return Err(DataTableUpdateError::NonF64Column { column: i });
            };
            if let Some(slot) = v.get_mut(row_index) {
                *slot = row[i];
            } else {
                return Err(DataTableUpdateError::RowOutOfBounds {
                    row: row_index,
                    row_count: v.len().min(self.row_count),
                });
            }
        }

        self.revision.bump();
        Ok(())
    }

    /// Updates multiple rows in a table that contains only f64 columns.
    ///
    /// `columns` is column-major: each slice corresponds to one f64 column and all slices must have
    /// the same length. The update writes into `row_start..row_start+len`.
    pub fn update_columns_f64(
        &mut self,
        row_start: usize,
        columns: &[&[f64]],
    ) -> Result<(), DataTableUpdateError> {
        if columns.len() != self.columns.len() {
            return Err(DataTableUpdateError::ColumnCountMismatch {
                expected: self.columns.len(),
                actual: columns.len(),
            });
        }

        let len = columns.first().map(|c| c.len()).unwrap_or(0);
        for (i, col) in columns.iter().enumerate() {
            if col.len() != len {
                return Err(DataTableUpdateError::ColumnLenMismatch {
                    expected: len,
                    actual: col.len(),
                    column: i,
                });
            }
        }

        if row_start > self.row_count || row_start.saturating_add(len) > self.row_count {
            return Err(DataTableUpdateError::RowRangeOutOfBounds {
                row_start,
                len,
                row_count: self.row_count,
            });
        }

        for (i, col) in self.columns.iter_mut().enumerate() {
            let Column::F64(v) = col else {
                return Err(DataTableUpdateError::NonF64Column { column: i });
            };
            let end = row_start + len;
            v[row_start..end].copy_from_slice(columns[i]);
        }

        self.revision.bump();
        Ok(())
    }

    pub fn column_f64(&self, index: usize) -> Option<&[f64]> {
        self.columns
            .get(index)?
            .as_f64_slice()
            .map(|v| &v[..self.row_count])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataTableUpdateError {
    RowOutOfBounds {
        row: usize,
        row_count: usize,
    },
    RowRangeOutOfBounds {
        row_start: usize,
        len: usize,
        row_count: usize,
    },
    ColumnCountMismatch {
        expected: usize,
        actual: usize,
    },
    ColumnLenMismatch {
        expected: usize,
        actual: usize,
        column: usize,
    },
    NonF64Column {
        column: usize,
    },
}

impl core::fmt::Display for DataTableUpdateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::RowOutOfBounds { row, row_count } => {
                write!(
                    f,
                    "row index {row} is out of bounds (row_count={row_count})"
                )
            }
            Self::RowRangeOutOfBounds {
                row_start,
                len,
                row_count,
            } => write!(
                f,
                "row range {row_start}..{end} is out of bounds (row_count={row_count})",
                end = row_start.saturating_add(*len)
            ),
            Self::ColumnCountMismatch { expected, actual } => write!(
                f,
                "column count mismatch (expected {expected}, got {actual})"
            ),
            Self::ColumnLenMismatch {
                expected,
                actual,
                column,
            } => write!(
                f,
                "column len mismatch at index {column} (expected {expected}, got {actual})"
            ),
            Self::NonF64Column { column } => write!(f, "non-f64 column at index {column}"),
        }
    }
}

impl std::error::Error for DataTableUpdateError {}

#[cfg(test)]
mod tests {
    use crate::transform::RowSelection;

    use super::*;

    #[test]
    fn append_row_f64_bumps_revision_and_row_count() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::F64(vec![2.0]));

        let rev0 = table.revision();
        table.append_row_f64(&[3.0, 4.0]).unwrap();

        assert!(table.revision().0 > rev0.0);
        assert_eq!(table.row_count(), 2);
        assert_eq!(table.column_f64(0).unwrap(), &[1.0, 3.0]);
        assert_eq!(table.column_f64(1).unwrap(), &[2.0, 4.0]);
    }

    #[test]
    fn append_row_f64_rejects_wrong_width() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0]));
        let err = table.append_row_f64(&[1.0, 2.0]).unwrap_err();
        assert!(matches!(
            err,
            DataTableAppendError::ColumnCountMismatch { .. }
        ));
    }

    #[test]
    fn append_row_f64_rejects_non_f64_columns() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::Bool(vec![true]));

        let err = table.append_row_f64(&[2.0, 3.0]).unwrap_err();
        assert!(matches!(
            err,
            DataTableAppendError::NonF64Column { column: 1 }
        ));
    }

    #[test]
    fn append_columns_f64_appends_all_rows_and_bumps_revision_once() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::F64(vec![2.0]));

        let rev0 = table.revision();
        table
            .append_columns_f64(&[&[3.0, 5.0], &[4.0, 6.0]])
            .unwrap();

        assert!(table.revision().0 > rev0.0);
        assert_eq!(table.row_count(), 3);
        assert_eq!(table.column_f64(0).unwrap(), &[1.0, 3.0, 5.0]);
        assert_eq!(table.column_f64(1).unwrap(), &[2.0, 4.0, 6.0]);
    }

    #[test]
    fn update_row_f64_updates_values_and_bumps_revision() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0, 2.0, 3.0]));
        table.push_column(Column::F64(vec![10.0, 20.0, 30.0]));

        let rev0 = table.revision();
        table.update_row_f64(1, &[200.0, 999.0]).unwrap();

        assert!(table.revision().0 > rev0.0);
        assert_eq!(table.row_count(), 3);
        assert_eq!(table.column_f64(0).unwrap(), &[1.0, 200.0, 3.0]);
        assert_eq!(table.column_f64(1).unwrap(), &[10.0, 999.0, 30.0]);
    }

    #[test]
    fn update_row_f64_rejects_out_of_bounds() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0]));
        let err = table.update_row_f64(1, &[2.0]).unwrap_err();
        assert!(matches!(err, DataTableUpdateError::RowOutOfBounds { .. }));
    }

    #[test]
    fn update_row_f64_rejects_wrong_width() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::F64(vec![2.0]));
        let err = table.update_row_f64(0, &[3.0]).unwrap_err();
        assert!(matches!(
            err,
            DataTableUpdateError::ColumnCountMismatch { .. }
        ));
    }

    #[test]
    fn update_row_f64_rejects_non_f64_columns() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::Bool(vec![true]));
        let err = table.update_row_f64(0, &[2.0, 3.0]).unwrap_err();
        assert!(matches!(
            err,
            DataTableUpdateError::NonF64Column { column: 1 }
        ));
    }

    #[test]
    fn append_columns_f64_rejects_mismatched_column_lengths() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::F64(vec![2.0]));

        let err = table
            .append_columns_f64(&[&[3.0, 5.0], &[4.0]])
            .unwrap_err();
        assert!(matches!(
            err,
            DataTableAppendError::ColumnLenMismatch { .. }
        ));
    }

    #[test]
    fn append_columns_f64_rejects_non_f64_columns() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::Bool(vec![true]));

        let err = table.append_columns_f64(&[&[3.0], &[4.0]]).unwrap_err();
        assert!(matches!(
            err,
            DataTableAppendError::NonF64Column { column: 1 }
        ));
    }

    #[test]
    fn dataset_store_insert_and_lookup() {
        let mut store = DatasetStore::default();
        let dataset_id = DatasetId::new(1);

        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0, 2.0, 3.0]));

        store.insert(dataset_id, table);

        let got = store
            .dataset(dataset_id)
            .expect("dataset should be present");
        assert_eq!(got.row_count(), 3);
        assert_eq!(got.column_f64(0).unwrap(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn data_table_view_maps_indices_to_raw() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![10.0, 20.0, 30.0, 40.0]));

        let sel = RowSelection::Indices(vec![1u32, 3u32].into());
        let view = DataTableView::new(&table, sel);

        assert_eq!(view.raw_len(), 4);
        assert_eq!(view.len(), 2);
        assert_eq!(view.get_raw_index(0), Some(1));
        assert_eq!(view.get_raw_index(1), Some(3));
        assert_eq!(view.column_f64(0).unwrap(), &[10.0, 20.0, 30.0, 40.0]);
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DatasetStore {
    pub datasets: BTreeMap<DatasetId, DataTable>,
}

impl DatasetStore {
    pub fn dataset(&self, id: DatasetId) -> Option<&DataTable> {
        self.datasets.get(&id)
    }

    pub fn dataset_mut(&mut self, id: DatasetId) -> Option<&mut DataTable> {
        self.datasets.get_mut(&id)
    }

    pub fn insert(&mut self, id: DatasetId, table: DataTable) -> Option<DataTable> {
        self.datasets.insert(id, table)
    }
}
