use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_core::AppWindowId;

use super::super::tab_overflow::TabOverflowMenuState;
use super::super::viewport::ViewportCaptureState;

mod types;

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

    pub(super) fn begin_floating_drag(
        &mut self,
        window: AppWindowId,
        drag: DeclarativeFloatingDrag,
    ) {
        self.floating_drag.insert(window, drag);
    }

    pub(super) fn take_floating_drag(
        &mut self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativeFloatingDrag> {
        self.floating_drag
            .get(&window)
            .is_some_and(|drag| drag.pointer_id == pointer_id)
            .then(|| self.floating_drag.remove(&window))
            .flatten()
    }

    pub(super) fn begin_divider_drag(&mut self, window: AppWindowId, drag: DeclarativeDividerDrag) {
        self.divider_drag
            .entry(window)
            .or_default()
            .insert(drag.pointer_id, drag);
    }

    pub(super) fn divider_drag(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativeDividerDrag> {
        self.divider_drag
            .get(&window)
            .and_then(|drags| drags.get(&pointer_id))
            .cloned()
    }

    pub(super) fn take_divider_drag(
        &mut self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativeDividerDrag> {
        let drag = self
            .divider_drag
            .get_mut(&window)
            .and_then(|drags| drags.remove(&pointer_id));
        if self
            .divider_drag
            .get(&window)
            .is_some_and(|drags| drags.is_empty())
        {
            self.divider_drag.remove(&window);
        }
        drag
    }

    pub(super) fn begin_pending_dock_drag(
        &mut self,
        window: AppWindowId,
        pending: DeclarativePendingDockDrag,
    ) {
        self.pending_dock_drags
            .entry(window)
            .or_default()
            .insert(pending.pointer_id, pending);
    }

    pub(super) fn pending_dock_drag(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativePendingDockDrag> {
        self.pending_dock_drags
            .get(&window)
            .and_then(|pending| pending.get(&pointer_id))
            .cloned()
    }

    pub(super) fn take_pending_dock_drag(
        &mut self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativePendingDockDrag> {
        let pending = self
            .pending_dock_drags
            .get_mut(&window)
            .and_then(|pending| pending.remove(&pointer_id));
        if self
            .pending_dock_drags
            .get(&window)
            .is_some_and(|pending| pending.is_empty())
        {
            self.pending_dock_drags.remove(&window);
        }
        pending
    }

    pub(super) fn begin_pending_dock_tabs_drag(
        &mut self,
        window: AppWindowId,
        pending: DeclarativePendingDockTabsDrag,
    ) {
        self.pending_dock_tabs_drags
            .entry(window)
            .or_default()
            .insert(pending.pointer_id, pending);
    }

    pub(super) fn pending_dock_tabs_drag(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativePendingDockTabsDrag> {
        self.pending_dock_tabs_drags
            .get(&window)
            .and_then(|pending| pending.get(&pointer_id))
            .cloned()
    }

    pub(super) fn take_pending_dock_tabs_drag(
        &mut self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativePendingDockTabsDrag> {
        let pending = self
            .pending_dock_tabs_drags
            .get_mut(&window)
            .and_then(|pending| pending.remove(&pointer_id));
        if self
            .pending_dock_tabs_drags
            .get(&window)
            .is_some_and(|pending| pending.is_empty())
        {
            self.pending_dock_tabs_drags.remove(&window);
        }
        pending
    }

    pub(super) fn begin_viewport_capture(
        &mut self,
        window: AppWindowId,
        capture: ViewportCaptureState,
    ) {
        self.viewport_capture
            .entry(window)
            .or_default()
            .insert(capture.pointer_id, capture);
    }

    pub(super) fn viewport_capture(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<ViewportCaptureState> {
        self.viewport_capture
            .get(&window)
            .and_then(|captures| captures.get(&pointer_id))
            .cloned()
    }

    pub(super) fn take_viewport_capture(
        &mut self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<ViewportCaptureState> {
        let capture = self
            .viewport_capture
            .get_mut(&window)
            .and_then(|captures| captures.remove(&pointer_id));
        if self
            .viewport_capture
            .get(&window)
            .is_some_and(|captures| captures.is_empty())
        {
            self.viewport_capture.remove(&window);
        }
        capture
    }

    pub(super) fn has_viewport_capture_for_window(&self, window: AppWindowId) -> bool {
        self.viewport_capture
            .get(&window)
            .is_some_and(|captures| !captures.is_empty())
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
