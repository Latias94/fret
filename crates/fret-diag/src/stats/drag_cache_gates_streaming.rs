use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

#[derive(Debug, Clone, Default)]
struct SemanticsLite {
    target_node_id: Option<u64>,
    parents: HashMap<u64, u64>,
}

fn semantics_lite_from_nodes(nodes: &[serde_json::Value], test_id: &str) -> SemanticsLite {
    let mut out = SemanticsLite::default();
    for node in nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        if let Some(parent) = node.get("parent").and_then(|v| v.as_u64()) {
            out.parents.insert(id, parent);
        }
        if out.target_node_id.is_none()
            && node
                .get("test_id")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s == test_id)
        {
            out.target_node_id = Some(id);
        }
    }
    out
}

#[derive(Debug, Clone, Default)]
struct CacheRootLite {
    reused: bool,
    contained_relayout_in_frame: bool,
}

#[derive(Debug, Clone, Default)]
struct Out {
    examined_snapshots: u64,
    good_frames: u64,
    missing_target_count: u64,
    any_view_cache_active: bool,
    seen_good: bool,
    bad_frames_total: u64,
    bad_frames_sample: Vec<String>,
    fatal_error: Option<String>,
}

#[derive(Debug, Clone)]
struct Cfg {
    bundle_path: PathBuf,
    test_id: String,
    warmup_frames: u64,
}

#[derive(Debug)]
struct State {
    cfg: Cfg,
    out: Out,
    table_sem_cache: HashMap<(u64, u64), Option<Rc<SemanticsLite>>>,
}

impl State {
    fn is_done(&self) -> bool {
        self.out.fatal_error.is_some()
            || (self.out.seen_good && self.out.bad_frames_sample.len() >= 10)
    }

    fn resolve_table_semantics_lite(
        &mut self,
        semantics_window_id: u64,
        semantics_fingerprint: u64,
    ) -> Result<Option<Rc<SemanticsLite>>, String> {
        if let Some(cached) = self
            .table_sem_cache
            .get(&(semantics_window_id, semantics_fingerprint))
            .cloned()
        {
            return Ok(cached);
        }

        let nodes = crate::json_bundle::stream_read_semantics_table_nodes(
            &self.cfg.bundle_path,
            semantics_window_id,
            semantics_fingerprint,
        )?;
        let sem = nodes.map(|nodes| Rc::new(semantics_lite_from_nodes(&nodes, &self.cfg.test_id)));
        self.table_sem_cache
            .insert((semantics_window_id, semantics_fingerprint), sem.clone());
        Ok(sem)
    }

    fn process_snapshot(
        &mut self,
        window_id: u64,
        frame_id: u64,
        semantics_window_id: u64,
        semantics_fingerprint: Option<u64>,
        view_cache_active: bool,
        inline_sem: Option<SemanticsLite>,
        cache_roots: Option<HashMap<u64, CacheRootLite>>,
        dirty_roots: &HashSet<u64>,
    ) -> Result<(), String> {
        if frame_id < self.cfg.warmup_frames {
            return Ok(());
        }

        self.out.examined_snapshots = self.out.examined_snapshots.saturating_add(1);

        self.out.any_view_cache_active |= view_cache_active;
        if !view_cache_active {
            return Ok(());
        }

        let sem = if let Some(inline) = inline_sem {
            Some(Rc::new(inline))
        } else if let Some(fp) = semantics_fingerprint {
            self.resolve_table_semantics_lite(semantics_window_id, fp)?
        } else {
            None
        };

        let Some(sem) = sem else {
            self.out.missing_target_count = self.out.missing_target_count.saturating_add(1);
            return Ok(());
        };

        let Some(target_node_id) = sem.target_node_id else {
            self.out.missing_target_count = self.out.missing_target_count.saturating_add(1);
            return Ok(());
        };

        let cache_roots = cache_roots
            .ok_or_else(|| "invalid bundle artifact: missing debug.cache_roots".to_string())?;

        let mut current = target_node_id;
        let mut cache_root_node: Option<u64> = None;
        loop {
            if cache_roots.contains_key(&current) {
                cache_root_node = Some(current);
                break;
            }
            let Some(parent) = sem.parents.get(&current).copied() else {
                break;
            };
            current = parent;
        }

        let Some(cache_root_node) = cache_root_node else {
            return Err(format!(
                "could not resolve a cache root ancestor for test_id={} (node_id={}) in bundle: {}",
                self.cfg.test_id,
                target_node_id,
                self.cfg.bundle_path.display()
            ));
        };

        let root = cache_roots
            .get(&cache_root_node)
            .ok_or_else(|| "internal error: cache root missing".to_string())?;

        let dirty = dirty_roots.contains(&cache_root_node);
        let ok = root.reused && !root.contained_relayout_in_frame && !dirty;
        if ok {
            self.out.good_frames = self.out.good_frames.saturating_add(1);
            self.out.seen_good = true;
            return Ok(());
        }

        if self.out.seen_good {
            self.out.bad_frames_total = self.out.bad_frames_total.saturating_add(1);
            if self.out.bad_frames_sample.len() < 10 {
                self.out.bad_frames_sample.push(format!(
                    "window={window_id} frame_id={frame_id} cache_root={cache_root_node} reused={} contained_relayout_in_frame={} dirty={dirty}",
                    root.reused, root.contained_relayout_in_frame
                ));
            }
        }

        Ok(())
    }
}

pub(crate) fn check_bundle_for_drag_cache_root_paint_only_streaming(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    const STOP_MARKER: &str = "__FRET_DIAG_DRAG_CACHE_STOP__";

    struct RootSeed {
        state: Rc<std::cell::RefCell<State>>,
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
        state: Rc<std::cell::RefCell<State>>,
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
        state: Rc<std::cell::RefCell<State>>,
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
        state: Rc<std::cell::RefCell<State>>,
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
        state: Rc<std::cell::RefCell<State>>,
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
        state: Rc<std::cell::RefCell<State>>,
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
                        window_id = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "snapshots" => {
                        map.next_value_seed(SnapshotsSeed {
                            state: self.state.clone(),
                            window_id_default: window_id,
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
        state: Rc<std::cell::RefCell<State>>,
        window_id_default: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotsSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(SnapshotsVisitor {
                state: self.state,
                window_id_default: self.window_id_default,
            })
        }
    }

    struct SnapshotsVisitor {
        state: Rc<std::cell::RefCell<State>>,
        window_id_default: u64,
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
            while seq
                .next_element_seed(SnapshotSeed {
                    state: self.state.clone(),
                    window_id_default: self.window_id_default,
                })?
                .is_some()
            {
                let done = { self.state.borrow().is_done() };
                if done {
                    return Err(serde::de::Error::custom(STOP_MARKER));
                }
            }
            Ok(())
        }
    }

    struct SnapshotSeed {
        state: Rc<std::cell::RefCell<State>>,
        window_id_default: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotVisitor {
                state: self.state,
                window_id_default: self.window_id_default,
                frame_id: 0,
                semantics_fingerprint: None,
                semantics_window_id: self.window_id_default,
                view_cache_active: false,
                cache_roots: None,
                dirty_roots: HashSet::new(),
                inline_sem: None,
            })
        }
    }

    struct SnapshotVisitor {
        state: Rc<std::cell::RefCell<State>>,
        window_id_default: u64,
        frame_id: u64,
        semantics_fingerprint: Option<u64>,
        semantics_window_id: u64,
        view_cache_active: bool,
        cache_roots: Option<HashMap<u64, CacheRootLite>>,
        dirty_roots: HashSet<u64>,
        inline_sem: Option<SemanticsLite>,
    }

    impl<'de> Visitor<'de> for SnapshotVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshot object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "frame_id" | "frameId" => {
                        self.frame_id = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "window" | "window_id" | "windowId" => {
                        self.semantics_window_id = map
                            .next_value::<Option<u64>>()?
                            .unwrap_or(self.window_id_default);
                    }
                    "semantics_fingerprint" | "semanticsFingerprint" => {
                        self.semantics_fingerprint = map.next_value::<Option<u64>>()?;
                    }
                    "debug" => {
                        let debug = map.next_value_seed(DebugSeed {
                            test_id: self.state.borrow().cfg.test_id.clone(),
                        })?;
                        self.view_cache_active = debug.view_cache_active;
                        self.cache_roots = debug.cache_roots;
                        self.dirty_roots = debug.dirty_roots;
                        if self.inline_sem.is_none() {
                            self.inline_sem = debug.semantics;
                        }
                    }
                    "semantics" | "semantic_tree" | "semanticTree" | "tree" => {
                        if self.inline_sem.is_none() {
                            self.inline_sem = map.next_value_seed(SemanticsLiteSeed {
                                test_id: self.state.borrow().cfg.test_id.clone(),
                            })?;
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            let window_id = self.semantics_window_id;
            let res = self.state.borrow_mut().process_snapshot(
                window_id,
                self.frame_id,
                self.semantics_window_id,
                self.semantics_fingerprint,
                self.view_cache_active,
                self.inline_sem,
                self.cache_roots,
                &self.dirty_roots,
            );

            if let Err(err) = res {
                self.state.borrow_mut().out.fatal_error = Some(err);
            }

            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct DebugLite {
        view_cache_active: bool,
        cache_roots: Option<HashMap<u64, CacheRootLite>>,
        dirty_roots: HashSet<u64>,
        semantics: Option<SemanticsLite>,
    }

    struct DebugSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for DebugSeed {
        type Value = DebugLite;

        fn deserialize<D>(self, deserializer: D) -> Result<DebugLite, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugVisitor {
                test_id: self.test_id,
                out: DebugLite::default(),
            })
        }
    }

    struct DebugVisitor {
        test_id: String,
        out: DebugLite,
    }

    impl<'de> Visitor<'de> for DebugVisitor {
        type Value = DebugLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<DebugLite, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "stats" => {
                        self.out.view_cache_active = map.next_value_seed(StatsSeed)?;
                    }
                    "cache_roots" => {
                        self.out.cache_roots = Some(map.next_value_seed(CacheRootsSeed)?);
                    }
                    "dirty_views" => {
                        self.out.dirty_roots = map.next_value_seed(DirtyViewsSeed)?;
                    }
                    "semantics" => {
                        self.out.semantics = map.next_value_seed(SemanticsLiteSeed {
                            test_id: self.test_id.clone(),
                        })?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(self.out)
        }
    }

    struct StatsSeed;

    impl<'de> DeserializeSeed<'de> for StatsSeed {
        type Value = bool;

        fn deserialize<D>(self, deserializer: D) -> Result<bool, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(StatsVisitor {
                view_cache_active: false,
            })
        }
    }

    struct StatsVisitor {
        view_cache_active: bool,
    }

    impl<'de> Visitor<'de> for StatsVisitor {
        type Value = bool;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "stats object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<bool, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "view_cache_active" | "viewCacheActive" => {
                        self.view_cache_active = map.next_value::<Option<bool>>()?.unwrap_or(false);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(self.view_cache_active)
        }
    }

    struct CacheRootsSeed;

    impl<'de> DeserializeSeed<'de> for CacheRootsSeed {
        type Value = HashMap<u64, CacheRootLite>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(CacheRootsVisitor {
                out: HashMap::new(),
            })
        }
    }

    struct CacheRootsVisitor {
        out: HashMap<u64, CacheRootLite>,
    }

    impl<'de> Visitor<'de> for CacheRootsVisitor {
        type Value = HashMap<u64, CacheRootLite>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "cache_roots array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some((root, lite)) = seq.next_element_seed(CacheRootSeed)? {
                let Some(root) = root else {
                    continue;
                };
                self.out.insert(root, lite);
            }
            Ok(self.out)
        }
    }

    struct CacheRootSeed;

    impl<'de> DeserializeSeed<'de> for CacheRootSeed {
        type Value = (Option<u64>, CacheRootLite);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(CacheRootVisitor {
                root: None,
                reused: false,
                contained_relayout_in_frame: false,
            })
        }
    }

    struct CacheRootVisitor {
        root: Option<u64>,
        reused: bool,
        contained_relayout_in_frame: bool,
    }

    impl<'de> Visitor<'de> for CacheRootVisitor {
        type Value = (Option<u64>, CacheRootLite);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "cache root object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "root" => {
                        self.root = map.next_value::<Option<u64>>()?;
                    }
                    "reused" => {
                        self.reused = map.next_value::<Option<bool>>()?.unwrap_or(false);
                    }
                    "contained_relayout_in_frame" | "containedRelayoutInFrame" => {
                        self.contained_relayout_in_frame =
                            map.next_value::<Option<bool>>()?.unwrap_or(false);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            Ok((
                self.root,
                CacheRootLite {
                    reused: self.reused,
                    contained_relayout_in_frame: self.contained_relayout_in_frame,
                },
            ))
        }
    }

    struct DirtyViewsSeed;

    impl<'de> DeserializeSeed<'de> for DirtyViewsSeed {
        type Value = HashSet<u64>;

        fn deserialize<D>(self, deserializer: D) -> Result<HashSet<u64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(DirtyViewsVisitor {
                out: HashSet::new(),
            })
        }
    }

    struct DirtyViewsVisitor {
        out: HashSet<u64>,
    }

    impl<'de> Visitor<'de> for DirtyViewsVisitor {
        type Value = HashSet<u64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "dirty_views array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<HashSet<u64>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some(root) = seq.next_element_seed(DirtyViewSeed)? {
                let Some(root) = root else {
                    continue;
                };
                self.out.insert(root);
            }
            Ok(self.out)
        }
    }

    struct DirtyViewSeed;

    impl<'de> DeserializeSeed<'de> for DirtyViewSeed {
        type Value = Option<u64>;

        fn deserialize<D>(self, deserializer: D) -> Result<Option<u64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DirtyViewVisitor { root: None })
        }
    }

    struct DirtyViewVisitor {
        root: Option<u64>,
    }

    impl<'de> Visitor<'de> for DirtyViewVisitor {
        type Value = Option<u64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "dirty view object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<u64>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "root_node" | "rootNode" => {
                        self.root = map.next_value::<Option<u64>>()?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(self.root)
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
                    "id" => {
                        self.id = map.next_value::<Option<u64>>()?;
                    }
                    "parent" => {
                        self.parent = map.next_value::<Option<u64>>()?;
                    }
                    "test_id" | "testId" => {
                        self.test_id = map.next_value::<Option<String>>()?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((self.id, self.parent, self.test_id))
        }
    }

    let state: Rc<std::cell::RefCell<State>> = Rc::new(std::cell::RefCell::new(State {
        cfg: Cfg {
            bundle_path: bundle_path.to_path_buf(),
            test_id: test_id.to_string(),
            warmup_frames,
        },
        out: Out::default(),
        table_sem_cache: HashMap::new(),
    }));
    crate::json_stream::with_bundle_json_deserializer_allow_stop(bundle_path, STOP_MARKER, |de| {
        RootSeed {
            state: state.clone(),
        }
        .deserialize(de)
    })?;

    let state = state.borrow();
    if let Some(err) = state.out.fatal_error.as_deref() {
        return Err(err.to_string());
    }

    if state.out.bad_frames_total > 0 {
        let mut msg = String::new();
        msg.push_str("expected paint-only drag indicator updates (cache-root reuse, no contained relayout, no dirty view), but found violations after reuse began\n");
        msg.push_str(&format!("bundle: {}\n", state.cfg.bundle_path.display()));
        msg.push_str(&format!("test_id: {}\n", state.cfg.test_id));
        for line in state.out.bad_frames_sample.iter().take(10) {
            msg.push_str("  ");
            msg.push_str(line);
            msg.push('\n');
        }
        return Err(msg);
    }

    if state.out.good_frames == 0 {
        return Err(format!(
            "did not observe any cache-root-reuse paint-only frames for test_id={} \
(any_view_cache_active={}, warmup_frames={}, examined_snapshots={}, missing_target_count={}) \
in bundle: {}",
            state.cfg.test_id,
            state.out.any_view_cache_active,
            state.cfg.warmup_frames,
            state.out.examined_snapshots,
            state.out.missing_target_count,
            state.cfg.bundle_path.display()
        ));
    }

    Ok(())
}
