use std::sync::Arc;

use super::{SelectEntry, SelectItem, SelectLabel, SelectTriggerLabelPolicy};

#[derive(Clone)]
pub(super) enum SelectRow {
    Item(SelectItem),
    Label(SelectLabel),
    Separator,
}

pub(super) struct SelectRows {
    rows: Vec<SelectRow>,
    disabled: Arc<[bool]>,
    labels: Arc<[Arc<str>]>,
    values_by_row: Arc<[Option<Arc<str>>]>,
    item_count: usize,
}

impl SelectRows {
    pub(super) fn from_entries(entries: &[SelectEntry], enabled: bool) -> Self {
        let mut builder = SelectRowsBuilder::default();
        builder.extend(entries, enabled);
        builder.finish()
    }

    pub(super) fn len(&self) -> usize {
        self.rows.len()
    }

    pub(super) fn rows(&self) -> &[SelectRow] {
        &self.rows
    }

    pub(super) fn disabled(&self) -> &[bool] {
        self.disabled.as_ref()
    }

    pub(super) fn disabled_arc(&self) -> Arc<[bool]> {
        self.disabled.clone()
    }

    pub(super) fn disabled_at(&self, row_idx: usize) -> bool {
        self.disabled.get(row_idx).copied().unwrap_or(true)
    }

    pub(super) fn labels_arc(&self) -> Arc<[Arc<str>]> {
        self.labels.clone()
    }

    pub(super) fn values_by_row_arc(&self) -> Arc<[Option<Arc<str>>]> {
        self.values_by_row.clone()
    }

    pub(super) fn item_count(&self) -> usize {
        self.item_count
    }

    pub(super) fn selected_row_index(&self, selected: &str) -> Option<usize> {
        self.rows.iter().position(|row| match row {
            SelectRow::Item(item) => item.value.as_ref() == selected,
            SelectRow::Label(_) | SelectRow::Separator => false,
        })
    }

    pub(super) fn contains_item_value(&self, value: &str) -> bool {
        self.rows.iter().any(|row| match row {
            SelectRow::Item(item) => item.value.as_ref() == value,
            SelectRow::Label(_) | SelectRow::Separator => false,
        })
    }
}

#[derive(Default)]
struct SelectRowsBuilder {
    rows: Vec<SelectRow>,
    disabled: Vec<bool>,
    labels: Vec<Arc<str>>,
    values_by_row: Vec<Option<Arc<str>>>,
    item_count: usize,
}

impl SelectRowsBuilder {
    fn extend(&mut self, entries: &[SelectEntry], enabled: bool) {
        for entry in entries {
            match entry {
                SelectEntry::Item(item) => {
                    self.rows.push(SelectRow::Item(item.clone()));
                    self.disabled.push(item.disabled || !enabled);
                    self.labels.push(item.label.clone());
                    self.values_by_row.push(Some(item.value.clone()));
                    self.item_count = self.item_count.saturating_add(1);
                }
                SelectEntry::Label(label) => {
                    self.rows.push(SelectRow::Label(label.clone()));
                    self.disabled.push(true);
                    self.labels.push(Arc::from(""));
                    self.values_by_row.push(None);
                }
                SelectEntry::Group(group) => {
                    self.extend(&group.entries, enabled);
                }
                SelectEntry::Separator(_) => {
                    self.rows.push(SelectRow::Separator);
                    self.disabled.push(true);
                    self.labels.push(Arc::from(""));
                    self.values_by_row.push(None);
                }
            }
        }
    }

    fn finish(self) -> SelectRows {
        SelectRows {
            rows: self.rows,
            disabled: Arc::from(self.disabled.into_boxed_slice()),
            labels: Arc::from(self.labels.into_boxed_slice()),
            values_by_row: Arc::from(self.values_by_row.into_boxed_slice()),
            item_count: self.item_count,
        }
    }
}

pub(super) fn find_item_label_overrides(
    entries: &[SelectEntry],
    value: &str,
) -> Option<(
    Arc<str>,
    Vec<fret_core::TextFontFeatureSetting>,
    Vec<fret_core::TextFontAxisSetting>,
)> {
    for entry in entries {
        match entry {
            SelectEntry::Item(it) => {
                if it.value.as_ref() == value {
                    return Some((
                        it.label.clone(),
                        it.label_features_override.clone(),
                        it.label_axes_override.clone(),
                    ));
                }
            }
            SelectEntry::Group(group) => {
                if let Some(out) = find_item_label_overrides(&group.entries, value) {
                    return Some(out);
                }
            }
            SelectEntry::Label(_) | SelectEntry::Separator(_) => {}
        }
    }
    None
}

pub(super) fn trigger_value_text(
    selected: Option<Arc<str>>,
    entries: &[SelectEntry],
    placeholder: &Arc<str>,
    trigger_label_policy: SelectTriggerLabelPolicy,
) -> Arc<str> {
    if trigger_label_policy == SelectTriggerLabelPolicy::Value
        && let Some(value) = selected.as_ref()
    {
        return value.clone();
    }

    selected
        .as_ref()
        .and_then(|v| {
            find_item_label_overrides(entries, v.as_ref())
                .map(|(label, _label_features_override, _label_axes_override)| label)
        })
        .unwrap_or_else(|| placeholder.clone())
}

pub(super) fn count_items(entries: &[SelectEntry]) -> usize {
    let mut count: usize = 0;
    for entry in entries {
        match entry {
            SelectEntry::Item(_) => count = count.saturating_add(1),
            SelectEntry::Group(group) => count = count.saturating_add(count_items(&group.entries)),
            SelectEntry::Label(_) | SelectEntry::Separator(_) => {}
        }
    }
    count
}

pub(super) fn select_group_label(entries: &[SelectEntry]) -> Option<Arc<str>> {
    entries.iter().find_map(|entry| match entry {
        SelectEntry::Label(label) => Some(label.text.clone()),
        _ => None,
    })
}

pub(super) fn flatten_items_for_typeahead(
    entries: &[SelectEntry],
    enabled: bool,
    values: &mut Vec<Arc<str>>,
    labels: &mut Vec<Arc<str>>,
    disabled: &mut Vec<bool>,
) {
    for entry in entries {
        match entry {
            SelectEntry::Item(item) => {
                values.push(item.value.clone());
                labels.push(item.label.clone());
                disabled.push(item.disabled || !enabled);
            }
            SelectEntry::Group(group) => {
                flatten_items_for_typeahead(&group.entries, enabled, values, labels, disabled);
            }
            SelectEntry::Label(_) | SelectEntry::Separator(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::select::{SelectGroup, SelectSeparator};

    #[test]
    fn select_rows_flattens_groups_and_caches_row_metadata() {
        let entries = vec![
            SelectLabel::new("Fruits").into(),
            SelectItem::new("apple", "Apple").into(),
            SelectGroup::new([
                SelectLabel::new("Citrus").into(),
                SelectItem::new("orange", "Orange").disabled(true).into(),
                SelectSeparator.into(),
                SelectItem::new("lemon", "Lemon").into(),
            ])
            .into(),
        ];

        let rows = SelectRows::from_entries(&entries, true);

        assert_eq!(rows.len(), 6);
        assert_eq!(rows.item_count(), 3);
        assert_eq!(rows.disabled(), &[true, false, true, true, false, false]);
        assert_eq!(rows.disabled_at(99), true);
        assert_eq!(rows.selected_row_index("apple"), Some(1));
        assert_eq!(rows.selected_row_index("lemon"), Some(5));
        assert_eq!(rows.selected_row_index("missing"), None);
        assert!(rows.contains_item_value("orange"));
        assert!(!rows.contains_item_value("missing"));

        let labels_arc = rows.labels_arc();
        let labels: Vec<&str> = labels_arc.iter().map(|label| label.as_ref()).collect();
        assert_eq!(labels, ["", "Apple", "", "Orange", "", "Lemon"]);

        let values_arc = rows.values_by_row_arc();
        let values: Vec<Option<&str>> = values_arc.iter().map(|value| value.as_deref()).collect();
        assert_eq!(
            values,
            [
                None,
                Some("apple"),
                None,
                Some("orange"),
                None,
                Some("lemon")
            ]
        );
    }

    #[test]
    fn select_rows_marks_all_items_disabled_when_root_is_disabled() {
        let entries = vec![
            SelectItem::new("small", "Small").into(),
            SelectItem::new("large", "Large").into(),
        ];

        let rows = SelectRows::from_entries(&entries, false);

        assert_eq!(rows.disabled(), &[true, true]);
        assert_eq!(rows.item_count(), 2);
    }
}
