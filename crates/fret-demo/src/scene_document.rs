use serde::{Deserialize, Serialize};

use crate::hierarchy::DemoHierarchy;
use crate::world::{DemoEntity, DemoWorld};

#[derive(Debug, Clone)]
pub struct SceneSnapshot {
    pub hierarchy: DemoHierarchy,
    pub world: DemoWorld,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoSceneFileV1 {
    pub version: u32,
    pub roots: Vec<DemoSceneNodeV1>,
}

impl DemoSceneFileV1 {
    pub fn from_snapshot(snapshot: &SceneSnapshot) -> Self {
        let roots = snapshot
            .hierarchy
            .roots
            .iter()
            .map(|n| DemoSceneNodeV1::from_tree_node(snapshot, n))
            .collect();
        Self { version: 1, roots }
    }

    pub fn to_snapshot(&self) -> SceneSnapshot {
        let mut hierarchy = DemoHierarchy { roots: Vec::new() };
        let mut world = DemoWorld::default();
        for node in &self.roots {
            let (tree, entities) = node.to_tree_and_entities();
            hierarchy.roots.push(tree);
            for (id, e) in entities {
                *world.entity_mut(id) = e;
            }
        }
        SceneSnapshot { hierarchy, world }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoSceneNodeV1 {
    pub id: u64,
    pub entity: DemoEntity,
    pub children: Vec<DemoSceneNodeV1>,
}

impl DemoSceneNodeV1 {
    fn from_tree_node(snapshot: &SceneSnapshot, node: &fret_ui::TreeNode) -> Self {
        let mut entity = snapshot.world.entity_snapshot(node.id);
        entity.name = node.label.clone();
        let children = node
            .children
            .iter()
            .map(|c| Self::from_tree_node(snapshot, c))
            .collect();
        Self {
            id: node.id,
            entity,
            children,
        }
    }

    fn to_tree_and_entities(&self) -> (fret_ui::TreeNode, Vec<(u64, DemoEntity)>) {
        let mut entities: Vec<(u64, DemoEntity)> = vec![(self.id, self.entity.clone())];
        let children: Vec<fret_ui::TreeNode> = self
            .children
            .iter()
            .map(|c| {
                let (tree, mut child_entities) = c.to_tree_and_entities();
                entities.append(&mut child_entities);
                tree
            })
            .collect();
        (
            fret_ui::TreeNode::new(self.id, self.entity.name.clone()).with_children(children),
            entities,
        )
    }
}

#[derive(Debug, Default)]
pub struct SceneDocumentService {
    dirty: bool,
    revision: u64,
}

impl SceneDocumentService {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        if self.dirty == dirty {
            return;
        }
        self.dirty = dirty;
        self.revision = self.revision.saturating_add(1);
    }
}

pub fn parse_scene_bytes(bytes: &[u8]) -> Option<DemoSceneFileV1> {
    serde_json::from_slice::<DemoSceneFileV1>(bytes).ok()
}

pub fn write_scene_pretty(file: &DemoSceneFileV1) -> Vec<u8> {
    // Fall back to compact JSON if pretty fails (should not happen).
    serde_json::to_vec_pretty(file).unwrap_or_else(|_| serde_json::to_vec(file).unwrap_or_default())
}
