use std::collections::HashMap;
use std::sync::Arc;

use super::ColumnDef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderGroupSnapshot {
    pub depth: usize,
    pub id: Arc<str>,
    pub headers: Vec<HeaderSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderSnapshot {
    pub id: Arc<str>,
    pub column_id: Arc<str>,
    pub depth: usize,
    pub index: usize,
    pub is_placeholder: bool,
    pub placeholder_id: Option<Arc<str>>,
    pub col_span: usize,
    pub row_span: usize,
    pub sub_header_ids: Vec<Arc<str>>,
}

#[derive(Debug, Clone)]
struct ColumnMeta {
    depth: usize,
    parent: Option<Arc<str>>,
    children: Vec<Arc<str>>,
}

#[derive(Debug, Clone)]
struct HeaderNode {
    id: Arc<str>,
    column_id: Arc<str>,
    depth: usize,
    index: usize,
    is_placeholder: bool,
    placeholder_id: Option<Arc<str>>,
    col_span: usize,
    row_span: usize,
    sub_headers: Vec<usize>,
}

fn build_column_meta<TData>(
    columns: &[ColumnDef<TData>],
    depth: usize,
    parent: Option<Arc<str>>,
    out: &mut HashMap<Arc<str>, ColumnMeta>,
) {
    for col in columns {
        let id = col.id.clone();
        let children: Vec<Arc<str>> = col.columns.iter().map(|c| c.id.clone()).collect();
        out.insert(
            id.clone(),
            ColumnMeta {
                depth,
                parent: parent.clone(),
                children,
            },
        );
        if !col.columns.is_empty() {
            build_column_meta(&col.columns, depth + 1, Some(id), out);
        }
    }
}

fn column_is_visible(
    id: &str,
    meta: &HashMap<Arc<str>, ColumnMeta>,
    leaf_visible: &dyn Fn(&str) -> bool,
    cache: &mut HashMap<Arc<str>, bool>,
) -> bool {
    if let Some(&v) = cache.get(id) {
        return v;
    }

    let Some(m) = meta.get(id) else {
        let v = leaf_visible(id);
        cache.insert(Arc::<str>::from(id), v);
        return v;
    };

    let v = if m.children.is_empty() {
        leaf_visible(id)
    } else {
        m.children
            .iter()
            .any(|child| column_is_visible(child.as_ref(), meta, leaf_visible, cache))
    };

    cache.insert(Arc::<str>::from(id), v);
    v
}

fn find_max_depth<TData>(
    columns: &[ColumnDef<TData>],
    depth_1_based: usize,
    meta: &HashMap<Arc<str>, ColumnMeta>,
    leaf_visible: &dyn Fn(&str) -> bool,
    visible_cache: &mut HashMap<Arc<str>, bool>,
    max_depth: &mut usize,
) {
    *max_depth = (*max_depth).max(depth_1_based);
    for col in columns {
        if !column_is_visible(col.id.as_ref(), meta, leaf_visible, visible_cache) {
            continue;
        }
        if !col.columns.is_empty() {
            find_max_depth(
                &col.columns,
                depth_1_based + 1,
                meta,
                leaf_visible,
                visible_cache,
                max_depth,
            );
        }
    }
}

fn header_group_id(header_family: Option<&str>, depth: usize) -> Arc<str> {
    if let Some(family) = header_family {
        Arc::<str>::from(format!("{}_{}", family, depth))
    } else {
        Arc::<str>::from(depth.to_string())
    }
}

fn header_id(
    header_family: Option<&str>,
    depth: usize,
    column_id: &str,
    child_header_id: &str,
) -> Arc<str> {
    if let Some(family) = header_family {
        Arc::<str>::from(format!(
            "{}_{}_{}_{}",
            family, depth, column_id, child_header_id
        ))
    } else {
        Arc::<str>::from(format!("{}_{}_{}", depth, column_id, child_header_id))
    }
}

fn create_header_group(
    headers_to_group: Vec<usize>,
    depth: usize,
    meta: &HashMap<Arc<str>, ColumnMeta>,
    leaf_visible: &dyn Fn(&str) -> bool,
    visible_cache: &mut HashMap<Arc<str>, bool>,
    header_family: Option<&str>,
    header_groups: &mut Vec<(usize, Arc<str>, Vec<usize>)>,
    arena: &mut Vec<HeaderNode>,
) {
    let group_id = header_group_id(header_family, depth);
    let mut pending_parent_headers: Vec<usize> = Vec::new();

    for &header_to_group_idx in &headers_to_group {
        let column_id = arena[header_to_group_idx].column_id.clone();
        let Some(col_meta) = meta.get(column_id.as_ref()) else {
            continue;
        };

        let is_leaf_header = col_meta.depth == depth;

        let mut parent_column_id: Arc<str> = column_id.clone();
        let mut is_placeholder = false;

        if is_leaf_header {
            if let Some(parent) = col_meta.parent.clone() {
                parent_column_id = parent;
            } else {
                is_placeholder = true;
            }
        } else {
            is_placeholder = true;
        }

        if let Some(&latest_idx) = pending_parent_headers.last() {
            if arena[latest_idx].column_id.as_ref() == parent_column_id.as_ref() {
                arena[latest_idx].sub_headers.push(header_to_group_idx);
                continue;
            }
        }

        let placeholder_id = if is_placeholder {
            let count = pending_parent_headers
                .iter()
                .filter(|&&idx| arena[idx].column_id.as_ref() == parent_column_id.as_ref())
                .count();
            Some(Arc::<str>::from(count.to_string()))
        } else {
            None
        };

        let child_header_id = arena[header_to_group_idx].id.clone();

        let header_idx = arena.len();
        arena.push(HeaderNode {
            id: header_id(
                header_family,
                depth,
                parent_column_id.as_ref(),
                child_header_id.as_ref(),
            ),
            column_id: parent_column_id,
            depth,
            index: pending_parent_headers.len(),
            is_placeholder,
            placeholder_id,
            col_span: 0,
            row_span: 0,
            sub_headers: vec![header_to_group_idx],
        });
        pending_parent_headers.push(header_idx);
    }

    header_groups.push((depth, group_id, headers_to_group));

    if depth > 0 {
        create_header_group(
            pending_parent_headers,
            depth - 1,
            meta,
            leaf_visible,
            visible_cache,
            header_family,
            header_groups,
            arena,
        );
    }
}

fn recurse_headers_for_spans(
    headers: &[usize],
    meta: &HashMap<Arc<str>, ColumnMeta>,
    leaf_visible: &dyn Fn(&str) -> bool,
    visible_cache: &mut HashMap<Arc<str>, bool>,
    arena: &mut [HeaderNode],
) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for &idx in headers {
        if !column_is_visible(
            arena[idx].column_id.as_ref(),
            meta,
            leaf_visible,
            visible_cache,
        ) {
            continue;
        }

        let mut col_span = 0usize;
        let mut row_span = 0usize;
        let mut child_row_spans = vec![0usize];

        let sub_headers = arena[idx].sub_headers.clone();
        if !sub_headers.is_empty() {
            child_row_spans.clear();
            for (child_col_span, child_row_span) in
                recurse_headers_for_spans(&sub_headers, meta, leaf_visible, visible_cache, arena)
            {
                col_span += child_col_span;
                child_row_spans.push(child_row_span);
            }
        } else {
            col_span = 1;
        }

        let min_child_row_span = child_row_spans.into_iter().min().unwrap_or(0);
        row_span += min_child_row_span;

        arena[idx].col_span = col_span;
        arena[idx].row_span = row_span;

        out.push((col_span, row_span));
    }
    out
}

fn snapshot_group(headers: &[usize], arena: &[HeaderNode]) -> Vec<HeaderSnapshot> {
    headers
        .iter()
        .copied()
        .map(|idx| HeaderSnapshot {
            id: arena[idx].id.clone(),
            column_id: arena[idx].column_id.clone(),
            depth: arena[idx].depth,
            index: arena[idx].index,
            is_placeholder: arena[idx].is_placeholder,
            placeholder_id: arena[idx].placeholder_id.clone(),
            col_span: arena[idx].col_span,
            row_span: arena[idx].row_span,
            sub_header_ids: arena[idx]
                .sub_headers
                .iter()
                .map(|&child| arena[child].id.clone())
                .collect(),
        })
        .collect()
}

/// TanStack-aligned header-group builder (`buildHeaderGroups`).
///
/// - `all_columns`: the full column def tree (group columns + leaf columns).
/// - `columns_to_group`: ordered leaf column ids (after pinning re-order, if any).
/// - `leaf_visible`: leaf-level visibility predicate (group visibility is derived from children).
pub fn build_header_groups<TData>(
    all_columns: &[ColumnDef<TData>],
    columns_to_group: &[Arc<str>],
    leaf_visible: &dyn Fn(&str) -> bool,
    header_family: Option<&str>,
) -> Vec<HeaderGroupSnapshot> {
    let mut meta: HashMap<Arc<str>, ColumnMeta> = HashMap::new();
    build_column_meta(all_columns, 0, None, &mut meta);

    let mut visible_cache: HashMap<Arc<str>, bool> = HashMap::new();

    let mut max_depth = 0usize;
    find_max_depth(
        all_columns,
        1,
        &meta,
        leaf_visible,
        &mut visible_cache,
        &mut max_depth,
    );

    if max_depth == 0 {
        return Vec::new();
    }

    let mut arena: Vec<HeaderNode> = Vec::new();
    let bottom_headers: Vec<usize> = columns_to_group
        .iter()
        .enumerate()
        .map(|(index, col_id)| {
            let idx = arena.len();
            arena.push(HeaderNode {
                id: col_id.clone(),
                column_id: col_id.clone(),
                depth: max_depth,
                index,
                is_placeholder: false,
                placeholder_id: None,
                col_span: 0,
                row_span: 0,
                sub_headers: Vec::new(),
            });
            idx
        })
        .collect();

    let mut header_groups: Vec<(usize, Arc<str>, Vec<usize>)> = Vec::new();
    create_header_group(
        bottom_headers,
        max_depth.saturating_sub(1),
        &meta,
        leaf_visible,
        &mut visible_cache,
        header_family,
        &mut header_groups,
        &mut arena,
    );

    header_groups.reverse();

    if let Some((_, _, headers)) = header_groups.first() {
        recurse_headers_for_spans(
            headers,
            &meta,
            leaf_visible,
            &mut visible_cache,
            arena.as_mut_slice(),
        );
    }

    header_groups
        .into_iter()
        .map(|(depth, id, headers)| HeaderGroupSnapshot {
            depth,
            id,
            headers: snapshot_group(&headers, &arena),
        })
        .collect()
}
