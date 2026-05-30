use fret_core::Px;
use fret_ui::element::{Length, PointerRegionProps};

use super::visual::TABLE_RESIZE_HANDLE_MIN_HEIGHT;

const TABLE_RESIZE_HANDLE_HIT_WIDTH: Px = Px(12.0);

pub(super) fn table_resize_handle_props(enabled: bool) -> PointerRegionProps {
    let mut props = PointerRegionProps::default();
    props.enabled = enabled;
    props.layout.size.width = Length::Px(TABLE_RESIZE_HANDLE_HIT_WIDTH);
    props.layout.size.height = Length::Auto;
    props.layout.size.min_height = Some(Length::Px(TABLE_RESIZE_HANDLE_MIN_HEIGHT));
    props.layout.flex.shrink = 0.0;
    props
}
