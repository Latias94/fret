use fret_authoring::Response;
use fret_ui::GlobalElementId;

use super::ResponseExt;

impl ResponseExt {
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
