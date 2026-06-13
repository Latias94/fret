use std::sync::Arc;

use super::{SelectEntry, SelectItem, SelectLabel, SelectTriggerLabelPolicy};

#[derive(Clone)]
pub(super) enum SelectRow {
    Item(SelectItem),
    Label(SelectLabel),
    Separator,
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

pub(super) fn flatten_entries(into: &mut Vec<SelectRow>, entries: &[SelectEntry]) {
    for entry in entries {
        match entry {
            SelectEntry::Item(item) => into.push(SelectRow::Item(item.clone())),
            SelectEntry::Label(label) => into.push(SelectRow::Label(label.clone())),
            SelectEntry::Group(group) => flatten_entries(into, &group.entries),
            SelectEntry::Separator(_) => into.push(SelectRow::Separator),
        }
    }
}

pub(super) fn flattened_rows(entries: &[SelectEntry]) -> Vec<SelectRow> {
    let mut rows = Vec::new();
    flatten_entries(&mut rows, entries);
    rows
}

pub(super) fn row_item_count(rows: &[SelectRow]) -> usize {
    rows.iter()
        .filter(|row| matches!(row, SelectRow::Item(_)))
        .count()
}

pub(super) fn row_disabled_mask(rows: &[SelectRow], enabled: bool) -> Vec<bool> {
    rows.iter()
        .map(|row| match row {
            SelectRow::Item(item) => item.disabled || !enabled,
            SelectRow::Label(_) | SelectRow::Separator => true,
        })
        .collect()
}

pub(super) fn row_labels(rows: &[SelectRow]) -> Vec<Arc<str>> {
    rows.iter()
        .map(|row| match row {
            SelectRow::Item(item) => item.label.clone(),
            SelectRow::Label(_) | SelectRow::Separator => Arc::from(""),
        })
        .collect()
}

pub(super) fn row_values(rows: &[SelectRow]) -> Vec<Option<Arc<str>>> {
    rows.iter()
        .map(|row| match row {
            SelectRow::Item(item) => Some(item.value.clone()),
            SelectRow::Label(_) | SelectRow::Separator => None,
        })
        .collect()
}

pub(super) fn selected_row_index(rows: &[SelectRow], selected: &str) -> Option<usize> {
    rows.iter().position(|row| match row {
        SelectRow::Item(item) => item.value.as_ref() == selected,
        SelectRow::Label(_) | SelectRow::Separator => false,
    })
}

pub(super) fn contains_item_value(rows: &[SelectRow], value: &str) -> bool {
    rows.iter().any(|row| match row {
        SelectRow::Item(item) => item.value.as_ref() == value,
        SelectRow::Label(_) | SelectRow::Separator => false,
    })
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
