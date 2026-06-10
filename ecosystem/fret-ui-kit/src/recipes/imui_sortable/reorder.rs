use super::SortableInsertionSide;

/// Reorder a vector by stable keys while keeping the actual mutation app-owned.
pub fn reorder_vec_by_key<T, K: ?Sized + PartialEq>(
    items: &mut Vec<T>,
    active_key: &K,
    over_key: &K,
    side: SortableInsertionSide,
    mut key_of: impl for<'a> FnMut(&'a T) -> &'a K,
) -> bool {
    if active_key == over_key {
        return false;
    }

    let Some(from_index) = items.iter().position(|item| key_of(item) == active_key) else {
        return false;
    };
    let Some(over_index_before_remove) = items.iter().position(|item| key_of(item) == over_key)
    else {
        return false;
    };

    let moving = items.remove(from_index);
    let mut insert_index = items
        .iter()
        .position(|item| key_of(item) == over_key)
        .unwrap_or(over_index_before_remove.min(items.len()));
    if side == SortableInsertionSide::After {
        insert_index = insert_index.saturating_add(1).min(items.len());
    }

    items.insert(insert_index, moving);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestItem {
        id: &'static str,
    }

    #[test]
    fn reorder_vec_by_key_moves_item_after_target() {
        let mut items = vec![
            TestItem { id: "camera" },
            TestItem { id: "cube" },
            TestItem { id: "light" },
        ];

        assert!(reorder_vec_by_key(
            &mut items,
            "camera",
            "cube",
            SortableInsertionSide::After,
            |item| item.id,
        ));
        assert_eq!(
            items.iter().map(|item| item.id).collect::<Vec<_>>(),
            vec!["cube", "camera", "light"]
        );
    }

    #[test]
    fn reorder_vec_by_key_moves_item_before_target() {
        let mut items = vec![
            TestItem { id: "camera" },
            TestItem { id: "cube" },
            TestItem { id: "light" },
        ];

        assert!(reorder_vec_by_key(
            &mut items,
            "light",
            "cube",
            SortableInsertionSide::Before,
            |item| item.id,
        ));
        assert_eq!(
            items.iter().map(|item| item.id).collect::<Vec<_>>(),
            vec!["camera", "light", "cube"]
        );
    }
}
