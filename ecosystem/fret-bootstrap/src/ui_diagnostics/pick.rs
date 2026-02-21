use fret_app::App;
use fret_core::Point;
use fret_ui::UiTree;

pub(super) fn pick_best_match<'a>(
    nodes: impl Iterator<Item = &'a fret_core::SemanticsNode>,
    index: &super::SemanticsIndex<'a>,
) -> Option<&'a fret_core::SemanticsNode> {
    let mut best: Option<(&'a fret_core::SemanticsNode, (u32, u32, u64))> = None;
    for n in nodes {
        let id = n.id.data().as_ffi();
        let rank = (index.root_z_for(id), index.depth_for(id), id);
        match best {
            None => best = Some((n, rank)),
            Some((_, best_rank)) if rank > best_rank => best = Some((n, rank)),
            _ => {}
        }
    }
    best.map(|(n, _)| n)
}

pub(super) fn pick_semantics_node_at<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    ui: &UiTree<App>,
    position: Point,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = super::SemanticsIndex::new(snapshot);

    let hit = ui.debug_hit_test(position).hit;
    if let Some(hit) = hit {
        let mut cur = Some(hit.data().as_ffi());
        while let Some(id) = cur {
            if index.is_selectable(id)
                && let Some(node) = index.by_id.get(&id).copied()
            {
                return Some(node);
            }
            cur = index
                .by_id
                .get(&id)
                .and_then(|n| n.parent.map(|p| p.data().as_ffi()));
        }
    }

    pick_semantics_node_by_bounds(snapshot, position)
}

pub(super) fn pick_semantics_node_at_routing<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    ui: &mut UiTree<App>,
    position: Point,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = super::SemanticsIndex::new(snapshot);

    let hit = ui.debug_hit_test_routing(position).hit;
    if let Some(hit) = hit {
        let mut cur = Some(hit.data().as_ffi());
        while let Some(id) = cur {
            if index.is_selectable(id)
                && let Some(node) = index.by_id.get(&id).copied()
            {
                return Some(node);
            }
            cur = index
                .by_id
                .get(&id)
                .and_then(|n| n.parent.map(|p| p.data().as_ffi()));
        }
    }

    pick_semantics_node_by_bounds(snapshot, position)
}

pub(crate) fn pick_semantics_node_by_bounds<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    position: Point,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = super::SemanticsIndex::new(snapshot);
    pick_best_match(
        snapshot.nodes.iter().filter(|n| {
            let id = n.id.data().as_ffi();
            index.is_selectable(id) && n.bounds.contains(position)
        }),
        &index,
    )
}
