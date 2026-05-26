use serde::{Deserialize, Serialize};

/// Persistence-friendly snapshot of runtime table-column visibility overrides.
///
/// This is only a data shape. Callers own where and when it is stored, and the IMUI helper keeps
/// using caller-owned `ImUiTableColumnVisibilityState` models at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TableColumnVisibilitySnapshot {
    #[serde(default)]
    pub columns: Vec<TableColumnVisibilityEntry>,
}

/// One stable column visibility override inside [`TableColumnVisibilitySnapshot`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableColumnVisibilityEntry {
    #[serde(rename = "id")]
    pub column_id: String,
    #[serde(rename = "visible")]
    pub is_visible: bool,
}

impl TableColumnVisibilitySnapshot {
    pub fn new<I, S>(columns: I) -> Self
    where
        I: IntoIterator<Item = (S, bool)>,
        S: Into<String>,
    {
        let mut snapshot = Self::default();
        for (id, visible) in columns {
            let id = id.into();
            if id.is_empty() {
                continue;
            }
            snapshot.columns.push(TableColumnVisibilityEntry {
                column_id: id,
                is_visible: visible,
            });
        }
        snapshot
    }

    pub fn columns(&self) -> &[TableColumnVisibilityEntry] {
        &self.columns
    }

    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

impl TableColumnVisibilityEntry {
    pub fn new(id: impl Into<String>, visible: bool) -> Self {
        Self {
            column_id: id.into(),
            is_visible: visible,
        }
    }

    pub fn id(&self) -> &str {
        self.column_id.as_str()
    }

    pub fn visible(&self) -> bool {
        self.is_visible
    }
}
