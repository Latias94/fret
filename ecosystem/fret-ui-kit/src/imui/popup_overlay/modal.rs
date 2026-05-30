use fret_ui::{GlobalElementId, UiHost};

use super::{ImUiFacade, PopupModalOptions, UiWriterImUiFacadeExt};

mod dismiss;
mod layer;
mod layout;
mod request;
mod state;

pub(super) fn begin_popup_modal_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupModalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    ui.with_cx_mut(|cx| {
        let open = state::popup_modal_open_model(cx, id);
        if !state::popup_modal_is_open(cx, &open) {
            return false;
        }

        state::refresh_popup_modal_keep_alive(cx, id);

        let target = request::popup_modal_overlay_target(cx, id);

        let palette = layout::popup_modal_palette(fret_ui::Theme::global(&*cx.app));
        let panel_layout = layout::centered_panel_layout(cx.bounds, options.size);

        let close_on_outside_press = options.close_on_outside_press;
        let on_dismiss_request =
            dismiss::modal_dismiss_request(open.clone(), close_on_outside_press);

        let focus_state = layer::modal_focus_state();
        let focus_state_for_build = focus_state.clone();

        let built_layer = layer::build_popup_modal_layer(
            cx,
            layer::PopupModalLayerInput {
                id,
                root_name: target.root_name.as_str(),
                open: open.clone(),
                palette,
                panel_layout,
                close_on_outside_press,
                on_dismiss_request: on_dismiss_request.clone(),
                focus_state_for_build,
                build: f,
            },
        );

        request::request_popup_modal_overlay(
            cx,
            request::PopupModalOverlayRequestInput {
                target,
                trigger,
                open: open.clone(),
                on_dismiss_request,
                layer: built_layer.layer,
                initial_focus: focus_state.get().or(built_layer.panel_id_for_focus),
            },
        );

        true
    })
}
