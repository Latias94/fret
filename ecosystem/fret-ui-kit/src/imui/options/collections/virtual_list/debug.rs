use super::options::VirtualListOptions;

impl std::fmt::Debug for VirtualListOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualListOptions")
            .field("viewport_height", &self.viewport_height)
            .field("estimate_row_height", &self.estimate_row_height)
            .field("overscan", &self.overscan)
            .field("items_revision", &self.items_revision)
            .field("measure_mode", &self.measure_mode)
            .field("key_cache", &self.key_cache)
            .field("keep_alive", &self.keep_alive)
            .field("gap", &self.gap)
            .field("scroll_margin", &self.scroll_margin)
            .field("known_row_height_at", &self.known_row_height_at.is_some())
            .field("handle", &self.handle.is_some())
            .field("test_id", &self.test_id)
            .finish()
    }
}
