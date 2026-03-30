use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

pub(crate) fn pick_last_snapshot_after_warmup(
    snaps: &[Value],
    warmup_frames: u64,
) -> Option<&Value> {
    snaps
        .iter()
        .rev()
        .find(|s| snapshot_frame_id(s) >= warmup_frames)
        .or_else(|| snaps.last())
}

pub(crate) fn pick_last_snapshot_with_resolved_semantics_after_warmup<'a>(
    snaps: &'a [Value],
    warmup_frames: u64,
    semantics: &SemanticsResolver<'a>,
) -> Option<&'a Value> {
    snaps
        .iter()
        .rev()
        .find(|s| snapshot_frame_id(s) >= warmup_frames && semantics.nodes(s).is_some())
        .or_else(|| snaps.iter().rev().find(|s| semantics.nodes(s).is_some()))
        .or_else(|| pick_last_snapshot_after_warmup(snaps, warmup_frames))
}

pub(crate) fn snapshot_frame_id(snapshot: &Value) -> u64 {
    snapshot
        .get("frame_id")
        .and_then(|v| v.as_u64())
        .or_else(|| snapshot.get("frameId").and_then(|v| v.as_u64()))
        .unwrap_or(0)
}

pub(crate) fn snapshot_window_snapshot_seq(snapshot: &Value) -> Option<u64> {
    snapshot
        .get("window_snapshot_seq")
        .and_then(|v| v.as_u64())
        .or_else(|| snapshot.get("windowSnapshotSeq").and_then(|v| v.as_u64()))
}

pub(crate) fn snapshot_window_id(snapshot: &Value) -> Option<u64> {
    snapshot
        .get("window")
        .and_then(|v| v.as_u64())
        .or_else(|| snapshot.get("window_id").and_then(|v| v.as_u64()))
        .or_else(|| snapshot.get("windowId").and_then(|v| v.as_u64()))
}

pub(crate) fn snapshot_semantics_fingerprint(snapshot: &Value) -> Option<u64> {
    snapshot
        .get("semantics_fingerprint")
        .and_then(|v| v.as_u64())
        .or_else(|| {
            snapshot
                .get("semanticsFingerprint")
                .and_then(|v| v.as_u64())
        })
}

pub(crate) fn snapshot_semantics(snapshot: &Value) -> Option<&Value> {
    snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .or_else(|| snapshot.get("semantics"))
        .or_else(|| snapshot.get("semantic_tree"))
        .or_else(|| snapshot.get("semanticTree"))
        .or_else(|| snapshot.get("tree"))
}

pub(crate) fn snapshot_semantics_nodes(snapshot: &Value) -> Option<&[Value]> {
    snapshot_semantics(snapshot)?
        .get("nodes")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
}

pub(crate) fn semantics_node_for_test_id<'a>(
    semantics: &SemanticsResolver<'a>,
    snapshot: &'a Value,
    test_id: &str,
) -> Option<&'a Value> {
    let nodes = semantics.nodes(snapshot)?;
    nodes.iter().find(|n| {
        n.get("test_id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id == test_id)
    })
}

pub(crate) fn semantics_node_for_test_id_trimmed<'a>(
    semantics: &SemanticsResolver<'a>,
    snapshot: &'a Value,
    test_id: &str,
) -> Option<&'a Value> {
    let target = test_id.trim();
    if target.is_empty() {
        return None;
    }

    let nodes = semantics.nodes(snapshot)?;
    nodes.iter().find(|n| {
        n.get("test_id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id.trim() == target)
    })
}

pub(crate) fn build_semantics_table_entries_from_windows(windows: &[Value]) -> Vec<Value> {
    let mut table: BTreeMap<(u64, u64), Value> = BTreeMap::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v.as_slice());
        for s in snaps {
            let Some(sem) = snapshot_semantics(s) else {
                continue;
            };
            if sem.is_null() {
                continue;
            }
            let Some(fp) = snapshot_semantics_fingerprint(s) else {
                continue;
            };
            let snap_window = snapshot_window_id(s).unwrap_or(window_id);
            table
                .entry((snap_window, fp))
                .or_insert_with(|| sem.clone());
        }
    }

    table
        .into_iter()
        .map(|((window, fp), semantics)| {
            serde_json::json!({
                "window": window,
                "semantics_fingerprint": fp,
                "semantics": semantics,
            })
        })
        .collect()
}

fn bundle_semantics_table_entries(bundle: &Value) -> Option<&[Value]> {
    bundle
        .get("tables")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("entries"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
}

pub(crate) struct SemanticsResolver<'a> {
    entries: Option<&'a [Value]>,
    by_window_fp: HashMap<(u64, u64), usize>,
}

impl<'a> SemanticsResolver<'a> {
    pub(crate) fn new(bundle: &'a Value) -> Self {
        let entries = bundle_semantics_table_entries(bundle);
        let mut by_window_fp: HashMap<(u64, u64), usize> = HashMap::new();
        if let Some(entries) = entries {
            for (idx, e) in entries.iter().enumerate() {
                let Some(window) = e.get("window").and_then(|v| v.as_u64()) else {
                    continue;
                };
                let Some(fp) = e.get("semantics_fingerprint").and_then(|v| v.as_u64()) else {
                    continue;
                };
                by_window_fp.insert((window, fp), idx);
            }
        }
        Self {
            entries,
            by_window_fp,
        }
    }

    pub(crate) fn semantics_snapshot(&self, snapshot: &'a Value) -> Option<&'a Value> {
        if let Some(sem) = snapshot_semantics(snapshot) {
            // Treat explicit nulls as "missing" so schema2 table semantics can still be resolved.
            if !sem.is_null() {
                return Some(sem);
            }
        }
        let entries = self.entries?;
        let window = snapshot_window_id(snapshot)?;
        let fp = snapshot_semantics_fingerprint(snapshot)?;
        let idx = *self.by_window_fp.get(&(window, fp))?;
        entries
            .get(idx)
            .and_then(|e| e.get("semantics"))
            .or_else(|| entries.get(idx).and_then(|e| e.get("semantic")))
    }

    pub(crate) fn table_entries(&self) -> &'a [Value] {
        self.entries.unwrap_or(&[])
    }

    pub(crate) fn table_entries_total(&self) -> usize {
        self.table_entries().len()
    }

    pub(crate) fn table_unique_keys_total(&self) -> usize {
        self.by_window_fp.len()
    }

    pub(crate) fn table_unique_keys_total_for_window(&self, window: u64) -> usize {
        self.by_window_fp
            .keys()
            .filter(|(w, _fp)| *w == window)
            .count()
    }

    pub(crate) fn nodes(&self, snapshot: &'a Value) -> Option<&'a [Value]> {
        if let Some(nodes) = snapshot_semantics_nodes(snapshot) {
            return Some(nodes);
        }
        self.semantics_snapshot(snapshot)?
            .get("nodes")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
    }
}

/// Owned semantics presence helper for schema conversion and other in-place mutation workflows.
///
/// Unlike [`SemanticsResolver`], this type does not borrow the bundle JSON, so it can be used
/// while mutating the bundle structure.
pub(crate) struct SemanticsTablePresence {
    table_keys: HashSet<(u64, u64)>,
}

impl SemanticsTablePresence {
    pub(crate) fn new(bundle: &Value) -> Self {
        let mut table_keys: HashSet<(u64, u64)> = HashSet::new();
        if let Some(entries) = bundle_semantics_table_entries(bundle) {
            for e in entries {
                let Some(window) = e.get("window").and_then(|v| v.as_u64()) else {
                    continue;
                };
                let Some(fp) = e.get("semantics_fingerprint").and_then(|v| v.as_u64()) else {
                    continue;
                };
                table_keys.insert((window, fp));
            }
        }
        Self { table_keys }
    }

    pub(crate) fn snapshot_has_semantics(&self, snapshot: &Value, default_window: u64) -> bool {
        if let Some(sem) = snapshot_semantics(snapshot) {
            // In schema conversion flows, explicit null semantics is an intentional "cleared"
            // marker; do not fall back to table semantics in that case.
            return !sem.is_null();
        }
        let window = snapshot_window_id(snapshot).unwrap_or(default_window);
        let Some(fp) = snapshot_semantics_fingerprint(snapshot) else {
            return false;
        };
        self.table_keys.contains(&(window, fp))
    }
}

/// Streaming reader for schema2 semantics table nodes (`tables.semantics.entries[*].semantics.nodes`).
///
/// This avoids materializing the entire bundle artifact in memory for large bundles.
pub(crate) fn stream_read_semantics_table_nodes(
    bundle_path: &Path,
    window_id: u64,
    semantics_fingerprint: u64,
) -> Result<Option<Vec<Value>>, String> {
    const FOUND_TABLE_MARKER: &str = "__FRET_DIAG_FOUND_TABLE__";

    struct RootSeed {
        window_id: u64,
        semantics_fingerprint: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> DeserializeSeed<'de> for RootSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(RootVisitor {
                window_id: self.window_id,
                semantics_fingerprint: self.semantics_fingerprint,
                out: self.out,
            })
        }
    }

    struct RootVisitor {
        window_id: u64,
        semantics_fingerprint: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> Visitor<'de> for RootVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "bundle artifact object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "tables" => {
                        map.next_value_seed(TablesSeed {
                            window_id: self.window_id,
                            semantics_fingerprint: self.semantics_fingerprint,
                            out: self.out.clone(),
                        })?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(())
        }
    }

    struct TablesSeed {
        window_id: u64,
        semantics_fingerprint: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> DeserializeSeed<'de> for TablesSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(TablesVisitor {
                window_id: self.window_id,
                semantics_fingerprint: self.semantics_fingerprint,
                out: self.out,
            })
        }
    }

    struct TablesVisitor {
        window_id: u64,
        semantics_fingerprint: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> Visitor<'de> for TablesVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "tables object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "semantics" => {
                        map.next_value_seed(SemanticsTableSeed {
                            window_id: self.window_id,
                            semantics_fingerprint: self.semantics_fingerprint,
                            out: self.out.clone(),
                        })?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(())
        }
    }

    struct SemanticsTableSeed {
        window_id: u64,
        semantics_fingerprint: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> DeserializeSeed<'de> for SemanticsTableSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SemanticsTableVisitor {
                window_id: self.window_id,
                semantics_fingerprint: self.semantics_fingerprint,
                out: self.out,
            })
        }
    }

    struct SemanticsTableVisitor {
        window_id: u64,
        semantics_fingerprint: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> Visitor<'de> for SemanticsTableVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "semantics table object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "entries" => {
                        map.next_value_seed(EntriesSeed {
                            window_id: self.window_id,
                            semantics_fingerprint: self.semantics_fingerprint,
                            out: self.out.clone(),
                        })?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(())
        }
    }

    struct EntriesSeed {
        window_id: u64,
        semantics_fingerprint: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> DeserializeSeed<'de> for EntriesSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(EntriesVisitor {
                window_id: self.window_id,
                semantics_fingerprint: self.semantics_fingerprint,
                out: self.out,
            })
        }
    }

    struct EntriesVisitor {
        window_id: u64,
        semantics_fingerprint: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> Visitor<'de> for EntriesVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "semantics entries array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
        where
            A: SeqAccess<'de>,
        {
            while seq
                .next_element_seed(EntrySeed {
                    window_id: self.window_id,
                    semantics_fingerprint: self.semantics_fingerprint,
                    out: self.out.clone(),
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct EntrySeed {
        window_id: u64,
        semantics_fingerprint: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> DeserializeSeed<'de> for EntrySeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(EntryVisitor {
                window_id: self.window_id,
                semantics_fingerprint: self.semantics_fingerprint,
                out: self.out,
            })
        }
    }

    struct EntryVisitor {
        window_id: u64,
        semantics_fingerprint: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> Visitor<'de> for EntryVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "semantics entry object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut window: Option<u64> = None;
            let mut fp: Option<u64> = None;
            let mut nodes: Option<Vec<Value>> = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "window" => {
                        window = Some(map.next_value::<u64>()?);
                    }
                    "semantics_fingerprint" | "semanticsFingerprint" => {
                        fp = Some(map.next_value::<u64>()?);
                    }
                    "semantics" => {
                        let is_match = window == Some(self.window_id)
                            && fp == Some(self.semantics_fingerprint);
                        if is_match {
                            map.next_value_seed(SemanticsSeed { nodes: &mut nodes })?;
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            if window == Some(self.window_id)
                && fp == Some(self.semantics_fingerprint)
                && let Some(nodes) = nodes
            {
                self.out.borrow_mut().replace(nodes);
                return Err(serde::de::Error::custom(FOUND_TABLE_MARKER));
            }
            Ok(())
        }
    }

    struct SemanticsSeed<'a> {
        nodes: &'a mut Option<Vec<Value>>,
    }

    impl<'de> DeserializeSeed<'de> for SemanticsSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_option(SemanticsOptVisitor { nodes: self.nodes })
        }
    }

    struct SemanticsOptVisitor<'a> {
        nodes: &'a mut Option<Vec<Value>>,
    }

    impl<'de> Visitor<'de> for SemanticsOptVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "semantics object or null")
        }

        fn visit_none<E>(self) -> Result<(), E>
        where
            E: serde::de::Error,
        {
            Ok(())
        }

        fn visit_unit<E>(self) -> Result<(), E>
        where
            E: serde::de::Error,
        {
            Ok(())
        }

        fn visit_some<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SemanticsVisitor { nodes: self.nodes })
        }
    }

    struct SemanticsVisitor<'a> {
        nodes: &'a mut Option<Vec<Value>>,
    }

    impl<'de> Visitor<'de> for SemanticsVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "semantics object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "nodes" => {
                        *self.nodes = Some(map.next_value::<Vec<Value>>()?);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(())
        }
    }

    let out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));
    crate::json_stream::with_bundle_json_deserializer_allow_stop(
        bundle_path,
        FOUND_TABLE_MARKER,
        |de| {
            RootSeed {
                window_id,
                semantics_fingerprint,
                out: out.clone(),
            }
            .deserialize(de)
        },
    )?;

    Ok(out.borrow_mut().take())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn build_semantics_table_entries_from_windows_dedups_by_window_and_fingerprint() {
        let windows = json!([{
            "window": 1,
            "snapshots": [
                {
                    "window": 1,
                    "semantics_fingerprint": 42,
                    "debug": { "semantics": { "nodes": [{ "id": 1 }] } }
                },
                {
                    "window": 1,
                    "semantics_fingerprint": 42,
                    "debug": { "semantics": { "nodes": [{ "id": 2 }] } }
                },
                {
                    "window": 1,
                    "semantics_fingerprint": 7,
                    "debug": { "semantics": { "nodes": [{ "id": 3 }] } }
                }
            ]
        }]);

        let entries = build_semantics_table_entries_from_windows(
            windows.as_array().expect("windows must be an array"),
        );
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["window"].as_u64(), Some(1));
        assert_eq!(entries[0]["semantics_fingerprint"].as_u64(), Some(7));
        assert_eq!(entries[1]["semantics_fingerprint"].as_u64(), Some(42));
    }

    #[test]
    fn semantics_resolver_reads_from_table_when_inline_missing() {
        let bundle = json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 1,
                    "window": 1,
                    "semantics_fingerprint": 42,
                    "debug": {}
                }]
            }],
            "tables": {
                "semantics": {
                    "entries": [{
                        "window": 1,
                        "semantics_fingerprint": 42,
                        "semantics": { "nodes": [{ "id": 7, "test_id": "foo" }] }
                    }]
                }
            }
        });

        let semantics = SemanticsResolver::new(&bundle);
        let snap = &bundle["windows"][0]["snapshots"][0];
        let nodes = semantics.nodes(snap).expect("expected nodes");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["id"].as_u64(), Some(7));
        assert_eq!(nodes[0]["test_id"].as_str(), Some("foo"));
    }

    #[test]
    fn semantics_resolver_prefers_inline_semantics_over_table() {
        let bundle = json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 1,
                    "window": 1,
                    "semantics_fingerprint": 42,
                    "debug": {
                        "semantics": { "nodes": [{ "id": 1, "test_id": "inline" }] }
                    }
                }]
            }],
            "tables": {
                "semantics": {
                    "entries": [{
                        "window": 1,
                        "semantics_fingerprint": 42,
                        "semantics": { "nodes": [{ "id": 2, "test_id": "table" }] }
                    }]
                }
            }
        });

        let semantics = SemanticsResolver::new(&bundle);
        let snap = &bundle["windows"][0]["snapshots"][0];
        let nodes = semantics.nodes(snap).expect("expected nodes");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["id"].as_u64(), Some(1));
        assert_eq!(nodes[0]["test_id"].as_str(), Some("inline"));
    }

    #[test]
    fn semantics_node_for_test_id_trimmed_matches_whitespace_padded_test_id() {
        let bundle = json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 1,
                    "window": 1,
                    "debug": {
                        "semantics": { "nodes": [{ "id": 1, "test_id": "  foo  " }] }
                    }
                }]
            }]
        });

        let semantics = SemanticsResolver::new(&bundle);
        let snap = &bundle["windows"][0]["snapshots"][0];
        let node = semantics_node_for_test_id_trimmed(&semantics, snap, "foo")
            .expect("expected node match");
        assert_eq!(node.get("id").and_then(|v| v.as_u64()), Some(1));
    }

    #[test]
    fn semantics_resolver_falls_back_to_table_when_inline_is_explicit_null() {
        let bundle = json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [{
                    "frame_id": 1,
                    "window": 1,
                    "semantics_fingerprint": 42,
                    "debug": { "semantics": null }
                }]
            }],
            "tables": {
                "semantics": {
                    "entries": [{
                        "window": 1,
                        "semantics_fingerprint": 42,
                        "semantics": { "nodes": [{ "id": 9, "test_id": "bar" }] }
                    }]
                }
            }
        });

        let semantics = SemanticsResolver::new(&bundle);
        let snap = &bundle["windows"][0]["snapshots"][0];
        let nodes = semantics.nodes(snap).expect("expected nodes");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["id"].as_u64(), Some(9));
        assert_eq!(nodes[0]["test_id"].as_str(), Some("bar"));
    }

    #[test]
    fn pick_last_snapshot_with_resolved_semantics_respects_warmup() {
        let bundle = json!({
            "schema_version": 2,
            "windows": [{
                "window": 1,
                "snapshots": [
                    { "frame_id": 0, "window": 1, "semantics_fingerprint": 1, "debug": {} },
                    { "frame_id": 5, "window": 1, "semantics_fingerprint": 1, "debug": {} }
                ]
            }],
            "tables": {
                "semantics": {
                    "entries": [{
                        "window": 1,
                        "semantics_fingerprint": 1,
                        "semantics": { "nodes": [{ "id": 9, "test_id": "x" }] }
                    }]
                }
            }
        });
        let snaps = bundle["windows"][0]["snapshots"].as_array().unwrap();
        let semantics = SemanticsResolver::new(&bundle);

        let picked = pick_last_snapshot_with_resolved_semantics_after_warmup(snaps, 1, &semantics)
            .expect("expected a snapshot");
        assert_eq!(snapshot_frame_id(picked), 5);
    }
}
