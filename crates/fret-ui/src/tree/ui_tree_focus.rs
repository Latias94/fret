use super::*;

impl<H: UiHost> UiTree<H> {
    pub fn set_window(&mut self, window: AppWindowId) {
        self.window = Some(window);
    }

    pub fn focus(&self) -> Option<NodeId> {
        self.focus
    }

    #[track_caller]
    pub fn set_focus(&mut self, focus: Option<NodeId>) {
        #[cfg(debug_assertions)]
        let debug_focus_scope = std::env::var_os("FRET_TEST_DEBUG_FOCUS_SCOPE").is_some();
        #[cfg(debug_assertions)]
        if debug_focus_scope && self.focus != focus {
            let loc = std::panic::Location::caller();
            eprintln!(
                "debug: set_focus at {}:{}:{}: {:?} -> {:?}",
                loc.file(),
                loc.line(),
                loc.column(),
                self.focus,
                focus
            );
        }

        if let Some(focus) = focus {
            let (active_roots, barrier_root) = self.active_focus_layers();
            if barrier_root.is_some()
                && !self.is_reachable_from_any_root_via_children(focus, active_roots.as_slice())
            {
                return;
            }
        }
        if self.focus != focus {
            self.ime_composing = false;
        }
        let changed = self.focus != focus;
        self.focus = focus;
        if changed {
            self.request_post_layout_window_runtime_snapshot_refine_if_layout_active();
        }
    }

    /// Internal focus mutation helper that skips focus-barrier gating.
    ///
    /// This is used by mechanism code that must clear or adjust focus (e.g. scope enforcement)
    /// without re-entering policy checks.
    #[track_caller]
    pub(in crate::tree) fn set_focus_unchecked(
        &mut self,
        focus: Option<NodeId>,
        reason: &'static str,
    ) {
        #[cfg(debug_assertions)]
        {
            let debug_focus_scope = std::env::var_os("FRET_TEST_DEBUG_FOCUS_SCOPE").is_some();
            if debug_focus_scope && self.focus != focus {
                let loc = std::panic::Location::caller();
                eprintln!(
                    "debug: set_focus_unchecked({reason}) at {}:{}:{}: {:?} -> {:?}",
                    loc.file(),
                    loc.line(),
                    loc.column(),
                    self.focus,
                    focus
                );
            }
        }

        if self.focus != focus {
            self.ime_composing = false;
        }
        let changed = self.focus != focus;
        self.focus = focus;
        if changed {
            self.request_post_layout_window_runtime_snapshot_refine_if_layout_active();
        }
    }

    const TOUCH_POINTER_DOWN_OUTSIDE_SLOP_PX: f32 = 6.0;

    pub(in crate::tree) fn update_touch_pointer_down_outside_move(
        &mut self,
        pointer_id: PointerId,
        position: Point,
    ) {
        let Some(candidate) = self
            .touch_pointer_down_outside_candidates
            .get_mut(&pointer_id)
        else {
            return;
        };
        if candidate.moved {
            return;
        }
        let dx = position.x.0 - candidate.start_pos.x.0;
        let dy = position.y.0 - candidate.start_pos.y.0;
        if (dx * dx + dy * dy).sqrt() > Self::TOUCH_POINTER_DOWN_OUTSIDE_SLOP_PX {
            candidate.moved = true;
        }
    }
}
