use std::sync::Arc;

use fret_core::{Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::GlobalElementId;

use crate::primitives::popper::PopperContentPlacement;

pub(in crate::imui::tooltip_overlay) struct TooltipPanelBuildOptions {
    pub(in crate::imui::tooltip_overlay) trigger_id: GlobalElementId,
    pub(in crate::imui::tooltip_overlay) trigger_rect: Option<Rect>,
    pub(in crate::imui::tooltip_overlay) panel_size: Size,
    pub(in crate::imui::tooltip_overlay) placement: PopperContentPlacement,
    pub(in crate::imui::tooltip_overlay) window_margin: Px,
    pub(in crate::imui::tooltip_overlay) panel_id_model: Model<Option<GlobalElementId>>,
    pub(in crate::imui::tooltip_overlay) panel_test_id: Option<Arc<str>>,
}
