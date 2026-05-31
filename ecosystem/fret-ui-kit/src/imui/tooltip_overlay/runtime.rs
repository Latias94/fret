use fret_ui::UiHost;

use crate::imui::{ImUiFacade, ResponseExt, TooltipOptions, UiWriterImUiFacadeExt};
use crate::primitives::tooltip as radix_tooltip;

use super::request::{TooltipOverlayRequestModels, request_tooltip_overlay};

mod interaction;
mod layout;
mod models;

use interaction::update_tooltip_runtime_interaction;
use layout::resolve_tooltip_runtime_layout;
use models::prepare_tooltip_runtime_models;

pub(in crate::imui) fn tooltip_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    options: TooltipOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    let Some(trigger_id) = trigger.id() else {
        return false;
    };

    ui.with_cx_mut(|cx| {
        let overlay_key = format!("fret-ui-kit.imui.tooltip.overlay.{id}");
        cx.named(overlay_key.as_str(), |cx| {
            let tooltip_id = cx.root_id();
            let models = prepare_tooltip_runtime_models(cx, trigger_id);

            let provider_cfg = radix_tooltip::current_config(cx);
            let disable_hoverable_content = options
                .disable_hoverable_content
                .unwrap_or(provider_cfg.disable_hoverable_content);

            let tooltip_layout = resolve_tooltip_runtime_layout(
                cx,
                trigger_id,
                trigger.rect(),
                &models.panel_id,
                &options,
            );

            let open = update_tooltip_runtime_interaction(
                cx,
                trigger.pointer_hovered_raw(),
                trigger.focused(),
                tooltip_layout.anchor_bounds,
                tooltip_layout.floating_bounds,
                &models.open,
                &models.event_models,
                models.last_pointer.clone(),
                &options,
                disable_hoverable_content,
            );
            if !open {
                return false;
            }

            request_tooltip_overlay(
                cx,
                tooltip_id,
                trigger_id,
                trigger.rect(),
                tooltip_layout.panel_size,
                disable_hoverable_content,
                &options,
                TooltipOverlayRequestModels {
                    open: models.open,
                    panel_id: models.panel_id,
                    event_models: models.event_models,
                    last_pointer: models.last_pointer,
                },
                f,
            );

            true
        })
    })
}
