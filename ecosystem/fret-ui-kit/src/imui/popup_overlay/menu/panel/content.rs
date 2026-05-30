use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::layout::{PopupMenuPanelPalette, popup_menu_panel_column_props, popup_menu_panel_props};
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::imui::popup_overlay::{ImUiFacade, ImUiPopupMenuPolicyState};

pub(super) fn popup_menu_panel_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    palette: PopupMenuPanelPalette,
    popup_policy: ImUiPopupMenuPolicyState,
    menubar_policy: Option<ImUiMenubarPolicyState>,
    build: &mut Option<impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)>,
) -> Vec<AnyElement> {
    vec![cx.container(popup_menu_panel_props(palette), move |cx| {
        vec![cx.column(popup_menu_panel_column_props(), move |cx| {
            let render_children = move |cx: &mut ElementContext<'_, H>| {
                let mut out: Vec<AnyElement> = Vec::new();
                let mut ui = ImUiFacade {
                    cx,
                    out: &mut out,
                    build_focus: None,
                };
                if let Some(build) = build.take() {
                    build(&mut ui);
                }
                out
            };
            if let Some(menubar_policy) = menubar_policy.clone() {
                cx.provide(menubar_policy, move |cx| {
                    cx.provide(popup_policy.clone(), render_children)
                })
            } else {
                cx.provide(popup_policy.clone(), render_children)
            }
        })]
    })]
}
