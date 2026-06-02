use fret_core::{Modifiers, Point, PointerId, Px};

use super::super::EditSession;

#[derive(Debug)]
pub(super) struct DragState<T> {
    pub(super) current_value: T,
    pub(super) session: EditSession<T>,
    pub(super) edited_during_session: bool,
    pub(super) armed: bool,
    pub(super) dragging: bool,
    pub(super) pointer_id: Option<PointerId>,
    pub(super) down_pos: Point,
    pub(super) start_x: f64,
    pub(super) start_value: T,
}

impl<T: Copy + Default> Default for DragState<T> {
    fn default() -> Self {
        Self {
            current_value: T::default(),
            session: EditSession::default(),
            edited_during_session: false,
            armed: false,
            dragging: false,
            pointer_id: None,
            down_pos: Point::new(Px(0.0), Px(0.0)),
            start_x: 0.0,
            start_value: T::default(),
        }
    }
}

impl<T: Copy + Default + PartialEq> DragState<T> {
    pub(super) fn begin_session(&mut self, pointer_id: PointerId, position: Point) {
        let current_value = self.current_value;
        self.session.begin(current_value);
        self.edited_during_session = false;
        self.armed = true;
        self.dragging = false;
        self.pointer_id = Some(pointer_id);
        self.down_pos = position;
        self.start_x = position.x.0 as f64;
        self.start_value = current_value;
    }

    pub(super) fn apply_live_value(&mut self, next: T) -> bool {
        if self.current_value == next {
            return false;
        }

        self.current_value = next;
        self.edited_during_session = true;
        true
    }

    pub(super) fn commit_session(&mut self) -> bool {
        let edited = self.edited_during_session;
        if self.session.is_active() {
            let _ = self.session.commit();
        }
        self.edited_during_session = false;
        edited
    }

    pub(super) fn cancel_session(&mut self) -> Option<T> {
        self.edited_during_session = false;
        self.session.cancel()
    }

    pub(super) fn clear_pointer_session(&mut self) {
        self.armed = false;
        self.dragging = false;
        self.pointer_id = None;
    }
}

pub(super) enum DragValueCoreMoveAction<T> {
    None,
    Live(T),
    Commit { edited: bool },
    Cancel(Option<T>),
}

pub(super) fn resolve_scrub_multiplier(mods: Modifiers, slow: f64, fast: f64) -> f64 {
    let mut out = 1.0;
    if mods.shift {
        out *= slow;
    }
    if mods.alt {
        out *= fast;
    }
    out
}
