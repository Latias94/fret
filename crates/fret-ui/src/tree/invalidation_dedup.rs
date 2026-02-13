use super::*;

#[derive(Debug, Default, Clone, Copy)]
struct InvalidationDedupEntry {
    generation: u32,
    mask: u8,
}

#[derive(Debug, Default)]
pub(super) struct InvalidationDedupTable {
    generation: u32,
    entries: SecondaryMap<NodeId, InvalidationDedupEntry>,
}

impl InvalidationDedupTable {
    pub(super) fn begin(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.generation = 1;
            self.entries.clear();
        }
    }

    fn get(&self, node: NodeId) -> u8 {
        self.entries
            .get(node)
            .filter(|e| e.generation == self.generation)
            .map(|e| e.mask)
            .unwrap_or_default()
    }

    fn insert(&mut self, node: NodeId, mask: u8) {
        self.entries.insert(
            node,
            InvalidationDedupEntry {
                generation: self.generation,
                mask,
            },
        );
    }
}

pub(super) trait InvalidationVisited {
    fn mask(&self, node: NodeId) -> u8;
    fn set_mask(&mut self, node: NodeId, mask: u8);
}

impl InvalidationVisited for HashMap<NodeId, u8> {
    fn mask(&self, node: NodeId) -> u8 {
        self.get(&node).copied().unwrap_or_default()
    }

    fn set_mask(&mut self, node: NodeId, mask: u8) {
        self.insert(node, mask);
    }
}

impl InvalidationVisited for InvalidationDedupTable {
    fn mask(&self, node: NodeId) -> u8 {
        self.get(node)
    }

    fn set_mask(&mut self, node: NodeId, mask: u8) {
        self.insert(node, mask);
    }
}
