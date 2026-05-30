use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::layout::{self, PopupModalPalette, PopupModalPanelLayout};
use crate::imui::popup_overlay::ImUiFacade;

pub(super) struct PopupModalPanelInput<'a, Build> {
    pub(super) id: &'a str,
    pub(super) palette: &'a PopupModalPalette,
    pub(super) panel_layout: PopupModalPanelLayout,
    pub(super) focus_state_for_build: Rc<Cell<Option<GlobalElementId>>>,
    pub(super) build: Build,
}

pub(super) struct PopupModalPanelBuilt {
    pub(super) element: AnyElement,
    pub(super) panel_id_for_focus: Option<GlobalElementId>,
}

pub(super) fn popup_modal_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: PopupModalPanelInput<'_, impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)>,
) -> PopupModalPanelBuilt {
    let PopupModalPanelInput {
        id,
        palette,
        panel_layout,
        focus_state_for_build,
        build,
    } = input;

    let mut build = Some(build);
    let mut panel_id_for_focus = None;
    let element = cx.named("fret-ui-kit.imui.popup_modal.panel", |cx| {
        let semantics = layout::modal_panel_semantics(id, panel_layout);
        let panel_props = layout::modal_panel_props(palette);
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

    PopupModalPanelBuilt {
        element,
        panel_id_for_focus,
    }
}
