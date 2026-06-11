use super::ResponseExt;

impl ResponseExt {
    pub fn pointer_hovered_raw(self) -> bool {
        self.pointer_hovered_raw
    }

    pub fn pointer_hovered_raw_below_barrier(self) -> bool {
        self.pointer_hovered_raw_below_barrier
    }

    pub fn hover_stationary_met(self) -> bool {
        self.hover_stationary_met
    }

    pub fn hover_delay_short_met(self) -> bool {
        self.hover_delay_short_met
    }

    pub fn hover_delay_normal_met(self) -> bool {
        self.hover_delay_normal_met
    }

    pub fn hover_delay_short_shared_met(self) -> bool {
        self.hover_delay_short_shared_met
    }

    pub fn hover_delay_normal_shared_met(self) -> bool {
        self.hover_delay_normal_shared_met
    }

    pub fn hover_blocked_by_active_item(self) -> bool {
        self.hover_blocked_by_active_item
    }

    pub fn nav_highlighted(self) -> bool {
        self.nav_highlighted
    }
}
