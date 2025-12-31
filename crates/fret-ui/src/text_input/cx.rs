use crate::widget::{CommandCx, EventCx};
use crate::{Invalidation, UiHost};

pub(super) trait TextInputUiCx {
    fn invalidate_self(&mut self, kind: Invalidation);
    fn request_redraw(&mut self);
}

impl<'a, H: UiHost> TextInputUiCx for EventCx<'a, H> {
    fn invalidate_self(&mut self, kind: Invalidation) {
        EventCx::invalidate_self(self, kind);
    }

    fn request_redraw(&mut self) {
        EventCx::request_redraw(self);
    }
}

impl<'a, H: UiHost> TextInputUiCx for CommandCx<'a, H> {
    fn invalidate_self(&mut self, kind: Invalidation) {
        CommandCx::invalidate_self(self, kind);
    }

    fn request_redraw(&mut self) {
        CommandCx::request_redraw(self);
    }
}
