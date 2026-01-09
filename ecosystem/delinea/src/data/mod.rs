#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::ids::{DatasetId, Revision, StringId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataTableAppendError {
    ColumnCountMismatch { expected: usize, actual: usize },
    NonF64Column { column: usize },
}

impl core::fmt::Display for DataTableAppendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ColumnCountMismatch { expected, actual } => write!(
                f,
                "column count mismatch: expected {expected} values, got {actual}"
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
    pub revision: Revision,
    pub columns: Vec<Column>,
    pub row_count: usize,
}

impl DataTable {
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

    pub fn column_f64(&self, index: usize) -> Option<&[f64]> {
        self.columns
            .get(index)?
            .as_f64_slice()
            .map(|v| &v[..self.row_count])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_row_f64_bumps_revision_and_row_count() {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::F64(vec![2.0]));

        let rev0 = table.revision;
        table.append_row_f64(&[3.0, 4.0]).unwrap();

        assert!(table.revision.0 > rev0.0);
        assert_eq!(table.row_count, 2);
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
    fn dataset_store_insert_and_lookup() {
        let mut store = DatasetStore::default();
        let dataset_id = DatasetId::new(1);

        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![1.0, 2.0, 3.0]));

        store.insert(dataset_id, table);

        let got = store
            .dataset(dataset_id)
            .expect("dataset should be present");
        assert_eq!(got.row_count, 3);
        assert_eq!(got.column_f64(0).unwrap(), &[1.0, 2.0, 3.0]);
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
