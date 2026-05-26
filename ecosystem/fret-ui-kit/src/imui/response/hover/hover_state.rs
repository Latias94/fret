use super::ResponseExt;

impl ResponseExt {
    pub(crate) fn set_pointer_hovered_raw(&mut self, pointer_hovered_raw: bool) {
        self.pointer_hovered_raw = pointer_hovered_raw;
    }

    pub(crate) fn set_pointer_hovered_raw_below_barrier(
        &mut self,
        pointer_hovered_raw_below_barrier: bool,
    ) {
        self.pointer_hovered_raw_below_barrier = pointer_hovered_raw_below_barrier;
    }

    pub(crate) fn set_hover_stationary_met(&mut self, hover_stationary_met: bool) {
        self.hover_stationary_met = hover_stationary_met;
    }

    pub(crate) fn set_hover_delay_short_met(&mut self, hover_delay_short_met: bool) {
        self.hover_delay_short_met = hover_delay_short_met;
    }

    pub(crate) fn set_hover_delay_normal_met(&mut self, hover_delay_normal_met: bool) {
        self.hover_delay_normal_met = hover_delay_normal_met;
    }

    pub(crate) fn set_hover_delay_short_shared_met(&mut self, hover_delay_short_shared_met: bool) {
        self.hover_delay_short_shared_met = hover_delay_short_shared_met;
    }

    pub(crate) fn set_hover_delay_normal_shared_met(
        &mut self,
        hover_delay_normal_shared_met: bool,
    ) {
        self.hover_delay_normal_shared_met = hover_delay_normal_shared_met;
    }

    pub(crate) fn set_hover_blocked_by_active_item(&mut self, hover_blocked_by_active_item: bool) {
        self.hover_blocked_by_active_item = hover_blocked_by_active_item;
    }

    pub(crate) fn set_nav_highlighted(&mut self, nav_highlighted: bool) {
        self.nav_highlighted = nav_highlighted;
    }

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
