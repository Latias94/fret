use std::cell::Cell;
use std::rc::Rc;

use fret_runtime::Model;
use fret_ui::GlobalElementId;
use fret_ui::action::OnDismissRequest;
use fret_ui::element::AnyElement;

use super::super::layout::{PopupModalPalette, PopupModalPanelLayout};

pub(in crate::imui::popup_overlay::modal) struct PopupModalLayerInput<'a, Build> {
    pub(in crate::imui::popup_overlay::modal) id: &'a str,
    pub(in crate::imui::popup_overlay::modal) root_name: &'a str,
    pub(in crate::imui::popup_overlay::modal) open: Model<bool>,
    pub(in crate::imui::popup_overlay::modal) palette: PopupModalPalette,
    pub(in crate::imui::popup_overlay::modal) panel_layout: PopupModalPanelLayout,
    pub(in crate::imui::popup_overlay::modal) close_on_outside_press: bool,
    pub(in crate::imui::popup_overlay::modal) on_dismiss_request: OnDismissRequest,
    pub(in crate::imui::popup_overlay::modal) focus_state_for_build:
        Rc<Cell<Option<GlobalElementId>>>,
    pub(in crate::imui::popup_overlay::modal) build: Build,
}

pub(in crate::imui::popup_overlay::modal) struct PopupModalLayerBuilt {
    pub(in crate::imui::popup_overlay::modal) layer: AnyElement,
    pub(in crate::imui::popup_overlay::modal) panel_id_for_focus: Option<GlobalElementId>,
}
