use std::cell::Cell;
use std::rc::Rc;

use fret_runtime::Model;
use fret_ui::action::OnDismissRequest;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::layout::{self, PopupModalPalette, PopupModalPanelLayout};
use crate::imui::popup_overlay::ImUiFacade;
use crate::primitives::dialog;

pub(super) struct PopupModalLayerInput<'a, Build> {
    pub(super) id: &'a str,
    pub(super) root_name: &'a str,
    pub(super) open: Model<bool>,
    pub(super) palette: PopupModalPalette,
    pub(super) panel_layout: PopupModalPanelLayout,
    pub(super) close_on_outside_press: bool,
    pub(super) on_dismiss_request: OnDismissRequest,
    pub(super) focus_state_for_build: Rc<Cell<Option<GlobalElementId>>>,
    pub(super) build: Build,
}

pub(super) struct PopupModalLayerBuilt {
    pub(super) layer: AnyElement,
    pub(super) panel_id_for_focus: Option<GlobalElementId>,
}

pub(super) fn modal_focus_state() -> Rc<Cell<Option<GlobalElementId>>> {
    Rc::new(Cell::new(None::<GlobalElementId>))
}

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

    PopupModalLayerBuilt {
        layer,
        panel_id_for_focus,
    }
}
