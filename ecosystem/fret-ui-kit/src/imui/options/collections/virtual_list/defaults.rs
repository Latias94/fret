use fret_core::Px;

use super::options::VirtualListOptions;

impl Default for VirtualListOptions {
    fn default() -> Self {
        Self {
            viewport_height: Px(240.0),
            estimate_row_height: Px(28.0),
            overscan: 6,
            items_revision: 0,
            measure_mode: fret_ui::element::VirtualListMeasureMode::Measured,
            key_cache: fret_ui::element::VirtualListKeyCacheMode::AllKeys,
            keep_alive: 0,
            gap: Px(0.0),
            scroll_margin: Px(0.0),
            known_row_height_at: None,
            handle: None,
            test_id: None,
        }
    }
}
