use super::*;

pub(crate) fn largest_semantics_node<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    role: SemanticsRole,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes.iter().filter(|n| n.role == role).max_by(|a, b| {
        let a_area = a.bounds.size.width.0 * a.bounds.size.height.0;
        let b_area = b.bounds.size.width.0 * b.bounds.size.height.0;
        a_area
            .partial_cmp(&b_area)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

pub(crate) fn fret_find_active_listbox_option<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
) -> Option<&'a fret_core::SemanticsNode> {
    if let Some(focused) = snap
        .nodes
        .iter()
        .find(|n| n.flags.focused && n.role == SemanticsRole::ListBoxOption)
    {
        return Some(focused);
    }

    for owner in snap.nodes.iter().filter(|n| n.active_descendant.is_some()) {
        let active_id = owner.active_descendant?;
        let target = snap.nodes.iter().find(|n| n.id == active_id)?;
        if target.role == SemanticsRole::ListBoxOption {
            return Some(target);
        }
    }

    None
}

pub(crate) fn fret_find_topmost_menu_item_in_menu<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    menu_bounds: Rect,
) -> Option<&'a fret_core::SemanticsNode> {
    snap.nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::MenuItem)
        .filter(|n| rect_intersection_area(n.bounds, menu_bounds) > 0.01)
        .min_by(|a, b| {
            let ay = a.bounds.origin.y.0;
            let by = b.bounds.origin.y.0;
            let ax = a.bounds.origin.x.0;
            let bx = b.bounds.origin.x.0;
            ay.total_cmp(&by).then_with(|| ax.total_cmp(&bx))
        })
}

pub(crate) fn fret_find_active_menu_item<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
) -> Option<&'a fret_core::SemanticsNode> {
    if let Some(focused) = snap
        .nodes
        .iter()
        .find(|n| n.flags.focused && n.role == SemanticsRole::MenuItem)
    {
        return Some(focused);
    }

    for owner in snap.nodes.iter().filter(|n| n.active_descendant.is_some()) {
        let active_id = owner.active_descendant?;
        let target = snap.nodes.iter().find(|n| n.id == active_id)?;
        if target.role == SemanticsRole::MenuItem {
            return Some(target);
        }
    }

    None
}
