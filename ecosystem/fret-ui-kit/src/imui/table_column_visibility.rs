//! Runtime table-column visibility helpers for IMUI table authoring.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};
use serde::{Deserialize, Serialize};

use super::label_identity::parse_label_identity;
use super::{
    MenuItemOptions, PopupMenuOptions, ResponseExt, TableColumn, TableResponse,
    UiWriterImUiFacadeExt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableColumnVisibilityOverride {
    id: Arc<str>,
    visible: bool,
}

/// Model state for runtime table-column visibility.
///
/// This intentionally stays policy-only: it maps stable column ids to visible flags and then
/// produces a new `TableColumn` list. Persistence, freeze panes, and durable column storage are
/// separate table policies.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImUiTableColumnVisibilityState {
    overrides: Vec<TableColumnVisibilityOverride>,
}

/// Persistence-friendly snapshot of runtime table-column visibility overrides.
///
/// This is only a data shape. Callers own where and when it is stored, and the IMUI helper keeps
/// using caller-owned `ImUiTableColumnVisibilityState` models at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TableColumnVisibilitySnapshot {
    #[serde(default)]
    pub columns: Vec<TableColumnVisibilityEntry>,
}

/// One stable column visibility override inside [`TableColumnVisibilitySnapshot`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableColumnVisibilityEntry {
    #[serde(rename = "id")]
    pub column_id: String,
    #[serde(rename = "visible")]
    pub is_visible: bool,
}

/// Options for composing a group of table-column visibility menu items.
#[derive(Debug, Clone, Default)]
pub struct TableColumnVisibilityMenuOptions {
    /// Base options cloned into every generated checkbox menu item.
    pub item_options: MenuItemOptions,
    /// Optional test-id prefix. When set, item test ids are `{prefix}{stable_column_id_slug}`.
    pub test_id_prefix: Option<Arc<str>>,
}

/// Options for wiring a table header context menu to table-column visibility items.
#[derive(Debug, Clone)]
pub struct TableColumnVisibilityHeaderContextMenuOptions {
    pub popup: PopupMenuOptions,
    pub menu: TableColumnVisibilityMenuOptions,
}

/// Aggregated response for a helper-composed table-column visibility menu section.
#[derive(Debug, Clone, Default)]
pub struct TableColumnVisibilityMenuResponse {
    items: Vec<TableColumnVisibilityMenuItemResponse>,
}

/// Response for helper-composed table header context-menu visibility policy.
#[derive(Debug, Clone, Default)]
pub struct TableColumnVisibilityHeaderContextMenuResponse {
    open: bool,
    items: TableColumnVisibilityMenuResponse,
}

/// Response for one generated table-column visibility menu item.
#[derive(Debug, Clone)]
pub struct TableColumnVisibilityMenuItemResponse {
    column_id: Arc<str>,
    visible: bool,
    response: ResponseExt,
}

impl ImUiTableColumnVisibilityState {
    pub fn new<I, S>(overrides: I) -> Self
    where
        I: IntoIterator<Item = (S, bool)>,
        S: Into<Arc<str>>,
    {
        let mut state = Self::default();
        for (id, visible) in overrides {
            state.set_visible(id, visible);
        }
        state
    }

    pub fn is_empty(&self) -> bool {
        self.overrides.is_empty()
    }

    pub fn len(&self) -> usize {
        self.overrides.len()
    }

    pub fn visibility_for(&self, id: &str) -> Option<bool> {
        self.overrides
            .iter()
            .find(|entry| entry.id.as_ref() == id)
            .map(|entry| entry.visible)
    }

    pub fn is_visible(&self, id: &str, default_visible: bool) -> bool {
        self.visibility_for(id).unwrap_or(default_visible)
    }

    pub fn set_visible(&mut self, id: impl Into<Arc<str>>, visible: bool) {
        let id = id.into();
        if id.is_empty() {
            return;
        }

        if let Some(entry) = self
            .overrides
            .iter_mut()
            .find(|entry| entry.id.as_ref() == id.as_ref())
        {
            entry.visible = visible;
            return;
        }

        self.overrides
            .push(TableColumnVisibilityOverride { id, visible });
    }

    pub fn show(&mut self, id: impl Into<Arc<str>>) {
        self.set_visible(id, true);
    }

    pub fn hide(&mut self, id: impl Into<Arc<str>>) {
        self.set_visible(id, false);
    }

    pub fn toggle(&mut self, id: impl Into<Arc<str>>, default_visible: bool) -> bool {
        let id = id.into();
        if id.is_empty() {
            return default_visible;
        }

        let visible = !self.is_visible(id.as_ref(), default_visible);
        self.set_visible(id, visible);
        visible
    }

    pub fn remove(&mut self, id: &str) -> Option<bool> {
        let index = self
            .overrides
            .iter()
            .position(|entry| entry.id.as_ref() == id)?;
        Some(self.overrides.remove(index).visible)
    }

    pub fn clear(&mut self) {
        self.overrides.clear();
    }

    pub fn snapshot(&self) -> TableColumnVisibilitySnapshot {
        TableColumnVisibilitySnapshot {
            columns: self
                .overrides
                .iter()
                .filter(|entry| !entry.id.is_empty())
                .map(|entry| TableColumnVisibilityEntry {
                    column_id: entry.id.to_string(),
                    is_visible: entry.visible,
                })
                .collect(),
        }
    }

    pub fn from_snapshot(snapshot: TableColumnVisibilitySnapshot) -> Self {
        let mut state = Self::default();
        for entry in snapshot.columns {
            state.set_visible(entry.column_id, entry.is_visible);
        }
        state
    }

    pub fn replace_from_snapshot(&mut self, snapshot: TableColumnVisibilitySnapshot) {
        self.clear();
        for entry in snapshot.columns {
            self.set_visible(entry.column_id, entry.is_visible);
        }
    }

    pub fn apply_to_columns(&self, columns: &[TableColumn]) -> Vec<TableColumn> {
        columns
            .iter()
            .cloned()
            .map(|mut column| {
                if let Some(id) = column.id.as_deref() {
                    if let Some(visible) = self.visibility_for(id) {
                        column.visible = visible;
                    }
                }
                column
            })
            .collect()
    }
}

impl TableColumnVisibilitySnapshot {
    pub fn new<I, S>(columns: I) -> Self
    where
        I: IntoIterator<Item = (S, bool)>,
        S: Into<String>,
    {
        let mut snapshot = Self::default();
        for (id, visible) in columns {
            let id = id.into();
            if id.is_empty() {
                continue;
            }
            snapshot.columns.push(TableColumnVisibilityEntry {
                column_id: id,
                is_visible: visible,
            });
        }
        snapshot
    }

    pub fn columns(&self) -> &[TableColumnVisibilityEntry] {
        &self.columns
    }

    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

impl TableColumnVisibilityEntry {
    pub fn new(id: impl Into<String>, visible: bool) -> Self {
        Self {
            column_id: id.into(),
            is_visible: visible,
        }
    }

    pub fn id(&self) -> &str {
        self.column_id.as_str()
    }

    pub fn visible(&self) -> bool {
        self.is_visible
    }
}

impl TableColumnVisibilityMenuResponse {
    pub fn items(&self) -> &[TableColumnVisibilityMenuItemResponse] {
        &self.items
    }

    pub fn item(&self, column_id: &str) -> Option<&TableColumnVisibilityMenuItemResponse> {
        self.items
            .iter()
            .find(|item| item.column_id.as_ref() == column_id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn changed(&self) -> bool {
        self.items.iter().any(|item| item.changed())
    }
}

impl TableColumnVisibilityHeaderContextMenuResponse {
    pub fn open(&self) -> bool {
        self.open
    }

    pub fn items(&self) -> &TableColumnVisibilityMenuResponse {
        &self.items
    }

    pub fn changed(&self) -> bool {
        self.items.changed()
    }
}

impl Default for TableColumnVisibilityHeaderContextMenuOptions {
    fn default() -> Self {
        Self {
            popup: PopupMenuOptions {
                estimated_size: fret_core::Size::new(fret_core::Px(180.0), fret_core::Px(160.0)),
                ..Default::default()
            },
            menu: TableColumnVisibilityMenuOptions::default(),
        }
    }
}

impl TableColumnVisibilityMenuItemResponse {
    pub fn column_id(&self) -> &str {
        self.column_id.as_ref()
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn response(&self) -> ResponseExt {
        self.response
    }

    pub fn clicked(&self) -> bool {
        self.response.clicked()
    }

    pub fn changed(&self) -> bool {
        self.response.changed()
    }
}

/// Opens and renders a table-column visibility context menu from table header context requests.
///
/// This is a kit-layer composition policy: callers still own the visibility model and decide when
/// to apply it to their column list, while the helper wires table header context-menu requests to
/// the existing popup/menu-item policy.
pub fn table_column_visibility_header_context_menu<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    table: &TableResponse,
    columns: &[TableColumn],
    model: &Model<ImUiTableColumnVisibilityState>,
    options: TableColumnVisibilityHeaderContextMenuOptions,
) -> TableColumnVisibilityHeaderContextMenuResponse {
    let mut trigger = None;
    let mut fallback_trigger = None;
    for header in table.headers() {
        let response = header.response();
        if fallback_trigger.is_none() && response.id().is_some() {
            fallback_trigger = Some(response);
        }
        if response.context_menu_requested() {
            trigger = Some(response);
            break;
        }
    }
    let trigger = trigger.or(fallback_trigger).unwrap_or_default();

    let mut items = TableColumnVisibilityMenuResponse::default();
    let open = ui.begin_popup_context_menu_with_options(id, trigger, options.popup, |ui| {
        items = table_column_visibility_menu_items(ui, columns, model, options.menu);
    });

    TableColumnVisibilityHeaderContextMenuResponse { open, items }
}

/// Returns a controllable visibility model for an immediate table column set.
pub fn table_column_visibility_use_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<ImUiTableColumnVisibilityState>>,
    default_value: impl FnOnce() -> ImUiTableColumnVisibilityState,
) -> crate::primitives::controllable_state::ControllableModel<ImUiTableColumnVisibilityState> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

pub fn table_column_visibility_menu_items<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    columns: &[TableColumn],
    model: &Model<ImUiTableColumnVisibilityState>,
    options: TableColumnVisibilityMenuOptions,
) -> TableColumnVisibilityMenuResponse {
    let mut items = Vec::new();

    for (index, column) in columns.iter().enumerate() {
        let Some(column_id) = menu_column_id(column) else {
            continue;
        };
        if visible_menu_label(column).is_none() {
            continue;
        }

        let mut item_options = options.item_options.clone();
        if let Some(prefix) = options.test_id_prefix.as_ref() {
            item_options.test_id = Some(Arc::from(format!(
                "{}{}",
                prefix,
                menu_test_id_suffix(column_id.as_ref(), index)
            )));
        }

        let Some(response) = table_column_visibility_menu_item(ui, column, model, item_options)
        else {
            continue;
        };
        let visible = ui.with_cx_mut(|cx| {
            cx.read_model(model, fret_ui::Invalidation::Paint, |_app, state| {
                state.is_visible(column_id.as_ref(), column.visible)
            })
            .unwrap_or(column.visible)
        });
        items.push(TableColumnVisibilityMenuItemResponse {
            column_id,
            visible,
            response,
        });
    }

    TableColumnVisibilityMenuResponse { items }
}

pub fn table_column_visibility_menu_item<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    column: &TableColumn,
    model: &Model<ImUiTableColumnVisibilityState>,
    options: MenuItemOptions,
) -> Option<ResponseExt> {
    let id = column.id.clone()?;
    if id.is_empty() {
        return None;
    }

    let label = column
        .header
        .clone()
        .unwrap_or_else(|| Arc::from(id.as_ref()));
    let visible = ui.with_cx_mut(|cx| {
        cx.read_model(model, fret_ui::Invalidation::Paint, |_app, state| {
            state.is_visible(id.as_ref(), column.visible)
        })
        .unwrap_or(column.visible)
    });

    let mut response = ui.menu_item_checkbox_with_options(label, visible, options);
    if response.clicked() {
        let changed_to = !visible;
        let mut changed = false;
        let _ = ui.with_cx_mut(|cx| {
            cx.app.models_mut().update(model, |state| {
                if state.is_visible(id.as_ref(), column.visible) != changed_to {
                    state.set_visible(id.clone(), changed_to);
                    changed = true;
                }
            })
        });
        response.set_core_changed(changed);
        response.merge_edited(changed);
    }

    Some(response)
}

fn menu_column_id(column: &TableColumn) -> Option<Arc<str>> {
    let id = column.id.clone()?;
    (!id.is_empty()).then_some(id)
}

fn visible_menu_label(column: &TableColumn) -> Option<&str> {
    let header = column.header.as_deref()?;
    let parts = parse_label_identity(header);
    (!parts.visible.is_empty()).then_some(parts.visible)
}

fn menu_test_id_suffix(id: &str, index: usize) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_core::Px;

    #[test]
    fn visibility_state_applies_runtime_overrides_by_stable_column_id() {
        let columns = vec![
            TableColumn::fill("Name###name"),
            TableColumn::px("Status###status", Px(96.0)),
            TableColumn::px("Owner###owner", Px(88.0)),
        ];
        let state = ImUiTableColumnVisibilityState::new([
            (Arc::from("status"), false),
            (Arc::from("owner"), true),
        ]);

        let applied = state.apply_to_columns(&columns);

        assert!(applied[0].visible);
        assert!(!applied[1].visible);
        assert!(applied[2].visible);
        assert_eq!(applied[1].id.as_deref(), Some("status"));
        assert_eq!(state.visibility_for("status"), Some(false));
    }

    #[test]
    fn visibility_state_leaves_unlisted_and_unidentified_columns_at_declared_visibility() {
        let columns = vec![
            TableColumn::fill("Name###name"),
            TableColumn::px("Static Hidden###hidden", Px(96.0)).hidden(),
            TableColumn::unlabeled(super::super::TableColumnWidth::px(Px(64.0))),
        ];
        let state = ImUiTableColumnVisibilityState::new([(Arc::from("name"), false)]);

        let applied = state.apply_to_columns(&columns);

        assert!(!applied[0].visible);
        assert!(!applied[1].visible);
        assert!(applied[2].visible);
    }

    #[test]
    fn visibility_state_toggle_uses_current_override_or_default_visibility() {
        let mut state = ImUiTableColumnVisibilityState::default();

        assert!(!state.toggle("status", true));
        assert_eq!(state.visibility_for("status"), Some(false));
        assert!(state.toggle("status", true));
        assert_eq!(state.visibility_for("status"), Some(true));
        assert_eq!(state.remove("status"), Some(true));
        assert!(state.visibility_for("status").is_none());
    }

    #[test]
    fn visibility_state_snapshot_roundtrips_stable_column_ids() {
        let state = ImUiTableColumnVisibilityState::new([
            (Arc::from("status"), false),
            (Arc::from("owner"), true),
        ]);

        let snapshot = state.snapshot();
        let encoded = serde_json::to_string(&snapshot).expect("snapshot should serialize");
        let decoded: TableColumnVisibilitySnapshot =
            serde_json::from_str(&encoded).expect("snapshot should deserialize");
        let restored = ImUiTableColumnVisibilityState::from_snapshot(decoded);

        assert_eq!(snapshot.columns().len(), 2);
        assert_eq!(snapshot.columns()[0].id(), "status");
        assert!(!snapshot.columns()[0].visible());
        assert_eq!(restored.visibility_for("status"), Some(false));
        assert_eq!(restored.visibility_for("owner"), Some(true));
    }

    #[test]
    fn visibility_state_snapshot_restore_ignores_empty_ids_and_last_entry_wins() {
        let snapshot = TableColumnVisibilitySnapshot {
            columns: vec![
                TableColumnVisibilityEntry::new("", false),
                TableColumnVisibilityEntry::new("status", false),
                TableColumnVisibilityEntry::new("status", true),
                TableColumnVisibilityEntry::new("owner", false),
            ],
        };

        let mut state = ImUiTableColumnVisibilityState::new([("stale", false)]);
        state.replace_from_snapshot(snapshot);

        assert_eq!(state.len(), 2);
        assert!(state.visibility_for("").is_none());
        assert!(state.visibility_for("stale").is_none());
        assert_eq!(state.visibility_for("status"), Some(true));
        assert_eq!(state.visibility_for("owner"), Some(false));
    }

    #[test]
    fn menu_group_filters_to_stable_human_labeled_columns() {
        let columns = [
            TableColumn::fill("Name###name"),
            TableColumn::unlabeled(super::super::TableColumnWidth::px(Px(64.0))).with_id("actions"),
            TableColumn::px("###internal", Px(48.0)),
            TableColumn::px("State###state", Px(80.0)),
        ];

        assert_eq!(menu_column_id(&columns[0]).as_deref(), Some("name"));
        assert_eq!(visible_menu_label(&columns[0]), Some("Name"));
        assert_eq!(menu_column_id(&columns[1]).as_deref(), Some("actions"));
        assert!(visible_menu_label(&columns[1]).is_none());
        assert_eq!(menu_column_id(&columns[2]).as_deref(), Some("internal"));
        assert!(visible_menu_label(&columns[2]).is_none());
        assert_eq!(visible_menu_label(&columns[3]), Some("State"));
    }

    #[test]
    fn menu_group_test_id_suffix_uses_stable_column_id_slug() {
        assert_eq!(menu_test_id_suffix("asset-status", 7), "asset-status");
        assert_eq!(menu_test_id_suffix("Asset Status!", 7), "asset-status");
        assert_eq!(menu_test_id_suffix("###", 7), "7");
    }
}
