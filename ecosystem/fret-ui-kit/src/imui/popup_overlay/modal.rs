use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{GlobalElementId, UiHost};

use super::{ImUiFacade, PopupModalOptions, UiWriterImUiFacadeExt};
use crate::primitives::dialog;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

mod dismiss;
mod layout;

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

        let palette = layout::popup_modal_palette(fret_ui::Theme::global(&*cx.app));
        let panel_layout = layout::centered_panel_layout(cx.bounds, options.size);

        let close_on_outside_press = options.close_on_outside_press;
        let on_dismiss_request =
            dismiss::modal_dismiss_request(open.clone(), close_on_outside_press);

        let focus_state = Rc::new(Cell::new(None::<GlobalElementId>));
        let focus_state_for_build = focus_state.clone();
        let mut panel_id_for_focus: Option<GlobalElementId> = None;
        let mut build = Some(f);

        let layer = cx.with_root_name(root_name.as_str(), |cx| {
            cx.named("fret-ui-kit.imui.popup_modal.layer", |cx| {
                cx.stack_props(layout::modal_layer_stack_props(), |cx| {
                    let backdrop_visual = cx
                        .container(layout::modal_backdrop_props(palette.dim), |_cx| {
                            Vec::<AnyElement>::new()
                        });
                    let backdrop = dialog::modal_barrier_with_dismiss_handler(
                        cx,
                        open.clone(),
                        close_on_outside_press,
                        Some(on_dismiss_request.clone()),
                        [backdrop_visual],
                    );

                    let panel = cx.named("fret-ui-kit.imui.popup_modal.panel", |cx| {
                        let semantics = layout::modal_panel_semantics(id, panel_layout);
                        let panel_props = layout::modal_panel_props(&palette);
                        let modal = cx.semantics_with_id(semantics, move |cx, _id| {
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
