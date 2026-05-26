use super::ResponseExt;

impl ResponseExt {
    pub(crate) fn set_activated(&mut self, activated: bool) {
        self.activated = activated;
    }

    pub(crate) fn set_deactivated(&mut self, deactivated: bool) {
        self.deactivated = deactivated;
    }

    pub(crate) fn set_edited(&mut self, edited: bool) {
        self.edited = edited;
    }

    pub(crate) fn set_deactivated_after_edit(&mut self, deactivated_after_edit: bool) {
        self.deactivated_after_edit = deactivated_after_edit;
    }

    pub(crate) fn merge_activated(&mut self, activated: bool) {
        self.activated |= activated;
    }

    pub(crate) fn merge_deactivated(&mut self, deactivated: bool) {
        self.deactivated |= deactivated;
    }

    pub(crate) fn merge_edited(&mut self, edited: bool) {
        self.edited |= edited;
    }

    pub(crate) fn merge_deactivated_after_edit(&mut self, deactivated_after_edit: bool) {
        self.deactivated_after_edit |= deactivated_after_edit;
    }

    pub(crate) fn clear_lifecycle_signals(&mut self) {
        self.activated = false;
        self.deactivated = false;
        self.edited = false;
        self.deactivated_after_edit = false;
    }

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
