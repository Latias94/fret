use std::sync::Arc;

use super::{SelectEntry, SelectTriggerLabelPolicy};

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
