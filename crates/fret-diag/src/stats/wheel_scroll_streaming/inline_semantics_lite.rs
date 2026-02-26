use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

use super::types::SemanticsLite;

pub(super) fn stream_read_inline_semantics_lite_for_pairs(
    bundle_path: &Path,
    wanted: &HashMap<u64, HashSet<u64>>,
    test_id: &str,
) -> Result<HashMap<(u64, u64), Option<SemanticsLite>>, String> {
    const FOUND_MARKER: &str = "__FRET_DIAG_FOUND_SEM_LITE__";

    #[derive(Debug, Clone)]
    struct State {
        wanted: HashMap<u64, HashSet<u64>>,
        found: HashMap<(u64, u64), Option<SemanticsLite>>,
        total: usize,
        test_id: String,
    }

    impl State {
        fn is_done(&self) -> bool {
            self.found.len() >= self.total
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
                    "window" | "window_id" | "windowId" => {
                        window_id = map.next_value::<u64>()?;
                    }
                    "snapshots" => {
                        let wanted_frames = {
                            let st = self.state.borrow();
                            st.wanted.get(&window_id).cloned().unwrap_or_default()
                        };
                        if wanted_frames.is_empty() {
                            map.next_value::<IgnoredAny>()?;
                            continue;
                        }
                        map.next_value_seed(SnapshotsSeed {
                            window_id,
                            wanted_frames,
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
        window_id: u64,
        wanted_frames: HashSet<u64>,
        state: std::rc::Rc<std::cell::RefCell<State>>,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotsSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(SnapshotsVisitor {
                window_id: self.window_id,
                wanted_frames: self.wanted_frames,
                state: self.state,
            })
        }
    }

    struct SnapshotsVisitor {
        window_id: u64,
        wanted_frames: HashSet<u64>,
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
            let test_id = { self.state.borrow().test_id.clone() };
            while let Some(v) = seq.next_element_seed(SnapshotSemLiteSeed {
                wanted_frames: self.wanted_frames.clone(),
                test_id: test_id.clone(),
            })? {
                let Some((frame_id, sem)) = v else {
                    continue;
                };
                self.state
                    .borrow_mut()
                    .found
                    .insert((self.window_id, frame_id), sem);
                let done = { self.state.borrow().is_done() };
                if done {
                    return Err(serde::de::Error::custom(FOUND_MARKER));
                }
            }
            Ok(())
        }
    }

    struct SnapshotSemLiteSeed {
        wanted_frames: HashSet<u64>,
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotSemLiteSeed {
        type Value = Option<(u64, Option<SemanticsLite>)>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotSemLiteVisitor {
                wanted_frames: self.wanted_frames,
                test_id: self.test_id,
                frame_id: 0,
                sem: None,
            })
        }
    }

    struct SnapshotSemLiteVisitor {
        wanted_frames: HashSet<u64>,
        test_id: String,
        frame_id: u64,
        sem: Option<Option<SemanticsLite>>,
    }

    impl<'de> Visitor<'de> for SnapshotSemLiteVisitor {
        type Value = Option<(u64, Option<SemanticsLite>)>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshot object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "frame_id" | "frameId" => {
                        self.frame_id = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "debug" => {
                        if self.wanted_frames.contains(&self.frame_id) {
                            self.sem = Some(map.next_value_seed(DebugSemLiteSeed {
                                test_id: self.test_id.clone(),
                            })?);
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    "semantics" | "semantic_tree" | "semanticTree" | "tree" => {
                        if self.wanted_frames.contains(&self.frame_id) && self.sem.is_none() {
                            self.sem = Some(map.next_value_seed(SemanticsLiteSeed {
                                test_id: self.test_id.clone(),
                            })?);
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            if !self.wanted_frames.contains(&self.frame_id) {
                return Ok(None);
            }

            Ok(Some((self.frame_id, self.sem.unwrap_or(None))))
        }
    }

    struct DebugSemLiteSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for DebugSemLiteSeed {
        type Value = Option<SemanticsLite>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugSemLiteVisitor {
                test_id: self.test_id,
                sem: None,
            })
        }
    }

    struct DebugSemLiteVisitor {
        test_id: String,
        sem: Option<Option<SemanticsLite>>,
    }

    impl<'de> Visitor<'de> for DebugSemLiteVisitor {
        type Value = Option<SemanticsLite>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<SemanticsLite>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "semantics" => {
                        self.sem = Some(map.next_value_seed(SemanticsLiteSeed {
                            test_id: self.test_id.clone(),
                        })?);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(self.sem.unwrap_or(None))
        }
    }

    struct SemanticsLiteSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for SemanticsLiteSeed {
        type Value = Option<SemanticsLite>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(SemanticsLiteVisitor {
                test_id: self.test_id,
                out: SemanticsLite::default(),
                has_nodes: false,
            })
        }
    }

    struct SemanticsLiteVisitor {
        test_id: String,
        out: SemanticsLite,
        has_nodes: bool,
    }

    impl<'de> Visitor<'de> for SemanticsLiteVisitor {
        type Value = Option<SemanticsLite>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "a semantics object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<SemanticsLite>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "nodes" {
                    self.has_nodes = true;
                    self.out = map.next_value_seed(SemanticsNodesLiteSeed {
                        test_id: self.test_id.clone(),
                    })?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            if self.has_nodes {
                Ok(Some(self.out))
            } else {
                Ok(None)
            }
        }

        fn visit_unit<E>(self) -> Result<Option<SemanticsLite>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_none<E>(self) -> Result<Option<SemanticsLite>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    struct SemanticsNodesLiteSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for SemanticsNodesLiteSeed {
        type Value = SemanticsLite;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(SemanticsNodesLiteVisitor {
                test_id: self.test_id,
                out: SemanticsLite::default(),
            })
        }
    }

    struct SemanticsNodesLiteVisitor {
        test_id: String,
        out: SemanticsLite,
    }

    impl<'de> Visitor<'de> for SemanticsNodesLiteVisitor {
        type Value = SemanticsLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "nodes array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<SemanticsLite, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some((id, parent, test_id)) = seq.next_element_seed(NodeLiteSeed)? {
                if let (Some(id), Some(parent)) = (id, parent) {
                    self.out.parents.insert(id, parent);
                }
                if self.out.target_node_id.is_none()
                    && test_id.as_deref() == Some(self.test_id.as_str())
                {
                    self.out.target_node_id = id;
                }
            }
            Ok(self.out)
        }
    }

    struct NodeLiteSeed;

    impl<'de> DeserializeSeed<'de> for NodeLiteSeed {
        type Value = (Option<u64>, Option<u64>, Option<String>);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(NodeLiteVisitor {
                id: None,
                parent: None,
                test_id: None,
            })
        }
    }

    struct NodeLiteVisitor {
        id: Option<u64>,
        parent: Option<u64>,
        test_id: Option<String>,
    }

    impl<'de> Visitor<'de> for NodeLiteVisitor {
        type Value = (Option<u64>, Option<u64>, Option<String>);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "node object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "id" => self.id = map.next_value::<Option<u64>>()?,
                    "parent" => self.parent = map.next_value::<Option<u64>>()?,
                    "test_id" => self.test_id = map.next_value::<Option<String>>()?,
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((self.id, self.parent, self.test_id))
        }
    }

    let total = wanted.values().map(|s| s.len()).sum::<usize>();
    let state = std::rc::Rc::new(std::cell::RefCell::new(State {
        wanted: wanted.clone(),
        found: HashMap::new(),
        total,
        test_id: test_id.to_string(),
    }));

    crate::json_stream::with_bundle_json_deserializer_allow_stop(
        bundle_path,
        FOUND_MARKER,
        |de| {
            RootSeed {
                state: state.clone(),
            }
            .deserialize(de)
        },
    )?;

    Ok(state.borrow().found.clone())
}
