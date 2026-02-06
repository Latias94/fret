use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    ColumnFilter, ColumnId, ColumnPinningState, ColumnSizingInfoState, ColumnSizingState,
    ColumnVisibilityState, ExpandingState, GroupingState, PaginationState, RowKey, RowModel,
    RowPinningState, SortSpec, TableState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TanStackSortingSpec {
    pub id: String,
    pub desc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TanStackColumnFilter {
    pub id: String,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TanStackPaginationState {
    #[serde(rename = "pageIndex")]
    pub page_index: usize,
    #[serde(rename = "pageSize")]
    pub page_size: usize,
}

pub type TanStackColumnSizingState = BTreeMap<String, f32>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TanStackColumnSizingInfoState {
    #[serde(default, rename = "columnSizingStart")]
    pub column_sizing_start: Vec<(String, f32)>,
    #[serde(default, rename = "deltaOffset")]
    pub delta_offset: Option<f32>,
    #[serde(default, rename = "deltaPercentage")]
    pub delta_percentage: Option<f32>,
    #[serde(default, rename = "isResizingColumn")]
    pub is_resizing_column: Value,
    #[serde(default, rename = "startOffset")]
    pub start_offset: Option<f32>,
    #[serde(default, rename = "startSize")]
    pub start_size: Option<f32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TanStackColumnPinningState {
    #[serde(default)]
    pub left: Vec<String>,
    #[serde(default)]
    pub right: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TanStackTableState {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sorting: Vec<TanStackSortingSpec>,
    #[serde(
        default,
        rename = "columnFilters",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub column_filters: Vec<TanStackColumnFilter>,
    #[serde(default, rename = "globalFilter")]
    pub global_filter: Option<Value>,
    #[serde(default)]
    pub pagination: Option<TanStackPaginationState>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grouping: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expanded: Option<TanStackExpandedState>,
    #[serde(default, rename = "rowPinning")]
    pub row_pinning: Option<TanStackRowPinningState>,
    #[serde(default, rename = "rowSelection")]
    pub row_selection: Option<BTreeMap<String, bool>>,
    #[serde(default, rename = "columnPinning")]
    pub column_pinning: Option<TanStackColumnPinningState>,
    #[serde(default, rename = "columnOrder", skip_serializing_if = "Vec::is_empty")]
    pub column_order: Vec<String>,
    #[serde(default, rename = "columnVisibility")]
    pub column_visibility: Option<BTreeMap<String, bool>>,
    #[serde(
        default,
        rename = "columnSizing",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub column_sizing: TanStackColumnSizingState,
    #[serde(default, rename = "columnSizingInfo")]
    pub column_sizing_info: Option<TanStackColumnSizingInfoState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TanStackExpandedState {
    All(bool),
    Map(BTreeMap<String, bool>),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TanStackRowPinningState {
    #[serde(default)]
    pub top: Vec<String>,
    #[serde(default)]
    pub bottom: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TanStackStateError {
    InvalidRowSelectionKey { row_id: String },
    InvalidExpandedKey { row_id: String },
    InvalidRowPinningKey { row_id: String },
    UnresolvedRowId { field: &'static str, row_id: String },
    InvalidIsResizingColumnValue { value: Value },
}

impl std::fmt::Display for TanStackStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRowSelectionKey { row_id } => {
                write!(
                    f,
                    "tanstack rowSelection key must parse as u64 (row_id={row_id})"
                )
            }
            Self::InvalidExpandedKey { row_id } => {
                write!(
                    f,
                    "tanstack expanded key must parse as u64 (row_id={row_id})"
                )
            }
            Self::InvalidRowPinningKey { row_id } => {
                write!(
                    f,
                    "tanstack rowPinning key must parse as u64 (row_id={row_id})"
                )
            }
            Self::UnresolvedRowId { field, row_id } => {
                write!(
                    f,
                    "tanstack row id must resolve via row model (field={field}, row_id={row_id})"
                )
            }
            Self::InvalidIsResizingColumnValue { value } => {
                write!(
                    f,
                    "tanstack columnSizingInfo.isResizingColumn must be false|null|string (value={value:?})"
                )
            }
        }
    }
}

impl std::error::Error for TanStackStateError {}

impl TanStackTableState {
    pub fn from_json(value: &Value) -> serde_json::Result<Self> {
        serde_json::from_value(value.clone())
    }

    pub fn from_table_state(state: &TableState) -> Self {
        let sorting = state
            .sorting
            .iter()
            .map(|s| TanStackSortingSpec {
                id: s.column.as_ref().to_string(),
                desc: s.desc,
            })
            .collect();

        let column_filters = state
            .column_filters
            .iter()
            .map(|f| TanStackColumnFilter {
                id: f.column.as_ref().to_string(),
                value: f.value.clone(),
            })
            .collect();

        let global_filter = state
            .global_filter
            .as_ref()
            .and_then(|v| if v.is_null() { None } else { Some(v.clone()) });

        let pagination = if state.pagination.page_index == 0 && state.pagination.page_size == 10 {
            None
        } else {
            Some(TanStackPaginationState {
                page_index: state.pagination.page_index,
                page_size: state.pagination.page_size,
            })
        };

        let grouping = state
            .grouping
            .iter()
            .map(|s| s.as_ref().to_string())
            .collect();

        let expanded = match &state.expanding {
            ExpandingState::All => Some(TanStackExpandedState::All(true)),
            ExpandingState::Keys(keys) if keys.is_empty() => None,
            ExpandingState::Keys(keys) => Some(TanStackExpandedState::Map(
                keys.iter().map(|k| (k.0.to_string(), true)).collect(),
            )),
        };

        let row_pinning = if state.row_pinning.top.is_empty() && state.row_pinning.bottom.is_empty()
        {
            None
        } else {
            Some(TanStackRowPinningState {
                top: state
                    .row_pinning
                    .top
                    .iter()
                    .map(|k| k.0.to_string())
                    .collect(),
                bottom: state
                    .row_pinning
                    .bottom
                    .iter()
                    .map(|k| k.0.to_string())
                    .collect(),
            })
        };

        let row_selection = if state.row_selection.is_empty() {
            None
        } else {
            Some(
                state
                    .row_selection
                    .iter()
                    .map(|k| (k.0.to_string(), true))
                    .collect(),
            )
        };

        let column_pinning =
            if state.column_pinning.left.is_empty() && state.column_pinning.right.is_empty() {
                None
            } else {
                Some(TanStackColumnPinningState {
                    left: state
                        .column_pinning
                        .left
                        .iter()
                        .map(|s| s.as_ref().to_string())
                        .collect(),
                    right: state
                        .column_pinning
                        .right
                        .iter()
                        .map(|s| s.as_ref().to_string())
                        .collect(),
                })
            };

        let column_order = state
            .column_order
            .iter()
            .map(|s| s.as_ref().to_string())
            .collect();

        let column_visibility = if state.column_visibility.is_empty() {
            None
        } else {
            Some(
                state
                    .column_visibility
                    .iter()
                    .map(|(k, v)| (k.as_ref().to_string(), *v))
                    .collect(),
            )
        };

        let column_sizing: TanStackColumnSizingState = state
            .column_sizing
            .iter()
            .map(|(k, v)| (k.as_ref().to_string(), *v))
            .collect();

        let column_sizing_info = {
            let info = &state.column_sizing_info;
            let is_default = info.column_sizing_start.is_empty()
                && info.delta_offset.is_none()
                && info.delta_percentage.is_none()
                && info.is_resizing_column.is_none()
                && info.start_offset.is_none()
                && info.start_size.is_none();
            if is_default {
                None
            } else {
                Some(TanStackColumnSizingInfoState {
                    column_sizing_start: info
                        .column_sizing_start
                        .iter()
                        .map(|(id, size)| (id.as_ref().to_string(), *size))
                        .collect(),
                    delta_offset: info.delta_offset,
                    delta_percentage: info.delta_percentage,
                    is_resizing_column: match info.is_resizing_column.as_ref() {
                        None => Value::Bool(false),
                        Some(id) => Value::String(id.as_ref().to_string()),
                    },
                    start_offset: info.start_offset,
                    start_size: info.start_size,
                })
            }
        };

        Self {
            sorting,
            column_filters,
            global_filter,
            pagination,
            grouping,
            expanded,
            row_pinning,
            row_selection,
            column_pinning,
            column_order,
            column_visibility,
            column_sizing,
            column_sizing_info,
        }
    }

    pub fn to_json(&self) -> serde_json::Result<Value> {
        serde_json::to_value(self)
    }

    pub fn to_table_state(&self) -> Result<TableState, TanStackStateError> {
        let mut out = TableState::default();

        out.sorting = self
            .sorting
            .iter()
            .map(|s| SortSpec {
                column: Arc::<str>::from(s.id.as_str()),
                desc: s.desc,
            })
            .collect();

        out.column_filters = self
            .column_filters
            .iter()
            .map(|f| {
                Ok(ColumnFilter {
                    column: Arc::<str>::from(f.id.as_str()),
                    value: f.value.clone(),
                })
            })
            .collect::<Result<_, _>>()?;

        out.global_filter = match self.global_filter.as_ref() {
            None | Some(Value::Null) => None,
            Some(v) => Some(v.clone()),
        };

        if let Some(p) = self.pagination {
            out.pagination = PaginationState {
                page_index: p.page_index,
                page_size: p.page_size,
            };
        }

        if !self.grouping.is_empty() {
            out.grouping = self
                .grouping
                .iter()
                .map(|s| ColumnId::from(s.as_str()))
                .collect::<GroupingState>();
        }

        if let Some(expanded) = self.expanded.as_ref() {
            out.expanding = match expanded {
                TanStackExpandedState::All(true) => ExpandingState::All,
                TanStackExpandedState::All(false) => ExpandingState::default(),
                TanStackExpandedState::Map(map) => ExpandingState::from_iter(
                    map.iter()
                        .filter_map(|(k, v)| {
                            if !*v {
                                return None;
                            }
                            let row_id = k.parse::<u64>().ok()?;
                            Some(RowKey(row_id))
                        })
                        .collect::<Vec<_>>(),
                ),
            };

            // Validate row ids if expanded is a map.
            if let TanStackExpandedState::Map(map) = expanded {
                for (k, v) in map {
                    if !*v {
                        continue;
                    }
                    if k.parse::<u64>().is_err() {
                        return Err(TanStackStateError::InvalidExpandedKey { row_id: k.clone() });
                    }
                }
            }
        }

        if let Some(pinning) = self.row_pinning.as_ref() {
            let mut next = RowPinningState::default();
            for k in &pinning.top {
                let row_id = k
                    .parse::<u64>()
                    .map_err(|_| TanStackStateError::InvalidRowPinningKey { row_id: k.clone() })?;
                next.top.push(RowKey(row_id));
            }
            for k in &pinning.bottom {
                let row_id = k
                    .parse::<u64>()
                    .map_err(|_| TanStackStateError::InvalidRowPinningKey { row_id: k.clone() })?;
                next.bottom.push(RowKey(row_id));
            }
            out.row_pinning = next;
        }

        if let Some(sel) = self.row_selection.as_ref() {
            for (k, v) in sel {
                if !*v {
                    continue;
                }
                let row_id =
                    k.parse::<u64>()
                        .map_err(|_| TanStackStateError::InvalidRowSelectionKey {
                            row_id: k.clone(),
                        })?;
                out.row_selection.insert(super::RowKey(row_id));
            }
        }

        if let Some(pinning) = self.column_pinning.as_ref() {
            out.column_pinning = ColumnPinningState {
                left: pinning
                    .left
                    .iter()
                    .map(|s| ColumnId::from(s.as_str()))
                    .collect(),
                right: pinning
                    .right
                    .iter()
                    .map(|s| ColumnId::from(s.as_str()))
                    .collect(),
            };
        }

        if let Some(vis) = self.column_visibility.as_ref() {
            let mut next: ColumnVisibilityState = ColumnVisibilityState::default();
            for (k, v) in vis {
                if *v {
                    continue;
                }
                next.insert(ColumnId::from(k.as_str()), false);
            }
            out.column_visibility = next;
        }

        if !self.column_sizing.is_empty() {
            let mut next: ColumnSizingState = ColumnSizingState::default();
            for (k, v) in &self.column_sizing {
                next.insert(ColumnId::from(k.as_str()), *v);
            }
            out.column_sizing = next;
        }

        if let Some(info) = self.column_sizing_info.as_ref() {
            let is_resizing_column: Option<ColumnId> = match &info.is_resizing_column {
                Value::Bool(false) | Value::Null => None,
                Value::String(s) => Some(ColumnId::from(s.as_str())),
                v => {
                    return Err(TanStackStateError::InvalidIsResizingColumnValue {
                        value: v.clone(),
                    });
                }
            };

            out.column_sizing_info = ColumnSizingInfoState {
                column_sizing_start: info
                    .column_sizing_start
                    .iter()
                    .map(|(id, size)| (ColumnId::from(id.as_str()), *size))
                    .collect(),
                delta_offset: info.delta_offset,
                delta_percentage: info.delta_percentage,
                is_resizing_column,
                start_offset: info.start_offset,
                start_size: info.start_size,
            };
        }

        out.column_order = self
            .column_order
            .iter()
            .map(|s| ColumnId::from(s.as_str()))
            .collect();

        Ok(out)
    }

    pub fn to_table_state_with_row_model<'a, TData>(
        &self,
        core_row_model: &RowModel<'a, TData>,
    ) -> Result<TableState, TanStackStateError> {
        fn resolve_row_key<'a, TData>(
            core: &RowModel<'a, TData>,
            field: &'static str,
            row_id: &str,
        ) -> Result<RowKey, TanStackStateError> {
            if let Some(index) = core.row_by_id(row_id) {
                let row = core
                    .row(index)
                    .ok_or_else(|| TanStackStateError::UnresolvedRowId {
                        field,
                        row_id: row_id.to_string(),
                    })?;
                return Ok(row.key);
            }
            if let Ok(parsed) = row_id.parse::<u64>() {
                return Ok(RowKey(parsed));
            }
            Err(TanStackStateError::UnresolvedRowId {
                field,
                row_id: row_id.to_string(),
            })
        }

        let mut base = self.clone();
        base.expanded = None;
        base.row_pinning = None;
        base.row_selection = None;

        let mut out = base.to_table_state()?;

        if let Some(sel) = self.row_selection.as_ref() {
            out.row_selection = sel
                .iter()
                .filter(|(_k, v)| **v)
                .map(|(k, _)| resolve_row_key(core_row_model, "rowSelection", k))
                .collect::<Result<_, _>>()?;
        }

        if let Some(expanded) = self.expanded.as_ref() {
            out.expanding = match expanded {
                TanStackExpandedState::All(true) => ExpandingState::All,
                TanStackExpandedState::All(false) => ExpandingState::default(),
                TanStackExpandedState::Map(map) => ExpandingState::from_iter(
                    map.iter()
                        .filter(|(_k, v)| **v)
                        .map(|(k, _)| resolve_row_key(core_row_model, "expanded", k))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
            };
        }

        if let Some(pinning) = self.row_pinning.as_ref() {
            let mut next = RowPinningState::default();
            for id in &pinning.top {
                next.top
                    .push(resolve_row_key(core_row_model, "rowPinning.top", id)?);
            }
            for id in &pinning.bottom {
                next.bottom
                    .push(resolve_row_key(core_row_model, "rowPinning.bottom", id)?);
            }
            out.row_pinning = next;
        }

        Ok(out)
    }
}
