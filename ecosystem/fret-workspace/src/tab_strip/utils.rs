use std::sync::Arc;

use fret_core::{FontId, FontWeight, Point, Px, Rect, TextStyle};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{Theme};
use fret_ui_kit::dnd as ui_dnd;

use crate::tab_drag::WorkspaceTabHitRect;

pub(super) fn tab_text_style(theme: &Theme) -> TextStyle {
    let px = theme.metric_by_key("font.size").unwrap_or(Px(13.0));
    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::MEDIUM,
        slant: Default::default(),
        line_height: None,
        letter_spacing_em: None,
        ..Default::default()
    }
}

pub(super) fn scroll_rect_into_view_x(handle: &ScrollHandle, viewport: Rect, child: Rect) {
    let margin = Px(12.0);

    let current = handle.offset();
    let view_left = viewport.origin.x;
    let view_right = Px(viewport.origin.x.0 + viewport.size.width.0);
    let child_left = child.origin.x;
    let child_right = Px(child.origin.x.0 + child.size.width.0);

    let next_x = if child_left.0 < (view_left.0 + margin.0) {
        Px(current.x.0 + (child_left.0 - (view_left.0 + margin.0)))
    } else if child_right.0 > (view_right.0 - margin.0) {
        Px(current.x.0 + (child_right.0 - (view_right.0 - margin.0)))
    } else {
        current.x
    };

    if next_x != current.x {
        handle.set_offset(Point::new(next_x, current.y));
    }
}

fn fnv1a64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub(super) fn dnd_scope_for_pane(pane_id: Option<&Arc<str>>) -> ui_dnd::DndScopeId {
    pane_id
        .map(|id| ui_dnd::DndScopeId(fnv1a64(id.as_ref())))
        .unwrap_or(ui_dnd::DND_SCOPE_DEFAULT)
}

fn tab_pinned_flag_for_id(
    pinned_by_id: &std::collections::HashMap<Arc<str>, bool>,
    id: &str,
) -> bool {
    pinned_by_id.get(id).copied().unwrap_or(false)
}

pub(super) fn resolve_end_drop_target(
    pinned_by_id: &std::collections::HashMap<Arc<str>, bool>,
    rects: &[WorkspaceTabHitRect],
    dragged: &str,
) -> Option<Arc<str>> {
    let dragged_pinned = tab_pinned_flag_for_id(pinned_by_id, dragged);

    let mut best: Option<(Arc<str>, f32)> = None;
    for r in rects.iter().filter(|r| r.id.as_ref() != dragged) {
        if tab_pinned_flag_for_id(pinned_by_id, r.id.as_ref()) != dragged_pinned {
            continue;
        }
        let right = r.rect.origin.x.0 + r.rect.size.width.0;
        if best
            .as_ref()
            .map(|(_id, prev)| right > *prev)
            .unwrap_or(true)
        {
            best = Some((r.id.clone(), right));
        }
    }

    best.map(|(id, _)| id)
}

