use std::hash::{Hash, Hasher};
use std::panic::Location;

use fret_core::AppWindowId;

use super::GlobalElementId;

pub fn global_root(window: AppWindowId, name: &str) -> GlobalElementId {
    let mut hasher = Fnv1a64::default();
    window.hash(&mut hasher);
    hasher.write(name.as_bytes());
    GlobalElementId(hasher.finish())
}

pub(crate) fn derive_child_id(
    parent: GlobalElementId,
    callsite: u64,
    child_salt: u64,
) -> GlobalElementId {
    let mut hasher = Fnv1a64::default();
    hasher.write_u64(parent.0);
    hasher.write_u64(callsite);
    hasher.write_u64(child_salt);
    GlobalElementId(hasher.finish())
}

pub(crate) fn stable_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = Fnv1a64::default();
    value.hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn callsite_hash(loc: &Location<'_>) -> u64 {
    let mut hasher = Fnv1a64::default();
    hasher.write(loc.file().as_bytes());
    hasher.write_u32(loc.line());
    hasher.write_u32(loc.column());
    hasher.finish()
}

#[derive(Default)]
struct Fnv1a64(u64);

impl Hasher for Fnv1a64 {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 {
            0xcbf29ce484222325
        } else {
            self.0
        };
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.0 = hash;
    }
}
