use fret_core::Px;

use crate::imui::ScrollOptions;

use super::ListBoxOptions;

impl Default for ListBoxOptions {
    fn default() -> Self {
        Self {
            layout: crate::LayoutRefinement::default().h_px(Px(160.0)),
            scroll: ScrollOptions::default(),
            label: None,
            multiselectable: false,
            test_id: None,
            content_test_id: None,
        }
    }
}
