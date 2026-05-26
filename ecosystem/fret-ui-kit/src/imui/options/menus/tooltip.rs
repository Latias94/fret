use std::sync::Arc;

use fret_core::{Px, Size};

use crate::primitives::popper;

#[derive(Debug, Clone)]
pub struct TooltipOptions {
    pub placement: popper::PopperContentPlacement,
    pub estimated_size: Size,
    pub window_margin: Px,
    pub open_delay_frames_override: Option<u32>,
    pub close_delay_frames_override: Option<u32>,
    pub disable_hoverable_content: Option<bool>,
    pub test_id: Option<Arc<str>>,
}

impl Default for TooltipOptions {
    fn default() -> Self {
        Self {
            placement: popper::PopperContentPlacement::new(
                popper::LayoutDirection::Ltr,
                popper::Side::Top,
                popper::Align::Center,
                Px(6.0),
            )
            .with_shift_cross_axis(true),
            estimated_size: Size::new(Px(180.0), Px(32.0)),
            window_margin: Px(8.0),
            open_delay_frames_override: None,
            close_delay_frames_override: None,
            disable_hoverable_content: None,
            test_id: None,
        }
    }
}
