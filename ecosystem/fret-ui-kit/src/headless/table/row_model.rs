use std::cell::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Stable identifier for a row in the table.
///
/// This is intentionally string-based (TanStack-compatible), because the default row-id strategy
/// uses index paths like `0.1.2`. For data sourced from a backend, callers should supply their own
/// stable IDs (e.g. a database primary key).
pub type RowId = Arc<str>;

/// Index into a [`RowModel`] arena.
pub type RowIndex = usize;

#[derive(Debug)]
pub struct Row<'a, TData> {
    pub id: RowId,
    pub original: &'a TData,
    pub index: usize,
    pub depth: u16,
    pub parent: Option<RowIndex>,
    pub parent_id: Option<RowId>,
    pub sub_rows: Vec<RowIndex>,
}

impl<'a, TData> Clone for Row<'a, TData> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            original: self.original,
            index: self.index,
            depth: self.depth,
            parent: self.parent,
            parent_id: self.parent_id.clone(),
            sub_rows: self.sub_rows.clone(),
        }
    }
}

#[derive(Debug)]
pub struct RowModel<'a, TData> {
    pub(super) root_rows: Vec<RowIndex>,
    pub(super) flat_rows: Vec<RowIndex>,
    pub(super) rows_by_id: HashMap<RowId, RowIndex>,
    pub(super) arena: Vec<Row<'a, TData>>,
}

impl<'a, TData> Clone for RowModel<'a, TData> {
    fn clone(&self) -> Self {
        Self {
            root_rows: self.root_rows.clone(),
            flat_rows: self.flat_rows.clone(),
            rows_by_id: self.rows_by_id.clone(),
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

    pub fn row_by_id(&self, id: &str) -> Option<RowIndex> {
        self.rows_by_id.get(id).copied()
    }

    pub fn rows_by_id(&self) -> &HashMap<RowId, RowIndex> {
        &self.rows_by_id
    }

    pub fn arena(&self) -> &[Row<'a, TData>] {
        &self.arena
    }
}

type GetRowIdFn<'a, TData> = Box<dyn Fn(&TData, usize, Option<&RowId>) -> RowId + 'a>;
type GetSubRowsFn<'a, TData> = Box<dyn for<'r> Fn(&'r TData, usize) -> Option<&'r [TData]> + 'a>;

pub struct TableBuilder<'a, TData> {
    data: &'a [TData],
    columns: Vec<super::ColumnDef<TData>>,
    get_row_id: Option<GetRowIdFn<'a, TData>>,
    get_sub_rows: Option<GetSubRowsFn<'a, TData>>,
    state: super::TableState,
}

impl<'a, TData> TableBuilder<'a, TData> {
    pub fn new(data: &'a [TData]) -> Self {
        Self {
            data,
            columns: Vec::new(),
            get_row_id: None,
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

    pub fn get_row_id(mut self, f: impl Fn(&TData, usize, Option<&RowId>) -> RowId + 'a) -> Self {
        self.get_row_id = Some(Box::new(f));
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
    get_row_id: GetRowIdFn<'a, TData>,
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
        let get_row_id = builder
            .get_row_id
            .unwrap_or_else(|| Box::new(default_row_id_for_index_path));
        Self {
            data: builder.data,
            columns: builder.columns,
            get_row_id,
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

    pub fn state(&self) -> &super::TableState {
        &self.state
    }

    pub fn core_row_model(&self) -> &RowModel<'a, TData> {
        self.core_row_model.get_or_init(|| {
            build_core_row_model(self.data, &*self.get_row_id, self.get_sub_rows.as_deref())
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

fn default_row_id_for_index_path<TData>(_: &TData, index: usize, parent: Option<&RowId>) -> RowId {
    if let Some(parent) = parent {
        Arc::from(format!("{parent}.{index}"))
    } else {
        Arc::from(index.to_string())
    }
}

fn build_core_row_model<'a, TData>(
    data: &'a [TData],
    get_row_id: &dyn Fn(&TData, usize, Option<&RowId>) -> RowId,
    get_sub_rows: Option<&dyn for<'r> Fn(&'r TData, usize) -> Option<&'r [TData]>>,
) -> RowModel<'a, TData> {
    let mut root_rows: Vec<RowIndex> = Vec::new();
    let mut flat_rows: Vec<RowIndex> = Vec::new();
    let mut rows_by_id: HashMap<RowId, RowIndex> = HashMap::new();
    let mut arena: Vec<Row<'a, TData>> = Vec::new();

    fn access_rows<'a, TData>(
        original_rows: &'a [TData],
        depth: u16,
        parent: Option<RowIndex>,
        parent_id: Option<&RowId>,
        get_row_id: &dyn Fn(&TData, usize, Option<&RowId>) -> RowId,
        get_sub_rows: Option<&dyn for<'r> Fn(&'r TData, usize) -> Option<&'r [TData]>>,
        root_out: &mut Vec<RowIndex>,
        flat_out: &mut Vec<RowIndex>,
        rows_by_id: &mut HashMap<RowId, RowIndex>,
        arena: &mut Vec<Row<'a, TData>>,
    ) -> Vec<RowIndex> {
        let mut rows: Vec<RowIndex> = Vec::with_capacity(original_rows.len());
        for (index, original) in original_rows.iter().enumerate() {
            let id = get_row_id(original, index, parent_id);
            let row_index = arena.len();
            arena.push(Row {
                id: id.clone(),
                original,
                index,
                depth,
                parent,
                parent_id: parent_id.cloned(),
                sub_rows: Vec::new(),
            });
            flat_out.push(row_index);
            rows_by_id.insert(id.clone(), row_index);
            rows.push(row_index);

            if let Some(get_sub_rows) = get_sub_rows
                && let Some(sub) = get_sub_rows(original, index)
                && !sub.is_empty()
            {
                let children = access_rows(
                    sub,
                    depth.saturating_add(1),
                    Some(row_index),
                    Some(&id),
                    get_row_id,
                    Some(get_sub_rows),
                    root_out,
                    flat_out,
                    rows_by_id,
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
        get_row_id,
        get_sub_rows,
        &mut root_rows,
        &mut flat_rows,
        &mut rows_by_id,
        &mut arena,
    );

    RowModel {
        root_rows,
        flat_rows,
        rows_by_id,
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
    fn core_row_model_produces_flat_rows_and_id_map() {
        let data = make_people(3, 0);
        let table = Table::builder(&data).build();
        let model = table.core_row_model();

        assert_eq!(model.root_rows().len(), 3);
        assert_eq!(model.flat_rows().len(), 3);
        assert!(model.row_by_id("0").is_some());
        assert!(model.row_by_id("1").is_some());
        assert!(model.row_by_id("2").is_some());
    }

    #[test]
    fn core_row_model_recurses_into_sub_rows_and_uses_index_path_ids() {
        let data = make_people(3, 2);
        let table = Table::builder(&data)
            .get_sub_rows(|p, _| p.sub_rows.as_deref())
            .build();
        let model = table.core_row_model();

        assert_eq!(model.root_rows().len(), 3);
        assert_eq!(model.flat_rows().len(), 3 + 3 * 2);

        let root_0 = model.row(model.root_rows()[0]).expect("root row 0");
        assert_eq!(root_0.sub_rows.len(), 2);

        assert!(model.row_by_id("0.0").is_some());
        assert!(model.row_by_id("0.1").is_some());
        assert!(model.row_by_id("2.1").is_some());
    }

    #[test]
    fn core_row_model_allows_custom_stable_row_ids() {
        let data = make_people(2, 0);
        let table = Table::builder(&data)
            .get_row_id(|p, _i, _parent| Arc::from(p.name.as_str()))
            .build();
        let model = table.core_row_model();

        assert!(model.row_by_id("Person 0").is_some());
        assert!(model.row_by_id("Person 1").is_some());
        assert!(model.row_by_id("0").is_none());
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
        let ids = sorted
            .root_rows()
            .iter()
            .filter_map(|&i| sorted.row(i).map(|r| r.id.as_ref().to_string()))
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["1", "0", "2"]);
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
        let ids = paged
            .root_rows()
            .iter()
            .filter_map(|&i| paged.row(i).map(|r| r.id.as_ref().to_string()))
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["3", "1"]);
        assert!(std::ptr::eq(paged, table.row_model()));
    }

    #[test]
    fn table_selected_row_model_uses_state_row_selection() {
        let data = make_people(3, 0);
        let table = Table::builder(&data)
            .state(TableState {
                row_selection: [("1", true)]
                    .into_iter()
                    .map(|(id, v)| (Arc::from(id), v))
                    .collect(),
                ..TableState::default()
            })
            .build();

        let selected = table.selected_row_model();
        assert_eq!(selected.root_rows().len(), 1);
        assert!(selected.row_by_id("1").is_some());
        assert!(std::ptr::eq(selected, table.selected_row_model()));
    }
}
