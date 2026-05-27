use std::sync::Arc;

use fret_core::Px;
use fret_ui::action::DismissReason;
use fret_ui::{GlobalElementId, UiHost};

use crate::OverlayPresence;
use crate::declarative::ModelWatchExt;
use crate::declarative::scheduling;
use crate::imui::{ImUiFacade, ResponseExt, TooltipOptions, UiWriterImUiFacadeExt};
use crate::overlay;
use crate::primitives::tooltip as radix_tooltip;

use super::panel::{TooltipPanelBuildOptions, tooltip_overlay_children};
use super::trigger::install_pointer_move_open_gate_for;

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
            let open = cx.local_model_keyed("open", || false);
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
            let gates = radix_tooltip::tooltip_trigger_update_gates(
                cx,
                trigger.pointer_hovered_raw(),
                trigger.focused(),
                &event_models,
            );

            let anchor_bounds =
                overlay::anchor_bounds_for_element(cx, trigger_id).or(trigger.rect());
            let panel_size = cx
                .watch_model(&panel_id)
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

            let update = radix_tooltip::tooltip_update_interaction(
                cx,
                gates.trigger_hovered,
                gates.trigger_focused,
                gates.force_close,
                last_pointer.clone(),
                anchor_bounds,
                floating_bounds,
                radix_tooltip::TooltipInteractionConfig {
                    disable_hoverable_content,
                    open_delay_ticks_override: options.open_delay_frames_override.map(u64::from),
                    close_delay_ticks_override: options.close_delay_frames_override.map(u64::from),
                    safe_hover_buffer: Px(5.0),
                },
            );
            scheduling::set_continuous_frames(cx, update.wants_continuous_ticks);

            let open_now = cx.watch_model(&open).layout().copied().unwrap_or(false);
            if open_now != update.open {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&open, |value| *value = update.open);
            }

            if !update.open {
                return false;
            }

            let root_name = radix_tooltip::tooltip_root_name(tooltip_id);
            let overlay_children = tooltip_overlay_children(
                cx,
                root_name.as_str(),
                TooltipPanelBuildOptions {
                    trigger_id,
                    trigger_rect: trigger.rect(),
                    panel_size,
                    placement: options.placement,
                    window_margin: options.window_margin,
                    panel_id_model: panel_id.clone(),
                    panel_test_id: options.test_id.clone(),
                },
                f,
            );

            let mut request = radix_tooltip::tooltip_request(
                tooltip_id,
                open.clone(),
                OverlayPresence::instant(true),
                overlay_children,
            );
            request.trigger = Some(trigger_id);
            request.dismissible_on_dismiss_request = Some(Arc::new({
                let close_requested = event_models.close_requested.clone();
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
                radix_tooltip::tooltip_install_pointer_move_tracker(&mut request, last_pointer);
            }
            radix_tooltip::request_tooltip(cx, request);

            true
        })
    })
}
