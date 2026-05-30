//! Immediate multi-select state storage and normalization.

mod selection;

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
