use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_core::AppWindowId;

use super::super::tab_overflow::TabOverflowMenuState;
use super::super::viewport::ViewportCaptureState;

mod arbitration;
mod drag_sessions;
mod types;

pub(super) use arbitration::{
    DeclarativePointerCancelOwner, DeclarativePointerMoveOwner, DeclarativePointerUpOwner,
};
pub(super) use types::{
    DeclarativeDividerDrag, DeclarativeFloatingDrag, DeclarativeFloatingHover,
    DeclarativePendingDockDrag, DeclarativePendingDockTabsDrag, DeclarativePressedFloatingClose,
    DeclarativePressedTabClose, DeclarativeTabHover,
};

#[derive(Default)]
pub(super) struct DeclarativeDockInteractionService {
    pressed_tab_close: HashMap<AppWindowId, DeclarativePressedTabClose>,
    pressed_floating_close: HashMap<AppWindowId, DeclarativePressedFloatingClose>,
    floating_drag: HashMap<AppWindowId, DeclarativeFloatingDrag>,
    divider_drag: HashMap<AppWindowId, HashMap<fret_core::PointerId, DeclarativeDividerDrag>>,
    pending_dock_drags:
        HashMap<AppWindowId, HashMap<fret_core::PointerId, DeclarativePendingDockDrag>>,
    pending_dock_tabs_drags:
        HashMap<AppWindowId, HashMap<fret_core::PointerId, DeclarativePendingDockTabsDrag>>,
    viewport_capture: HashMap<AppWindowId, HashMap<fret_core::PointerId, ViewportCaptureState>>,
    tab_overflow_menu: HashMap<AppWindowId, TabOverflowMenuState>,
    tab_scroll: HashMap<AppWindowId, HashMap<fret_core::DockNodeId, fret_core::Px>>,
    tab_widths: HashMap<AppWindowId, HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>>>,
    tab_drag_auto_scroll_last_frame:
        HashMap<AppWindowId, HashMap<fret_core::DockNodeId, fret_runtime::FrameId>>,
    tab_hover: HashMap<AppWindowId, DeclarativeTabHover>,
    floating_hover: HashMap<AppWindowId, DeclarativeFloatingHover>,
}

impl DeclarativeDockInteractionService {
    pub(in crate::dock::declarative) fn clear_window(&mut self, window: AppWindowId) -> bool {
        let mut changed = false;
        changed |= self.pressed_tab_close.remove(&window).is_some();
        changed |= self.pressed_floating_close.remove(&window).is_some();
        changed |= self.floating_drag.remove(&window).is_some();
        changed |= self.divider_drag.remove(&window).is_some();
        changed |= self.pending_dock_drags.remove(&window).is_some();
        changed |= self.pending_dock_tabs_drags.remove(&window).is_some();
        changed |= self.viewport_capture.remove(&window).is_some();
        changed |= self.tab_overflow_menu.remove(&window).is_some();
        changed |= self.tab_scroll.remove(&window).is_some();
        changed |= self.tab_widths.remove(&window).is_some();
        changed |= self
            .tab_drag_auto_scroll_last_frame
            .remove(&window)
            .is_some();
        changed |= self.tab_hover.remove(&window).is_some();
        changed |= self.floating_hover.remove(&window).is_some();
        changed
    }

    #[cfg(test)]
    pub(in crate::dock::declarative) fn has_window_state(&self, window: AppWindowId) -> bool {
        self.pressed_tab_close.contains_key(&window)
            || self.pressed_floating_close.contains_key(&window)
            || self.floating_drag.contains_key(&window)
            || self.divider_drag.contains_key(&window)
            || self.pending_dock_drags.contains_key(&window)
            || self.pending_dock_tabs_drags.contains_key(&window)
            || self.viewport_capture.contains_key(&window)
            || self.tab_overflow_menu.contains_key(&window)
            || self.tab_scroll.contains_key(&window)
            || self.tab_widths.contains_key(&window)
            || self.tab_drag_auto_scroll_last_frame.contains_key(&window)
            || self.tab_hover.contains_key(&window)
            || self.floating_hover.contains_key(&window)
    }

    pub(in crate::dock::declarative) fn pointer_move_owner(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> DeclarativePointerMoveOwner {
        arbitration::arbitrate_pointer_move(self.interaction_snapshot(window, pointer_id))
    }

    pub(in crate::dock::declarative) fn pointer_up_owner(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> DeclarativePointerUpOwner {
        arbitration::arbitrate_pointer_up(self.interaction_snapshot(window, pointer_id))
    }

    pub(in crate::dock::declarative) fn pointer_cancel_owner(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> DeclarativePointerCancelOwner {
        arbitration::arbitrate_pointer_cancel(self.interaction_snapshot(window, pointer_id))
    }

    fn interaction_snapshot(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> arbitration::DeclarativeInteractionSnapshot {
        arbitration::DeclarativeInteractionSnapshot {
            viewport_capture_for_pointer: self
                .viewport_capture
                .get(&window)
                .is_some_and(|captures| captures.contains_key(&pointer_id)),
            viewport_capture_in_window: self
                .viewport_capture
                .get(&window)
                .is_some_and(|captures| !captures.is_empty()),
            divider_drag_for_pointer: self
                .divider_drag
                .get(&window)
                .is_some_and(|drags| drags.contains_key(&pointer_id)),
            floating_close_for_pointer: self
                .pressed_floating_close
                .get(&window)
                .is_some_and(|pressed| pressed.pointer_id == pointer_id),
            floating_drag_for_pointer: self
                .floating_drag
                .get(&window)
                .is_some_and(|drag| drag.pointer_id == pointer_id),
            pending_panel_drag_for_pointer: self
                .pending_dock_drags
                .get(&window)
                .is_some_and(|pending| pending.contains_key(&pointer_id)),
            pending_tabs_drag_for_pointer: self
                .pending_dock_tabs_drags
                .get(&window)
                .is_some_and(|pending| pending.contains_key(&pointer_id)),
            tab_close_for_pointer: self
                .pressed_tab_close
                .get(&window)
                .is_some_and(|pressed| pressed.pointer_id == pointer_id),
        }
    }

    pub(super) fn begin_tab_close(
        &mut self,
        window: AppWindowId,
        pressed: DeclarativePressedTabClose,
    ) {
        self.pressed_tab_close.insert(window, pressed);
    }

    pub(super) fn take_tab_close(
        &mut self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativePressedTabClose> {
        self.pressed_tab_close
            .get(&window)
            .is_some_and(|pressed| pressed.pointer_id == pointer_id)
            .then(|| self.pressed_tab_close.remove(&window))
            .flatten()
    }

    pub(super) fn begin_floating_close(
        &mut self,
        window: AppWindowId,
        pressed: DeclarativePressedFloatingClose,
    ) {
        self.pressed_floating_close.insert(window, pressed);
    }

    pub(super) fn take_floating_close(
        &mut self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativePressedFloatingClose> {
        self.pressed_floating_close
            .get(&window)
            .is_some_and(|pressed| pressed.pointer_id == pointer_id)
            .then(|| self.pressed_floating_close.remove(&window))
            .flatten()
    }

    pub(super) fn pressed_floating_close(
        &self,
        window: AppWindowId,
    ) -> Option<fret_core::DockNodeId> {
        self.pressed_floating_close
            .get(&window)
            .map(|pressed| pressed.floating)
    }

    pub(super) fn tab_overflow_menu(&self, window: AppWindowId) -> Option<TabOverflowMenuState> {
        self.tab_overflow_menu.get(&window).cloned()
    }

    pub(super) fn set_tab_overflow_menu(
        &mut self,
        window: AppWindowId,
        menu: Option<TabOverflowMenuState>,
    ) {
        match menu {
            Some(menu) => {
                self.tab_overflow_menu.insert(window, menu);
            }
            None => {
                self.tab_overflow_menu.remove(&window);
            }
        }
    }

    pub(super) fn tab_overflow_menu_matches(
        &self,
        window: AppWindowId,
        menu: &Option<TabOverflowMenuState>,
    ) -> bool {
        match (self.tab_overflow_menu.get(&window), menu.as_ref()) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                a.tabs == b.tabs
                    && a.items == b.items
                    && a.scroll == b.scroll
                    && a.hovered == b.hovered
            }
            _ => false,
        }
    }

    pub(super) fn tab_scroll_for(
        &self,
        window: AppWindowId,
    ) -> HashMap<fret_core::DockNodeId, fret_core::Px> {
        self.tab_scroll.get(&window).cloned().unwrap_or_default()
    }

    pub(super) fn tab_widths_for(
        &self,
        window: AppWindowId,
    ) -> HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>> {
        self.tab_widths.get(&window).cloned().unwrap_or_default()
    }

    pub(super) fn set_tab_widths_for_window(
        &mut self,
        window: AppWindowId,
        widths: HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>>,
    ) {
        if widths.is_empty() {
            self.tab_widths.remove(&window);
        } else {
            self.tab_widths.insert(window, widths);
        }
    }

    pub(super) fn set_tab_scroll_for(
        &mut self,
        window: AppWindowId,
        tabs: fret_core::DockNodeId,
        scroll: fret_core::Px,
    ) {
        if scroll.0 <= 0.0 {
            if let Some(window_scroll) = self.tab_scroll.get_mut(&window) {
                window_scroll.remove(&tabs);
                if window_scroll.is_empty() {
                    self.tab_scroll.remove(&window);
                }
            }
            return;
        }

        self.tab_scroll
            .entry(window)
            .or_default()
            .insert(tabs, scroll);
    }

    pub(super) fn retain_tab_scroll_for_window(
        &mut self,
        window: AppWindowId,
        visible_tabs: &HashSet<fret_core::DockNodeId>,
    ) {
        if let Some(window_scroll) = self.tab_scroll.get_mut(&window) {
            window_scroll.retain(|tabs, _| visible_tabs.contains(tabs));
            if window_scroll.is_empty() {
                self.tab_scroll.remove(&window);
            }
        }
    }

    pub(super) fn should_auto_scroll_tab_drag(
        &mut self,
        window: AppWindowId,
        tabs: fret_core::DockNodeId,
        frame_id: fret_runtime::FrameId,
    ) -> bool {
        let window_frames = self
            .tab_drag_auto_scroll_last_frame
            .entry(window)
            .or_default();
        if window_frames
            .get(&tabs)
            .is_some_and(|last_frame| *last_frame == frame_id)
        {
            return false;
        }
        window_frames.insert(tabs, frame_id);
        true
    }

    pub(super) fn tab_hover(&self, window: AppWindowId) -> DeclarativeTabHover {
        self.tab_hover.get(&window).copied().unwrap_or_default()
    }

    pub(super) fn set_tab_hover(
        &mut self,
        window: AppWindowId,
        hover: DeclarativeTabHover,
    ) -> bool {
        if hover == DeclarativeTabHover::default() {
            return self.tab_hover.remove(&window).is_some();
        }
        if self.tab_hover.get(&window).copied() == Some(hover) {
            return false;
        }
        self.tab_hover.insert(window, hover);
        true
    }

    pub(super) fn floating_hover(&self, window: AppWindowId) -> DeclarativeFloatingHover {
        self.floating_hover
            .get(&window)
            .copied()
            .unwrap_or_default()
    }

    pub(super) fn set_floating_hover(
        &mut self,
        window: AppWindowId,
        hover: DeclarativeFloatingHover,
    ) -> bool {
        if hover == DeclarativeFloatingHover::default() {
            return self.floating_hover.remove(&window).is_some();
        }
        if self.floating_hover.get(&window).copied() == Some(hover) {
            return false;
        }
        self.floating_hover.insert(window, hover);
        true
    }
}
