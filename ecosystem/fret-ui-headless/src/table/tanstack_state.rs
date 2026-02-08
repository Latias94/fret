use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::{
    ColumnFilter, ColumnId, ColumnPinningState, ColumnSizingInfoState, ColumnSizingState,
    ColumnVisibilityState, ExpandingState, GroupedRowModel, GroupingState, PaginationState, RowKey,
    RowModel, RowPinningState, SortSpec, TableState,
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

#[derive(Debug, Clone, Copy, Default)]
struct TanStackStatePresence {
    sorting: bool,
    column_filters: bool,
    global_filter: bool,
    pagination: bool,
    grouping: bool,
    expanded: bool,
    row_pinning: bool,
    row_selection: bool,
    column_pinning: bool,
    column_order: bool,
    column_visibility: bool,
    column_sizing: bool,
    column_sizing_info: bool,
}

impl TanStackStatePresence {
    fn from_json(value: &Value) -> Self {
        let mut out = Self::default();
        let Some(map) = value.as_object() else {
            return out;
        };

        out.sorting = map.contains_key("sorting");
        out.column_filters = map.contains_key("columnFilters");
        out.global_filter = map.contains_key("globalFilter");
        out.pagination = map.contains_key("pagination");
        out.grouping = map.contains_key("grouping");
        out.expanded = map.contains_key("expanded");
        out.row_pinning = map.contains_key("rowPinning");
        out.row_selection = map.contains_key("rowSelection");
        out.column_pinning = map.contains_key("columnPinning");
        out.column_order = map.contains_key("columnOrder");
        out.column_visibility = map.contains_key("columnVisibility");
        out.column_sizing = map.contains_key("columnSizing");
        out.column_sizing_info = map.contains_key("columnSizingInfo");
        out
    }
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
    #[serde(
        default,
        rename = "globalFilter",
        skip_serializing_if = "Option::is_none"
    )]
    pub global_filter: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pagination: Option<TanStackPaginationState>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grouping: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expanded: Option<TanStackExpandedState>,
    #[serde(
        default,
        rename = "rowPinning",
        skip_serializing_if = "Option::is_none"
    )]
    pub row_pinning: Option<TanStackRowPinningState>,
    #[serde(
        default,
        rename = "rowSelection",
        skip_serializing_if = "Option::is_none"
    )]
    pub row_selection: Option<BTreeMap<String, bool>>,
    #[serde(
        default,
        rename = "columnPinning",
        skip_serializing_if = "Option::is_none"
    )]
    pub column_pinning: Option<TanStackColumnPinningState>,
    #[serde(default, rename = "columnOrder", skip_serializing_if = "Vec::is_empty")]
    pub column_order: Vec<String>,
    #[serde(
        default,
        rename = "columnVisibility",
        skip_serializing_if = "Option::is_none"
    )]
    pub column_visibility: Option<BTreeMap<String, bool>>,
    #[serde(
        default,
        rename = "columnSizing",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub column_sizing: TanStackColumnSizingState,
    #[serde(
        default,
        rename = "columnSizingInfo",
        skip_serializing_if = "Option::is_none"
    )]
    pub column_sizing_info: Option<TanStackColumnSizingInfoState>,
    #[serde(skip)]
    presence: TanStackStatePresence,
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
    InvalidRowSelectionKey {
        row_id: String,
    },
    InvalidExpandedKey {
        row_id: String,
    },
    InvalidRowPinningKey {
        row_id: String,
    },
    UnresolvedRowId {
        field: &'static str,
        row_id: String,
    },
    UnresolvedRowKey {
        field: &'static str,
        row_key: RowKey,
    },
    InvalidIsResizingColumnValue {
        value: Value,
    },
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
            Self::UnresolvedRowKey { field, row_key } => {
                write!(
                    f,
                    "tanstack row key must resolve via row model (field={field}, row_key={})",
                    row_key.0
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
        let mut out: Self = serde_json::from_value(value.clone())?;
        out.presence = TanStackStatePresence::from_json(value);
        Ok(out)
    }

    pub fn from_table_state(state: &TableState) -> Self {
        let sorting: Vec<TanStackSortingSpec> = state
            .sorting
            .iter()
            .map(|s| TanStackSortingSpec {
                id: s.column.as_ref().to_string(),
                desc: s.desc,
            })
            .collect();

        let column_filters: Vec<TanStackColumnFilter> = state
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

        let grouping: Vec<String> = state
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

        let column_order: Vec<String> = state
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

        let presence = TanStackStatePresence {
            sorting: !sorting.is_empty(),
            column_filters: !column_filters.is_empty(),
            global_filter: global_filter.is_some(),
            pagination: pagination.is_some(),
            grouping: !grouping.is_empty(),
            expanded: expanded.is_some(),
            row_pinning: row_pinning.is_some(),
            row_selection: row_selection.is_some(),
            column_pinning: column_pinning.is_some(),
            column_order: !column_order.is_empty(),
            column_visibility: column_visibility.is_some(),
            column_sizing: !column_sizing.is_empty(),
            column_sizing_info: column_sizing_info.is_some(),
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
            presence,
        }
    }

    pub fn to_json(&self) -> serde_json::Result<Value> {
        let mut out = Map::new();

        if self.presence.sorting || !self.sorting.is_empty() {
            out.insert("sorting".to_string(), serde_json::to_value(&self.sorting)?);
        }

        if self.presence.column_filters || !self.column_filters.is_empty() {
            out.insert(
                "columnFilters".to_string(),
                serde_json::to_value(&self.column_filters)?,
            );
        }

        if self.presence.global_filter || self.global_filter.is_some() {
            out.insert(
                "globalFilter".to_string(),
                self.global_filter.clone().unwrap_or(Value::Null),
            );
        }

        if self.presence.pagination || self.pagination.is_some() {
            out.insert(
                "pagination".to_string(),
                match self.pagination.as_ref() {
                    Some(v) => serde_json::to_value(v)?,
                    None => Value::Null,
                },
            );
        }

        if self.presence.grouping || !self.grouping.is_empty() {
            out.insert(
                "grouping".to_string(),
                serde_json::to_value(&self.grouping)?,
            );
        }

        if self.presence.expanded || self.expanded.is_some() {
            out.insert(
                "expanded".to_string(),
                match self.expanded.as_ref() {
                    Some(v) => serde_json::to_value(v)?,
                    None => Value::Null,
                },
            );
        }

        if self.presence.row_pinning || self.row_pinning.is_some() {
            out.insert(
                "rowPinning".to_string(),
                match self.row_pinning.as_ref() {
                    Some(v) => serde_json::to_value(v)?,
                    None => Value::Null,
                },
            );
        }

        if self.presence.row_selection || self.row_selection.is_some() {
            out.insert(
                "rowSelection".to_string(),
                match self.row_selection.as_ref() {
                    Some(v) => serde_json::to_value(v)?,
                    None => Value::Null,
                },
            );
        }

        if self.presence.column_pinning || self.column_pinning.is_some() {
            out.insert(
                "columnPinning".to_string(),
                match self.column_pinning.as_ref() {
                    Some(v) => serde_json::to_value(v)?,
                    None => Value::Null,
                },
            );
        }

        if self.presence.column_order || !self.column_order.is_empty() {
            out.insert(
                "columnOrder".to_string(),
                serde_json::to_value(&self.column_order)?,
            );
        }

        if self.presence.column_visibility || self.column_visibility.is_some() {
            out.insert(
                "columnVisibility".to_string(),
                match self.column_visibility.as_ref() {
                    Some(v) => serde_json::to_value(v)?,
                    None => Value::Null,
                },
            );
        }

        if self.presence.column_sizing || !self.column_sizing.is_empty() {
            out.insert(
                "columnSizing".to_string(),
                serde_json::to_value(&self.column_sizing)?,
            );
        }

        if self.presence.column_sizing_info || self.column_sizing_info.is_some() {
            out.insert(
                "columnSizingInfo".to_string(),
                match self.column_sizing_info.as_ref() {
                    Some(v) => serde_json::to_value(v)?,
                    None => Value::Null,
                },
            );
        }

        Ok(Value::Object(out))
    }

    fn with_presence_hint(mut self, source: &Self) -> Self {
        self.presence = source.presence;

        if source.presence.expanded && self.expanded.is_none() {
            self.expanded = source.expanded.clone();
        }
        if source.presence.row_pinning && self.row_pinning.is_none() {
            self.row_pinning = source.row_pinning.clone();
        }
        if source.presence.row_selection && self.row_selection.is_none() {
            self.row_selection = source.row_selection.clone();
        }
        if source.presence.column_pinning && self.column_pinning.is_none() {
            self.column_pinning = source.column_pinning.clone();
        }
        if source.presence.column_visibility && self.column_visibility.is_none() {
            self.column_visibility = source.column_visibility.clone();
        }
        if source.presence.column_sizing_info && self.column_sizing_info.is_none() {
            self.column_sizing_info = source.column_sizing_info.clone();
        }
        if source.presence.pagination && self.pagination.is_none() {
            self.pagination = source.pagination;
        }
        if source.presence.global_filter && self.global_filter.is_none() {
            self.global_filter = source.global_filter.clone();
        }

        self
    }

    pub fn from_table_state_with_shape(state: &TableState, source: &Self) -> Self {
        Self::from_table_state(state).with_presence_hint(source)
    }

    fn resolve_row_id_for_key<'a, TData>(
        core_row_model: &RowModel<'a, TData>,
        grouped_row_model: Option<&GroupedRowModel>,
        field: &'static str,
        row_key: RowKey,
    ) -> Result<String, TanStackStateError> {
        if let Some(grouped) = grouped_row_model {
            if let Some(index) = grouped.row_by_key(row_key) {
                if let Some(row) = grouped.row(index) {
                    return Ok(row.id.as_str().to_string());
                }
            }
        }

        let index = core_row_model
            .row_by_key(row_key)
            .ok_or(TanStackStateError::UnresolvedRowKey { field, row_key })?;
        let row = core_row_model
            .row(index)
            .ok_or(TanStackStateError::UnresolvedRowKey { field, row_key })?;
        Ok(row.id.as_str().to_string())
    }

    pub fn from_table_state_with_row_models<'a, TData>(
        state: &TableState,
        core_row_model: &RowModel<'a, TData>,
        grouped_row_model: Option<&GroupedRowModel>,
    ) -> Result<Self, TanStackStateError> {
        let mut out = Self::from_table_state(state);

        out.expanded = match &state.expanding {
            ExpandingState::All => Some(TanStackExpandedState::All(true)),
            ExpandingState::Keys(keys) if keys.is_empty() => None,
            ExpandingState::Keys(keys) => Some(TanStackExpandedState::Map(
                keys.iter()
                    .map(|row_key| {
                        let row_id = Self::resolve_row_id_for_key(
                            core_row_model,
                            grouped_row_model,
                            "expanded",
                            *row_key,
                        )?;
                        Ok((row_id, true))
                    })
                    .collect::<Result<BTreeMap<_, _>, TanStackStateError>>()?,
            )),
        };

        out.row_pinning = if state.row_pinning.top.is_empty() && state.row_pinning.bottom.is_empty()
        {
            None
        } else {
            Some(TanStackRowPinningState {
                top: state
                    .row_pinning
                    .top
                    .iter()
                    .map(|row_key| {
                        Self::resolve_row_id_for_key(
                            core_row_model,
                            grouped_row_model,
                            "rowPinning.top",
                            *row_key,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                bottom: state
                    .row_pinning
                    .bottom
                    .iter()
                    .map(|row_key| {
                        Self::resolve_row_id_for_key(
                            core_row_model,
                            grouped_row_model,
                            "rowPinning.bottom",
                            *row_key,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            })
        };

        out.row_selection = if state.row_selection.is_empty() {
            None
        } else {
            Some(
                state
                    .row_selection
                    .iter()
                    .map(|row_key| {
                        let row_id = Self::resolve_row_id_for_key(
                            core_row_model,
                            grouped_row_model,
                            "rowSelection",
                            *row_key,
                        )?;
                        Ok((row_id, true))
                    })
                    .collect::<Result<BTreeMap<_, _>, TanStackStateError>>()?,
            )
        };

        out.presence.expanded = out.expanded.is_some();
        out.presence.row_pinning = out.row_pinning.is_some();
        out.presence.row_selection = out.row_selection.is_some();

        Ok(out)
    }

    pub fn from_table_state_with_row_model<'a, TData>(
        state: &TableState,
        core_row_model: &RowModel<'a, TData>,
    ) -> Result<Self, TanStackStateError> {
        Self::from_table_state_with_row_models(state, core_row_model, None)
    }

    pub fn from_table_state_with_row_models_and_shape<'a, TData>(
        state: &TableState,
        core_row_model: &RowModel<'a, TData>,
        grouped_row_model: Option<&GroupedRowModel>,
        source: &Self,
    ) -> Result<Self, TanStackStateError> {
        Ok(
            Self::from_table_state_with_row_models(state, core_row_model, grouped_row_model)?
                .with_presence_hint(source),
        )
    }

    pub fn from_table_state_with_row_model_and_shape<'a, TData>(
        state: &TableState,
        core_row_model: &RowModel<'a, TData>,
        source: &Self,
    ) -> Result<Self, TanStackStateError> {
        Self::from_table_state_with_row_models_and_shape(state, core_row_model, None, source)
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

    pub fn to_table_state_with_row_models<'a, TData>(
        &self,
        core_row_model: &RowModel<'a, TData>,
        grouped_row_model: Option<&GroupedRowModel>,
    ) -> Result<TableState, TanStackStateError> {
        fn resolve_row_key<'a, TData>(
            core: &RowModel<'a, TData>,
            grouped: Option<&GroupedRowModel>,
            field: &'static str,
            row_id: &str,
        ) -> Result<RowKey, TanStackStateError> {
            if let Some(grouped) = grouped {
                if let Some(index) = grouped.row_by_id(row_id) {
                    let row =
                        grouped
                            .row(index)
                            .ok_or_else(|| TanStackStateError::UnresolvedRowId {
                                field,
                                row_id: row_id.to_string(),
                            })?;
                    return Ok(row.key);
                }
            }
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
                .map(|(k, _)| resolve_row_key(core_row_model, grouped_row_model, "rowSelection", k))
                .collect::<Result<_, _>>()?;
        }

        if let Some(expanded) = self.expanded.as_ref() {
            out.expanding = match expanded {
                TanStackExpandedState::All(true) => ExpandingState::All,
                TanStackExpandedState::All(false) => ExpandingState::default(),
                TanStackExpandedState::Map(map) => ExpandingState::from_iter(
                    map.iter()
                        .filter(|(_k, v)| **v)
                        .map(|(k, _)| {
                            resolve_row_key(core_row_model, grouped_row_model, "expanded", k)
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                ),
            };
        }

        if let Some(pinning) = self.row_pinning.as_ref() {
            let mut next = RowPinningState::default();
            for id in &pinning.top {
                next.top.push(resolve_row_key(
                    core_row_model,
                    grouped_row_model,
                    "rowPinning.top",
                    id,
                )?);
            }
            for id in &pinning.bottom {
                next.bottom.push(resolve_row_key(
                    core_row_model,
                    grouped_row_model,
                    "rowPinning.bottom",
                    id,
                )?);
            }
            out.row_pinning = next;
        }

        Ok(out)
    }

    pub fn to_table_state_with_row_model<'a, TData>(
        &self,
        core_row_model: &RowModel<'a, TData>,
    ) -> Result<TableState, TanStackStateError> {
        self.to_table_state_with_row_models(core_row_model, None)
    }
}
