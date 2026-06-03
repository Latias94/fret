use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod backdrop;
mod focus;
mod panel;
mod types;

pub(super) use focus::modal_focus_state;
pub(super) use types::{PopupModalLayerBuilt, PopupModalLayerInput};

use super::layout;
use crate::imui::popup_overlay::ImUiFacade;

pub(super) fn build_popup_modal_layer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: PopupModalLayerInput<'_, impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)>,
) -> PopupModalLayerBuilt {
    let PopupModalLayerInput {
        id,
        root_name,
        open,
        palette,
        panel_layout,
        close_on_outside_press,
        on_dismiss_request,
        focus_state_for_build,
        build,
    } = input;

    let mut panel_id_for_focus: Option<GlobalElementId> = None;
    let mut build = Some(build);

    let layer = cx.with_root_name(root_name, |cx| {
        cx.named("fret-ui-kit.imui.popup_modal.layer", |cx| {
            cx.stack_props(layout::modal_layer_stack_props(), |cx| {
                let backdrop = backdrop::popup_modal_backdrop(
                    cx,
                    open.clone(),
                    palette.dim,
                    close_on_outside_press,
                    on_dismiss_request.clone(),
                );
                let panel = panel::popup_modal_panel(
                    cx,
                    panel::PopupModalPanelInput {
                        id,
                        palette: &palette,
                        panel_layout,
                        focus_state_for_build: focus_state_for_build.clone(),
                        build: build
                            .take()
                            .expect("popup modal body builder should be available"),
                    },
                );
                panel_id_for_focus = panel.panel_id_for_focus;

                vec![backdrop, panel.element]
            })
        })
    });

    PopupModalLayerBuilt {
        layer,
        panel_id_for_focus,
    }
}
