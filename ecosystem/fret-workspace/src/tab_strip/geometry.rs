use std::sync::Arc;

use fret_core::Rect;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::tab_drag::WorkspaceTabHitRect;

pub(super) fn bounds_for_optional_element_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    element_id: Option<GlobalElementId>,
) -> Option<Rect> {
    element_id.and_then(|id| cx.last_bounds_for_element(id))
}

pub(super) fn collect_tab_hit_rects<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    elements: &[(Arc<str>, GlobalElementId)],
) -> Vec<WorkspaceTabHitRect> {
    let mut rects: Vec<WorkspaceTabHitRect> = Vec::new();
    for (id, el) in elements.iter() {
        if let Some(rect) = cx.last_bounds_for_element(*el) {
            rects.push(WorkspaceTabHitRect {
                id: id.clone(),
                rect,
            });
        }
    }
    rects
}

