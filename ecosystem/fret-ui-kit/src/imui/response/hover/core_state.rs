use fret_ui::GlobalElementId;

use super::ResponseExt;

mod accessors;

impl ResponseExt {
    pub(crate) fn set_core_hovered(&mut self, hovered: bool) {
        self.core.hovered = hovered;
    }

    pub(crate) fn set_core_pressed(&mut self, pressed: bool) {
        self.core.pressed = pressed;
    }

    pub(crate) fn set_core_focused(&mut self, focused: bool) {
        self.core.focused = focused;
    }

    pub(crate) fn set_core_clicked(&mut self, clicked: bool) {
        self.core.clicked = clicked;
    }

    pub(crate) fn set_core_changed(&mut self, changed: bool) {
        self.core.changed = changed;
    }

    pub(crate) fn merge_core_changed(&mut self, changed: bool) {
        self.core.changed |= changed;
    }

    pub(crate) fn set_core_rect(&mut self, rect: Option<fret_core::Rect>) {
        self.core.rect = rect;
    }

    pub(crate) fn set_id(&mut self, id: Option<GlobalElementId>) {
        self.id = id;
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
