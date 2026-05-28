use fret_core::{Px, Size};

use crate::primitives::popper;

#[derive(Debug, Clone, Copy)]
pub struct PopupMenuOptions {
    pub placement: popper::PopperContentPlacement,
    pub estimated_size: Size,
    pub modal: bool,
    pub auto_focus: bool,
}

impl Default for PopupMenuOptions {
    fn default() -> Self {
        Self {
            placement: popper::PopperContentPlacement::new(
                popper::LayoutDirection::Ltr,
                popper::Side::Bottom,
                popper::Align::Start,
                Px(4.0),
            ),
            estimated_size: Size::new(Px(160.0), Px(120.0)),
            modal: true,
            auto_focus: true,
        }
    }
}
