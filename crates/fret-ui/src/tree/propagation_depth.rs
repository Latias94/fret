use super::*;

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct PropagationDepthCacheEntry {
    pub(super) generation: u32,
    pub(super) depth: u32,
}

pub(super) fn propagation_depth_for<H: UiHost>(tree: &mut UiTree<H>, start: NodeId) -> u32 {
    let generation = tree.propagation_depth_generation;
    if let Some(entry) = tree.propagation_depth_cache.get(start)
        && entry.generation == generation
    {
        return entry.depth;
    }

    tree.propagation_chain.clear();

    let mut current = Some(start);
    while let Some(node) = current {
        if let Some(entry) = tree.propagation_depth_cache.get(node)
            && entry.generation == generation
        {
            let mut d = entry.depth;
            for id in tree.propagation_chain.drain(..).rev() {
                d = d.saturating_add(1);
                tree.propagation_depth_cache.insert(
                    id,
                    PropagationDepthCacheEntry {
                        generation,
                        depth: d,
                    },
                );
            }
            return tree
                .propagation_depth_cache
                .get(start)
                .and_then(|e| (e.generation == generation).then_some(e.depth))
                .unwrap_or_default();
        }

        tree.propagation_chain.push(node);
        current = tree.nodes.get(node).and_then(|n| n.parent);
    }

    let mut d = 0u32;
    for id in tree.propagation_chain.drain(..).rev() {
        tree.propagation_depth_cache.insert(
            id,
            PropagationDepthCacheEntry {
                generation,
                depth: d,
            },
        );
        d = d.saturating_add(1);
    }

    tree.propagation_depth_cache
        .get(start)
        .and_then(|e| (e.generation == generation).then_some(e.depth))
        .unwrap_or_default()
}
