use std::sync::Arc;

use super::super::EnumSelectItem;

#[cfg(test)]
mod tests;

pub(super) fn filter_enum_select_items(
    items: &[EnumSelectItem],
    query: &str,
) -> Arc<[EnumSelectItem]> {
    let q = query.trim().to_lowercase();
    let matches = |s: &str| q.is_empty() || s.to_lowercase().contains(&q);

    items
        .iter()
        .filter(|it| matches(it.label.as_ref()) || matches(it.value.as_ref()))
        .cloned()
        .collect::<Vec<_>>()
        .into()
}
