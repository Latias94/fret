use fret_core::{AppWindowId, Point, Rect};
use fret_runtime::{CommandId, InputContext, Menu};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MenuBarContextMenuEntry {
    pub index: usize,
    pub bounds: Rect,
    pub menu: Menu,
}

#[derive(Debug, Clone)]
pub struct MenuBarContextMenu {
    pub open_index: usize,
    pub entries: Vec<MenuBarContextMenuEntry>,
}

#[derive(Debug, Clone)]
pub struct ContextMenuRequest {
    pub position: Point,
    pub menu: Menu,
    pub input_ctx: InputContext,
    pub menu_bar: Option<MenuBarContextMenu>,
}

#[derive(Debug, Default)]
pub struct ContextMenuService {
    next_serial: u64,
    by_window: HashMap<AppWindowId, ContextMenuEntry>,
}

#[derive(Debug)]
struct ContextMenuEntry {
    serial: u64,
    request: ContextMenuRequest,
    pending_action: Option<CommandId>,
}

impl ContextMenuService {
    pub fn set_request(&mut self, window: AppWindowId, request: ContextMenuRequest) {
        self.next_serial = self.next_serial.saturating_add(1);
        let serial = self.next_serial;
        self.by_window.insert(
            window,
            ContextMenuEntry {
                serial,
                request,
                pending_action: None,
            },
        );
    }

    pub fn request(&self, window: AppWindowId) -> Option<(u64, &ContextMenuRequest)> {
        let entry = self.by_window.get(&window)?;
        Some((entry.serial, &entry.request))
    }

    pub fn set_pending_action(&mut self, window: AppWindowId, action: Option<CommandId>) {
        let Some(entry) = self.by_window.get_mut(&window) else {
            return;
        };
        entry.pending_action = action;
    }

    pub fn take_pending_action(&mut self, window: AppWindowId) -> Option<CommandId> {
        self.by_window
            .get_mut(&window)
            .and_then(|e| e.pending_action.take())
    }

    pub fn clear(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }
}
