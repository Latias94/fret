use std::collections::{HashMap, HashSet};

use super::{ColumnDef, ColumnId, RowKey, RowModel, TableOptions};

/// TanStack-compatible grouping state: an ordered list of grouped column ids.
pub type GroupingState = Vec<ColumnId>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupedColumnMode {
    /// Do not reorder or remove grouped columns.
    None,
    /// Move grouped columns to the start of the leaf column list (TanStack default).
    Reorder,
    /// Remove grouped columns from the leaf column list.
    Remove,
}

pub fn is_column_grouped(state: &GroupingState, column: &ColumnId) -> bool {
    state.iter().any(|c| c.as_ref() == column.as_ref())
}

pub fn grouped_index(state: &GroupingState, column: &ColumnId) -> Option<usize> {
    state.iter().position(|c| c.as_ref() == column.as_ref())
}

pub fn set_grouping(state: &mut GroupingState, ids: impl IntoIterator<Item = ColumnId>) {
    state.clear();
    for id in ids {
        if state.iter().any(|c| c.as_ref() == id.as_ref()) {
            continue;
        }
        state.push(id);
    }
}

pub fn toggle_column_grouping(state: &mut GroupingState, column: &ColumnId) {
    if let Some(i) = grouped_index(state, column) {
        state.remove(i);
    } else {
        state.push(column.clone());
    }
}

pub fn toggle_column_grouping_value(
    state: &mut GroupingState,
    column: &ColumnId,
    grouped: Option<bool>,
) {
    let should_group = grouped.unwrap_or_else(|| !is_column_grouped(state, column));
    let is_grouped = is_column_grouped(state, column);

    match (is_grouped, should_group) {
        (true, false) => {
            if let Some(i) = grouped_index(state, column) {
                state.remove(i);
            }
        }
        (false, true) => state.push(column.clone()),
        _ => {}
    }
}

pub fn toggled_column_grouping(state: &GroupingState, column: &ColumnId) -> GroupingState {
    let mut next = state.clone();
    toggle_column_grouping(&mut next, column);
    next
}

pub fn toggled_column_grouping_value(
    state: &GroupingState,
    column: &ColumnId,
    grouped: Option<bool>,
) -> GroupingState {
    let mut next = state.clone();
    toggle_column_grouping_value(&mut next, column, grouped);
    next
}

pub fn order_columns_for_grouping<'c, TData>(
    leaf_columns: &'c [ColumnDef<TData>],
    grouping: &[ColumnId],
    mode: GroupedColumnMode,
) -> Vec<&'c ColumnDef<TData>> {
    if leaf_columns.is_empty() {
        return Vec::new();
    }
    if grouping.is_empty() || mode == GroupedColumnMode::None {
        return leaf_columns.iter().collect();
    }

    let is_grouped = |c: &&ColumnDef<TData>| grouping.iter().any(|id| id.as_ref() == c.id.as_ref());

    let non_grouped: Vec<&ColumnDef<TData>> =
        leaf_columns.iter().filter(|c| !is_grouped(c)).collect();
    if mode == GroupedColumnMode::Remove {
        return non_grouped;
    }

    let by_id: HashMap<&str, &ColumnDef<TData>> =
        leaf_columns.iter().map(|c| (c.id.as_ref(), c)).collect();
    let mut grouped_cols: Vec<&ColumnDef<TData>> = Vec::new();
    for id in grouping {
        if let Some(col) = by_id.get(id.as_ref()).copied() {
            grouped_cols.push(col);
        }
    }

    grouped_cols.extend(non_grouped);
    grouped_cols
}

pub fn order_column_refs_for_grouping<'c, TData>(
    leaf_columns: &'c [&'c ColumnDef<TData>],
    grouping: &[ColumnId],
    mode: GroupedColumnMode,
) -> Vec<&'c ColumnDef<TData>> {
    if leaf_columns.is_empty() {
        return Vec::new();
    }
    if grouping.is_empty() || mode == GroupedColumnMode::None {
        return leaf_columns.to_vec();
    }

    let is_grouped = |c: &&ColumnDef<TData>| grouping.iter().any(|id| id.as_ref() == c.id.as_ref());

    let non_grouped: Vec<&ColumnDef<TData>> = leaf_columns
        .iter()
        .copied()
        .filter(|c| !is_grouped(&c))
        .collect();
    if mode == GroupedColumnMode::Remove {
        return non_grouped;
    }

    let by_id: HashMap<&str, &ColumnDef<TData>> = leaf_columns
        .iter()
        .copied()
        .map(|c| (c.id.as_ref(), c))
        .collect();

    let mut grouped_cols: Vec<&ColumnDef<TData>> = Vec::new();
    for id in grouping {
        if let Some(col) = by_id.get(id.as_ref()).copied() {
            grouped_cols.push(col);
        }
    }

    grouped_cols.extend(non_grouped);
    grouped_cols
}

pub fn column_can_group<TData>(options: TableOptions, column: &ColumnDef<TData>) -> bool {
    if !(options.enable_grouping && column.enable_grouping) {
        return false;
    }
    column.facet_key_fn.is_some() || column.facet_str_fn.is_some()
}

pub type GroupedRowIndex = usize;

#[derive(Debug, Clone)]
pub enum GroupedRowKind {
    Group {
        grouping_column: ColumnId,
        grouping_value: u64,
        first_leaf_row_key: RowKey,
        leaf_row_count: usize,
    },
    Leaf {
        row_key: RowKey,
    },
}

#[derive(Debug, Clone)]
pub struct GroupedRow {
    pub key: RowKey,
    pub kind: GroupedRowKind,
    pub depth: usize,
    pub parent: Option<GroupedRowIndex>,
    pub parent_key: Option<RowKey>,
    pub sub_rows: Vec<GroupedRowIndex>,
}

#[derive(Debug, Clone, Default)]
pub struct GroupedRowModel {
    root_rows: Vec<GroupedRowIndex>,
    flat_rows: Vec<GroupedRowIndex>,
    rows_by_key: HashMap<RowKey, GroupedRowIndex>,
    arena: Vec<GroupedRow>,
}

impl GroupedRowModel {
    pub fn root_rows(&self) -> &[GroupedRowIndex] {
        &self.root_rows
    }

    pub fn flat_rows(&self) -> &[GroupedRowIndex] {
        &self.flat_rows
    }

    pub fn row(&self, index: GroupedRowIndex) -> Option<&GroupedRow> {
        self.arena.get(index)
    }

    pub fn row_by_key(&self, key: RowKey) -> Option<GroupedRowIndex> {
        self.rows_by_key.get(&key).copied()
    }

    pub fn is_leaf(&self, index: GroupedRowIndex) -> bool {
        self.row(index)
            .is_some_and(|r| matches!(r.kind, GroupedRowKind::Leaf { .. }))
    }
}

fn fnv1a64_bytes(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001B3;

    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

fn mix64(mut h: u64, v: u64) -> u64 {
    const PRIME: u64 = 0x00000100000001B3;
    h ^= v;
    h = h.wrapping_mul(PRIME);
    h ^= h >> 32;
    h
}

fn group_row_key_hash(parent: Option<RowKey>, column: &ColumnId, value: u64, attempt: u64) -> u64 {
    let mut h = fnv1a64_bytes(column.as_ref().as_bytes());
    h = mix64(h, parent.map(|k| k.0).unwrap_or(0));
    h = mix64(h, value);
    mix64(h, attempt)
}

fn alloc_group_row_key(
    used: &mut HashSet<RowKey>,
    parent: Option<RowKey>,
    column: &ColumnId,
    value: u64,
) -> RowKey {
    for attempt in 0u64.. {
        let candidate = RowKey(group_row_key_hash(parent, column, value, attempt));
        if used.insert(candidate) {
            return candidate;
        }
    }
    unreachable!("u64 key space exhausted")
}

fn grouping_value_for_row<TData>(column: &ColumnDef<TData>, row: &TData) -> u64 {
    if let Some(f) = column.facet_key_fn.as_ref() {
        return f(row);
    }
    if let Some(f) = column.facet_str_fn.as_ref() {
        return fnv1a64_bytes(f(row).as_bytes());
    }
    0
}

pub fn grouped_row_model_from_leaf<'a, TData>(row_model: &RowModel<'a, TData>) -> GroupedRowModel {
    let mut out = GroupedRowModel::default();
    if row_model.root_rows().is_empty() {
        return out;
    }

    out.arena.reserve(row_model.root_rows().len());
    for &root in row_model.root_rows() {
        let Some(row) = row_model.row(root) else {
            continue;
        };
        let index = out.arena.len();
        out.arena.push(GroupedRow {
            key: row.key,
            kind: GroupedRowKind::Leaf { row_key: row.key },
            depth: 0,
            parent: None,
            parent_key: None,
            sub_rows: Vec::new(),
        });
        out.root_rows.push(index);
        out.flat_rows.push(index);
        out.rows_by_key.insert(row.key, index);
    }

    out
}

pub fn group_row_model<'a, TData>(
    row_model: &RowModel<'a, TData>,
    columns: &[ColumnDef<TData>],
    grouping: &[ColumnId],
) -> GroupedRowModel {
    if grouping.is_empty() {
        return grouped_row_model_from_leaf(row_model);
    }

    let mut columns_by_id: HashMap<&str, &ColumnDef<TData>> = HashMap::new();
    for col in columns {
        columns_by_id.insert(col.id.as_ref(), col);
    }

    let mut used_keys: HashSet<RowKey> = HashSet::new();
    for &root in row_model.root_rows() {
        if let Some(r) = row_model.row(root) {
            used_keys.insert(r.key);
        }
    }

    let mut out = GroupedRowModel::default();

    fn build_groups<'a, TData>(
        row_model: &RowModel<'a, TData>,
        columns_by_id: &HashMap<&str, &ColumnDef<TData>>,
        grouping: &[ColumnId],
        used_keys: &mut HashSet<RowKey>,
        parent: Option<GroupedRowIndex>,
        parent_key: Option<RowKey>,
        depth: usize,
        row_keys: &[RowKey],
        out: &mut GroupedRowModel,
    ) -> Vec<GroupedRowIndex> {
        let Some(grouping_column_id) = grouping.first() else {
            return Vec::new();
        };
        let Some(column) = columns_by_id.get(grouping_column_id.as_ref()).copied() else {
            return Vec::new();
        };

        let mut buckets: Vec<(u64, Vec<RowKey>)> = Vec::new();
        let mut bucket_index_by_value: HashMap<u64, usize> = HashMap::new();

        for &row_key in row_keys {
            let Some(i) = row_model.row_by_key(row_key) else {
                continue;
            };
            let Some(row) = row_model.row(i) else {
                continue;
            };
            let value = grouping_value_for_row(column, row.original);

            let bucket_idx = match bucket_index_by_value.get(&value).copied() {
                Some(i) => i,
                None => {
                    let i = buckets.len();
                    buckets.push((value, Vec::new()));
                    bucket_index_by_value.insert(value, i);
                    i
                }
            };
            buckets[bucket_idx].1.push(row_key);
        }

        let mut out_children: Vec<GroupedRowIndex> = Vec::with_capacity(buckets.len());

        for (value, rows) in buckets {
            let first_leaf_row_key = rows.first().copied().unwrap_or(RowKey(0));
            let leaf_row_count = rows.len();
            let group_key = alloc_group_row_key(used_keys, parent_key, &column.id, value);

            let index = out.arena.len();
            out.arena.push(GroupedRow {
                key: group_key,
                kind: GroupedRowKind::Group {
                    grouping_column: column.id.clone(),
                    grouping_value: value,
                    first_leaf_row_key,
                    leaf_row_count,
                },
                depth,
                parent,
                parent_key,
                sub_rows: Vec::new(),
            });
            out.rows_by_key.insert(group_key, index);

            let sub_rows: Vec<GroupedRowIndex> = if grouping.len() > 1 {
                build_groups(
                    row_model,
                    columns_by_id,
                    &grouping[1..],
                    used_keys,
                    Some(index),
                    Some(group_key),
                    depth + 1,
                    &rows,
                    out,
                )
            } else {
                let mut leaf_nodes: Vec<GroupedRowIndex> = Vec::with_capacity(rows.len());
                for &leaf_key in &rows {
                    let leaf_index = out.arena.len();
                    out.arena.push(GroupedRow {
                        key: leaf_key,
                        kind: GroupedRowKind::Leaf { row_key: leaf_key },
                        depth: depth + 1,
                        parent: Some(index),
                        parent_key: Some(group_key),
                        sub_rows: Vec::new(),
                    });
                    out.rows_by_key.insert(leaf_key, leaf_index);
                    leaf_nodes.push(leaf_index);
                    // TanStack-compatible: leaf rows are included in flat rows during recursion,
                    // and may appear again when their parent group appends its `subRows`.
                    out.flat_rows.push(leaf_index);
                }
                leaf_nodes
            };

            // TanStack-compatible: group rows append their sub rows (not themselves) to the flat
            // row list.
            out.flat_rows.extend(sub_rows.iter().copied());
            if let Some(node) = out.arena.get_mut(index) {
                node.sub_rows = sub_rows;
            }
            out_children.push(index);
        }

        out_children
    }

    let mut root_keys: Vec<RowKey> = Vec::new();
    root_keys.reserve(row_model.root_rows().len());
    for &root in row_model.root_rows() {
        if let Some(r) = row_model.row(root) {
            root_keys.push(r.key);
        }
    }

    out.root_rows = build_groups(
        row_model,
        &columns_by_id,
        grouping,
        &mut used_keys,
        None,
        None,
        0,
        &root_keys,
        &mut out,
    );

    // TanStack-compatible: root grouped rows are appended to the flat row list at the end.
    out.flat_rows.extend(out.root_rows.iter().copied());

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::{Table, create_column_helper};
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    struct Item {
        role: Arc<str>,
        score: u64,
    }

    #[test]
    fn grouping_state_toggle_adds_and_removes() {
        let mut g: GroupingState = Vec::new();
        let a: ColumnId = "a".into();
        let b: ColumnId = "b".into();

        toggle_column_grouping(&mut g, &a);
        assert_eq!(g.iter().map(|c| c.as_ref()).collect::<Vec<_>>(), vec!["a"]);
        toggle_column_grouping(&mut g, &b);
        assert_eq!(
            g.iter().map(|c| c.as_ref()).collect::<Vec<_>>(),
            vec!["a", "b"]
        );
        toggle_column_grouping(&mut g, &a);
        assert_eq!(g.iter().map(|c| c.as_ref()).collect::<Vec<_>>(), vec!["b"]);
    }

    #[test]
    fn group_row_model_groups_by_facet_key_and_preserves_first_seen_order() {
        let data = vec![
            Item {
                role: "Admin".into(),
                score: 1,
            },
            Item {
                role: "Member".into(),
                score: 2,
            },
            Item {
                role: "Admin".into(),
                score: 3,
            },
        ];

        let helper = create_column_helper::<Item>();
        let columns = vec![
            helper
                .clone()
                .accessor("role", |it| it.role.clone())
                .facet_key_by(|it| fnv1a64_bytes(it.role.as_bytes())),
            helper.accessor("score", |it| it.score),
        ];

        let mut state = crate::table::TableState::default();
        state.grouping = vec![ColumnId::from("role")];
        let table = Table::builder(&data).columns(columns).state(state).build();

        let grouped = table.grouped_row_model();
        assert_eq!(grouped.root_rows().len(), 2);

        let g0 = grouped.row(grouped.root_rows()[0]).unwrap();
        let g1 = grouped.row(grouped.root_rows()[1]).unwrap();

        assert!(matches!(g0.kind, GroupedRowKind::Group { .. }));
        assert!(matches!(g1.kind, GroupedRowKind::Group { .. }));

        // First group should be "Admin" (first seen), then "Member".
        let GroupedRowKind::Group {
            grouping_value: v0,
            first_leaf_row_key: first0,
            leaf_row_count: count0,
            ..
        } = &g0.kind
        else {
            panic!("expected group row");
        };
        let GroupedRowKind::Group {
            grouping_value: v1,
            first_leaf_row_key: first1,
            leaf_row_count: count1,
            ..
        } = &g1.kind
        else {
            panic!("expected group row");
        };

        assert_eq!(*v0, fnv1a64_bytes("Admin".as_bytes()));
        assert_eq!(*v1, fnv1a64_bytes("Member".as_bytes()));
        assert_eq!((*first0, *count0), (RowKey(0), 2));
        assert_eq!((*first1, *count1), (RowKey(1), 1));
    }
}
