use std::sync::Arc;

use fret_core::{Edges, PointerType, Px, SemanticsRole};
use fret_ui::action::DismissReason;
use fret_ui::element::{
    ColumnProps, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle,
    SemanticsDecoration, SpacingLength,
};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{ImUiFacade, ResponseExt, TooltipOptions, UiWriterImUiFacadeExt};
use crate::OverlayPresence;
use crate::declarative::ModelWatchExt;
use crate::declarative::scheduling;
use crate::overlay;
use crate::primitives::tooltip as radix_tooltip;

fn install_pointer_move_open_gate_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger: GlobalElementId,
    models: radix_tooltip::TooltipTriggerEventModels,
    pointer_in_transit_buffer: Px,
    last_pointer: fret_runtime::Model<Option<fret_core::Point>>,
) {
    cx.pressable_add_on_pointer_move_for(
        trigger,
        Arc::new(move |host, action_cx, mv| {
            if mv.pointer_type == PointerType::Touch {
                return false;
            }

            let _ = host
                .models_mut()
                .update(&last_pointer, |value| *value = Some(mv.position));

            let geometry = host
                .models_mut()
                .read(&models.pointer_transit_geometry, |value| *value)
                .ok()
                .flatten();
            if let Some((anchor, floating)) = geometry
                && radix_tooltip::tooltip_pointer_in_transit(
                    mv.position,
                    anchor,
                    floating,
                    pointer_in_transit_buffer,
                )
            {
                return false;
            }

            let already = host
                .models_mut()
                .read(&models.has_pointer_move_opened, |value| *value)
                .ok()
                .unwrap_or(false);
            if !already {
                let _ = host
                    .models_mut()
                    .update(&models.has_pointer_move_opened, |value| *value = true);
                host.request_redraw(action_cx.window);
            }

            false
        }),
    );
}

pub(super) fn tooltip_text_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    text: Arc<str>,
    options: TooltipOptions,
) -> bool {
    tooltip_with_options(ui, id, trigger, options, move |ui| {
        ui.text(text);
    })
}

pub(super) fn tooltip_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    options: TooltipOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    let Some(trigger_id) = trigger.id else {
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
                trigger.pointer_hovered_raw,
                trigger.core.focused,
                &event_models,
            );

            let anchor_bounds =
                overlay::anchor_bounds_for_element(cx, trigger_id).or(trigger.core.rect);
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
            let panel_test_id = options.test_id.clone();
            let placement = options.placement;
            let window_margin = options.window_margin;
            let panel_id_model = panel_id.clone();
            let mut build = Some(f);

            let overlay_children = cx.with_root_name(root_name.as_str(), |cx| {
                let Some(anchor) =
                    overlay::anchor_bounds_for_element(cx, trigger_id).or(trigger.core.rect)
                else {
                    return Vec::new();
                };

                let outer = overlay::outer_bounds_with_window_margin_for_environment(
                    cx,
                    fret_ui::Invalidation::Layout,
                    window_margin,
                );
                let layout = crate::primitives::popper::popper_content_layout_sized(
                    outer, anchor, panel_size, placement,
                );

                vec![cx.named("fret-ui-kit.imui.tooltip.panel", |cx| {
                    let current_panel_id = cx.root_id();
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&panel_id_model, |value| *value = Some(current_panel_id));

                    let mut panel_props = ContainerProps::default();
                    let theme = fret_ui::Theme::global(&*cx.app);
                    panel_props.layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            left: Some(layout.rect.origin.x).into(),
                            top: Some(layout.rect.origin.y).into(),
                            ..Default::default()
                        },
                        size: fret_ui::element::SizeStyle {
                            width: Length::Auto,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        overflow: Overflow::Visible,
                        ..Default::default()
                    };
                    panel_props.padding = Edges::all(Px(6.0)).into();
                    panel_props.background = Some(theme.color_token("popover"));
                    panel_props.border = Edges::all(Px(1.0));
                    panel_props.border_color = Some(theme.color_token("border"));
                    panel_props.corner_radii = fret_core::Corners::all(Px(6.0));

                    let mut panel = cx.container(panel_props, move |cx| {
                        let mut column = ColumnProps::default();
                        column.layout.size.width = Length::Auto;
                        column.layout.size.height = Length::Auto;
                        column.gap = SpacingLength::Px(Px(4.0));

                        vec![cx.column(column, move |cx| {
                            let mut out = Vec::new();
                            let mut ui = ImUiFacade {
                                cx,
                                out: &mut out,
                                build_focus: None,
                            };
                            if let Some(build) = build.take() {
                                build(&mut ui);
                            }
                            out
                        })]
                    });

                    let mut semantics = SemanticsDecoration::default().role(SemanticsRole::Tooltip);
                    if let Some(test_id) = panel_test_id.as_ref() {
                        semantics = semantics.test_id(test_id.clone());
                    }
                    panel = panel.attach_semantics(semantics);
                    panel
                })]
            });

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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_authoring::UiWriter;
    use fret_ui::element::AnyElement;

    struct TestWriter<'cx, 'a, H: UiHost> {
        cx: &'cx mut ElementContext<'a, H>,
        out: &'cx mut Vec<AnyElement>,
    }

    impl<'cx, 'a, H: UiHost> UiWriter<H> for TestWriter<'cx, 'a, H> {
        fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
            f(self.cx)
        }

        fn add(&mut self, element: AnyElement) {
            self.out.push(element);
        }
    }

    #[test]
    fn tooltip_returns_false_without_trigger_id() {
        let mut app = App::new();
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            |cx| {
                let mut out = Vec::new();
                let mut ui = TestWriter { cx, out: &mut out };
                assert!(!tooltip_text_with_options(
                    &mut ui,
                    "tooltip",
                    ResponseExt::default(),
                    Arc::from("tip"),
                    TooltipOptions::default(),
                ));
                assert!(out.is_empty());
            },
        );
    }

    #[test]
    fn tooltip_default_options_use_top_center_placement() {
        let options = TooltipOptions::default();
        assert_eq!(options.placement.side, crate::primitives::popper::Side::Top);
        assert_eq!(
            options.placement.align,
            crate::primitives::popper::Align::Center
        );
        assert_eq!(options.window_margin, Px(8.0));
        assert_eq!(options.open_delay_frames_override, None);
        assert_eq!(options.close_delay_frames_override, None);
        assert!(options.test_id.is_none());
    }
}
