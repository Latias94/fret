use std::sync::Arc;

use fret_core::{Event, KeyCode, Modifiers, Px, Size};
use fret_runtime::{CommandId, Model};
use fret_ui::{
    Column, EventCx, FixedPanel, Invalidation, LayoutCx, PaintCx, UiHost, UiTree, Widget,
};

use crate::Size as ComponentSize;
use crate::command::{CommandItem, CommandList, visible_item_ids};
use crate::text_field::TextField;

pub struct CommandPaletteHandles {
    pub query: Model<String>,
    pub selection: Model<Option<Arc<str>>>,
}

pub struct CommandPalette {
    items: Model<Vec<CommandItem>>,
    query: Model<String>,
    selection: Model<Option<Arc<str>>>,
    close_command: CommandId,
    last_items_revision: Option<u64>,
    last_query_revision: Option<u64>,
}

impl CommandPalette {
    pub fn new(
        items: Model<Vec<CommandItem>>,
        query: Model<String>,
        selection: Model<Option<Arc<str>>>,
    ) -> Self {
        Self {
            items,
            query,
            selection,
            close_command: CommandId::from("command_palette.close"),
            last_items_revision: None,
            last_query_revision: None,
        }
    }

    pub fn with_close_command(mut self, command: CommandId) -> Self {
        self.close_command = command;
        self
    }

    fn ensure_selection<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) {
        cx.observe_model(self.items, Invalidation::Layout);
        cx.observe_model(self.query, Invalidation::Layout);
        cx.observe_model(self.selection, Invalidation::Paint);

        let items_rev = cx.app.models().revision(self.items);
        let query_rev = cx.app.models().revision(self.query);
        let changed =
            self.last_items_revision != items_rev || self.last_query_revision != query_rev;
        self.last_items_revision = items_rev;
        self.last_query_revision = query_rev;

        if !changed {
            return;
        }

        let items = cx.app.models().get(self.items).cloned().unwrap_or_default();
        let query = cx.app.models().get(self.query).cloned().unwrap_or_default();
        let visible = visible_item_ids(&items, &query);

        let current = cx.app.models().get(self.selection).cloned().unwrap_or(None);
        let next = match current {
            Some(id) if visible.iter().any(|v| v.as_ref() == id.as_ref()) => Some(id),
            _ => visible.first().cloned(),
        };

        let _ = cx.app.models_mut().update(self.selection, |v| *v = next);
    }

    fn move_selection<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>, dir: i32) -> bool {
        let items = cx.app.models().get(self.items).cloned().unwrap_or_default();
        let query = cx.app.models().get(self.query).cloned().unwrap_or_default();
        let visible = visible_item_ids(&items, &query);
        if visible.is_empty() {
            return false;
        }

        let current = cx.app.models().get(self.selection).cloned().unwrap_or(None);
        let idx = current.and_then(|id| {
            visible
                .iter()
                .position(|v| v.as_ref() == id.as_ref())
                .map(|i| i as i32)
        });

        let next_idx = match idx {
            None => {
                if dir >= 0 {
                    0
                } else {
                    (visible.len() as i32).saturating_sub(1)
                }
            }
            Some(i) => (i + dir).clamp(0, (visible.len() as i32).saturating_sub(1)),
        };

        let next = visible
            .get(next_idx as usize)
            .cloned()
            .or_else(|| visible.first().cloned());
        let _ = cx.app.models_mut().update(self.selection, |v| *v = next);
        cx.request_redraw();
        true
    }

    fn activate_selected<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) -> bool {
        let Some(id) = cx.app.models().get(self.selection).cloned().unwrap_or(None) else {
            return false;
        };
        cx.dispatch_command(self.close_command.clone());
        cx.dispatch_command(CommandId::new(id));
        cx.stop_propagation();
        true
    }

    fn is_plain(mods: Modifiers) -> bool {
        !mods.shift && !mods.ctrl && !mods.alt && !mods.alt_gr && !mods.meta
    }
}

impl<H: UiHost> Widget<H> for CommandPalette {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if let Event::KeyDown { key, modifiers, .. } = event
            && cx.focus.is_some()
            && Self::is_plain(*modifiers)
        {
            match key {
                KeyCode::ArrowDown => {
                    if self.move_selection(cx, 1) {
                        cx.stop_propagation();
                        return;
                    }
                }
                KeyCode::ArrowUp => {
                    if self.move_selection(cx, -1) {
                        cx.stop_propagation();
                        return;
                    }
                }
                KeyCode::Enter => {
                    if self.activate_selected(cx) {
                        return;
                    }
                }
                _ => {}
            }
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.ensure_selection(cx);

        if let Some(&child) = cx.children.first() {
            let _ = cx.layout_in(child, cx.bounds);
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        if let Some(&child) = cx.children.first() {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}

/// Builds a command palette subtree under the provided overlay root (`WindowOverlays::command_palette_node()`).
///
/// This helper is intentionally UI-tree oriented: it provides “one-call install” ergonomics for apps.
pub fn install_command_palette<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    overlay_root: fret_core::NodeId,
    items: Model<Vec<CommandItem>>,
    size: ComponentSize,
) -> CommandPaletteHandles {
    let query = app.models_mut().insert(String::new());
    let selection = app.models_mut().insert(None::<Arc<str>>);

    let palette = ui.create_node(CommandPalette::new(items, query, selection));
    ui.add_child(overlay_root, palette);

    let content = ui.create_node(Column::new());
    ui.add_child(palette, content);

    let input = ui.create_node(
        TextField::new(query)
            .with_size(size)
            .with_cancel_command(CommandId::from("command_palette.close")),
    );
    ui.add_child(content, input);

    let list_h = match size {
        ComponentSize::XSmall => Px(240.0),
        ComponentSize::Small => Px(260.0),
        ComponentSize::Medium => Px(300.0),
        ComponentSize::Large => Px(320.0),
    };
    let list_panel = ui.create_node(FixedPanel::new(list_h, fret_core::Color::TRANSPARENT));
    ui.add_child(content, list_panel);

    let list = ui.create_node(
        CommandList::new(items, query)
            .with_size(size)
            .with_selection_model(selection)
            .with_close_command(CommandId::from("command_palette.close"))
            .activate_on_enter(true),
    );
    ui.add_child(list_panel, list);

    CommandPaletteHandles { query, selection }
}
