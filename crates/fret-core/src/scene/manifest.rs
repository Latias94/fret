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
        let mut hash = 0x982d_3fc7_c4f1_6a5bu64;
        hash = mix_u64(hash, self.chunk.fingerprint());
        hash = mix_u64(hash, self.chunk.closure().fingerprint());
        hash = mix_u64(hash, self.chunk.ops_len() as u64);
        hash = mix_rect(hash, self.local_bounds);
        mix_point(hash, self.scene_origin)
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
        if self.entries.is_empty() {
            return 0;
        }
        let mut hash = 0x3c79_ac49_2ba7_b653u64;
        hash = mix_u64(hash, self.entries.len() as u64);
        for entry in &self.entries {
            hash = mix_u64(hash, entry.fingerprint());
        }
        hash
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

fn mix_u64(mut hash: u64, value: u64) -> u64 {
    hash ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    hash = hash.rotate_left(7);
    hash.wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

fn mix_px(hash: u64, value: crate::geometry::Px) -> u64 {
    mix_u64(hash, u64::from(value.0.to_bits()))
}

fn mix_point(mut hash: u64, point: Point) -> u64 {
    hash = mix_px(hash, point.x);
    mix_px(hash, point.y)
}

fn mix_rect(mut hash: u64, rect: Rect) -> u64 {
    hash = mix_point(hash, rect.origin);
    hash = mix_px(hash, rect.size.width);
    mix_px(hash, rect.size.height)
}
