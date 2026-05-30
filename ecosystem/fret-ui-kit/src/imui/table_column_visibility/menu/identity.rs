use std::sync::Arc;

use crate::imui::TableColumn;
use crate::imui::label_identity::parse_label_identity;

pub(in crate::imui::table_column_visibility) fn menu_column_id(
    column: &TableColumn,
) -> Option<Arc<str>> {
    let id = column.id_arc()?;
    (!id.is_empty()).then_some(id)
}

pub(in crate::imui::table_column_visibility) fn visible_menu_label(
    column: &TableColumn,
) -> Option<&str> {
    let header = column.header()?;
    let parts = parse_label_identity(header);
    (!parts.visible.is_empty()).then_some(parts.visible)
}

pub(in crate::imui::table_column_visibility) fn menu_test_id_suffix(
    id: &str,
    index: usize,
) -> String {
    let mut out = String::new();
    let mut last_was_separator = false;

    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !out.is_empty() && !last_was_separator {
            out.push('-');
            last_was_separator = true;
        }
    }

    if out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        index.to_string()
    } else {
        out
    }
}
