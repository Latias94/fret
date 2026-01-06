use std::cmp::Ordering;
use std::sync::Arc;

pub type ColumnId = Arc<str>;

pub type SortCmpFn<TData> = Arc<dyn Fn(&TData, &TData) -> Ordering>;
pub type FilterFn<TData> = Arc<dyn Fn(&TData, &str) -> bool>;
pub type FacetKeyFn<TData> = Arc<dyn Fn(&TData) -> u64>;
pub type FacetStrFn<TData> = Arc<dyn for<'r> Fn(&'r TData) -> &'r str>;

#[derive(Clone)]
pub struct ColumnDef<TData> {
    pub id: ColumnId,
    pub sort_cmp: Option<SortCmpFn<TData>>,
    pub filter_fn: Option<FilterFn<TData>>,
    pub facet_key_fn: Option<FacetKeyFn<TData>>,
    pub facet_str_fn: Option<FacetStrFn<TData>>,
    pub enable_hiding: bool,
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
            facet_key_fn: None,
            facet_str_fn: None,
            enable_hiding: true,
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

    /// Provide a stable `u64` facet key for this column (TanStack-aligned faceting, Rust-native).
    pub fn facet_key_by(mut self, f: impl Fn(&TData) -> u64 + 'static) -> Self {
        self.facet_key_fn = Some(Arc::new(f));
        self
    }

    /// Provide a string view for this column's facet value (borrowed from row data; no allocation).
    pub fn facet_str_by(mut self, f: impl for<'r> Fn(&'r TData) -> &'r str + 'static) -> Self {
        self.facet_str_fn = Some(Arc::new(f));
        self
    }

    pub fn enable_hiding(mut self, enabled: bool) -> Self {
        self.enable_hiding = enabled;
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
