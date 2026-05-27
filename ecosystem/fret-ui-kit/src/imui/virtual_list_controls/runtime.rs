//! Runtime option projection for immediate virtual lists.

use fret_ui::element::{
    LayoutStyle, Length, Overflow, VirtualListMeasureMode,
    VirtualListOptions as RuntimeVirtualListOptions,
};

use super::VirtualListOptions;

pub(super) fn runtime_options(
    options: &VirtualListOptions,
    measure_mode: VirtualListMeasureMode,
) -> RuntimeVirtualListOptions {
    let mut runtime = RuntimeVirtualListOptions::new(options.estimate_row_height, options.overscan);
    runtime.items_revision = options.items_revision;
    runtime.measure_mode = measure_mode;
    runtime.key_cache = options.key_cache;
    runtime.keep_alive = options.keep_alive;
    runtime.gap = options.gap;
    runtime.scroll_margin = options.scroll_margin;
    runtime.known_row_height_at = options.known_row_height_at.clone();
    runtime
}

pub(super) fn resolved_measure_mode(options: &VirtualListOptions) -> VirtualListMeasureMode {
    if matches!(options.measure_mode, VirtualListMeasureMode::Known)
        && options.known_row_height_at.is_none()
    {
        VirtualListMeasureMode::Measured
    } else {
        options.measure_mode
    }
}

pub(super) fn list_layout(options: &VirtualListOptions) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(options.viewport_height);
    layout.overflow = Overflow::Clip;
    layout
}
