use super::TableColumn;

impl TableColumn {
    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub(crate) fn set_visible_for_policy(&mut self, visible: bool) {
        self.visible = visible;
    }
}
