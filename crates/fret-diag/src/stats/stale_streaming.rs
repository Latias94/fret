use std::path::Path;

use serde::de::{DeserializeSeed, Error as _, IgnoredAny, MapAccess, SeqAccess, Visitor};

use crate::util::write_json_value;

use super::semantics::{semantics_diff_detail_nodes, semantics_diff_summary_nodes};
use super::stale::SemanticsChangedRepaintedScan;

#[derive(Debug, Clone, Copy)]
struct SnapshotMeta {
    tick_id: u64,
    frame_id: u64,
    scene_fingerprint: u64,
    semantics_fingerprint: u64,
    semantics_window_id: u64,
    paint_nodes_performed: u64,
    paint_cache_replayed_ops: u64,
}

#[derive(Debug, Clone, Copy)]
struct SuspiciousPair {
    window_id: u64,
    prev: SnapshotMeta,
    now: SnapshotMeta,
}

pub(crate) fn check_bundle_for_semantics_changed_repainted_streaming(
    bundle_path: &Path,
    warmup_frames: u64,
    dump_json: bool,
) -> Result<(), String> {
    let mut scan = scan_semantics_changed_repainted_streaming(bundle_path, warmup_frames)?;

    // Re-scan pairs from findings for enrichment (findings are stable and machine-readable).
    enrich_semantics_changed_repainted_findings(bundle_path, &mut scan)?;

    if dump_json && !scan.findings.is_empty() {
        let out_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
        let out_path = out_dir.join("check.semantics_changed_repainted.json");
        let payload = serde_json::json!({
            "schema_version": 1,
            "kind": "semantics_changed_repainted",
            "bundle_artifact": bundle_path.display().to_string(),
            "bundle_json": bundle_path.display().to_string(),
            "warmup_frames": warmup_frames,
            "findings": scan.findings,
        });
        let _ = write_json_value(&out_path, &payload);
    }

    if scan.missing_scene_fingerprint {
        return Err(format!(
            "semantics repaint check requires `scene_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if scan.missing_semantics_fingerprint {
        return Err(format!(
            "semantics repaint check requires `semantics_fingerprint` in snapshots (re-run the script with a newer target build): {}",
            bundle_path.display()
        ));
    }

    if scan.suspicious_lines.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "missing repaint suspected (semantics fingerprint changed but scene fingerprint did not)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in scan.suspicious_lines {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

fn scan_semantics_changed_repainted_streaming(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<SemanticsChangedRepaintedScan, String> {
    const STOP_MARKER: &str = "__FRET_DIAG_STOP_SEM_REPAINT_SCAN__";
    const MAX_SUSPICIOUS: usize = 8;

    #[derive(Debug)]
    struct State {
        warmup_frames: u64,
        scan: SemanticsChangedRepaintedScan,
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
            let mut prev_scene: Option<u64> = None;
            let mut prev_sem: Option<u64> = None;
            let mut prev_tick: u64 = 0;
            let mut prev_frame: u64 = 0;
            let mut prev_sem_window_id: u64 = self.window_id;

            while let Some(snap) = seq.next_element_seed(SnapshotMinSeed {
                window_id_default: self.window_id,
            })? {
                let warmup_frames = self.state.borrow().warmup_frames;

                if snap.frame_id < warmup_frames {
                    continue;
                }

                if snap.scene_fingerprint.is_none() {
                    self.state.borrow_mut().scan.missing_scene_fingerprint = true;
                }
                if snap.semantics_fingerprint.is_none() {
                    self.state.borrow_mut().scan.missing_semantics_fingerprint = true;
                }

                let (Some(scene_fp), Some(sem_fp)) = (snap.scene_fingerprint, snap.semantics_fingerprint) else {
                    prev_scene = None;
                    prev_sem = None;
                    prev_tick = snap.tick_id;
                    prev_frame = snap.frame_id;
                    prev_sem_window_id = snap.semantics_window_id;
                    continue;
                };

                if let (Some(prev_scene_fp), Some(prev_sem_fp)) = (prev_scene, prev_sem) {
                    let semantics_changed = sem_fp != prev_sem_fp;
                    let scene_unchanged = scene_fp == prev_scene_fp;
                    if semantics_changed && scene_unchanged {
                        let pair = SuspiciousPair {
                            window_id: self.window_id,
                            prev: SnapshotMeta {
                                tick_id: prev_tick,
                                frame_id: prev_frame,
                                scene_fingerprint: prev_scene_fp,
                                semantics_fingerprint: prev_sem_fp,
                                semantics_window_id: prev_sem_window_id,
                                paint_nodes_performed: 0,
                                paint_cache_replayed_ops: 0,
                            },
                            now: SnapshotMeta {
                                tick_id: snap.tick_id,
                                frame_id: snap.frame_id,
                                scene_fingerprint: scene_fp,
                                semantics_fingerprint: sem_fp,
                                semantics_window_id: snap.semantics_window_id,
                                paint_nodes_performed: snap.paint_nodes_performed,
                                paint_cache_replayed_ops: snap.paint_cache_replayed_ops,
                            },
                        };

                        let mut st = self.state.borrow_mut();
                        let scan = &mut st.scan;

                        // Defer semantics diffs until after the scan.
                        scan.findings.push(serde_json::json!({
                            "window": pair.window_id,
                            "prev": {
                                "tick_id": pair.prev.tick_id,
                                "frame_id": pair.prev.frame_id,
                                "scene_fingerprint": pair.prev.scene_fingerprint,
                                "semantics_fingerprint": pair.prev.semantics_fingerprint,
                                "semantics_window_id": pair.prev.semantics_window_id,
                            },
                            "now": {
                                "tick_id": pair.now.tick_id,
                                "frame_id": pair.now.frame_id,
                                "scene_fingerprint": pair.now.scene_fingerprint,
                                "semantics_fingerprint": pair.now.semantics_fingerprint,
                                "semantics_window_id": pair.now.semantics_window_id,
                            },
                            "paint_nodes_performed": pair.now.paint_nodes_performed,
                            "paint_cache_replayed_ops": pair.now.paint_cache_replayed_ops,
                            "semantics_diff": serde_json::Value::Null,
                        }));

                        scan.suspicious_lines.push(format!(
                            "window={} tick={} frame={} prev_tick={} prev_frame={} semantics_fingerprint=0x{:016x} prev_semantics_fingerprint=0x{:016x} scene_fingerprint=0x{:016x} paint_nodes_performed={} paint_cache_replayed_ops={}",
                            pair.window_id,
                            pair.now.tick_id,
                            pair.now.frame_id,
                            pair.prev.tick_id,
                            pair.prev.frame_id,
                            pair.now.semantics_fingerprint,
                            pair.prev.semantics_fingerprint,
                            pair.now.scene_fingerprint,
                            pair.now.paint_nodes_performed,
                            pair.now.paint_cache_replayed_ops,
                        ));

                        if scan.suspicious_lines.len() >= MAX_SUSPICIOUS {
                            return Err(A::Error::custom(STOP_MARKER));
                        }
                    }
                }

                prev_scene = Some(scene_fp);
                prev_sem = Some(sem_fp);
                prev_tick = snap.tick_id;
                prev_frame = snap.frame_id;
                prev_sem_window_id = snap.semantics_window_id;
            }

            Ok(())
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct SnapshotMin {
        tick_id: u64,
        frame_id: u64,
        scene_fingerprint: Option<u64>,
        semantics_fingerprint: Option<u64>,
        semantics_window_id: u64,
        paint_nodes_performed: u64,
        paint_cache_replayed_ops: u64,
    }

    struct SnapshotMinSeed {
        window_id_default: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotMinSeed {
        type Value = SnapshotMin;

        fn deserialize<D>(self, deserializer: D) -> Result<SnapshotMin, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotMinVisitor {
                window_id_default: self.window_id_default,
                tick_id: 0,
                frame_id: 0,
                scene_fingerprint: None,
                semantics_fingerprint: None,
                semantics_window_id: None,
                paint_nodes_performed: 0,
                paint_cache_replayed_ops: 0,
            })
        }
    }

    struct SnapshotMinVisitor {
        window_id_default: u64,
        tick_id: u64,
        frame_id: u64,
        scene_fingerprint: Option<u64>,
        semantics_fingerprint: Option<u64>,
        semantics_window_id: Option<u64>,
        paint_nodes_performed: u64,
        paint_cache_replayed_ops: u64,
    }

    impl<'de> Visitor<'de> for SnapshotMinVisitor {
        type Value = SnapshotMin;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshot object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<SnapshotMin, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "tick_id" | "tickId" => self.tick_id = map.next_value::<u64>()?,
                    "frame_id" | "frameId" => self.frame_id = map.next_value::<u64>()?,
                    "scene_fingerprint" | "sceneFingerprint" => {
                        self.scene_fingerprint = map.next_value::<Option<u64>>()?;
                    }
                    "semantics_fingerprint" | "semanticsFingerprint" => {
                        self.semantics_fingerprint = map.next_value::<Option<u64>>()?;
                    }
                    "semantics_window_id" | "semanticsWindowId" => {
                        self.semantics_window_id = map.next_value::<Option<u64>>()?;
                    }
                    "debug" => {
                        let (paint_nodes, paint_cache_ops) =
                            map.next_value_seed(DebugStatsSeed)?;
                        self.paint_nodes_performed = paint_nodes;
                        self.paint_cache_replayed_ops = paint_cache_ops;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            Ok(SnapshotMin {
                tick_id: self.tick_id,
                frame_id: self.frame_id,
                scene_fingerprint: self.scene_fingerprint,
                semantics_fingerprint: self.semantics_fingerprint,
                semantics_window_id: self.semantics_window_id.unwrap_or(self.window_id_default),
                paint_nodes_performed: self.paint_nodes_performed,
                paint_cache_replayed_ops: self.paint_cache_replayed_ops,
            })
        }
    }

    struct DebugStatsSeed;

    impl<'de> DeserializeSeed<'de> for DebugStatsSeed {
        type Value = (u64, u64);

        fn deserialize<D>(self, deserializer: D) -> Result<(u64, u64), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugStatsVisitor {
                paint_nodes_performed: 0,
                paint_cache_replayed_ops: 0,
            })
        }
    }

    struct DebugStatsVisitor {
        paint_nodes_performed: u64,
        paint_cache_replayed_ops: u64,
    }

    impl<'de> Visitor<'de> for DebugStatsVisitor {
        type Value = (u64, u64);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<(u64, u64), M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "stats" {
                    let (paint_nodes, paint_cache_ops) = map.next_value_seed(StatsSeed)?;
                    self.paint_nodes_performed = paint_nodes;
                    self.paint_cache_replayed_ops = paint_cache_ops;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok((self.paint_nodes_performed, self.paint_cache_replayed_ops))
        }
    }

    struct StatsSeed;

    impl<'de> DeserializeSeed<'de> for StatsSeed {
        type Value = (u64, u64);

        fn deserialize<D>(self, deserializer: D) -> Result<(u64, u64), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(StatsVisitor {
                paint_nodes_performed: 0,
                paint_cache_replayed_ops: 0,
            })
        }
    }

    struct StatsVisitor {
        paint_nodes_performed: u64,
        paint_cache_replayed_ops: u64,
    }

    impl<'de> Visitor<'de> for StatsVisitor {
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

    let state = std::rc::Rc::new(std::cell::RefCell::new(State {
        warmup_frames,
        scan: SemanticsChangedRepaintedScan::default(),
    }));

    let res = crate::json_stream::with_bundle_json_deserializer_allow_stop(
        bundle_path,
        STOP_MARKER,
        |de| RootSeed { state: state.clone() }.deserialize(de),
    );
    if let Err(msg) = res {
        return Err(msg);
    }

    Ok(state.borrow().scan.clone())
}

fn enrich_semantics_changed_repainted_findings(
    bundle_path: &Path,
    scan: &mut SemanticsChangedRepaintedScan,
) -> Result<(), String> {
    // The streaming scan pre-populates findings with stable fields and a null semantics_diff. Here
    // we attempt to fill the semantics diff from schema2 semantics tables for each finding.
    for (idx, finding) in scan.findings.iter_mut().enumerate() {
        let Some(prev) = finding.get("prev") else { continue };
        let Some(now) = finding.get("now") else { continue };

        let prev_fp = prev.get("semantics_fingerprint").and_then(|v| v.as_u64());
        let prev_window_id = prev.get("semantics_window_id").and_then(|v| v.as_u64());
        let now_fp = now.get("semantics_fingerprint").and_then(|v| v.as_u64());
        let now_window_id = now.get("semantics_window_id").and_then(|v| v.as_u64());

        let (Some(prev_fp), Some(prev_window_id), Some(now_fp), Some(now_window_id)) =
            (prev_fp, prev_window_id, now_fp, now_window_id)
        else {
            continue;
        };

        let prev_nodes = crate::json_bundle::stream_read_semantics_table_nodes(
            bundle_path,
            prev_window_id,
            prev_fp,
        )?;
        let now_nodes = crate::json_bundle::stream_read_semantics_table_nodes(
            bundle_path,
            now_window_id,
            now_fp,
        )?;

        let (Some(prev_nodes), Some(now_nodes)) = (prev_nodes, now_nodes) else {
            continue;
        };

        let detail = semantics_diff_detail_nodes(prev_nodes.as_slice(), now_nodes.as_slice());
        *finding
            .as_object_mut()
            .expect("finding must be object")
            .entry("semantics_diff")
            .or_insert(serde_json::Value::Null) = detail;

        let summary = semantics_diff_summary_nodes(prev_nodes.as_slice(), now_nodes.as_slice());
        if !summary.is_empty() {
            if let Some(line) = scan.suspicious_lines.get_mut(idx) {
                line.push(' ');
                line.push_str(&summary);
            }
        }
    }

    Ok(())
}
