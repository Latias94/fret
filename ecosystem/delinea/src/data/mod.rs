#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::ids::{DatasetId, Revision, StringId};

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

    pub fn column_f64(&self, index: usize) -> Option<&[f64]> {
        self.columns
            .get(index)?
            .as_f64_slice()
            .map(|v| &v[..self.row_count])
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DatasetStore {
    pub datasets: Vec<(DatasetId, DataTable)>,
}

impl DatasetStore {
    pub fn dataset_mut(&mut self, id: DatasetId) -> Option<&mut DataTable> {
        self.datasets
            .iter_mut()
            .find_map(|(k, v)| (*k == id).then_some(v))
    }
}
