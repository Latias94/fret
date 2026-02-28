use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::de::{DeserializeSeed, Error as _, IgnoredAny, MapAccess, SeqAccess, Visitor};

type SemanticsKey = (u64, u64); // (window_id, semantics_fingerprint)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TextFingerprint {
    len: usize,
    hash: u64,
}

impl TextFingerprint {
    fn new(s: &str) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut hasher);
        Self {
            len: s.len(),
            hash: hasher.finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct SemNodeFieldsLite {
    y: Option<f64>,
    label: Option<TextFingerprint>,
    value: Option<TextFingerprint>,
}

pub(crate) fn check_bundle_for_stale_paint_streaming(
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let wanted = collect_wanted_table_semantics_keys(bundle_path)?;
    let table = build_semantics_table_fields_map(bundle_path, &wanted, test_id)?;
    scan_stale_paint(bundle_path, &table, test_id, eps)
}

pub(crate) fn check_bundle_for_stale_scene_streaming(
    bundle_path: &Path,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    let wanted = collect_wanted_table_semantics_keys(bundle_path)?;
    let table = build_semantics_table_fields_map(bundle_path, &wanted, test_id)?;
    scan_stale_scene(bundle_path, &table, test_id, eps)
}

fn collect_wanted_table_semantics_keys(
    bundle_path: &Path,
) -> Result<HashSet<SemanticsKey>, String> {
    #[derive(Debug, Default)]
    struct State {
        wanted: HashSet<SemanticsKey>,
    }

    struct RootSeed {
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> DeserializeSeed<'de> for RootSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(RootVisitor { state: self.state })
        }
    }

    struct RootVisitor {
        state: std::rc::Rc<std::cell::RefCell<State>>,
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
                    "windows" => {
                        map.next_value_seed(WindowsSeed {
                            state: self.state.clone(),
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

    struct WindowsSeed {
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowsSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(WindowsVisitor { state: self.state })
        }
    }

    struct WindowsVisitor {
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> Visitor<'de> for WindowsVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "windows array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
        where
            A: SeqAccess<'de>,
        {
            while seq
                .next_element_seed(WindowSeed {
                    state: self.state.clone(),
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct WindowSeed {
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(WindowVisitor { state: self.state })
        }
    }

    struct WindowVisitor {
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> Visitor<'de> for WindowVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "window object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut window_id: u64 = 0;
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "window" | "window_id" | "windowId" => window_id = map.next_value::<u64>()?,
                    "snapshots" => {
                        map.next_value_seed(SnapshotsSeed {
                            window_id_default: window_id,
                            state: self.state.clone(),
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

    struct SnapshotsSeed {
        window_id_default: u64,
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotsSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(SnapshotsVisitor {
                window_id_default: self.window_id_default,
                state: self.state,
            })
        }
    }

    struct SnapshotsVisitor {
        window_id_default: u64,
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> Visitor<'de> for SnapshotsVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshots array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some(meta) = seq.next_element_seed(SnapshotMetaSeed {
                window_id_default: self.window_id_default,
            })? {
                if meta.has_inline_semantics_nodes {
                    continue;
                }
                if let Some(fp) = meta.semantics_fingerprint {
                    self.state
                        .borrow_mut()
                        .wanted
                        .insert((meta.semantics_window_id, fp));
                }
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct SnapshotSemMeta {
        semantics_fingerprint: Option<u64>,
        semantics_window_id: u64,
        has_inline_semantics_nodes: bool,
    }

    struct SnapshotMetaSeed {
        window_id_default: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotMetaSeed {
        type Value = SnapshotSemMeta;

        fn deserialize<D>(self, deserializer: D) -> Result<SnapshotSemMeta, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotMetaVisitor {
                window_id_default: self.window_id_default,
                semantics_fingerprint: None,
                semantics_window_id: None,
                has_inline_semantics_nodes: false,
            })
        }
    }

    struct SnapshotMetaVisitor {
        window_id_default: u64,
        semantics_fingerprint: Option<u64>,
        semantics_window_id: Option<u64>,
        has_inline_semantics_nodes: bool,
    }

    impl<'de> Visitor<'de> for SnapshotMetaVisitor {
        type Value = SnapshotSemMeta;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshot object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<SnapshotSemMeta, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "semantics_fingerprint" | "semanticsFingerprint" => {
                        self.semantics_fingerprint = map.next_value::<Option<u64>>()?;
                    }
                    "semantics_window_id" | "semanticsWindowId" => {
                        self.semantics_window_id = map.next_value::<Option<u64>>()?;
                    }
                    "debug" => {
                        self.has_inline_semantics_nodes =
                            map.next_value_seed(DebugHasInlineSemNodesSeed)?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            Ok(SnapshotSemMeta {
                semantics_fingerprint: self.semantics_fingerprint,
                semantics_window_id: self.semantics_window_id.unwrap_or(self.window_id_default),
                has_inline_semantics_nodes: self.has_inline_semantics_nodes,
            })
        }
    }

    struct DebugHasInlineSemNodesSeed;

    impl<'de> DeserializeSeed<'de> for DebugHasInlineSemNodesSeed {
        type Value = bool;

        fn deserialize<D>(self, deserializer: D) -> Result<bool, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugHasInlineSemNodesVisitor { found: false })
        }
    }

    struct DebugHasInlineSemNodesVisitor {
        found: bool,
    }

    impl<'de> Visitor<'de> for DebugHasInlineSemNodesVisitor {
        type Value = bool;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<bool, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "semantics" {
                    self.found = map.next_value_seed(SemanticsHasNodesSeed)?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(self.found)
        }
    }

    struct SemanticsHasNodesSeed;

    impl<'de> DeserializeSeed<'de> for SemanticsHasNodesSeed {
        type Value = bool;

        fn deserialize<D>(self, deserializer: D) -> Result<bool, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(SemanticsHasNodesVisitor { found: false })
        }
    }

    struct SemanticsHasNodesVisitor {
        found: bool,
    }

    impl<'de> Visitor<'de> for SemanticsHasNodesVisitor {
        type Value = bool;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "semantics object or null")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<bool, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "nodes" {
                    self.found = map.next_value_seed(NodesPresenceSeed)?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(self.found)
        }

        fn visit_unit<E>(self) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_none<E>(self) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }
    }

    struct NodesPresenceSeed;

    impl<'de> DeserializeSeed<'de> for NodesPresenceSeed {
        type Value = bool;

        fn deserialize<D>(self, deserializer: D) -> Result<bool, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(NodesPresenceVisitor)
        }
    }

    struct NodesPresenceVisitor;

    impl<'de> Visitor<'de> for NodesPresenceVisitor {
        type Value = bool;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "nodes array or null")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<bool, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while seq.next_element::<IgnoredAny>()?.is_some() {}
            Ok(true)
        }

        fn visit_unit<E>(self) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_none<E>(self) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }
    }

    let state = std::rc::Rc::new(std::cell::RefCell::new(State::default()));
    crate::json_stream::with_bundle_json_deserializer(bundle_path, |de| {
        RootSeed {
            state: state.clone(),
        }
        .deserialize(de)
    })?;

    Ok(state.borrow().wanted.clone())
}

fn build_semantics_table_fields_map(
    bundle_path: &Path,
    wanted: &HashSet<SemanticsKey>,
    test_id: &str,
) -> Result<HashMap<SemanticsKey, SemNodeFieldsLite>, String> {
    if wanted.is_empty() {
        return Ok(HashMap::new());
    }

    const FOUND_ALL_MARKER: &str = "__FRET_DIAG_FOUND_ALL_STALE_FIELDS__";

    #[derive(Debug)]
    struct State {
        wanted: HashSet<SemanticsKey>,
        test_id: String,
        out: HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl State {
        fn is_done(&self) -> bool {
            self.out.len() >= self.wanted.len()
        }
    }

    struct RootSeed {
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> DeserializeSeed<'de> for RootSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(RootVisitor { state: self.state })
        }
    }

    struct RootVisitor {
        state: std::rc::Rc<std::cell::RefCell<State>>,
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
                            state: self.state.clone(),
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
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> DeserializeSeed<'de> for TablesSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(TablesVisitor { state: self.state })
        }
    }

    struct TablesVisitor {
        state: std::rc::Rc<std::cell::RefCell<State>>,
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
                            state: self.state.clone(),
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
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> DeserializeSeed<'de> for SemanticsTableSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SemanticsTableVisitor { state: self.state })
        }
    }

    struct SemanticsTableVisitor {
        state: std::rc::Rc<std::cell::RefCell<State>>,
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
                            state: self.state.clone(),
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
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> DeserializeSeed<'de> for EntriesSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(EntriesVisitor { state: self.state })
        }
    }

    struct EntriesVisitor {
        state: std::rc::Rc<std::cell::RefCell<State>>,
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
                    state: self.state.clone(),
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct EntrySeed {
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> DeserializeSeed<'de> for EntrySeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(EntryVisitor { state: self.state })
        }
    }

    struct EntryVisitor {
        state: std::rc::Rc<std::cell::RefCell<State>>,
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
            let mut fields: Option<SemNodeFieldsLite> = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "window" => window = Some(map.next_value::<u64>()?),
                    "semantics_fingerprint" | "semanticsFingerprint" => {
                        fp = Some(map.next_value::<u64>()?);
                    }
                    "semantics" | "semantic" => {
                        let is_match = window
                            .zip(fp)
                            .is_some_and(|(w, fp)| self.state.borrow().wanted.contains(&(w, fp)));
                        if is_match {
                            let test_id = self.state.borrow().test_id.clone();
                            fields = Some(map.next_value_seed(SemanticsSeed { test_id })?);
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            if let (Some(w), Some(fp)) = (window, fp) {
                let key = (w, fp);
                if self.state.borrow().wanted.contains(&key) {
                    self.state
                        .borrow_mut()
                        .out
                        .insert(key, fields.unwrap_or_default());
                    if self.state.borrow().is_done() {
                        return Err(serde::de::Error::custom(FOUND_ALL_MARKER));
                    }
                }
            }

            Ok(())
        }
    }

    struct SemanticsSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for SemanticsSeed {
        type Value = SemNodeFieldsLite;

        fn deserialize<D>(self, deserializer: D) -> Result<SemNodeFieldsLite, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(SemanticsVisitor {
                test_id: self.test_id,
            })
        }
    }

    struct SemanticsVisitor {
        test_id: String,
    }

    impl<'de> Visitor<'de> for SemanticsVisitor {
        type Value = SemNodeFieldsLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "semantics object or null")
        }

        fn visit_map<M>(self, mut map: M) -> Result<SemNodeFieldsLite, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut out = SemNodeFieldsLite::default();
            while let Some(key) = map.next_key::<String>()? {
                if key == "nodes" {
                    out = map.next_value_seed(NodesSeed {
                        test_id: self.test_id.clone(),
                    })?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(out)
        }

        fn visit_unit<E>(self) -> Result<SemNodeFieldsLite, E>
        where
            E: serde::de::Error,
        {
            Ok(SemNodeFieldsLite::default())
        }

        fn visit_none<E>(self) -> Result<SemNodeFieldsLite, E>
        where
            E: serde::de::Error,
        {
            Ok(SemNodeFieldsLite::default())
        }
    }

    struct NodesSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for NodesSeed {
        type Value = SemNodeFieldsLite;

        fn deserialize<D>(self, deserializer: D) -> Result<SemNodeFieldsLite, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(NodesVisitor {
                test_id: self.test_id,
            })
        }
    }

    struct NodesVisitor {
        test_id: String,
    }

    impl<'de> Visitor<'de> for NodesVisitor {
        type Value = SemNodeFieldsLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "nodes array or null")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<SemNodeFieldsLite, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some(fields) = seq.next_element_seed(NodeFieldsSeed {
                test_id: self.test_id.clone(),
            })? {
                if fields.y.is_some() || fields.label.is_some() || fields.value.is_some() {
                    while seq.next_element::<IgnoredAny>()?.is_some() {}
                    return Ok(fields);
                }
            }
            Ok(SemNodeFieldsLite::default())
        }

        fn visit_unit<E>(self) -> Result<SemNodeFieldsLite, E>
        where
            E: serde::de::Error,
        {
            Ok(SemNodeFieldsLite::default())
        }

        fn visit_none<E>(self) -> Result<SemNodeFieldsLite, E>
        where
            E: serde::de::Error,
        {
            Ok(SemNodeFieldsLite::default())
        }
    }

    struct NodeFieldsSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for NodeFieldsSeed {
        type Value = SemNodeFieldsLite;

        fn deserialize<D>(self, deserializer: D) -> Result<SemNodeFieldsLite, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(NodeFieldsVisitor {
                test_id: self.test_id,
                out: SemNodeFieldsLite::default(),
                node_test_id: None,
            })
        }
    }

    struct NodeFieldsVisitor {
        test_id: String,
        out: SemNodeFieldsLite,
        node_test_id: Option<String>,
    }

    impl<'de> Visitor<'de> for NodeFieldsVisitor {
        type Value = SemNodeFieldsLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "node object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<SemNodeFieldsLite, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "test_id" | "testId" => {
                        self.node_test_id = map.next_value::<Option<String>>()?
                    }
                    "bounds" => self.out.y = map.next_value_seed(BoundsYSeed)?,
                    "label" => {
                        let v = map.next_value::<Option<String>>()?;
                        self.out.label = v.as_deref().map(TextFingerprint::new);
                    }
                    "value" => {
                        let v = map.next_value::<Option<String>>()?;
                        self.out.value = v.as_deref().map(TextFingerprint::new);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            if self
                .node_test_id
                .as_deref()
                .is_some_and(|s| s == self.test_id)
            {
                return Ok(self.out);
            }
            Ok(SemNodeFieldsLite::default())
        }
    }

    struct BoundsYSeed;

    impl<'de> DeserializeSeed<'de> for BoundsYSeed {
        type Value = Option<f64>;

        fn deserialize<D>(self, deserializer: D) -> Result<Option<f64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(BoundsYVisitor { y: None })
        }
    }

    struct BoundsYVisitor {
        y: Option<f64>,
    }

    impl<'de> Visitor<'de> for BoundsYVisitor {
        type Value = Option<f64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "bounds object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<f64>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "y" {
                    self.y = map.next_value::<Option<f64>>()?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(self.y)
        }
    }

    let state = std::rc::Rc::new(std::cell::RefCell::new(State {
        wanted: wanted.clone(),
        test_id: test_id.to_string(),
        out: HashMap::new(),
    }));

    crate::json_stream::with_bundle_json_deserializer_allow_stop(
        bundle_path,
        FOUND_ALL_MARKER,
        |de| {
            RootSeed {
                state: state.clone(),
            }
            .deserialize(de)
        },
    )?;

    Ok(state.borrow().out.clone())
}

fn scan_stale_paint(
    bundle_path: &Path,
    table: &HashMap<SemanticsKey, SemNodeFieldsLite>,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    const STOP_MARKER: &str = "__FRET_DIAG_STOP_STALE_PAINT__";

    #[derive(Debug, Default)]
    struct State {
        test_id: String,
        eps: f32,
        missing_scene_fingerprint: bool,
        suspicious: Vec<String>,
    }

    struct RootSeed<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> DeserializeSeed<'de> for RootSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(RootVisitor {
                state: self.state,
                table: self.table,
            })
        }
    }

    struct RootVisitor<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> Visitor<'de> for RootVisitor<'_> {
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
                    "windows" => {
                        map.next_value_seed(WindowsSeed {
                            state: self.state.clone(),
                            table: self.table,
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

    struct WindowsSeed<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> DeserializeSeed<'de> for WindowsSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(WindowsVisitor {
                state: self.state,
                table: self.table,
            })
        }
    }

    struct WindowsVisitor<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> Visitor<'de> for WindowsVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "windows array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
        where
            A: SeqAccess<'de>,
        {
            while seq
                .next_element_seed(WindowSeed {
                    state: self.state.clone(),
                    table: self.table,
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct WindowSeed<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> DeserializeSeed<'de> for WindowSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(WindowVisitor {
                state: self.state,
                table: self.table,
            })
        }
    }

    struct WindowVisitor<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> Visitor<'de> for WindowVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "window object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut window_id: u64 = 0;
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "window" | "window_id" | "windowId" => window_id = map.next_value::<u64>()?,
                    "snapshots" => {
                        map.next_value_seed(SnapshotsSeed {
                            window_id,
                            state: self.state.clone(),
                            table: self.table,
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

    struct SnapshotsSeed<'a> {
        window_id: u64,
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotsSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(SnapshotsVisitor {
                window_id: self.window_id,
                state: self.state,
                table: self.table,
            })
        }
    }

    struct SnapshotsVisitor<'a> {
        window_id: u64,
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> Visitor<'de> for SnapshotsVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshots array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut prev_y: Option<f64> = None;
            let mut prev_fp: Option<u64> = None;

            while let Some(snap) = seq.next_element_seed(SnapshotPaintSeed {
                window_id_default: self.window_id,
                test_id: self.state.borrow().test_id.clone(),
                table: self.table,
            })? {
                if snap.scene_fingerprint.is_none() {
                    self.state.borrow_mut().missing_scene_fingerprint = true;
                }

                let (Some(y), Some(fp)) = (snap.y, snap.scene_fingerprint) else {
                    prev_y = snap.y;
                    prev_fp = snap.scene_fingerprint;
                    continue;
                };

                if let (Some(prev_y), Some(prev_fp)) = (prev_y, prev_fp)
                    && (y - prev_y).abs() >= self.state.borrow().eps as f64
                    && fp == prev_fp
                {
                    let mut st = self.state.borrow_mut();
                    let test_id = st.test_id.clone();
                    st.suspicious.push(format!(
                        "window={} tick={} frame={} test_id={} delta_y={:.2} scene_fingerprint=0x{:016x} paint_nodes_performed={} paint_cache_replayed_ops={}",
                        self.window_id,
                        snap.tick_id,
                        snap.frame_id,
                        test_id,
                        y - prev_y,
                        fp,
                        snap.paint_nodes_performed,
                        snap.paint_cache_replayed_ops,
                    ));

                    if st.suspicious.len() >= 8 {
                        return Err(A::Error::custom(STOP_MARKER));
                    }
                }

                prev_y = Some(y);
                prev_fp = Some(fp);
            }
            Ok(())
        }
    }

    #[derive(Debug, Default, Clone)]
    struct SnapshotPaint {
        tick_id: u64,
        frame_id: u64,
        scene_fingerprint: Option<u64>,
        semantics_fingerprint: Option<u64>,
        semantics_window_id: u64,
        has_inline_nodes: bool,
        y: Option<f64>,
        paint_nodes_performed: u64,
        paint_cache_replayed_ops: u64,
    }

    struct SnapshotPaintSeed<'a> {
        window_id_default: u64,
        test_id: String,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotPaintSeed<'_> {
        type Value = SnapshotPaint;

        fn deserialize<D>(self, deserializer: D) -> Result<SnapshotPaint, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotPaintVisitor {
                window_id_default: self.window_id_default,
                test_id: self.test_id,
                table: self.table,
                out: SnapshotPaint::default(),
            })
        }
    }

    struct SnapshotPaintVisitor<'a> {
        window_id_default: u64,
        test_id: String,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
        out: SnapshotPaint,
    }

    impl<'de> Visitor<'de> for SnapshotPaintVisitor<'_> {
        type Value = SnapshotPaint;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshot object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<SnapshotPaint, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "tick_id" | "tickId" => self.out.tick_id = map.next_value::<u64>()?,
                    "frame_id" | "frameId" => self.out.frame_id = map.next_value::<u64>()?,
                    "scene_fingerprint" | "sceneFingerprint" => {
                        self.out.scene_fingerprint = map.next_value::<Option<u64>>()?;
                    }
                    "semantics_fingerprint" | "semanticsFingerprint" => {
                        self.out.semantics_fingerprint = map.next_value::<Option<u64>>()?;
                    }
                    "semantics_window_id" | "semanticsWindowId" => {
                        self.out.semantics_window_id = map
                            .next_value::<Option<u64>>()?
                            .unwrap_or(self.window_id_default);
                    }
                    "debug" => {
                        let (has_nodes, y, paint_nodes, paint_ops) =
                            map.next_value_seed(DebugPaintSeed {
                                test_id: self.test_id.clone(),
                            })?;
                        self.out.has_inline_nodes = has_nodes;
                        self.out.y = y;
                        self.out.paint_nodes_performed = paint_nodes;
                        self.out.paint_cache_replayed_ops = paint_ops;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            // Fallback to table semantics only when there are no inline semantics nodes.
            if !self.out.has_inline_nodes {
                if let Some(fp) = self.out.semantics_fingerprint {
                    if let Some(fields) =
                        self.table.get(&(self.out.semantics_window_id, fp)).copied()
                    {
                        self.out.y = fields.y;
                    }
                }
            }

            Ok(self.out)
        }
    }

    struct DebugPaintSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for DebugPaintSeed {
        type Value = (bool, Option<f64>, u64, u64); // has_nodes, y, paint_nodes, paint_ops

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugPaintVisitor {
                test_id: self.test_id,
                has_nodes: false,
                y: None,
                paint_nodes_performed: 0,
                paint_cache_replayed_ops: 0,
            })
        }
    }

    struct DebugPaintVisitor {
        test_id: String,
        has_nodes: bool,
        y: Option<f64>,
        paint_nodes_performed: u64,
        paint_cache_replayed_ops: u64,
    }

    impl<'de> Visitor<'de> for DebugPaintVisitor {
        type Value = (bool, Option<f64>, u64, u64);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "semantics" => {
                        let (has_nodes, y) = map.next_value_seed(SemanticsInlineYSeed {
                            test_id: self.test_id.clone(),
                        })?;
                        self.has_nodes = has_nodes;
                        self.y = y;
                    }
                    "stats" => {
                        let (paint_nodes, paint_ops) = map.next_value_seed(StatsPaintSeed)?;
                        self.paint_nodes_performed = paint_nodes;
                        self.paint_cache_replayed_ops = paint_ops;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((
                self.has_nodes,
                self.y,
                self.paint_nodes_performed,
                self.paint_cache_replayed_ops,
            ))
        }
    }

    struct StatsPaintSeed;

    impl<'de> DeserializeSeed<'de> for StatsPaintSeed {
        type Value = (u64, u64);

        fn deserialize<D>(self, deserializer: D) -> Result<(u64, u64), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(StatsPaintVisitor {
                paint_nodes_performed: 0,
                paint_cache_replayed_ops: 0,
            })
        }
    }

    struct StatsPaintVisitor {
        paint_nodes_performed: u64,
        paint_cache_replayed_ops: u64,
    }

    impl<'de> Visitor<'de> for StatsPaintVisitor {
        type Value = (u64, u64);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "stats object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<(u64, u64), M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "paint_nodes_performed" | "paintNodesPerformed" => {
                        self.paint_nodes_performed = map.next_value::<u64>()?;
                    }
                    "paint_cache_replayed_ops" | "paintCacheReplayedOps" => {
                        self.paint_cache_replayed_ops = map.next_value::<u64>()?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((self.paint_nodes_performed, self.paint_cache_replayed_ops))
        }
    }

    struct SemanticsInlineYSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for SemanticsInlineYSeed {
        type Value = (bool, Option<f64>); // has_nodes, y

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(SemanticsInlineYVisitor {
                test_id: self.test_id,
            })
        }
    }

    struct SemanticsInlineYVisitor {
        test_id: String,
    }

    impl<'de> Visitor<'de> for SemanticsInlineYVisitor {
        type Value = (bool, Option<f64>);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "semantics object or null")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(bool, Option<f64>), M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut has_nodes = false;
            let mut y: Option<f64> = None;
            while let Some(key) = map.next_key::<String>()? {
                if key == "nodes" {
                    let (present, found) = map.next_value_seed(NodesInlineYSeed {
                        test_id: self.test_id.clone(),
                    })?;
                    has_nodes = present;
                    y = found;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok((has_nodes, y))
        }

        fn visit_unit<E>(self) -> Result<(bool, Option<f64>), E>
        where
            E: serde::de::Error,
        {
            Ok((false, None))
        }

        fn visit_none<E>(self) -> Result<(bool, Option<f64>), E>
        where
            E: serde::de::Error,
        {
            Ok((false, None))
        }
    }

    struct NodesInlineYSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for NodesInlineYSeed {
        type Value = (bool, Option<f64>);

        fn deserialize<D>(self, deserializer: D) -> Result<(bool, Option<f64>), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(NodesInlineYVisitor {
                test_id: self.test_id,
            })
        }
    }

    struct NodesInlineYVisitor {
        test_id: String,
    }

    impl<'de> Visitor<'de> for NodesInlineYVisitor {
        type Value = (bool, Option<f64>);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "nodes array or null")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<(bool, Option<f64>), A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some(found) = seq.next_element_seed(NodeInlineYSeed {
                test_id: self.test_id.clone(),
            })? {
                if found.is_some() {
                    while seq.next_element::<IgnoredAny>()?.is_some() {}
                    return Ok((true, found));
                }
            }
            Ok((true, None))
        }

        fn visit_unit<E>(self) -> Result<(bool, Option<f64>), E>
        where
            E: serde::de::Error,
        {
            Ok((false, None))
        }

        fn visit_none<E>(self) -> Result<(bool, Option<f64>), E>
        where
            E: serde::de::Error,
        {
            Ok((false, None))
        }
    }

    struct NodeInlineYSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for NodeInlineYSeed {
        type Value = Option<f64>;

        fn deserialize<D>(self, deserializer: D) -> Result<Option<f64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(NodeInlineYVisitor {
                test_id: self.test_id,
                node_test_id: None,
                y: None,
            })
        }
    }

    struct NodeInlineYVisitor {
        test_id: String,
        node_test_id: Option<String>,
        y: Option<f64>,
    }

    impl<'de> Visitor<'de> for NodeInlineYVisitor {
        type Value = Option<f64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "node object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<f64>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "test_id" | "testId" => {
                        self.node_test_id = map.next_value::<Option<String>>()?
                    }
                    "bounds" => self.y = map.next_value_seed(BoundsYSeed)?,
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            if self
                .node_test_id
                .as_deref()
                .is_some_and(|s| s == self.test_id)
            {
                return Ok(self.y);
            }
            Ok(None)
        }
    }

    struct BoundsYSeed;

    impl<'de> DeserializeSeed<'de> for BoundsYSeed {
        type Value = Option<f64>;

        fn deserialize<D>(self, deserializer: D) -> Result<Option<f64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(BoundsYVisitor { y: None })
        }
    }

    struct BoundsYVisitor {
        y: Option<f64>,
    }

    impl<'de> Visitor<'de> for BoundsYVisitor {
        type Value = Option<f64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "bounds object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<f64>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "y" {
                    self.y = map.next_value::<Option<f64>>()?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(self.y)
        }
    }

    let state = std::rc::Rc::new(std::cell::RefCell::new(State {
        test_id: test_id.to_string(),
        eps,
        missing_scene_fingerprint: false,
        suspicious: Vec::new(),
    }));

    crate::json_stream::with_bundle_json_deserializer_allow_stop(bundle_path, STOP_MARKER, |de| {
        RootSeed {
            state: state.clone(),
            table,
        }
        .deserialize(de)
    })?;

    let st = state.borrow();
    if st.missing_scene_fingerprint {
        return Err(format!(
            "stale paint check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if st.suspicious.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "stale paint suspected (semantics bounds moved but scene fingerprint did not change)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in st.suspicious.iter() {
        msg.push_str("  ");
        msg.push_str(line);
        msg.push('\n');
    }
    Err(msg)
}

fn scan_stale_scene(
    bundle_path: &Path,
    table: &HashMap<SemanticsKey, SemNodeFieldsLite>,
    test_id: &str,
    eps: f32,
) -> Result<(), String> {
    const STOP_MARKER: &str = "__FRET_DIAG_STOP_STALE_SCENE__";

    #[derive(Debug, Default)]
    struct State {
        test_id: String,
        eps: f32,
        missing_scene_fingerprint: bool,
        suspicious: Vec<String>,
    }

    struct RootSeed<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> DeserializeSeed<'de> for RootSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(RootVisitor {
                state: self.state,
                table: self.table,
            })
        }
    }

    struct RootVisitor<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> Visitor<'de> for RootVisitor<'_> {
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
                    "windows" => {
                        map.next_value_seed(WindowsSeed {
                            state: self.state.clone(),
                            table: self.table,
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

    struct WindowsSeed<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> DeserializeSeed<'de> for WindowsSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(WindowsVisitor {
                state: self.state,
                table: self.table,
            })
        }
    }

    struct WindowsVisitor<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> Visitor<'de> for WindowsVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "windows array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
        where
            A: SeqAccess<'de>,
        {
            while seq
                .next_element_seed(WindowSeed {
                    state: self.state.clone(),
                    table: self.table,
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct WindowSeed<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> DeserializeSeed<'de> for WindowSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(WindowVisitor {
                state: self.state,
                table: self.table,
            })
        }
    }

    struct WindowVisitor<'a> {
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> Visitor<'de> for WindowVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "window object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut window_id: u64 = 0;
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "window" | "window_id" | "windowId" => window_id = map.next_value::<u64>()?,
                    "snapshots" => {
                        map.next_value_seed(SnapshotsSeed {
                            window_id,
                            state: self.state.clone(),
                            table: self.table,
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

    struct SnapshotsSeed<'a> {
        window_id: u64,
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotsSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(SnapshotsVisitor {
                window_id: self.window_id,
                state: self.state,
                table: self.table,
            })
        }
    }

    struct SnapshotsVisitor<'a> {
        window_id: u64,
        state: std::rc::Rc<std::cell::RefCell<State>>,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> Visitor<'de> for SnapshotsVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshots array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut prev_y: Option<f64> = None;
            let mut prev_label: Option<TextFingerprint> = None;
            let mut prev_value: Option<TextFingerprint> = None;
            let mut prev_fp: Option<u64> = None;

            while let Some(snap) = seq.next_element_seed(SnapshotSceneSeed {
                window_id_default: self.window_id,
                test_id: self.state.borrow().test_id.clone(),
                table: self.table,
            })? {
                if snap.scene_fingerprint.is_none() {
                    self.state.borrow_mut().missing_scene_fingerprint = true;
                }

                let Some(fp) = snap.scene_fingerprint else {
                    prev_y = snap.fields.y;
                    prev_label = snap.fields.label;
                    prev_value = snap.fields.value;
                    prev_fp = None;
                    continue;
                };

                if let (Some(prev_fp), Some(prev_y_val)) = (prev_fp, prev_y) {
                    let moved = snap
                        .fields
                        .y
                        .zip(Some(prev_y_val))
                        .is_some_and(|(y, prev_y)| {
                            (y - prev_y).abs() >= self.state.borrow().eps as f64
                        });
                    let label_changed = snap.fields.label != prev_label;
                    let value_changed = snap.fields.value != prev_value;
                    let changed = moved || label_changed || value_changed;

                    if changed && fp == prev_fp {
                        let mut st = self.state.borrow_mut();
                        let test_id = st.test_id.clone();
                        let label_len_prev = prev_label.map(|f| f.len).unwrap_or(0);
                        let label_len_now = snap.fields.label.map(|f| f.len).unwrap_or(0);
                        let value_len_prev = prev_value.map(|f| f.len).unwrap_or(0);
                        let value_len_now = snap.fields.value.map(|f| f.len).unwrap_or(0);
                        let delta_y = snap
                            .fields
                            .y
                            .zip(Some(prev_y_val))
                            .map(|(y, prev_y)| y - prev_y)
                            .unwrap_or(0.0);
                        st.suspicious.push(format!(
                            "window={} tick={} frame={} test_id={} changed={{moved={} label={} value={}}} delta_y={:.2} label_len={}->{} value_len={}->{} scene_fingerprint=0x{:016x}",
                            self.window_id,
                            snap.tick_id,
                            snap.frame_id,
                            test_id,
                            moved,
                            label_changed,
                            value_changed,
                            delta_y,
                            label_len_prev,
                            label_len_now,
                            value_len_prev,
                            value_len_now,
                            fp,
                        ));

                        if st.suspicious.len() >= 8 {
                            return Err(A::Error::custom(STOP_MARKER));
                        }
                    }
                }

                prev_y = snap.fields.y;
                prev_label = snap.fields.label;
                prev_value = snap.fields.value;
                prev_fp = Some(fp);
            }
            Ok(())
        }
    }

    #[derive(Debug, Default, Clone)]
    struct SnapshotScene {
        tick_id: u64,
        frame_id: u64,
        scene_fingerprint: Option<u64>,
        semantics_fingerprint: Option<u64>,
        semantics_window_id: u64,
        has_inline_nodes: bool,
        fields: SemNodeFieldsLite,
    }

    struct SnapshotSceneSeed<'a> {
        window_id_default: u64,
        test_id: String,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotSceneSeed<'_> {
        type Value = SnapshotScene;

        fn deserialize<D>(self, deserializer: D) -> Result<SnapshotScene, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotSceneVisitor {
                window_id_default: self.window_id_default,
                test_id: self.test_id,
                table: self.table,
                out: SnapshotScene::default(),
            })
        }
    }

    struct SnapshotSceneVisitor<'a> {
        window_id_default: u64,
        test_id: String,
        table: &'a HashMap<SemanticsKey, SemNodeFieldsLite>,
        out: SnapshotScene,
    }

    impl<'de> Visitor<'de> for SnapshotSceneVisitor<'_> {
        type Value = SnapshotScene;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshot object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<SnapshotScene, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "tick_id" | "tickId" => self.out.tick_id = map.next_value::<u64>()?,
                    "frame_id" | "frameId" => self.out.frame_id = map.next_value::<u64>()?,
                    "scene_fingerprint" | "sceneFingerprint" => {
                        self.out.scene_fingerprint = map.next_value::<Option<u64>>()?;
                    }
                    "semantics_fingerprint" | "semanticsFingerprint" => {
                        self.out.semantics_fingerprint = map.next_value::<Option<u64>>()?;
                    }
                    "semantics_window_id" | "semanticsWindowId" => {
                        self.out.semantics_window_id = map
                            .next_value::<Option<u64>>()?
                            .unwrap_or(self.window_id_default);
                    }
                    "debug" => {
                        let (has_nodes, fields) = map.next_value_seed(DebugSceneSeed {
                            test_id: self.test_id.clone(),
                        })?;
                        self.out.has_inline_nodes = has_nodes;
                        if let Some(fields) = fields {
                            self.out.fields = fields;
                        }
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            if !self.out.has_inline_nodes {
                if let Some(fp) = self.out.semantics_fingerprint {
                    if let Some(fields) =
                        self.table.get(&(self.out.semantics_window_id, fp)).copied()
                    {
                        self.out.fields = fields;
                    }
                }
            }

            Ok(self.out)
        }
    }

    struct DebugSceneSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for DebugSceneSeed {
        type Value = (bool, Option<SemNodeFieldsLite>);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugSceneVisitor {
                test_id: self.test_id,
                has_nodes: false,
                fields: None,
            })
        }
    }

    struct DebugSceneVisitor {
        test_id: String,
        has_nodes: bool,
        fields: Option<SemNodeFieldsLite>,
    }

    impl<'de> Visitor<'de> for DebugSceneVisitor {
        type Value = (bool, Option<SemNodeFieldsLite>);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "semantics" {
                    let (has_nodes, fields) = map.next_value_seed(SemanticsInlineFieldsSeed {
                        test_id: self.test_id.clone(),
                    })?;
                    self.has_nodes = has_nodes;
                    self.fields = fields;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok((self.has_nodes, self.fields))
        }
    }

    struct SemanticsInlineFieldsSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for SemanticsInlineFieldsSeed {
        type Value = (bool, Option<SemNodeFieldsLite>);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(SemanticsInlineFieldsVisitor {
                test_id: self.test_id,
            })
        }
    }

    struct SemanticsInlineFieldsVisitor {
        test_id: String,
    }

    impl<'de> Visitor<'de> for SemanticsInlineFieldsVisitor {
        type Value = (bool, Option<SemNodeFieldsLite>);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "semantics object or null")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut has_nodes = false;
            let mut fields: Option<SemNodeFieldsLite> = None;
            while let Some(key) = map.next_key::<String>()? {
                if key == "nodes" {
                    let (present, found) = map.next_value_seed(NodesInlineFieldsSeed {
                        test_id: self.test_id.clone(),
                    })?;
                    has_nodes = present;
                    fields = found;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok((has_nodes, fields))
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok((false, None))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok((false, None))
        }
    }

    struct NodesInlineFieldsSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for NodesInlineFieldsSeed {
        type Value = (bool, Option<SemNodeFieldsLite>);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(NodesInlineFieldsVisitor {
                test_id: self.test_id,
            })
        }
    }

    struct NodesInlineFieldsVisitor {
        test_id: String,
    }

    impl<'de> Visitor<'de> for NodesInlineFieldsVisitor {
        type Value = (bool, Option<SemNodeFieldsLite>);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "nodes array or null")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some(found) = seq.next_element_seed(NodeInlineFieldsSeed {
                test_id: self.test_id.clone(),
            })? {
                if found.y.is_some() || found.label.is_some() || found.value.is_some() {
                    while seq.next_element::<IgnoredAny>()?.is_some() {}
                    return Ok((true, Some(found)));
                }
            }
            Ok((true, None))
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok((false, None))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok((false, None))
        }
    }

    struct NodeInlineFieldsSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for NodeInlineFieldsSeed {
        type Value = SemNodeFieldsLite;

        fn deserialize<D>(self, deserializer: D) -> Result<SemNodeFieldsLite, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(NodeInlineFieldsVisitor {
                test_id: self.test_id,
                node_test_id: None,
                out: SemNodeFieldsLite::default(),
            })
        }
    }

    struct NodeInlineFieldsVisitor {
        test_id: String,
        node_test_id: Option<String>,
        out: SemNodeFieldsLite,
    }

    impl<'de> Visitor<'de> for NodeInlineFieldsVisitor {
        type Value = SemNodeFieldsLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "node object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<SemNodeFieldsLite, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "test_id" | "testId" => {
                        self.node_test_id = map.next_value::<Option<String>>()?
                    }
                    "bounds" => self.out.y = map.next_value_seed(BoundsYSeed)?,
                    "label" => {
                        let v = map.next_value::<Option<String>>()?;
                        self.out.label = v.as_deref().map(TextFingerprint::new);
                    }
                    "value" => {
                        let v = map.next_value::<Option<String>>()?;
                        self.out.value = v.as_deref().map(TextFingerprint::new);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            if self
                .node_test_id
                .as_deref()
                .is_some_and(|s| s == self.test_id)
            {
                return Ok(self.out);
            }
            Ok(SemNodeFieldsLite::default())
        }
    }

    struct BoundsYSeed;

    impl<'de> DeserializeSeed<'de> for BoundsYSeed {
        type Value = Option<f64>;

        fn deserialize<D>(self, deserializer: D) -> Result<Option<f64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(BoundsYVisitor { y: None })
        }
    }

    struct BoundsYVisitor {
        y: Option<f64>,
    }

    impl<'de> Visitor<'de> for BoundsYVisitor {
        type Value = Option<f64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "bounds object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<f64>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "y" {
                    self.y = map.next_value::<Option<f64>>()?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(self.y)
        }
    }

    let state = std::rc::Rc::new(std::cell::RefCell::new(State {
        test_id: test_id.to_string(),
        eps,
        missing_scene_fingerprint: false,
        suspicious: Vec::new(),
    }));

    crate::json_stream::with_bundle_json_deserializer_allow_stop(bundle_path, STOP_MARKER, |de| {
        RootSeed {
            state: state.clone(),
            table,
        }
        .deserialize(de)
    })?;

    let st = state.borrow();
    if st.missing_scene_fingerprint {
        return Err(format!(
            "stale scene check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if st.suspicious.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "stale scene suspected (semantics changed but scene fingerprint did not change)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in st.suspicious.iter() {
        msg.push_str("  ");
        msg.push_str(line);
        msg.push('\n');
    }
    Err(msg)
}
