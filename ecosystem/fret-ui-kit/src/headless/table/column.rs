use std::cmp::Ordering;
use std::sync::Arc;

pub type ColumnId = Arc<str>;

pub type SortCmpFn<TData> = Arc<dyn Fn(&TData, &TData) -> Ordering>;
pub type FilterFn<TData> = Arc<dyn Fn(&TData, &str) -> bool>;

#[derive(Clone)]
pub struct ColumnDef<TData> {
    pub id: ColumnId,
    pub sort_cmp: Option<SortCmpFn<TData>>,
    pub filter_fn: Option<FilterFn<TData>>,
}

impl<TData> std::fmt::Debug for ColumnDef<TData> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColumnDef")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl<TData> ColumnDef<TData> {
    pub fn new(id: impl Into<ColumnId>) -> Self {
        Self {
            id: id.into(),
            sort_cmp: None,
            filter_fn: None,
        }
    }

    pub fn sort_by(mut self, cmp: impl Fn(&TData, &TData) -> Ordering + 'static) -> Self {
        self.sort_cmp = Some(Arc::new(cmp));
        self
    }

    pub fn filter_by(mut self, f: impl Fn(&TData, &str) -> bool + 'static) -> Self {
        self.filter_fn = Some(Arc::new(f));
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnHelper<TData> {
    _marker: std::marker::PhantomData<TData>,
}

pub fn create_column_helper<TData>() -> ColumnHelper<TData> {
    ColumnHelper {
        _marker: std::marker::PhantomData,
    }
}

impl<TData> ColumnHelper<TData> {
    pub fn accessor<V>(
        self,
        id: impl Into<ColumnId>,
        accessor: impl Fn(&TData) -> V + 'static,
    ) -> ColumnDef<TData>
    where
        V: Ord,
    {
        let accessor = Arc::new(accessor);
        ColumnDef::new(id).sort_by(move |a, b| accessor(a).cmp(&accessor(b)))
    }
}
