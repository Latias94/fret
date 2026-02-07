use super::{RowIndex, RowModel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaginationState {
    pub page_index: usize,
    pub page_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PaginationBounds {
    pub page_index: usize,
    pub page_count: usize,
    pub can_prev: bool,
    pub can_next: bool,
    pub page_start: usize,
    pub page_end: usize,
}

impl Default for PaginationState {
    fn default() -> Self {
        Self {
            page_index: 0,
            page_size: 10,
        }
    }
}

pub fn pagination_bounds(total_rows: usize, pagination: PaginationState) -> PaginationBounds {
    if pagination.page_size == 0 || total_rows == 0 {
        return PaginationBounds {
            page_index: 0,
            page_count: 0,
            can_prev: false,
            can_next: false,
            page_start: 0,
            page_end: 0,
        };
    }

    let page_count = total_rows.div_ceil(pagination.page_size);
    let page_index = pagination.page_index.min(page_count.saturating_sub(1));
    let page_start = page_index.saturating_mul(pagination.page_size);
    let page_end = (page_start.saturating_add(pagination.page_size)).min(total_rows);

    PaginationBounds {
        page_index,
        page_count,
        can_prev: page_index > 0,
        can_next: page_index + 1 < page_count,
        page_start,
        page_end,
    }
}

pub fn paginate_row_model<'a, TData>(
    row_model: &RowModel<'a, TData>,
    pagination: PaginationState,
) -> RowModel<'a, TData> {
    if row_model.root_rows().is_empty() {
        return row_model.clone();
    }
    if pagination.page_size == 0 {
        return RowModel {
            root_rows: Vec::new(),
            flat_rows: Vec::new(),
            rows_by_key: row_model.rows_by_key().clone(),
            rows_by_id: row_model.rows_by_id().clone(),
            arena: row_model.arena().to_vec(),
        };
    }

    let page_start = pagination.page_index.saturating_mul(pagination.page_size);
    let page_end = page_start.saturating_add(pagination.page_size);

    let mut out = row_model.clone();
    let roots = out.root_rows.clone();
    let start = page_start.min(roots.len());
    let end = page_end.min(roots.len());
    out.root_rows = roots[start..end].to_vec();

    out.flat_rows.clear();
    fn push_flat<TData>(row_model: &mut RowModel<'_, TData>, row: RowIndex) {
        row_model.flat_rows.push(row);
        let Some(r) = row_model.row(row) else {
            return;
        };
        let children = r.sub_rows.clone();
        for child in children {
            push_flat(row_model, child);
        }
    }

    let roots = out.root_rows.clone();
    for root in roots {
        push_flat(&mut out, root);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::{RowKey, Table};

    #[derive(Debug, Clone)]
    struct Item {
        #[allow(dead_code)]
        value: i32,
    }

    #[test]
    fn paginate_row_model_slices_root_rows_and_rebuilds_flat_rows() {
        let data = (0..5).map(|i| Item { value: i }).collect::<Vec<_>>();
        let table = Table::builder(&data).build();
        let core = table.core_row_model();

        let paged = paginate_row_model(
            core,
            PaginationState {
                page_index: 1,
                page_size: 2,
            },
        );

        let ids = paged
            .root_rows()
            .iter()
            .filter_map(|&i| paged.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();
        assert_eq!(ids, vec![2, 3]);
        assert_eq!(paged.flat_rows().len(), 2);
        assert!(
            paged.row_by_key(RowKey::from_index(0)).is_some(),
            "rows_by_key remains full"
        );
        assert!(
            paged.row_by_key(RowKey::from_index(4)).is_some(),
            "rows_by_key remains full"
        );
    }

    #[test]
    fn paginate_row_model_clamps_page_end_to_row_count() {
        let data = (0..5).map(|i| Item { value: i }).collect::<Vec<_>>();
        let table = Table::builder(&data).build();
        let core = table.core_row_model();

        let paged = paginate_row_model(
            core,
            PaginationState {
                page_index: 0,
                page_size: 10,
            },
        );

        let ids = paged
            .root_rows()
            .iter()
            .filter_map(|&i| paged.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();
        assert_eq!(ids, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn pagination_bounds_clamps_and_reports_can_next_prev() {
        let b = pagination_bounds(
            5,
            PaginationState {
                page_index: 0,
                page_size: 2,
            },
        );
        assert_eq!(
            b,
            PaginationBounds {
                page_index: 0,
                page_count: 3,
                can_prev: false,
                can_next: true,
                page_start: 0,
                page_end: 2,
            }
        );

        let b = pagination_bounds(
            5,
            PaginationState {
                page_index: 10,
                page_size: 2,
            },
        );
        assert_eq!(b.page_index, 2);
        assert_eq!(b.page_count, 3);
        assert!(!b.can_next);
        assert!(b.can_prev);
        assert_eq!((b.page_start, b.page_end), (4, 5));

        let b = pagination_bounds(
            0,
            PaginationState {
                page_index: 0,
                page_size: 10,
            },
        );
        assert_eq!(b.page_count, 0);
        assert_eq!(b.page_index, 0);
        assert!(!b.can_prev);
        assert!(!b.can_next);
    }
}
