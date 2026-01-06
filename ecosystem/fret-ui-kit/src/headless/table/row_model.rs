use std::cell::OnceCell;
use std::collections::HashMap;

/// Stable identity for a row in the table.
///
/// This is aligned with TanStack Table's `getRowId` guidance, but uses an efficient numeric key so
/// it can be used in hot paths (selection, row maps, virtualization keys) without heap allocation.
///
/// The default key strategy is index-path based, so callers should supply their own stable key
/// (e.g. a database primary key) when the underlying data can reorder or change over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RowKey(pub u64);

impl RowKey {
    pub fn from_index(index: usize) -> Self {
        Self(index as u64)
    }
}

/// Index into a [`RowModel`] arena.
pub type RowIndex = usize;

#[derive(Debug)]
pub struct Row<'a, TData> {
    pub key: RowKey,
    pub original: &'a TData,
    pub index: usize,
    pub depth: u16,
    pub parent: Option<RowIndex>,
    pub parent_key: Option<RowKey>,
    pub sub_rows: Vec<RowIndex>,
}

impl<'a, TData> Clone for Row<'a, TData> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            original: self.original,
            index: self.index,
            depth: self.depth,
            parent: self.parent,
            parent_key: self.parent_key,
            sub_rows: self.sub_rows.clone(),
        }
    }
}

#[derive(Debug)]
pub struct RowModel<'a, TData> {
    pub(super) root_rows: Vec<RowIndex>,
    pub(super) flat_rows: Vec<RowIndex>,
    pub(super) rows_by_key: HashMap<RowKey, RowIndex>,
    pub(super) arena: Vec<Row<'a, TData>>,
}

impl<'a, TData> Clone for RowModel<'a, TData> {
    fn clone(&self) -> Self {
        Self {
            root_rows: self.root_rows.clone(),
            flat_rows: self.flat_rows.clone(),
            rows_by_key: self.rows_by_key.clone(),
            arena: self.arena.clone(),
        }
    }
}

impl<'a, TData> RowModel<'a, TData> {
    pub fn root_rows(&self) -> &[RowIndex] {
        &self.root_rows
    }

    pub fn flat_rows(&self) -> &[RowIndex] {
        &self.flat_rows
    }

    pub fn row(&self, index: RowIndex) -> Option<&Row<'a, TData>> {
        self.arena.get(index)
    }

    pub fn row_by_key(&self, key: RowKey) -> Option<RowIndex> {
        self.rows_by_key.get(&key).copied()
    }

    pub fn rows_by_key(&self) -> &HashMap<RowKey, RowIndex> {
        &self.rows_by_key
    }

    pub fn arena(&self) -> &[Row<'a, TData>] {
        &self.arena
    }
}

type GetRowKeyFn<'a, TData> = Box<dyn Fn(&TData, usize, Option<&RowKey>) -> RowKey + 'a>;
type GetSubRowsFn<'a, TData> = Box<dyn for<'r> Fn(&'r TData, usize) -> Option<&'r [TData]> + 'a>;

pub struct TableBuilder<'a, TData> {
    data: &'a [TData],
    columns: Vec<super::ColumnDef<TData>>,
    get_row_key: Option<GetRowKeyFn<'a, TData>>,
    get_sub_rows: Option<GetSubRowsFn<'a, TData>>,
    state: super::TableState,
}

impl<'a, TData> TableBuilder<'a, TData> {
    pub fn new(data: &'a [TData]) -> Self {
        Self {
            data,
            columns: Vec::new(),
            get_row_key: None,
            get_sub_rows: None,
            state: super::TableState::default(),
        }
    }

    pub fn columns(mut self, columns: Vec<super::ColumnDef<TData>>) -> Self {
        self.columns = columns;
        self
    }

    pub fn state(mut self, state: super::TableState) -> Self {
        self.state = state;
        self
    }

    pub fn get_row_key(
        mut self,
        f: impl Fn(&TData, usize, Option<&RowKey>) -> RowKey + 'a,
    ) -> Self {
        self.get_row_key = Some(Box::new(f));
        self
    }

    pub fn get_sub_rows(
        mut self,
        f: impl for<'r> Fn(&'r TData, usize) -> Option<&'r [TData]> + 'a,
    ) -> Self {
        self.get_sub_rows = Some(Box::new(f));
        self
    }

    pub fn build(self) -> Table<'a, TData> {
        Table::new(self)
    }
}

pub struct Table<'a, TData> {
    data: &'a [TData],
    columns: Vec<super::ColumnDef<TData>>,
    get_row_key: GetRowKeyFn<'a, TData>,
    get_sub_rows: Option<GetSubRowsFn<'a, TData>>,
    state: super::TableState,
    core_row_model: OnceCell<RowModel<'a, TData>>,
    sorted_row_model: OnceCell<RowModel<'a, TData>>,
    paginated_row_model: OnceCell<RowModel<'a, TData>>,
    selected_row_model: OnceCell<RowModel<'a, TData>>,
}

impl<'a, TData> Table<'a, TData> {
    pub fn builder(data: &'a [TData]) -> TableBuilder<'a, TData> {
        TableBuilder::new(data)
    }

    fn new(builder: TableBuilder<'a, TData>) -> Self {
        let get_row_key = builder
            .get_row_key
            .unwrap_or_else(|| Box::new(default_row_key_for_index_path));
        Self {
            data: builder.data,
            columns: builder.columns,
            get_row_key,
            get_sub_rows: builder.get_sub_rows,
            state: builder.state,
            core_row_model: OnceCell::new(),
            sorted_row_model: OnceCell::new(),
            paginated_row_model: OnceCell::new(),
            selected_row_model: OnceCell::new(),
        }
    }

    pub fn data(&self) -> &'a [TData] {
        self.data
    }

    pub fn columns(&self) -> &[super::ColumnDef<TData>] {
        &self.columns
    }

    pub fn column(&self, id: &str) -> Option<&super::ColumnDef<TData>> {
        self.columns.iter().find(|c| c.id.as_ref() == id)
    }

    pub fn state(&self) -> &super::TableState {
        &self.state
    }

    pub fn ordered_columns(&self) -> Vec<&super::ColumnDef<TData>> {
        super::order_columns(&self.columns, &self.state.column_order)
    }

    pub fn visible_columns(&self) -> Vec<&super::ColumnDef<TData>> {
        self.ordered_columns()
            .into_iter()
            .filter(|c| super::is_column_visible(&self.state.column_visibility, &c.id))
            .collect()
    }

    pub fn pinned_visible_columns(
        &self,
    ) -> (
        Vec<&super::ColumnDef<TData>>,
        Vec<&super::ColumnDef<TData>>,
        Vec<&super::ColumnDef<TData>>,
    ) {
        let visible = self.visible_columns();
        super::split_pinned_columns(visible.as_slice(), &self.state.column_pinning)
    }

    pub fn column_size(&self, id: &str) -> Option<f32> {
        let col = self.column(id)?;
        super::column_size(&self.state.column_sizing, &col.id)
    }

    pub fn core_row_model(&self) -> &RowModel<'a, TData> {
        self.core_row_model.get_or_init(|| {
            build_core_row_model(self.data, &*self.get_row_key, self.get_sub_rows.as_deref())
        })
    }

    pub fn pre_sorted_row_model(&self) -> &RowModel<'a, TData> {
        self.core_row_model()
    }

    pub fn sorted_row_model(&self) -> &RowModel<'a, TData> {
        self.sorted_row_model.get_or_init(|| {
            super::sort_row_model(
                self.pre_sorted_row_model(),
                &self.columns,
                &self.state.sorting,
            )
        })
    }

    pub fn pre_pagination_row_model(&self) -> &RowModel<'a, TData> {
        self.sorted_row_model()
    }

    pub fn row_model(&self) -> &RowModel<'a, TData> {
        self.paginated_row_model.get_or_init(|| {
            super::paginate_row_model(self.pre_pagination_row_model(), self.state.pagination)
        })
    }

    pub fn pre_selected_row_model(&self) -> &RowModel<'a, TData> {
        self.core_row_model()
    }

    pub fn selected_row_model(&self) -> &RowModel<'a, TData> {
        self.selected_row_model.get_or_init(|| {
            super::select_rows_fn(self.pre_selected_row_model(), &self.state.row_selection)
        })
    }
}

fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn default_row_key_for_index_path<TData>(
    _: &TData,
    index: usize,
    parent: Option<&RowKey>,
) -> RowKey {
    if let Some(parent) = parent {
        // Mix parent and child index in an order-sensitive way (avoid trivial collisions like
        // `(parent=0, i=1)` vs `(parent=1, i=0)` that happen with XOR).
        let z = parent
            .0
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add((index as u64).wrapping_add(0xbf58_476d_1ce4_e5b9));
        RowKey(splitmix64(z))
    } else {
        RowKey::from_index(index)
    }
}

fn build_core_row_model<'a, TData>(
    data: &'a [TData],
    get_row_key: &dyn Fn(&TData, usize, Option<&RowKey>) -> RowKey,
    get_sub_rows: Option<&dyn for<'r> Fn(&'r TData, usize) -> Option<&'r [TData]>>,
) -> RowModel<'a, TData> {
    let mut root_rows: Vec<RowIndex> = Vec::new();
    let mut flat_rows: Vec<RowIndex> = Vec::new();
    let mut rows_by_key: HashMap<RowKey, RowIndex> = HashMap::new();
    let mut arena: Vec<Row<'a, TData>> = Vec::new();

    fn access_rows<'a, TData>(
        original_rows: &'a [TData],
        depth: u16,
        parent: Option<RowIndex>,
        parent_key: Option<&RowKey>,
        get_row_key: &dyn Fn(&TData, usize, Option<&RowKey>) -> RowKey,
        get_sub_rows: Option<&dyn for<'r> Fn(&'r TData, usize) -> Option<&'r [TData]>>,
        root_out: &mut Vec<RowIndex>,
        flat_out: &mut Vec<RowIndex>,
        rows_by_key: &mut HashMap<RowKey, RowIndex>,
        arena: &mut Vec<Row<'a, TData>>,
    ) -> Vec<RowIndex> {
        let mut rows: Vec<RowIndex> = Vec::with_capacity(original_rows.len());
        for (index, original) in original_rows.iter().enumerate() {
            let key = get_row_key(original, index, parent_key);
            let row_index = arena.len();
            arena.push(Row {
                key,
                original,
                index,
                depth,
                parent,
                parent_key: parent_key.copied(),
                sub_rows: Vec::new(),
            });
            flat_out.push(row_index);
            rows_by_key.insert(key, row_index);
            rows.push(row_index);

            if let Some(get_sub_rows) = get_sub_rows
                && let Some(sub) = get_sub_rows(original, index)
                && !sub.is_empty()
            {
                let children = access_rows(
                    sub,
                    depth.saturating_add(1),
                    Some(row_index),
                    Some(&key),
                    get_row_key,
                    Some(get_sub_rows),
                    root_out,
                    flat_out,
                    rows_by_key,
                    arena,
                );
                if let Some(row) = arena.get_mut(row_index) {
                    row.sub_rows = children;
                }
            }
        }

        if depth == 0 {
            root_out.extend_from_slice(&rows);
        }

        rows
    }

    let _ = access_rows(
        data,
        0,
        None,
        None,
        get_row_key,
        get_sub_rows,
        &mut root_rows,
        &mut flat_rows,
        &mut rows_by_key,
        &mut arena,
    );

    RowModel {
        root_rows,
        flat_rows,
        rows_by_key,
        arena,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless::table::{PaginationState, SortSpec, TableState, create_column_helper};

    #[derive(Debug, Clone)]
    struct Person {
        name: String,
        sub_rows: Option<Vec<Person>>,
    }

    fn make_people(rows: usize, sub_rows: usize) -> Vec<Person> {
        (0..rows)
            .map(|i| Person {
                name: format!("Person {i}"),
                sub_rows: (sub_rows > 0).then(|| {
                    (0..sub_rows)
                        .map(|j| Person {
                            name: format!("Person {i}.{j}"),
                            sub_rows: None,
                        })
                        .collect()
                }),
            })
            .collect()
    }

    #[test]
    fn core_row_model_produces_flat_rows_and_key_map() {
        let data = make_people(3, 0);
        let table = Table::builder(&data).build();
        let model = table.core_row_model();

        assert_eq!(model.root_rows().len(), 3);
        assert_eq!(model.flat_rows().len(), 3);
        assert!(model.row_by_key(RowKey::from_index(0)).is_some());
        assert!(model.row_by_key(RowKey::from_index(1)).is_some());
        assert!(model.row_by_key(RowKey::from_index(2)).is_some());
    }

    #[test]
    fn core_row_model_recurses_into_sub_rows_and_assigns_unique_keys() {
        let data = make_people(3, 2);
        let table = Table::builder(&data)
            .get_sub_rows(|p, _| p.sub_rows.as_deref())
            .build();
        let model = table.core_row_model();

        assert_eq!(model.root_rows().len(), 3);
        assert_eq!(model.flat_rows().len(), 3 + 3 * 2);

        let root_0 = model.row(model.root_rows()[0]).expect("root row 0");
        assert_eq!(root_0.sub_rows.len(), 2);

        let c0 = model.row(root_0.sub_rows[0]).expect("root 0 child 0").key;
        let c1 = model.row(root_0.sub_rows[1]).expect("root 0 child 1").key;
        assert_ne!(c0, c1);
        assert_ne!(c0, root_0.key);
        assert_ne!(c1, root_0.key);
        assert!(model.row_by_key(c0).is_some());
        assert!(model.row_by_key(c1).is_some());
    }

    #[test]
    fn core_row_model_allows_custom_stable_row_keys() {
        let data = make_people(2, 0);
        let table = Table::builder(&data)
            .get_row_key(|_p, i, _parent| RowKey(10_000 + i as u64))
            .build();
        let model = table.core_row_model();

        assert!(model.row_by_key(RowKey(10_000)).is_some());
        assert!(model.row_by_key(RowKey(10_001)).is_some());
        assert!(model.row_by_key(RowKey::from_index(0)).is_none());
    }

    #[derive(Debug, Clone)]
    struct Item {
        value: i32,
    }

    #[test]
    fn table_sorted_row_model_uses_state_sorting() {
        let data = vec![Item { value: 2 }, Item { value: 1 }, Item { value: 3 }];

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];

        let table = Table::builder(&data)
            .columns(columns)
            .state(TableState {
                sorting: vec![SortSpec {
                    column: "value".into(),
                    desc: false,
                }],
                ..TableState::default()
            })
            .build();

        let sorted = table.sorted_row_model();
        let keys = sorted
            .root_rows()
            .iter()
            .filter_map(|&i| sorted.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(keys, vec![1, 0, 2]);
        assert!(std::ptr::eq(sorted, table.sorted_row_model()));
    }

    #[test]
    fn table_row_model_applies_pagination_after_sorting() {
        let data = vec![
            Item { value: 2 },
            Item { value: 1 },
            Item { value: 3 },
            Item { value: 0 },
        ];

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];

        let table = Table::builder(&data)
            .columns(columns)
            .state(TableState {
                sorting: vec![SortSpec {
                    column: "value".into(),
                    desc: false,
                }],
                pagination: PaginationState {
                    page_index: 0,
                    page_size: 2,
                },
                ..TableState::default()
            })
            .build();

        let paged = table.row_model();
        let keys = paged
            .root_rows()
            .iter()
            .filter_map(|&i| paged.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(keys, vec![3, 1]);
        assert!(std::ptr::eq(paged, table.row_model()));
    }

    #[test]
    fn table_selected_row_model_uses_state_row_selection() {
        let data = make_people(3, 0);
        let table = Table::builder(&data)
            .state(TableState {
                row_selection: [RowKey(1)].into_iter().collect(),
                ..TableState::default()
            })
            .build();

        let selected = table.selected_row_model();
        assert_eq!(selected.root_rows().len(), 1);
        assert!(selected.row_by_key(RowKey(1)).is_some());
        assert!(std::ptr::eq(selected, table.selected_row_model()));
    }

    #[test]
    fn table_visible_columns_respects_order_then_visibility() {
        let data = vec![Item { value: 1 }];

        let helper = create_column_helper::<Item>();
        let columns = vec![
            helper.clone().accessor("a", |it| it.value),
            helper.clone().accessor("b", |it| it.value),
            helper.accessor("c", |it| it.value),
        ];

        let mut state = TableState::default();
        state.column_order = vec!["c".into(), "a".into()];
        state.column_visibility.insert("b".into(), false);

        let table = Table::builder(&data).columns(columns).state(state).build();
        let visible = table.visible_columns();
        let ids = visible.iter().map(|c| c.id.as_ref()).collect::<Vec<_>>();

        assert_eq!(ids, vec!["c", "a"]);
    }

    #[test]
    fn table_column_size_reads_from_state() {
        let data = vec![Item { value: 1 }];

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];

        let mut state = TableState::default();
        state.column_sizing.insert("value".into(), 120.0);

        let table = Table::builder(&data).columns(columns).state(state).build();
        assert_eq!(table.column_size("value"), Some(120.0));
        assert_eq!(table.column_size("missing"), None);
    }
}
