use fret_core::{Corners, Edges, Px};
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

use super::super::super::control_chrome;

pub(super) fn input_padding() -> Edges {
    Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(3.0),
        bottom: Px(3.0),
    }
}

pub(super) fn input_border() -> Edges {
    Edges::all(Px(1.0))
}

pub(super) fn input_corner_radii() -> Corners {
    Corners::all(control_chrome::CONTROL_RADIUS)
}

pub(super) fn input_text_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Px(control_chrome::FIELD_MIN_HEIGHT),
            min_height: Some(Length::Px(control_chrome::FIELD_MIN_HEIGHT)),
            max_height: Some(Length::Px(control_chrome::FIELD_MIN_HEIGHT)),
            ..Default::default()
        },
        ..Default::default()
    }
}
