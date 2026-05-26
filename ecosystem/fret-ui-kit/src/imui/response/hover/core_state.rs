use fret_authoring::Response;
use fret_ui::GlobalElementId;

use super::ResponseExt;

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

    pub fn id(self) -> Option<GlobalElementId> {
        self.id
    }

    pub fn core(self) -> Response {
        self.core
    }

    pub fn from_core(core: Response) -> Self {
        Self {
            core,
            ..Default::default()
        }
    }

    pub fn rect(self) -> Option<fret_core::Rect> {
        self.core.rect
    }

    pub fn hovered(self) -> bool {
        self.core.hovered
    }

    pub fn pressed(self) -> bool {
        self.core.pressed
    }

    pub fn focused(self) -> bool {
        self.core.focused
    }

    pub fn clicked(self) -> bool {
        self.core.clicked()
    }

    pub fn changed(self) -> bool {
        self.core.changed()
    }

    pub fn enabled(self) -> bool {
        self.enabled
    }
}
