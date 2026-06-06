use super::super::{WinitAppDriver, WinitRunner};

#[derive(Debug, Clone, Copy)]
pub(super) struct MovingWindowUnderTarget {
    pub(super) window: Option<fret_core::AppWindowId>,
    pub(super) source: fret_runtime::WindowUnderCursorSource,
}

impl Default for MovingWindowUnderTarget {
    fn default() -> Self {
        Self {
            window: None,
            source: fret_runtime::WindowUnderCursorSource::Unknown,
        }
    }
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn resolve_internal_drag_moving_window(
        &self,
        drag_kind: fret_runtime::DragKindId,
        drag_source_window: fret_core::AppWindowId,
    ) -> Option<fret_core::AppWindowId> {
        self.dock_tearoff_follow
            .filter(|follow| follow.source_window == drag_source_window)
            .map(|follow| follow.window)
            .or_else(|| {
                matches!(
                    drag_kind,
                    fret_runtime::DRAG_KIND_DOCK_TABS | fret_runtime::DRAG_KIND_DOCK_PANEL
                )
                .then_some(drag_source_window)
                .filter(|w| self.main_window.is_some_and(|main| *w != main))
            })
    }

    pub(super) fn window_under_internal_drag_moving_window(
        &self,
        moving_window: Option<fret_core::AppWindowId>,
        screen_pos: winit::dpi::PhysicalPosition<f64>,
        allow_window_under_cursor: bool,
        reliable_window_under_cursor: bool,
    ) -> MovingWindowUnderTarget {
        let Some(moving_window) = moving_window else {
            return MovingWindowUnderTarget::default();
        };
        if !allow_window_under_cursor {
            return MovingWindowUnderTarget::default();
        }

        let mut source = fret_runtime::WindowUnderCursorSource::Unknown;
        for candidate in self.moving_window_under_target_candidates(moving_window, screen_pos) {
            let hit = if reliable_window_under_cursor {
                self.window_under_cursor_platform(candidate, Some(moving_window))
            } else {
                self.window_under_cursor_best_effort(candidate, Some(moving_window))
            };
            if matches!(source, fret_runtime::WindowUnderCursorSource::Unknown)
                && !matches!(hit.source, fret_runtime::WindowUnderCursorSource::Unknown)
            {
                source = hit.source;
            }
            if let Some(window) = hit.window.filter(|window| *window != moving_window) {
                return MovingWindowUnderTarget {
                    window: Some(window),
                    source: hit.source,
                };
            }
        }

        MovingWindowUnderTarget {
            window: None,
            source,
        }
    }

    fn moving_window_under_target_candidates(
        &self,
        moving_window: fret_core::AppWindowId,
        screen_pos: winit::dpi::PhysicalPosition<f64>,
    ) -> Vec<winit::dpi::PhysicalPosition<f64>> {
        // Scripted diagnostics inject cursor overrides in window-client coordinates. The
        // simulated cursor may briefly drift outside the moving window while the runner also
        // updates OS window positions, so sample a few stable points inside the moving window.
        let mut candidates = Vec::with_capacity(3);
        candidates.push(screen_pos);
        if !self.diag_pointer_input_isolation_active() {
            return candidates;
        }

        if let Some(clamped) = self.clamp_screen_pos_to_window_client(moving_window, screen_pos) {
            candidates.push(clamped);
        }
        if let Some((origin, size)) = self.window_client_rect_screen(moving_window) {
            candidates.push(winit::dpi::PhysicalPosition::new(
                origin.x + (size.width as f64) * 0.5,
                origin.y + (size.height as f64) * 0.5,
            ));
        }
        candidates
    }
}
