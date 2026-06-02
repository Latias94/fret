use fret_ui::element::AnyElement;
use fret_ui::{GlobalElementId, UiHost};

use super::policy::ImUiPopupMenuPolicyState;
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::imui::{ImUiFacade, PopupMenuOptions, UiWriterImUiFacadeExt};

mod assembly;
mod content;
mod layout;
mod state;

pub(super) struct PopupMenuBuilt {
    pub(super) children: Vec<AnyElement>,
    pub(super) first_item: Option<GlobalElementId>,
    pub(super) content_focus: Option<GlobalElementId>,
}

pub(super) fn build_popup_menu<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    root_name: &str,
    options: PopupMenuOptions,
    popup_policy: ImUiPopupMenuPolicyState,
    menubar_policy: Option<ImUiMenubarPolicyState>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> Option<PopupMenuBuilt> {
    ui.with_cx_mut(|cx| {
        let Some(panel_state) =
            state::prepare_popup_menu_panel_state(cx, id, options.estimated_size)
        else {
            return None;
        };
        let layout = layout::popup_menu_panel_layout(
            cx,
            panel_state.anchor,
            panel_state.desired,
            options.placement,
        );
        let palette = layout::popup_menu_panel_palette(fret_ui::Theme::global(&*cx.app));
        let mut build = Some(f);
        Some(assembly::assemble_popup_menu_panel(
            cx,
            id,
            root_name,
            layout.rect.origin,
            palette,
            popup_policy,
            menubar_policy,
            &mut build,
        ))
    })
}
