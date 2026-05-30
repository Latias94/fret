use fret_core::Px;
use fret_ui::{GlobalElementId, UiHost};

use crate::imui::{ImUiFacade, ResponseExt, TooltipOptions, UiWriterImUiFacadeExt};
use crate::primitives::tooltip as radix_tooltip;

use super::request::{TooltipOverlayRequestModels, request_tooltip_overlay};
use super::trigger::install_pointer_move_open_gate_for;

mod interaction;
mod layout;

use interaction::update_tooltip_runtime_interaction;
use layout::resolve_tooltip_runtime_layout;

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
            let open_model = cx.local_model_keyed("open", || false);
            let panel_id = cx.local_model_keyed("panel_id", || None::<GlobalElementId>);
            let event_models = radix_tooltip::tooltip_trigger_event_models(cx);
            let last_pointer = radix_tooltip::tooltip_last_pointer_model(cx);

            radix_tooltip::tooltip_install_default_trigger_dismiss_handlers(
                cx,
                trigger_id,
                event_models.clone(),
            );
            install_pointer_move_open_gate_for(
                cx,
                trigger_id,
                event_models.clone(),
                Px(5.0),
                last_pointer.clone(),
            );

            let provider_cfg = radix_tooltip::current_config(cx);
            let disable_hoverable_content = options
                .disable_hoverable_content
                .unwrap_or(provider_cfg.disable_hoverable_content);

            let tooltip_layout =
                resolve_tooltip_runtime_layout(cx, trigger_id, trigger.rect(), &panel_id, &options);

            let open = update_tooltip_runtime_interaction(
                cx,
                trigger.pointer_hovered_raw(),
                trigger.focused(),
                tooltip_layout.anchor_bounds,
                tooltip_layout.floating_bounds,
                &open_model,
                &event_models,
                last_pointer.clone(),
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
                    open: open_model,
                    panel_id,
                    event_models,
                    last_pointer,
                },
                f,
            );

            true
        })
    })
}
