use fret_core::Axis;
use fret_ui::element::{CrossAlign, FlexProps, Length, MainAlign};

use super::{ROW_GAP, STACK_GAP};

pub(in crate::imui) fn fill_row_props(justify: MainAlign) -> FlexProps {
    let mut props = FlexProps::default();
    props.direction = Axis::Horizontal;
    props.layout.size.width = Length::Fill;
    props.gap = ROW_GAP.into();
    props.justify = justify;
    props.align = CrossAlign::Center;
    props
}

pub(in crate::imui) fn centered_row_props() -> FlexProps {
    let mut props = FlexProps::default();
    props.direction = Axis::Horizontal;
    props.gap = ROW_GAP.into();
    props.justify = MainAlign::Center;
    props.align = CrossAlign::Center;
    props
}

pub(in crate::imui) fn fill_stack_props() -> FlexProps {
    let mut props = FlexProps::default();
    props.direction = Axis::Vertical;
    props.layout.size.width = Length::Fill;
    props.gap = STACK_GAP.into();
    props.align = CrossAlign::Stretch;
    props
}
