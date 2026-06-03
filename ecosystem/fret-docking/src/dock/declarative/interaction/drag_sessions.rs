use fret_core::AppWindowId;

use super::super::super::viewport::ViewportCaptureState;
use super::{
    DeclarativeDividerDrag, DeclarativeDockInteractionService, DeclarativeFloatingDrag,
    DeclarativePendingDockDrag, DeclarativePendingDockTabsDrag,
};

impl DeclarativeDockInteractionService {
    pub(in crate::dock::declarative) fn begin_floating_drag(
        &mut self,
        window: AppWindowId,
        drag: DeclarativeFloatingDrag,
    ) {
        self.floating_drag.insert(window, drag);
    }

    pub(in crate::dock::declarative) fn take_floating_drag(
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

    pub(in crate::dock::declarative) fn begin_divider_drag(
        &mut self,
        window: AppWindowId,
        drag: DeclarativeDividerDrag,
    ) {
        self.divider_drag
            .entry(window)
            .or_default()
            .insert(drag.pointer_id, drag);
    }

    pub(in crate::dock::declarative) fn divider_drag(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativeDividerDrag> {
        self.divider_drag
            .get(&window)
            .and_then(|drags| drags.get(&pointer_id))
            .cloned()
    }

    pub(in crate::dock::declarative) fn take_divider_drag(
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

    pub(in crate::dock::declarative) fn begin_pending_dock_drag(
        &mut self,
        window: AppWindowId,
        pending: DeclarativePendingDockDrag,
    ) {
        self.pending_dock_drags
            .entry(window)
            .or_default()
            .insert(pending.pointer_id, pending);
    }

    pub(in crate::dock::declarative) fn pending_dock_drag(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativePendingDockDrag> {
        self.pending_dock_drags
            .get(&window)
            .and_then(|pending| pending.get(&pointer_id))
            .cloned()
    }

    pub(in crate::dock::declarative) fn take_pending_dock_drag(
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

    pub(in crate::dock::declarative) fn begin_pending_dock_tabs_drag(
        &mut self,
        window: AppWindowId,
        pending: DeclarativePendingDockTabsDrag,
    ) {
        self.pending_dock_tabs_drags
            .entry(window)
            .or_default()
            .insert(pending.pointer_id, pending);
    }

    pub(in crate::dock::declarative) fn pending_dock_tabs_drag(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<DeclarativePendingDockTabsDrag> {
        self.pending_dock_tabs_drags
            .get(&window)
            .and_then(|pending| pending.get(&pointer_id))
            .cloned()
    }

    pub(in crate::dock::declarative) fn take_pending_dock_tabs_drag(
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

    pub(in crate::dock::declarative) fn begin_viewport_capture(
        &mut self,
        window: AppWindowId,
        capture: ViewportCaptureState,
    ) {
        self.viewport_capture
            .entry(window)
            .or_default()
            .insert(capture.pointer_id, capture);
    }

    pub(in crate::dock::declarative) fn viewport_capture(
        &self,
        window: AppWindowId,
        pointer_id: fret_core::PointerId,
    ) -> Option<ViewportCaptureState> {
        self.viewport_capture
            .get(&window)
            .and_then(|captures| captures.get(&pointer_id))
            .cloned()
    }

    pub(in crate::dock::declarative) fn take_viewport_capture(
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

    pub(in crate::dock::declarative) fn has_viewport_capture_for_window(
        &self,
        window: AppWindowId,
    ) -> bool {
        self.viewport_capture
            .get(&window)
            .is_some_and(|captures| !captures.is_empty())
    }
}
