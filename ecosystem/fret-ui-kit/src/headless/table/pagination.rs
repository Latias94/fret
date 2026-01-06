use super::{RowIndex, RowModel};

#[derive(Debug, Clone, Copy)]
pub struct PaginationState {
    pub page_index: usize,
    pub page_size: usize,
}

impl Default for PaginationState {
    fn default() -> Self {
        Self {
            page_index: 0,
            page_size: 10,
        }
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
    use crate::headless::table::{RowKey, Table};

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
}
