use super::ResponseExt;

impl ResponseExt {
    pub fn activated(self) -> bool {
        self.activated
    }

    pub fn deactivated(self) -> bool {
        self.deactivated
    }

    pub fn edited(self) -> bool {
        self.edited
    }

    pub fn deactivated_after_edit(self) -> bool {
        self.deactivated_after_edit
    }
}
