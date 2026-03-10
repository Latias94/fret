use std::path::Path;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

use crate::util::{now_unix_ms, write_json_value};

pub(crate) fn check_bundle_for_gc_sweep_liveness_streaming(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let scan = scan_gc_sweep_liveness_streaming(bundle_path, warmup_frames)?;

    // Always write evidence so debugging doesn't require re-running the harness.
    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.gc_sweep_liveness.json");
    let (bundle_artifact, bundle_json) = super::bundle_artifact_alias_pair(bundle_path);
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "gc_sweep_liveness",
        "bundle_artifact": bundle_artifact,
        "bundle_json": bundle_json,
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": scan.examined_snapshots,
        "removed_subtrees_total": scan.removed_subtrees_total,
        "removed_subtrees_offenders": scan.removed_subtrees_offenders,
        "offender_taxonomy_counts": scan.offender_taxonomy_counts,
        "offender_samples": scan.offender_samples,
        "debug_summary": {
            "element_runtime_node_entry_root_overwrites_total": scan.element_runtime_node_entry_root_overwrites_total,
            "element_runtime_view_cache_reuse_root_element_samples_total": scan.element_runtime_view_cache_reuse_root_element_samples_total,
            "element_runtime_retained_keep_alive_roots_total": scan.element_runtime_retained_keep_alive_roots_total,
        },
    });
    write_json_value(&evidence_path, &payload)?;

    if scan.offenders.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("GC sweep liveness violation: removed_subtrees contains entries that appear live or inconsistent with keep-alive/reuse bookkeeping\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "warmup_frames={warmup_frames} examined_snapshots={}\n",
        scan.examined_snapshots
    ));
    msg.push_str(&format!("evidence: {}\n", evidence_path.display()));
    for line in scan.offenders.into_iter().take(10) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

#[derive(Debug, Default, Clone)]
struct GcSweepLivenessScan {
    offenders: Vec<String>,
    offender_samples: Vec<serde_json::Value>,
    offender_taxonomy_counts: std::collections::BTreeMap<String, u64>,
    examined_snapshots: u64,
    removed_subtrees_total: u64,
    removed_subtrees_offenders: u64,

    element_runtime_node_entry_root_overwrites_total: u64,
    element_runtime_view_cache_reuse_root_element_samples_total: u64,
    element_runtime_retained_keep_alive_roots_total: u64,
}

fn scan_gc_sweep_liveness_streaming(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<GcSweepLivenessScan, String> {
    #[derive(Debug, Default)]
    struct SnapshotGcData {
        tick_id: u64,
        frame_id: u64,
        removed_subtrees: Vec<serde_json::Value>,
        node_entry_root_overwrites_len: u64,
        view_cache_reuse_root_element_samples_len: u64,
        retained_keep_alive_roots_len: u64,
    }

    #[derive(Debug, Default)]
    struct State {
        warmup_frames: u64,
        scan: GcSweepLivenessScan,
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
                        map.next_value_seed(SnapshotsSeed {
                            window_id,
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
                state: self.state,
            })
        }
    }

    struct SnapshotsVisitor {
        window_id: u64,
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
            while let Some(snapshot) = seq.next_element_seed(SnapshotSeed)? {
                let warmup_frames = self.state.borrow().warmup_frames;
                if snapshot.frame_id < warmup_frames {
                    continue;
                }

                let mut st = self.state.borrow_mut();
                let scan = &mut st.scan;

                scan.examined_snapshots = scan.examined_snapshots.saturating_add(1);

                scan.element_runtime_node_entry_root_overwrites_total = scan
                    .element_runtime_node_entry_root_overwrites_total
                    .saturating_add(snapshot.node_entry_root_overwrites_len);
                scan.element_runtime_view_cache_reuse_root_element_samples_total = scan
                    .element_runtime_view_cache_reuse_root_element_samples_total
                    .saturating_add(snapshot.view_cache_reuse_root_element_samples_len);
                scan.element_runtime_retained_keep_alive_roots_total = scan
                    .element_runtime_retained_keep_alive_roots_total
                    .saturating_add(snapshot.retained_keep_alive_roots_len);

                process_removed_subtrees_for_snapshot(
                    scan,
                    self.window_id,
                    snapshot.tick_id,
                    snapshot.frame_id,
                    snapshot.node_entry_root_overwrites_len,
                    snapshot.view_cache_reuse_root_element_samples_len,
                    snapshot.retained_keep_alive_roots_len,
                    snapshot.removed_subtrees.as_slice(),
                );
            }
            Ok(())
        }
    }

    struct SnapshotSeed;

    impl<'de> DeserializeSeed<'de> for SnapshotSeed {
        type Value = SnapshotGcData;

        fn deserialize<D>(self, deserializer: D) -> Result<SnapshotGcData, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotVisitor {
                out: SnapshotGcData::default(),
            })
        }
    }

    struct SnapshotVisitor {
        out: SnapshotGcData,
    }

    impl<'de> Visitor<'de> for SnapshotVisitor {
        type Value = SnapshotGcData;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshot object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<SnapshotGcData, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "tick_id" | "tickId" => self.out.tick_id = map.next_value::<u64>()?,
                    "frame_id" | "frameId" => self.out.frame_id = map.next_value::<u64>()?,
                    "debug" => {
                        let (
                            removed,
                            node_entry_overwrites_len,
                            reuse_root_samples_len,
                            keep_alive_roots_len,
                        ) = map.next_value_seed(DebugSeed)?;
                        self.out.removed_subtrees = removed;
                        self.out.node_entry_root_overwrites_len = node_entry_overwrites_len;
                        self.out.view_cache_reuse_root_element_samples_len = reuse_root_samples_len;
                        self.out.retained_keep_alive_roots_len = keep_alive_roots_len;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(self.out)
        }
    }

    struct DebugSeed;

    impl<'de> DeserializeSeed<'de> for DebugSeed {
        type Value = (Vec<serde_json::Value>, u64, u64, u64);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugVisitor {
                removed: Vec::new(),
                node_entry_root_overwrites_len: 0,
                view_cache_reuse_root_element_samples_len: 0,
                retained_keep_alive_roots_len: 0,
            })
        }
    }

    struct DebugVisitor {
        removed: Vec<serde_json::Value>,
        node_entry_root_overwrites_len: u64,
        view_cache_reuse_root_element_samples_len: u64,
        retained_keep_alive_roots_len: u64,
    }

    impl<'de> Visitor<'de> for DebugVisitor {
        type Value = (Vec<serde_json::Value>, u64, u64, u64);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "removed_subtrees" => {
                        self.removed = map.next_value::<Vec<serde_json::Value>>()?;
                    }
                    "element_runtime" => {
                        let (a, b, c) = map.next_value_seed(ElementRuntimeSeed)?;
                        self.node_entry_root_overwrites_len = a;
                        self.view_cache_reuse_root_element_samples_len = b;
                        self.retained_keep_alive_roots_len = c;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((
                self.removed,
                self.node_entry_root_overwrites_len,
                self.view_cache_reuse_root_element_samples_len,
                self.retained_keep_alive_roots_len,
            ))
        }
    }

    struct ElementRuntimeSeed;

    impl<'de> DeserializeSeed<'de> for ElementRuntimeSeed {
        type Value = (u64, u64, u64);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(ElementRuntimeVisitor {
                node_entry_root_overwrites_len: 0,
                view_cache_reuse_root_element_samples_len: 0,
                retained_keep_alive_roots_len: 0,
            })
        }
    }

    struct ElementRuntimeVisitor {
        node_entry_root_overwrites_len: u64,
        view_cache_reuse_root_element_samples_len: u64,
        retained_keep_alive_roots_len: u64,
    }

    impl<'de> Visitor<'de> for ElementRuntimeVisitor {
        type Value = (u64, u64, u64);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "element_runtime object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "node_entry_root_overwrites" => {
                        self.node_entry_root_overwrites_len =
                            map.next_value_seed(CountArrayLenSeed)? as u64;
                    }
                    "view_cache_reuse_root_element_samples" => {
                        self.view_cache_reuse_root_element_samples_len =
                            map.next_value_seed(CountArrayLenSeed)? as u64;
                    }
                    "retained_keep_alive_roots" => {
                        self.retained_keep_alive_roots_len =
                            map.next_value_seed(CountArrayLenSeed)? as u64;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((
                self.node_entry_root_overwrites_len,
                self.view_cache_reuse_root_element_samples_len,
                self.retained_keep_alive_roots_len,
            ))
        }
    }

    struct CountArrayLenSeed;

    impl<'de> DeserializeSeed<'de> for CountArrayLenSeed {
        type Value = usize;

        fn deserialize<D>(self, deserializer: D) -> Result<usize, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(CountArrayLenVisitor { len: 0 })
        }
    }

    struct CountArrayLenVisitor {
        len: usize,
    }

    impl<'de> Visitor<'de> for CountArrayLenVisitor {
        type Value = usize;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "array or null")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<usize, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while seq.next_element::<IgnoredAny>()?.is_some() {
                self.len += 1;
            }
            Ok(self.len)
        }

        fn visit_unit<E>(self) -> Result<usize, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_none<E>(self) -> Result<usize, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }
    }

    let state = std::rc::Rc::new(std::cell::RefCell::new(State {
        warmup_frames,
        scan: GcSweepLivenessScan::default(),
    }));

    crate::json_stream::with_bundle_json_deserializer(bundle_path, |de| {
        RootSeed {
            state: state.clone(),
        }
        .deserialize(de)
    })?;

    Ok(state.borrow().scan.clone())
}

fn process_removed_subtrees_for_snapshot(
    scan: &mut GcSweepLivenessScan,
    window_id: u64,
    tick_id: u64,
    frame_id: u64,
    snapshot_node_entry_root_overwrites_len: u64,
    snapshot_view_cache_reuse_root_element_samples_len: u64,
    snapshot_retained_keep_alive_roots_len: u64,
    removed: &[serde_json::Value],
) {
    for r in removed {
        scan.removed_subtrees_total = scan.removed_subtrees_total.saturating_add(1);

        let unreachable = r
            .get("unreachable_from_liveness_roots")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let reachable_from_layer_roots = r
            .get("reachable_from_layer_roots")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let reachable_from_view_cache_roots = r
            .get("reachable_from_view_cache_roots")
            .and_then(|v| v.as_bool());
        let root_layer_visible = r.get("root_layer_visible").and_then(|v| v.as_bool());
        let reuse_roots_len = r
            .get("view_cache_reuse_roots_len")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let under_reuse = reuse_roots_len > 0;
        let reuse_root_nodes_len = r
            .get("view_cache_reuse_root_nodes_len")
            .and_then(|v| v.as_u64());
        let trigger_in_keep_alive = r
            .get("trigger_element_in_view_cache_keep_alive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let trigger_listed_under_reuse_root = r
            .get("trigger_element_listed_under_reuse_root")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let taxonomy = if unreachable && !reachable_from_layer_roots {
            "swept_while_unreachable"
        } else if !unreachable && reachable_from_layer_roots {
            "swept_while_reachable"
        } else if unreachable && reachable_from_layer_roots {
            "unreachable_but_in_layer_roots"
        } else {
            "reachable_but_not_in_layer_roots"
        };

        let mut taxonomy_flags: Vec<&'static str> = Vec::new();
        if under_reuse {
            taxonomy_flags.push("under_reuse");
        }
        if trigger_in_keep_alive {
            taxonomy_flags.push("trigger_in_keep_alive");
        }
        if trigger_listed_under_reuse_root > 0 {
            taxonomy_flags.push("trigger_listed_under_reuse_root");
        }

        *scan
            .offender_taxonomy_counts
            .entry(taxonomy.to_string())
            .or_insert(0) += 1;

        // This matches the existing gate's offender criteria.
        let is_offender = taxonomy != "swept_while_unreachable"
            || trigger_in_keep_alive
            || trigger_listed_under_reuse_root > 0;

        if !is_offender {
            continue;
        }

        scan.removed_subtrees_offenders = scan.removed_subtrees_offenders.saturating_add(1);

        let root = r.get("root").and_then(|v| v.as_u64()).unwrap_or(0);
        let root_element_path = r
            .get("root_element_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let trigger_path = r
            .get("trigger_element_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let mut violations: Vec<&'static str> = Vec::new();
        if taxonomy == "swept_while_reachable" && !unreachable {
            violations.push("unreachable_from_liveness_roots=false");
        }
        if taxonomy == "swept_while_reachable" && reachable_from_layer_roots {
            violations.push("reachable_from_layer_roots=true");
        }
        if taxonomy == "swept_while_reachable" && reachable_from_view_cache_roots == Some(true) {
            violations.push("reachable_from_view_cache_roots=true");
        }
        if taxonomy == "swept_while_reachable" && root_layer_visible == Some(true) {
            violations.push("root_layer_visible=true");
        }

        scan.offenders.push(format!(
            "window={window_id} frame_id={frame_id} taxonomy={taxonomy} root={root} unreachable_from_liveness_roots={unreachable} reachable_from_layer_roots={reachable_from_layer_roots} reachable_from_view_cache_roots={reachable_from_view_cache_roots:?} root_layer_visible={root_layer_visible:?} reuse_roots_len={reuse_roots_len} reuse_root_nodes_len={reuse_root_nodes_len:?} trigger_in_keep_alive={trigger_in_keep_alive} trigger_listed_under_reuse_root={trigger_listed_under_reuse_root} root_element_path={root_element_path} trigger_element_path={trigger_path}"
        ));

        const MAX_SAMPLES: usize = 128;
        if scan.offender_samples.len() < MAX_SAMPLES {
            scan.offender_samples.push(serde_json::json!({
                "window": window_id,
                "frame_id": frame_id,
                "tick_id": tick_id,
                "taxonomy": taxonomy,
                "taxonomy_flags": taxonomy_flags,
                "root": r.get("root").and_then(|v| v.as_u64()).unwrap_or(0),
                "root_root": r.get("root_root").and_then(|v| v.as_u64()),
                "root_layer": r.get("root_layer").and_then(|v| v.as_u64()),
                "root_layer_visible": root_layer_visible,
                "reachable_from_layer_roots": reachable_from_layer_roots,
                "reachable_from_view_cache_roots": reachable_from_view_cache_roots,
                "unreachable_from_liveness_roots": unreachable,
                "violations": violations,
                "reuse_roots_len": reuse_roots_len,
                "reuse_root_nodes_len": reuse_root_nodes_len,
                "trigger_in_keep_alive": trigger_in_keep_alive,
                "trigger_listed_under_reuse_root": trigger_listed_under_reuse_root,
                "liveness_layer_roots_len": r.get("liveness_layer_roots_len").and_then(|v| v.as_u64()),
                "view_cache_reuse_roots_len": r.get("view_cache_reuse_roots_len").and_then(|v| v.as_u64()),
                "view_cache_reuse_root_nodes_len": r.get("view_cache_reuse_root_nodes_len").and_then(|v| v.as_u64()),
                "snapshot_node_entry_root_overwrites_len": snapshot_node_entry_root_overwrites_len,
                "snapshot_view_cache_reuse_root_element_samples_len": snapshot_view_cache_reuse_root_element_samples_len,
                "snapshot_retained_keep_alive_roots_len": snapshot_retained_keep_alive_roots_len,
                "root_element": r.get("root_element").and_then(|v| v.as_u64()),
                "root_element_path": r.get("root_element_path").and_then(|v| v.as_str()),
                "trigger_element": r.get("trigger_element").and_then(|v| v.as_u64()),
                "trigger_element_path": r.get("trigger_element_path").and_then(|v| v.as_str()),
                "trigger_element_in_view_cache_keep_alive": r.get("trigger_element_in_view_cache_keep_alive").and_then(|v| v.as_bool()),
                "trigger_element_listed_under_reuse_root": r.get("trigger_element_listed_under_reuse_root").and_then(|v| v.as_u64()),
                "root_root_parent_sever_parent": r.get("root_root_parent_sever_parent").and_then(|v| v.as_u64()),
                "root_root_parent_sever_location": r.get("root_root_parent_sever_location").and_then(|v| v.as_str()),
                "root_root_parent_sever_frame_id": r.get("root_root_parent_sever_frame_id").and_then(|v| v.as_u64()),
            }));
        }
    }
}
