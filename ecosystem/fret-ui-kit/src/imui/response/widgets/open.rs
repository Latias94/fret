use fret_ui::GlobalElementId;

use super::super::hover::ResponseExt;

#[derive(Debug, Clone, Copy)]
pub struct DisclosureResponse {
    pub(crate) trigger: ResponseExt,
    pub(crate) open: bool,
    pub(crate) toggled: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ComboResponse {
    pub(crate) trigger: ResponseExt,
    pub(crate) open: bool,
    pub(crate) toggled: bool,
}

impl DisclosureResponse {
    pub(crate) fn empty() -> Self {
        Self {
            trigger: ResponseExt::default(),
            open: false,
            toggled: false,
        }
    }

    pub fn id(self) -> Option<GlobalElementId> {
        self.trigger.id()
    }

    pub fn response(self) -> ResponseExt {
        self.trigger
    }

    pub fn open(self) -> bool {
        self.open
    }

    pub fn toggled(self) -> bool {
        self.toggled
    }

    pub fn clicked(self) -> bool {
        self.trigger.clicked()
    }

    pub fn opened(self) -> bool {
        self.toggled && self.open
    }

    pub fn closed(self) -> bool {
        self.toggled && !self.open
    }

    pub fn hovered_like_imgui(self) -> bool {
        self.trigger.hovered_like_imgui()
    }
}

impl ComboResponse {
    pub fn id(self) -> Option<GlobalElementId> {
        self.trigger.id()
    }

    pub fn response(self) -> ResponseExt {
        self.trigger
    }

    pub fn open(self) -> bool {
        self.open
    }

    pub fn toggled(self) -> bool {
        self.toggled
    }

    pub fn opened(self) -> bool {
        self.toggled && self.open
    }

    pub fn closed(self) -> bool {
        self.toggled && !self.open
    }

    pub fn clicked(self) -> bool {
        self.trigger.clicked()
    }

    pub fn hovered_like_imgui(self) -> bool {
        self.trigger.hovered_like_imgui()
    }
}
