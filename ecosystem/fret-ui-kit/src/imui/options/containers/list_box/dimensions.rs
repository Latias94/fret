use fret_core::Px;

use super::ListBoxOptions;

impl ListBoxOptions {
    pub fn height(mut self, height: Px) -> Self {
        self.layout = self.layout.h_px(height);
        self
    }

    pub fn width(mut self, width: Px) -> Self {
        self.layout = self.layout.w_px(width);
        self
    }

    pub fn size(mut self, width: Px, height: Px) -> Self {
        self.layout = self.layout.w_px(width).h_px(height);
        self
    }
}
