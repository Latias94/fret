use super::SceneChunk;
use crate::geometry::{Point, Rect};

#[derive(Debug, Clone)]
pub struct SceneChunkManifestEntry {
    chunk: SceneChunk,
    local_bounds: Rect,
    scene_origin: Point,
}

impl SceneChunkManifestEntry {
    pub fn new(chunk: SceneChunk, local_bounds: Rect, scene_origin: Point) -> Self {
        Self {
            chunk,
            local_bounds,
            scene_origin,
        }
    }

    pub fn chunk(&self) -> &SceneChunk {
        &self.chunk
    }

    pub fn local_bounds(&self) -> Rect {
        self.local_bounds
    }

    pub fn scene_origin(&self) -> Point {
        self.scene_origin
    }

    pub fn fingerprint(&self) -> u64 {
        self.chunk.fingerprint()
    }
}

#[derive(Debug, Default, Clone)]
pub struct SceneChunkManifest {
    entries: Vec<SceneChunkManifestEntry>,
}

impl SceneChunkManifest {
    pub fn push(&mut self, entry: SceneChunkManifestEntry) {
        if !entry.chunk.is_empty() {
            self.entries.push(entry);
        }
    }

    pub fn entries(&self) -> &[SceneChunkManifestEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn ops_len(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| entry.chunk().ops_len())
            .sum()
    }

    pub fn fingerprint(&self) -> u64 {
        self.entries
            .iter()
            .fold(0, |fingerprint, entry| fingerprint ^ entry.fingerprint())
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
