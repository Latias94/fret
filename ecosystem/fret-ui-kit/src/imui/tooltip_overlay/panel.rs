use std::sync::Arc;

use fret_core::{Px, Rect, Size};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::ImUiFacade;
use crate::overlay;
use crate::primitives::popper::{self, PopperContentPlacement};
use element::{TooltipPanelElementInput, tooltip_panel_element};

mod element;
mod layout;

pub(super) struct TooltipPanelBuildOptions {
    pub(super) trigger_id: GlobalElementId,
    pub(super) trigger_rect: Option<Rect>,
    pub(super) panel_size: Size,
    pub(super) placement: PopperContentPlacement,
    pub(super) window_margin: Px,
    pub(super) panel_id_model: fret_runtime::Model<Option<GlobalElementId>>,
    pub(super) panel_test_id: Option<Arc<str>>,
}

pub(super) fn tooltip_overlay_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_name: &str,
    options: TooltipPanelBuildOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> Vec<AnyElement> {
    let mut build = Some(f);
    cx.with_root_name(root_name, |cx| {
        let Some(anchor) =
            overlay::anchor_bounds_for_element(cx, options.trigger_id).or(options.trigger_rect)
        else {
            return Vec::new();
        };

        let outer = overlay::outer_bounds_with_window_margin_for_environment(
            cx,
            fret_ui::Invalidation::Layout,
            options.window_margin,
        );
        let layout = popper::popper_content_layout_sized(
            outer,
            anchor,
            options.panel_size,
            options.placement,
        );

        let Some(build) = build.take() else {
            return Vec::new();
        };

        vec![tooltip_panel_element(
            cx,
            TooltipPanelElementInput {
                origin: layout.rect.origin,
                panel_id_model: options.panel_id_model,
                panel_test_id: options.panel_test_id,
            },
            build,
        )]
    })
}
