//! Immediate multi-select state storage and normalization.

/// Model state for an immediate multi-select collection.
///
/// This is intentionally small:
/// - `selected` stores the currently selected keys,
/// - `anchor` stores the range-selection anchor used for shift-click expansion.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImUiMultiSelectState<K> {
    selected: Vec<K>,
    anchor: Option<K>,
}

impl<K> ImUiMultiSelectState<K> {
    pub fn new(selected: Vec<K>, anchor: Option<K>) -> Self {
        Self { selected, anchor }
    }

    pub fn selected(&self) -> &[K] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    pub fn anchor(&self) -> Option<&K> {
        self.anchor.as_ref()
    }

    pub fn first_selected(&self) -> Option<&K> {
        self.selected.first()
    }

    pub fn clear(&mut self) {
        self.selected.clear();
        self.anchor = None;
    }
}

impl<K: Clone> ImUiMultiSelectState<K> {
    pub fn single(key: K) -> Self {
        Self {
            selected: vec![key.clone()],
            anchor: Some(key),
        }
    }
}

impl<K: Clone + PartialEq> ImUiMultiSelectState<K> {
    pub fn from_ordered_selection(all_keys: &[K], selected: Vec<K>, anchor: Option<K>) -> Self {
        let selected = normalize_selection_order(all_keys, selected);
        let anchor = anchor
            .filter(|anchor| selected.iter().any(|item| item == anchor))
            .or_else(|| selected.first().cloned());
        Self { selected, anchor }
    }

    pub(super) fn replace_with_single(&mut self, key: &K) {
        self.selected = vec![key.clone()];
        self.anchor = Some(key.clone());
    }

    pub(super) fn range_select_from_anchor_or_single(&mut self, all_keys: &[K], key: &K) {
        let anchor = self.anchor.clone().unwrap_or_else(|| key.clone());
        let Some(anchor_index) = all_keys.iter().position(|item| item == &anchor) else {
            self.replace_with_single(key);
            return;
        };
        let Some(key_index) = all_keys.iter().position(|item| item == key) else {
            self.replace_with_single(key);
            return;
        };

        let (start, end) = if anchor_index <= key_index {
            (anchor_index, key_index)
        } else {
            (key_index, anchor_index)
        };
        self.selected = all_keys[start..=end].to_vec();
        self.anchor = Some(anchor);
    }

    pub(super) fn toggle_in_order(&mut self, all_keys: &[K], key: &K) {
        let mut selected = self.selected.clone();
        if let Some(index) = selected.iter().position(|item| item == key) {
            selected.remove(index);
        } else {
            selected.push(key.clone());
        }
        *self = Self::from_ordered_selection(all_keys, selected, Some(key.clone()));
    }
}

impl<K: PartialEq> ImUiMultiSelectState<K> {
    pub fn is_selected(&self, key: &K) -> bool {
        self.selected.iter().any(|item| item == key)
    }
}

fn normalize_selection_order<K: Clone + PartialEq>(all_keys: &[K], selected: Vec<K>) -> Vec<K> {
    let mut ordered = Vec::new();

    for key in all_keys {
        if selected.iter().any(|item| item == key) && !ordered.iter().any(|item| item == key) {
            ordered.push(key.clone());
        }
    }

    for key in selected {
        if !ordered.iter().any(|item| item == &key) {
            ordered.push(key);
        }
    }

    ordered
}
