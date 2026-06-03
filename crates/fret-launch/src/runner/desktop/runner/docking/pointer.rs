use super::super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(in crate::runner::desktop::runner) fn dock_drag_pointer_id(
        &self,
    ) -> Option<fret_core::PointerId> {
        use fret_runtime::DragHost as _;
        self.app.find_drag_pointer_id(|d| {
            d.cross_window_hover
                && (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                    || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
        })
    }

    pub(in crate::runner::desktop::runner) fn sync_dock_drag_pointer_capture(&mut self) {
        let Some(pointer_id) = self.dock_drag_pointer_id() else {
            self.dock_drag_pointer_capture = None;
            return;
        };
        let Some(drag) = self.app.drag(pointer_id) else {
            self.dock_drag_pointer_capture = None;
            return;
        };

        let desired_window = drag.source_window;
        let Some((captured_pointer, captured_window)) = self.dock_drag_pointer_capture else {
            self.dock_drag_pointer_capture = Some((pointer_id, desired_window));
            return;
        };

        if captured_pointer != pointer_id {
            self.dock_drag_pointer_capture = Some((pointer_id, desired_window));
            return;
        }

        if captured_window == desired_window {
            return;
        }

        // When docking tear-off migrates a drag session to a new window, the original window can
        // remain stuck in a "pointer down" state because the eventual `PointerUp` is delivered to
        // the new window. Once the drag's `source_window` changes, the old window is no longer
        // considered part of the drag (source/current), so we can safely send it a cancel to
        // release pending pointer capture/press state without terminating the active drag.
        self.deliver_dock_drag_pointer_cancel(captured_window, pointer_id);

        self.dock_drag_pointer_capture = Some((pointer_id, desired_window));
    }

    fn deliver_dock_drag_pointer_cancel(
        &mut self,
        window: fret_core::AppWindowId,
        pointer_id: fret_core::PointerId,
    ) {
        let modifiers = self
            .windows
            .get(window)
            .map(|w| w.platform.input.modifiers)
            .unwrap_or_default();
        let position = self
            .cursor_screen_pos
            .and_then(|screen| self.local_pos_for_window(window, screen));
        let buttons = fret_core::MouseButtons {
            left: self.left_mouse_down,
            right: false,
            middle: false,
        };
        self.deliver_window_event_now(
            window,
            &Event::PointerCancel(fret_core::PointerCancelEvent {
                pointer_id,
                position,
                buttons,
                modifiers,
                pointer_type: fret_core::PointerType::Mouse,
                reason: fret_core::PointerCancelReason::LeftWindow,
            }),
        );
    }
}
