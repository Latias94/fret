use std::sync::Arc;

use fret_core::{Point, Rect, Size};
use fret_runtime::Model;
use fret_ui::action::DismissReason;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::OverlayPresence;
use crate::imui::{ImUiFacade, TooltipOptions};
use crate::primitives::tooltip as radix_tooltip;

use super::panel::{TooltipPanelBuildOptions, tooltip_overlay_children};

pub(super) struct TooltipOverlayRequestModels {
    pub(super) open: Model<bool>,
    pub(super) panel_id: Model<Option<GlobalElementId>>,
    pub(super) event_models: radix_tooltip::TooltipTriggerEventModels,
    pub(super) last_pointer: Model<Option<Point>>,
}

pub(super) fn request_tooltip_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    tooltip_id: GlobalElementId,
    trigger_id: GlobalElementId,
    trigger_rect: Option<Rect>,
    panel_size: Size,
    disable_hoverable_content: bool,
    options: &TooltipOptions,
    models: TooltipOverlayRequestModels,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) {
    let root_name = radix_tooltip::tooltip_root_name(tooltip_id);
    let overlay_children = tooltip_overlay_children(
        cx,
        root_name.as_str(),
        TooltipPanelBuildOptions {
            trigger_id,
            trigger_rect,
            panel_size,
            placement: options.placement.clone(),
            window_margin: options.window_margin,
            panel_id_model: models.panel_id,
            panel_test_id: options.test_id.clone(),
        },
        f,
    );

    let mut request = radix_tooltip::tooltip_request(
        tooltip_id,
        models.open,
        OverlayPresence::instant(true),
        overlay_children,
    );
    request.trigger = Some(trigger_id);
    request.dismissible_on_dismiss_request = Some(Arc::new({
        let close_requested = models.event_models.close_requested.clone();
        move |host, action_cx, req| match req.reason {
            DismissReason::Escape | DismissReason::OutsidePress { .. } => {
                let _ = host
                    .models_mut()
                    .update(&close_requested, |value| *value = true);
                host.request_redraw(action_cx.window);
            }
            _ => req.prevent_default(),
        }
    }));
    if !disable_hoverable_content {
        radix_tooltip::tooltip_install_pointer_move_tracker(&mut request, models.last_pointer);
    }
    radix_tooltip::request_tooltip(cx, request);
}
