use super::*;

impl<H: UiHost> UiTree<H> {
    pub fn set_window(&mut self, window: AppWindowId) {
        self.window = Some(window);
    }

    pub fn focus(&self) -> Option<NodeId> {
        self.focus
    }

    pub fn set_focus(&mut self, focus: Option<NodeId>) {
        if let Some(focus) = focus {
            let (active_roots, barrier_root) = self.active_focus_layers();
            if barrier_root.is_some() && !self.node_in_any_layer(focus, active_roots.as_slice()) {
                return;
            }
        }
        if self.focus != focus {
            self.ime_composing = false;
        }
        self.focus = focus;
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
