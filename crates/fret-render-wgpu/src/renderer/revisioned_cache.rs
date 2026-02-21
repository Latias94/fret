use std::collections::HashMap;
use std::hash::Hash;

pub(super) struct RevisionedCache<K, V> {
    entries: HashMap<K, (u64, V)>,
}

impl<K, V> Default for RevisionedCache<K, V> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

impl<K, V> RevisionedCache<K, V>
where
    K: Eq + Hash + Copy,
{
    pub(super) fn get(&self, key: K) -> Option<&V> {
        self.entries.get(&key).map(|(_, v)| v)
    }

    pub(super) fn remove(&mut self, key: K) -> Option<V> {
        self.entries.remove(&key).map(|(_, v)| v)
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(super) fn ensure(&mut self, key: K, revision: u64, build: impl FnOnce() -> V) -> &V {
        let needs_rebuild = match self.entries.get(&key) {
            Some((cached, _)) => *cached != revision,
            None => true,
        };
        if needs_rebuild {
            self.entries.insert(key, (revision, build()));
        }
        self.entries
            .get(&key)
            .map(|(_, v)| v)
            .expect("entry exists")
    }
}
