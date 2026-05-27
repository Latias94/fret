use fret_core::Px;

use crate::style::MetricFallback;

pub const IMUI_ITEM_SPACING_X_TOKEN: &str = "component.imui.item_spacing_x_px";
pub const IMUI_ITEM_SPACING_Y_TOKEN: &str = "component.imui.item_spacing_y_px";
pub const IMUI_INDENT_SPACING_TOKEN: &str = "component.imui.indent_spacing_px";

pub(crate) fn imui_item_spacing_x() -> crate::MetricRef {
    crate::MetricRef::Token {
        key: IMUI_ITEM_SPACING_X_TOKEN,
        fallback: MetricFallback::Px(Px(8.0)),
    }
}

pub(crate) fn imui_item_spacing_y() -> crate::MetricRef {
    crate::MetricRef::Token {
        key: IMUI_ITEM_SPACING_Y_TOKEN,
        fallback: MetricFallback::Px(Px(4.0)),
    }
}

pub(crate) fn imui_indent_spacing() -> crate::MetricRef {
    crate::MetricRef::Token {
        key: IMUI_INDENT_SPACING_TOKEN,
        fallback: MetricFallback::Px(Px(21.0)),
    }
}
