use std::sync::Arc;

use fret_core::{Event, Size as UiSize};
use fret_runtime::{CommandId, Model};
use fret_ui::{
    Invalidation, LayoutCx, PaintCx, Theme, UiHost, VirtualList, VirtualListDataSource,
    VirtualListRow, VirtualListRowHeight, Widget,
};

use crate::list_style::list_style;
use crate::{Sizable, Size};

#[derive(Debug, Clone)]
pub struct CommandItem {
    pub id: Arc<str>,
    pub label: Arc<str>,
    pub keywords: Vec<Arc<str>>,
    pub detail: Option<Arc<str>>,
    pub shortcut: Option<Arc<str>>,
    pub group: Option<Arc<str>>,
    pub enabled: bool,
}

impl CommandItem {
    pub fn new(id: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            keywords: Vec::new(),
            detail: None,
            shortcut: None,
            group: None,
            enabled: true,
        }
    }

    pub fn keyword(mut self, keyword: impl Into<Arc<str>>) -> Self {
        self.keywords.push(keyword.into());
        self
    }

    pub fn detail(mut self, detail: impl Into<Arc<str>>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<Arc<str>>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn group(mut self, group: impl Into<Arc<str>>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Debug, Clone)]
enum CommandRowKind {
    Header {
        label: Arc<str>,
    },
    Separator,
    Item {
        id: Arc<str>,
        label: Arc<str>,
        detail: Option<Arc<str>>,
        shortcut: Option<Arc<str>>,
        enabled: bool,
    },
}

#[derive(Debug, Clone)]
struct CommandRow {
    key: u64,
    kind: CommandRowKind,
}

#[derive(Debug, Clone, Default)]
struct CommandDataSource {
    rows: Vec<CommandRow>,
}

impl VirtualListDataSource for CommandDataSource {
    type Key = u64;

    fn len(&self) -> usize {
        self.rows.len()
    }

    fn key_at(&self, index: usize) -> Self::Key {
        self.rows.get(index).map(|r| r.key).unwrap_or_default()
    }

    fn row_at(&self, index: usize) -> VirtualListRow<'_> {
        let Some(row) = self.rows.get(index) else {
            return VirtualListRow::new("").not_selectable();
        };

        match &row.kind {
            CommandRowKind::Header { label } => VirtualListRow::new(label.as_ref()).header(),
            CommandRowKind::Separator => VirtualListRow::separator(),
            CommandRowKind::Item {
                label,
                detail,
                shortcut,
                enabled,
                ..
            } => {
                let mut row = VirtualListRow::new(label.as_ref());
                if let Some(detail) = detail.as_ref() {
                    row = row.with_secondary_text(detail.as_ref());
                }
                if let Some(shortcut) = shortcut.as_ref() {
                    row = row.with_trailing_text(shortcut.as_ref());
                }
                if !*enabled {
                    row = row.disabled();
                }
                row
            }
        }
    }
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn stable_key(kind: &str, s: &str) -> u64 {
    let mut buf = Vec::with_capacity(kind.len() + 1 + s.len());
    buf.extend_from_slice(kind.as_bytes());
    buf.push(b'|');
    buf.extend_from_slice(s.as_bytes());
    fnv1a_64(&buf)
}

fn matches_query(item: &CommandItem, q: &str) -> bool {
    if q.is_empty() {
        return true;
    }
    let q = q.trim();
    if q.is_empty() {
        return true;
    }

    let label = item.label.as_ref().to_ascii_lowercase();
    if label.contains(q) {
        return true;
    }
    item.keywords
        .iter()
        .any(|k| k.as_ref().to_ascii_lowercase().contains(q))
}

/// shadcn-inspired command list (search + grouped results) backed by `fret-ui::VirtualList`.
///
/// This widget is intentionally generic: it does not execute app commands. It only exposes the
/// current selection via a model so apps can wire it to their own command registry.
pub struct CommandList {
    items: Model<Vec<CommandItem>>,
    query: Model<String>,
    selection: Option<Model<Option<Arc<str>>>>,
    close_command: Option<CommandId>,
    activate_on_enter: bool,
    size: Size,

    list: VirtualList<CommandDataSource>,
    last_items_revision: Option<u64>,
    last_query_revision: Option<u64>,
    last_selection_revision: Option<u64>,
    last_theme_revision: Option<u64>,
}

impl CommandList {
    pub fn new(items: Model<Vec<CommandItem>>, query: Model<String>) -> Self {
        Self {
            items,
            query,
            selection: None,
            close_command: None,
            activate_on_enter: false,
            size: Size::Medium,
            list: VirtualList::new(CommandDataSource::default()),
            last_items_revision: None,
            last_query_revision: None,
            last_selection_revision: None,
            last_theme_revision: None,
        }
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    pub fn with_selection_model(mut self, selection: Model<Option<Arc<str>>>) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn with_close_command(mut self, command: CommandId) -> Self {
        self.close_command = Some(command);
        self
    }

    pub fn activate_on_enter(mut self, enable: bool) -> Self {
        self.activate_on_enter = enable;
        self
    }

    fn sync_style(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        self.list.set_style(list_style(theme, self.size));
        self.list.set_row_height(VirtualListRowHeight::Measured {
            min: self.size.list_row_h(theme),
        });
    }

    fn rebuild_rows(&mut self, items: Vec<CommandItem>, query: String) {
        let q = query.trim().to_ascii_lowercase();
        let mut filtered: Vec<CommandItem> =
            items.into_iter().filter(|i| matches_query(i, &q)).collect();

        filtered.sort_by(|a, b| {
            let ag = a.group.as_deref().unwrap_or("");
            let bg = b.group.as_deref().unwrap_or("");
            match ag.cmp(bg) {
                std::cmp::Ordering::Equal => a.label.as_ref().cmp(b.label.as_ref()),
                other => other,
            }
        });

        let mut rows: Vec<CommandRow> = Vec::new();
        let mut current_group: Option<Arc<str>> = None;

        for item in filtered {
            let group = item.group.clone().unwrap_or_else(|| Arc::<str>::from(""));
            let group_changed = current_group
                .as_ref()
                .is_none_or(|g| g.as_ref() != group.as_ref());

            if group_changed {
                if !rows.is_empty() {
                    rows.push(CommandRow {
                        key: stable_key("sep", group.as_ref()),
                        kind: CommandRowKind::Separator,
                    });
                }

                if !group.is_empty() {
                    rows.push(CommandRow {
                        key: stable_key("header", group.as_ref()),
                        kind: CommandRowKind::Header {
                            label: group.clone(),
                        },
                    });
                }

                current_group = Some(group);
            }

            rows.push(CommandRow {
                key: stable_key("item", item.id.as_ref()),
                kind: CommandRowKind::Item {
                    id: item.id,
                    label: item.label,
                    detail: item.detail,
                    shortcut: item.shortcut,
                    enabled: item.enabled,
                },
            });
        }

        self.list.set_data(CommandDataSource { rows });
    }

    fn sync_models<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) {
        cx.observe_model(self.items, Invalidation::Layout);
        cx.observe_model(self.query, Invalidation::Layout);

        let items_rev = cx.app.models().revision(self.items);
        let query_rev = cx.app.models().revision(self.query);
        let needs_rebuild =
            items_rev != self.last_items_revision || query_rev != self.last_query_revision;

        if needs_rebuild {
            self.last_items_revision = items_rev;
            self.last_query_revision = query_rev;

            let items = cx.app.models().get(self.items).cloned().unwrap_or_default();
            let query = cx.app.models().get(self.query).cloned().unwrap_or_default();
            self.rebuild_rows(items, query);
        }

        if let Some(selection) = self.selection {
            cx.observe_model(selection, Invalidation::Paint);
            let sel_rev = cx.app.models().revision(selection);
            if sel_rev != self.last_selection_revision || needs_rebuild {
                self.last_selection_revision = sel_rev;
                let desired = cx.app.models().get(selection).cloned().unwrap_or(None);
                let desired_key = desired.as_ref().and_then(|id| {
                    self.list
                        .data()
                        .rows
                        .iter()
                        .find_map(|row| match &row.kind {
                            CommandRowKind::Item { id: row_id, .. }
                                if row_id.as_ref() == id.as_ref() =>
                            {
                                Some(row.key)
                            }
                            _ => None,
                        })
                });
                if self.list.selected_lead_key() != desired_key {
                    self.list.set_selected_key(desired_key);
                }
            }
        }
    }

    fn selected_id(&self) -> Option<Arc<str>> {
        let selected = self.list.selected_lead_key()?;
        let ds = self.list.data();
        let idx = ds.index_of_key(selected)?;
        match ds.rows.get(idx)?.kind.clone() {
            CommandRowKind::Item { id, .. } => Some(id),
            _ => None,
        }
    }
}

impl Sizable for CommandList {
    fn with_size(self, size: Size) -> Self {
        CommandList::with_size(self, size)
    }
}

impl<H: UiHost> Widget<H> for CommandList {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut fret_ui::EventCx<'_, H>, event: &Event) {
        let prev = self.selected_id();
        <VirtualList<CommandDataSource> as Widget<H>>::event(&mut self.list, cx, event);
        let next = self.selected_id();
        if prev != next
            && let Some(selection) = self.selection
        {
            let _ = cx.app.models_mut().update(selection, |v| *v = next.clone());
        }

        if self.activate_on_enter
            && cx.focus == Some(cx.node)
            && let Event::KeyDown { key, modifiers, .. } = event
            && *key == fret_core::KeyCode::Enter
            && !modifiers.shift
            && !modifiers.ctrl
            && !modifiers.alt
            && !modifiers.meta
            && let Some(id) = next
        {
            if let Some(cmd) = self.close_command.clone() {
                cx.dispatch_command(cmd);
            }
            cx.dispatch_command(CommandId::new(id));
            cx.stop_propagation();
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> UiSize {
        self.sync_style(cx.theme());
        self.sync_models(cx);
        <VirtualList<CommandDataSource> as Widget<H>>::layout(&mut self.list, cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        <VirtualList<CommandDataSource> as Widget<H>>::paint(&mut self.list, cx);
    }
}
