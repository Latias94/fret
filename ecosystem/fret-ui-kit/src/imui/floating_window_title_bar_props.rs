use fret_core::Px;
use fret_ui::element::{Length, RowProps, SpacingLength};

mod close_button;
mod drag_surface;

pub(super) use close_button::title_bar_close_button_props;
pub(super) use drag_surface::title_bar_drag_surface_props;

pub(super) fn title_bar_row_props(resizable_layout: bool) -> RowProps {
    let mut row = RowProps::default();
    row.layout.size.width = if resizable_layout {
        Length::Fill
    } else {
        Length::Auto
    };
    row.layout.size.height = Length::Fill;
    row.gap = SpacingLength::Px(Px(4.0));
    row.align = fret_ui::element::CrossAlign::Center;
    row
}
