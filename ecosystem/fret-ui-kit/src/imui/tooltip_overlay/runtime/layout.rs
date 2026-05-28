use fret_core::{Rect, Size};
use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::declarative::ModelWatchExt;
use crate::imui::TooltipOptions;
use crate::overlay;

pub(super) struct TooltipRuntimeLayout {
    pub(super) anchor_bounds: Option<Rect>,
    pub(super) panel_size: Size,
    pub(super) floating_bounds: Option<Rect>,
}

pub(super) fn resolve_tooltip_runtime_layout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    trigger_rect: Option<Rect>,
    panel_id: &Model<Option<GlobalElementId>>,
    options: &TooltipOptions,
) -> TooltipRuntimeLayout {
    let anchor_bounds = overlay::anchor_bounds_for_element(cx, trigger_id).or(trigger_rect);
    let panel_size = cx
        .watch_model(panel_id)
        .layout()
        .copied()
        .unwrap_or(None)
        .and_then(|panel_id| cx.last_bounds_for_element(panel_id).map(|rect| rect.size))
        .unwrap_or(options.estimated_size);
    let floating_bounds = anchor_bounds.map(|anchor| {
        let outer = overlay::outer_bounds_with_window_margin_for_environment(
            cx,
            fret_ui::Invalidation::Layout,
            options.window_margin,
        );
        crate::primitives::popper::popper_content_layout_sized(
            outer,
            anchor,
            panel_size,
            options.placement,
        )
        .rect
    });

    TooltipRuntimeLayout {
        anchor_bounds,
        panel_size,
        floating_bounds,
    }
}
