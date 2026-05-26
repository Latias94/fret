use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_ui::action::{DismissReason, DismissRequestCx, OnDismissRequest};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle,
};
use fret_ui::{GlobalElementId, UiHost};

use super::{ImUiFacade, PopupModalOptions, UiWriterImUiFacadeExt};
use crate::primitives::dialog;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

pub(super) fn begin_popup_modal_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupModalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    ui.with_cx_mut(|cx| {
        let open = super::super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
        let is_open = cx
            .read_model(&open, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);
        if !is_open {
            return false;
        }

        let keep_alive_generation = super::super::popup_render_generation_for_window(cx);
        super::super::with_popup_store_for_id(cx, id, move |st, _app| {
            st.keep_alive_generation = Some(keep_alive_generation);
        });

        let overlay_key = format!("fret-ui-kit.imui.popup_modal.overlay.{id}");
        let overlay_id = cx.named(overlay_key.as_str(), |cx| cx.root_id());

        let root_name = OverlayController::modal_root_name(overlay_id);

        let (popover, border) = {
            let theme = fret_ui::Theme::global(&*cx.app);
            (theme.color_token("popover"), theme.color_token("border"))
        };

        let dim = Color {
            a: 0.4,
            ..Color::from_srgb_hex_rgb(0x00_00_00)
        };

        let size = options.size;
        let left =
            Px(cx.bounds.origin.x.0 + (cx.bounds.size.width.0 - size.width.0).max(0.0) * 0.5);
        let top =
            Px(cx.bounds.origin.y.0 + (cx.bounds.size.height.0 - size.height.0).max(0.0) * 0.5);

        let close_on_outside_press = options.close_on_outside_press;
        let open_for_dismiss = open.clone();
        let on_dismiss_request: OnDismissRequest = Arc::new(
            move |host, acx, req: &mut DismissRequestCx| match req.reason {
                DismissReason::Escape => {
                    let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
                    host.notify(acx);
                }
                DismissReason::OutsidePress { .. } if close_on_outside_press => {
                    let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
                    host.notify(acx);
                }
                _ => {
                    req.prevent_default();
                }
            },
        );

        let focus_state = Rc::new(Cell::new(None::<GlobalElementId>));
        let focus_state_for_build = focus_state.clone();
        let mut panel_id_for_focus: Option<GlobalElementId> = None;
        let mut build = Some(f);

        let layer = cx.with_root_name(root_name.as_str(), |cx| {
            cx.named("fret-ui-kit.imui.popup_modal.layer", |cx| {
                let mut stack = fret_ui::element::StackProps::default();
                stack.layout.position = PositionStyle::Absolute;
                stack.layout.inset = InsetStyle {
                    left: Some(Px(0.0)).into(),
                    right: Some(Px(0.0)).into(),
                    top: Some(Px(0.0)).into(),
                    bottom: Some(Px(0.0)).into(),
                };
                stack.layout.size.width = Length::Fill;
                stack.layout.size.height = Length::Fill;
                stack.layout.overflow = Overflow::Visible;

                cx.stack_props(stack, |cx| {
                    let backdrop_visual = cx.container(
                        {
                            let mut props = ContainerProps::default();
                            props.layout.position = PositionStyle::Absolute;
                            props.layout.inset = InsetStyle {
                                left: Some(Px(0.0)).into(),
                                right: Some(Px(0.0)).into(),
                                top: Some(Px(0.0)).into(),
                                bottom: Some(Px(0.0)).into(),
                            };
                            props.layout.size.width = Length::Fill;
                            props.layout.size.height = Length::Fill;
                            props.background = Some(dim);
                            props
                        },
                        |_cx| Vec::<AnyElement>::new(),
                    );
                    let backdrop = dialog::modal_barrier_with_dismiss_handler(
                        cx,
                        open.clone(),
                        close_on_outside_press,
                        Some(on_dismiss_request.clone()),
                        [backdrop_visual],
                    );

                    let panel = cx.named("fret-ui-kit.imui.popup_modal.panel", |cx| {
                        let mut semantics = fret_ui::element::SemanticsProps::default();
                        semantics.role = SemanticsRole::Dialog;
                        semantics.test_id = Some(Arc::from(format!("imui-popup-modal-{id}")));
                        semantics.layout = LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                left: Some(left).into(),
                                top: Some(top).into(),
                                ..Default::default()
                            },
                            size: fret_ui::element::SizeStyle {
                                width: Length::Px(size.width),
                                height: Length::Px(size.height),
                                ..Default::default()
                            },
                            ..Default::default()
                        };

                        let modal = cx.semantics_with_id(semantics, move |cx, _id| {
                            let mut panel_props = ContainerProps::default();
                            panel_props.background = Some(popover);
                            panel_props.border = Edges::all(Px(1.0));
                            panel_props.border_color = Some(border);
                            panel_props.corner_radii =
                                Corners::all(super::super::control_chrome::PANEL_RADIUS);
                            panel_props.padding = Edges::all(Px(8.0)).into();
                            panel_props.layout.size.width = Length::Fill;
                            panel_props.layout.size.height = Length::Fill;

                            vec![cx.container(panel_props, move |cx| {
                                let mut out: Vec<AnyElement> = Vec::new();
                                {
                                    let mut ui = ImUiFacade {
                                        cx,
                                        out: &mut out,
                                        build_focus: Some(focus_state_for_build.clone()),
                                    };
                                    if let Some(f) = build.take() {
                                        f(&mut ui);
                                    }
                                }
                                out
                            })]
                        });
                        panel_id_for_focus = Some(modal.id);
                        modal
                    });

                    vec![backdrop, panel]
                })
            })
        });

        let mut req = OverlayRequest::modal(
            overlay_id,
            trigger,
            open.clone(),
            OverlayPresence::instant(true),
            vec![layer],
        );
        req.root_name = Some(root_name);
        req.dismissible_on_dismiss_request = Some(on_dismiss_request);
        req.initial_focus = focus_state.get().or(panel_id_for_focus);
        OverlayController::request(cx, req);

        true
    })
}
