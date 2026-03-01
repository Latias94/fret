use std::sync::Arc;

use fret_core::{FontId, FontWeight, Point, Px, Rect, TextStyle};
use fret_ui::Theme;
use fret_ui::scroll::ScrollHandle;
use fret_ui_kit::dnd as ui_dnd;

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

pub(super) fn resolve_end_drop_target_in_canonical_order(
    pinned_by_id: &std::collections::HashMap<Arc<str>, bool>,
    canonical_order: &[Arc<str>],
    dragged: &str,
) -> Option<Arc<str>> {
    let dragged_pinned = tab_pinned_flag_for_id(pinned_by_id, dragged);

    let mut best: Option<Arc<str>> = None;
    for id in canonical_order.iter().filter(|id| id.as_ref() != dragged) {
        if tab_pinned_flag_for_id(pinned_by_id, id.as_ref()) != dragged_pinned {
            continue;
        }
        best = Some(id.clone());
    }

    best
}

pub(super) fn predict_next_active_tab_after_close(
    active: &Arc<str>,
    canonical_order: &[Arc<str>],
    mru: Option<&[Arc<str>]>,
) -> Option<Arc<str>> {
    if let Some(mru) = mru {
        for id in mru {
            if id.as_ref() == active.as_ref() {
                continue;
            }
            if canonical_order.iter().any(|t| t.as_ref() == id.as_ref()) {
                return Some(id.clone());
            }
        }
    }

    canonical_order
        .iter()
        .find(|t| t.as_ref() != active.as_ref())
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arc(s: &str) -> Arc<str> {
        Arc::<str>::from(s)
    }

    #[test]
    fn end_drop_target_uses_canonical_order_and_respects_pinned_group() {
        let canonical: Vec<Arc<str>> = vec![arc("a"), arc("b"), arc("c"), arc("d")];
        let pinned_by_id: std::collections::HashMap<Arc<str>, bool> = [
            (arc("a"), true),
            (arc("b"), true),
            (arc("c"), false),
            (arc("d"), false),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            resolve_end_drop_target_in_canonical_order(&pinned_by_id, &canonical, "a").as_deref(),
            Some("b")
        );
        assert_eq!(
            resolve_end_drop_target_in_canonical_order(&pinned_by_id, &canonical, "c").as_deref(),
            Some("d")
        );
    }

    #[test]
    fn end_drop_target_returns_none_when_dragged_is_only_member_of_group() {
        let canonical: Vec<Arc<str>> = vec![arc("only"), arc("other")];
        let pinned_by_id: std::collections::HashMap<Arc<str>, bool> =
            [(arc("only"), true), (arc("other"), false)]
                .into_iter()
                .collect();

        assert_eq!(
            resolve_end_drop_target_in_canonical_order(&pinned_by_id, &canonical, "only"),
            None
        );
    }

    #[test]
    fn predict_next_active_tab_after_close_prefers_mru_fallback() {
        let canonical: Vec<Arc<str>> = vec![arc("a"), arc("b"), arc("c")];
        let mru: Vec<Arc<str>> = vec![arc("a"), arc("c"), arc("b")];

        assert_eq!(
            predict_next_active_tab_after_close(&arc("a"), &canonical, Some(&mru)).as_deref(),
            Some("c")
        );
    }

    #[test]
    fn predict_next_active_tab_after_close_falls_back_to_tab_order() {
        let canonical: Vec<Arc<str>> = vec![arc("a"), arc("b"), arc("c")];
        let mru: Vec<Arc<str>> = vec![arc("a")];

        assert_eq!(
            predict_next_active_tab_after_close(&arc("a"), &canonical, Some(&mru)).as_deref(),
            Some("b")
        );
    }
}
