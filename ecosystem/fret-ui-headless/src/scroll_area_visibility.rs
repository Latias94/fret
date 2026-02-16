use crate::scroll_area::ScrollAreaType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollVisibilityState {
    Hidden,
    Scrolling,
    Interacting,
    Idle,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollAreaVisibilityConfig {
    /// Mirrors Radix `scrollHideDelay` (default 600ms).
    ///
    /// Fret expresses this in monotonic "ticks" supplied by the driver.
    pub scroll_hide_delay_ticks: u64,
    /// Mirrors Radix's internal "scroll end" debounce (100ms).
    pub scroll_end_debounce_ticks: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollAreaVisibilityInput {
    pub ty: ScrollAreaType,
    pub hovered: bool,
    pub has_overflow: bool,
    pub scrolled: bool,
    pub tick: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollAreaVisibilityOutput {
    pub visible: bool,
    /// When true, the driver should keep scheduling frames so time-based transitions can fire.
    pub animating: bool,
}

#[derive(Debug, Clone)]
pub struct ScrollAreaVisibility {
    last_ty: Option<ScrollAreaType>,
    hover_visible_until: Option<u64>,
    was_hovered: bool,
    scroll_state: ScrollVisibilityState,
    last_scroll_tick: Option<u64>,
    scroll_hide_deadline: Option<u64>,
}

impl Default for ScrollAreaVisibility {
    fn default() -> Self {
        Self {
            last_ty: None,
            hover_visible_until: None,
            was_hovered: false,
            scroll_state: ScrollVisibilityState::Hidden,
            last_scroll_tick: None,
            scroll_hide_deadline: None,
        }
    }
}

impl ScrollAreaVisibility {
    pub fn update(
        &mut self,
        input: ScrollAreaVisibilityInput,
        config: ScrollAreaVisibilityConfig,
    ) -> ScrollAreaVisibilityOutput {
        if self.last_ty != Some(input.ty) {
            self.reset_for_type(input.ty);
        }

        if !input.has_overflow {
            self.reset_for_type(input.ty);
            return ScrollAreaVisibilityOutput {
                visible: false,
                animating: false,
            };
        }

        match input.ty {
            ScrollAreaType::Always | ScrollAreaType::Auto => ScrollAreaVisibilityOutput {
                visible: true,
                animating: false,
            },
            ScrollAreaType::Hover => self.update_hover(input, config),
            ScrollAreaType::Scroll => self.update_scroll(input, config),
        }
    }

    fn reset_for_type(&mut self, ty: ScrollAreaType) {
        self.last_ty = Some(ty);
        self.hover_visible_until = None;
        self.was_hovered = false;
        self.scroll_state = ScrollVisibilityState::Hidden;
        self.last_scroll_tick = None;
        self.scroll_hide_deadline = None;
    }

    fn update_hover(
        &mut self,
        input: ScrollAreaVisibilityInput,
        config: ScrollAreaVisibilityConfig,
    ) -> ScrollAreaVisibilityOutput {
        if input.hovered {
            self.was_hovered = true;
            self.hover_visible_until = None;
            return ScrollAreaVisibilityOutput {
                visible: true,
                animating: false,
            };
        }

        if self.was_hovered {
            self.was_hovered = false;
            self.hover_visible_until =
                Some(input.tick.saturating_add(config.scroll_hide_delay_ticks));
        }

        let Some(deadline) = self.hover_visible_until else {
            return ScrollAreaVisibilityOutput {
                visible: false,
                animating: false,
            };
        };

        let visible = input.tick < deadline;
        if !visible {
            self.hover_visible_until = None;
        }

        ScrollAreaVisibilityOutput {
            visible,
            animating: visible,
        }
    }

    fn update_scroll(
        &mut self,
        input: ScrollAreaVisibilityInput,
        config: ScrollAreaVisibilityConfig,
    ) -> ScrollAreaVisibilityOutput {
        if input.scrolled {
            self.last_scroll_tick = Some(input.tick);
            self.scroll_hide_deadline = None;
            match self.scroll_state {
                ScrollVisibilityState::Hidden | ScrollVisibilityState::Idle => {
                    self.scroll_state = ScrollVisibilityState::Scrolling;
                }
                ScrollVisibilityState::Scrolling | ScrollVisibilityState::Interacting => {}
            }
        }

        if input.hovered {
            match self.scroll_state {
                ScrollVisibilityState::Scrolling | ScrollVisibilityState::Idle => {
                    self.scroll_state = ScrollVisibilityState::Interacting;
                    self.scroll_hide_deadline = None;
                }
                ScrollVisibilityState::Hidden | ScrollVisibilityState::Interacting => {}
            }
        } else if self.scroll_state == ScrollVisibilityState::Interacting {
            self.scroll_state = ScrollVisibilityState::Idle;
            self.scroll_hide_deadline =
                Some(input.tick.saturating_add(config.scroll_hide_delay_ticks));
        }

        if self.scroll_state == ScrollVisibilityState::Scrolling
            && let Some(last) = self.last_scroll_tick
            && input.tick.saturating_sub(last) >= config.scroll_end_debounce_ticks
        {
            self.scroll_state = ScrollVisibilityState::Idle;
            self.scroll_hide_deadline =
                Some(input.tick.saturating_add(config.scroll_hide_delay_ticks));
        }

        if self.scroll_state == ScrollVisibilityState::Idle
            && let Some(deadline) = self.scroll_hide_deadline
            && input.tick >= deadline
        {
            self.scroll_state = ScrollVisibilityState::Hidden;
            self.scroll_hide_deadline = None;
        }

        let visible = self.scroll_state != ScrollVisibilityState::Hidden;
        let animating = match self.scroll_state {
            ScrollVisibilityState::Hidden | ScrollVisibilityState::Interacting => false,
            ScrollVisibilityState::Scrolling | ScrollVisibilityState::Idle => true,
        };

        ScrollAreaVisibilityOutput { visible, animating }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIDE: u64 = 10;
    const DEBOUNCE: u64 = 2;

    fn cfg() -> ScrollAreaVisibilityConfig {
        ScrollAreaVisibilityConfig {
            scroll_hide_delay_ticks: HIDE,
            scroll_end_debounce_ticks: DEBOUNCE,
        }
    }

    #[test]
    fn hover_hides_after_delay() {
        let mut vis = ScrollAreaVisibility::default();

        let out_init = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Hover,
                hovered: false,
                has_overflow: true,
                scrolled: false,
                tick: 0,
            },
            cfg(),
        );
        assert!(!out_init.visible);
        assert!(!out_init.animating);

        let out0 = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Hover,
                hovered: true,
                has_overflow: true,
                scrolled: false,
                tick: 1,
            },
            cfg(),
        );
        assert!(out0.visible);
        assert!(!out0.animating);

        let out1 = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Hover,
                hovered: false,
                has_overflow: true,
                scrolled: false,
                tick: 2,
            },
            cfg(),
        );
        assert!(out1.visible);
        assert!(out1.animating);

        let out2 = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Hover,
                hovered: false,
                has_overflow: true,
                scrolled: false,
                tick: 2 + HIDE,
            },
            cfg(),
        );
        assert!(!out2.visible);
        assert!(!out2.animating);

        let out3 = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Hover,
                hovered: false,
                has_overflow: true,
                scrolled: false,
                tick: 3 + HIDE,
            },
            cfg(),
        );
        assert!(
            !out3.visible,
            "hover mode should remain hidden after the delay"
        );
        assert!(!out3.animating);
    }

    #[test]
    fn scroll_shows_while_scrolling_then_hides() {
        let mut vis = ScrollAreaVisibility::default();

        let out0 = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Scroll,
                hovered: false,
                has_overflow: true,
                scrolled: false,
                tick: 1,
            },
            cfg(),
        );
        assert!(!out0.visible);

        let out1 = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Scroll,
                hovered: false,
                has_overflow: true,
                scrolled: true,
                tick: 2,
            },
            cfg(),
        );
        assert!(out1.visible);
        assert!(out1.animating);

        let out2 = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Scroll,
                hovered: false,
                has_overflow: true,
                scrolled: false,
                tick: 2 + DEBOUNCE,
            },
            cfg(),
        );
        assert!(out2.visible);
        assert!(out2.animating);

        let out3 = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Scroll,
                hovered: false,
                has_overflow: true,
                scrolled: false,
                tick: 2 + DEBOUNCE + HIDE,
            },
            cfg(),
        );
        assert!(!out3.visible);
        assert!(!out3.animating);
    }

    #[test]
    fn scroll_interaction_keeps_visible_until_leave() {
        let mut vis = ScrollAreaVisibility::default();

        let _ = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Scroll,
                hovered: false,
                has_overflow: true,
                scrolled: true,
                tick: 1,
            },
            cfg(),
        );

        let out0 = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Scroll,
                hovered: true,
                has_overflow: true,
                scrolled: false,
                tick: 2,
            },
            cfg(),
        );
        assert!(out0.visible);
        assert!(!out0.animating);

        let out1 = vis.update(
            ScrollAreaVisibilityInput {
                ty: ScrollAreaType::Scroll,
                hovered: false,
                has_overflow: true,
                scrolled: false,
                tick: 3,
            },
            cfg(),
        );
        assert!(out1.visible);
        assert!(out1.animating);
    }
}
