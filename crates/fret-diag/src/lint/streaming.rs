use std::path::Path;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde_json::Value;

use super::{LintOptions, LintReport};

#[derive(Debug, Clone)]
struct SnapshotMeta {
    window_snapshot_seq: u64,
    frame_id: u64,
    semantics_fingerprint: Option<u64>,
    window_bounds: Value,
}

#[derive(Debug, Clone)]
struct WindowPick {
    window_id: u64,
    candidates: Vec<SnapshotMeta>,
}

pub(super) fn lint_bundle_from_path_streaming(
    bundle_path: &Path,
    warmup_frames: u64,
    opts: LintOptions,
) -> Result<LintReport, String> {
    let picks = collect_window_picks(bundle_path, warmup_frames)?;

    let mut findings: Vec<Value> = Vec::new();

    for w in picks {
        let mut did_lint = false;
        for c in &w.candidates {
            let nodes = try_read_semantics_nodes_for_candidate(bundle_path, w.window_id, c)?;
            let Some(nodes) = nodes else {
                continue;
            };
            if nodes.is_empty() {
                continue;
            }

            super::lint_nodes_for_window(
                &mut findings,
                w.window_id,
                c.frame_id,
                &c.window_bounds,
                nodes.as_slice(),
                None,
                opts,
            );
            did_lint = true;
            break;
        }
        if !did_lint {
            continue;
        }
    }

    Ok(super::finish_lint_report(
        findings,
        bundle_path,
        warmup_frames,
        opts,
    ))
}

fn try_read_semantics_nodes_for_candidate(
    bundle_path: &Path,
    window_id: u64,
    cand: &SnapshotMeta,
) -> Result<Option<Vec<Value>>, String> {
    if let Some(fp) = cand.semantics_fingerprint {
        let nodes =
            crate::json_bundle::stream_read_semantics_table_nodes(bundle_path, window_id, fp)?;
        if let Some(nodes) = nodes
            && !nodes.is_empty()
        {
            return Ok(Some(nodes));
        }
    }

    stream_read_inline_semantics_nodes(bundle_path, window_id, cand.window_snapshot_seq)
}

fn collect_window_picks(bundle_path: &Path, warmup_frames: u64) -> Result<Vec<WindowPick>, String> {
    struct RootSeed {
        warmup_frames: u64,
        out: std::rc::Rc<std::cell::RefCell<Vec<WindowPick>>>,
    }

    impl<'de> DeserializeSeed<'de> for RootSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(RootVisitor {
                warmup_frames: self.warmup_frames,
                out: self.out,
            })
        }
    }

    struct RootVisitor {
        warmup_frames: u64,
        out: std::rc::Rc<std::cell::RefCell<Vec<WindowPick>>>,
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
                            warmup_frames: self.warmup_frames,
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

    struct WindowsSeed {
        warmup_frames: u64,
        out: std::rc::Rc<std::cell::RefCell<Vec<WindowPick>>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowsSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(self)
        }
    }

    impl<'de> Visitor<'de> for WindowsSeed {
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
                    warmup_frames: self.warmup_frames,
                    out: self.out.clone(),
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct WindowSeed {
        warmup_frames: u64,
        out: std::rc::Rc<std::cell::RefCell<Vec<WindowPick>>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(WindowVisitor {
                warmup_frames: self.warmup_frames,
                out: self.out,
            })
        }
    }

    struct WindowVisitor {
        warmup_frames: u64,
        out: std::rc::Rc<std::cell::RefCell<Vec<WindowPick>>>,
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
            let mut window_id: Option<u64> = None;
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "window" => {
                        window_id = Some(map.next_value::<u64>()?);
                    }
                    "snapshots" => {
                        let w = window_id.unwrap_or(0);
                        let pick = map.next_value_seed(SnapshotsSeed {
                            window_id: w,
                            warmup_frames: self.warmup_frames,
                        })?;
                        if !pick.candidates.is_empty() {
                            self.out.borrow_mut().push(pick);
                        }
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
        warmup_frames: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotsSeed {
        type Value = WindowPick;

        fn deserialize<D>(self, deserializer: D) -> Result<WindowPick, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(SnapshotsVisitor {
                window_id: self.window_id,
                warmup_frames: self.warmup_frames,
            })
        }
    }

    struct SnapshotsVisitor {
        window_id: u64,
        warmup_frames: u64,
    }

    impl<'de> Visitor<'de> for SnapshotsVisitor {
        type Value = WindowPick;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshots array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<WindowPick, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut last: Option<SnapshotMeta> = None;
            let mut last_after_warmup: Option<SnapshotMeta> = None;
            let mut last_after_warmup_with_fp: Option<SnapshotMeta> = None;

            let mut window_snapshot_seq = 0u64;
            while let Some(meta) = seq.next_element_seed(SnapshotMetaSeed {
                window_snapshot_seq,
            })? {
                if meta.frame_id >= self.warmup_frames {
                    last_after_warmup = Some(meta.clone());
                    if meta.semantics_fingerprint.is_some() {
                        last_after_warmup_with_fp = Some(meta.clone());
                    }
                }
                last = Some(meta);
                window_snapshot_seq = window_snapshot_seq.saturating_add(1);
            }

            let mut candidates: Vec<SnapshotMeta> = Vec::new();
            for c in [last_after_warmup_with_fp, last_after_warmup, last]
                .into_iter()
                .flatten()
            {
                if candidates
                    .iter()
                    .any(|e| e.window_snapshot_seq == c.window_snapshot_seq)
                {
                    continue;
                }
                candidates.push(c);
            }

            Ok(WindowPick {
                window_id: self.window_id,
                candidates,
            })
        }
    }

    struct SnapshotMetaSeed {
        window_snapshot_seq: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotMetaSeed {
        type Value = SnapshotMeta;

        fn deserialize<D>(self, deserializer: D) -> Result<SnapshotMeta, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotMetaVisitor {
                window_snapshot_seq: self.window_snapshot_seq,
            })
        }
    }

    struct SnapshotMetaVisitor {
        window_snapshot_seq: u64,
    }

    impl<'de> Visitor<'de> for SnapshotMetaVisitor {
        type Value = SnapshotMeta;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshot object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<SnapshotMeta, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut frame_id: u64 = 0;
            let mut semantics_fingerprint: Option<u64> = None;
            let mut window_bounds: Value = Value::Null;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "frame_id" | "frameId" => {
                        frame_id = map.next_value::<u64>()?;
                    }
                    "semantics_fingerprint" | "semanticsFingerprint" => {
                        semantics_fingerprint = Some(map.next_value::<u64>()?);
                    }
                    "window_bounds" => {
                        window_bounds = map.next_value::<Value>()?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            Ok(SnapshotMeta {
                window_snapshot_seq: self.window_snapshot_seq,
                frame_id,
                semantics_fingerprint,
                window_bounds,
            })
        }
    }

    let file = std::fs::File::open(bundle_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);

    let out: std::rc::Rc<std::cell::RefCell<Vec<WindowPick>>> =
        std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));

    RootSeed {
        warmup_frames,
        out: out.clone(),
    }
    .deserialize(&mut de)
    .map_err(|e| e.to_string())?;

    Ok(out.borrow_mut().drain(..).collect())
}

fn stream_read_inline_semantics_nodes(
    bundle_path: &Path,
    window_id: u64,
    window_snapshot_seq: u64,
) -> Result<Option<Vec<Value>>, String> {
    const FOUND_MARKER: &str = "__FRET_DIAG_FOUND_SEMANTICS_NODES__";

    struct RootSeed {
        window_id: u64,
        window_snapshot_seq: u64,
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
                window_snapshot_seq: self.window_snapshot_seq,
                out: self.out,
            })
        }
    }

    struct RootVisitor {
        window_id: u64,
        window_snapshot_seq: u64,
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
                    "windows" => {
                        map.next_value_seed(WindowsSeed {
                            window_id: self.window_id,
                            window_snapshot_seq: self.window_snapshot_seq,
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

    struct WindowsSeed {
        window_id: u64,
        window_snapshot_seq: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowsSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(self)
        }
    }

    impl<'de> Visitor<'de> for WindowsSeed {
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
                    window_id: self.window_id,
                    window_snapshot_seq: self.window_snapshot_seq,
                    out: self.out.clone(),
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct WindowSeed {
        window_id: u64,
        window_snapshot_seq: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(WindowVisitor {
                window_id: self.window_id,
                window_snapshot_seq: self.window_snapshot_seq,
                out: self.out,
            })
        }
    }

    struct WindowVisitor {
        window_id: u64,
        window_snapshot_seq: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
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
            let mut window_id: Option<u64> = None;
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "window" => {
                        window_id = Some(map.next_value::<u64>()?);
                    }
                    "snapshots" => {
                        let w = window_id.unwrap_or(0);
                        if w != self.window_id {
                            map.next_value::<IgnoredAny>()?;
                            continue;
                        }
                        map.next_value_seed(SnapshotsSeed {
                            window_snapshot_seq: self.window_snapshot_seq,
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

    struct SnapshotsSeed {
        window_snapshot_seq: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotsSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(self)
        }
    }

    impl<'de> Visitor<'de> for SnapshotsSeed {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshots array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut idx = 0u64;
            while seq
                .next_element_seed(SnapshotSeed {
                    want_idx: self.window_snapshot_seq,
                    idx,
                    out: self.out.clone(),
                })?
                .is_some()
            {
                idx = idx.saturating_add(1);
            }
            Ok(())
        }
    }

    struct SnapshotSeed {
        want_idx: u64,
        idx: u64,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotVisitor {
                want_nodes: self.idx == self.want_idx,
                out: self.out,
            })
        }
    }

    struct SnapshotVisitor {
        want_nodes: bool,
        out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>>,
    }

    impl<'de> Visitor<'de> for SnapshotVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshot object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut nodes: Option<Vec<Value>> = None;
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "debug" => {
                        if self.want_nodes {
                            map.next_value_seed(DebugSeed { nodes: &mut nodes })?;
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    "semantics" | "semantic_tree" | "semanticTree" | "tree" => {
                        if self.want_nodes {
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

            if self.want_nodes
                && let Some(nodes) = nodes
            {
                self.out.borrow_mut().replace(nodes);
                return Err(serde::de::Error::custom(FOUND_MARKER));
            }
            Ok(())
        }
    }

    struct DebugSeed<'a> {
        nodes: &'a mut Option<Vec<Value>>,
    }

    impl<'de> DeserializeSeed<'de> for DebugSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugVisitor { nodes: self.nodes })
        }
    }

    struct DebugVisitor<'a> {
        nodes: &'a mut Option<Vec<Value>>,
    }

    impl<'de> Visitor<'de> for DebugVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "semantics" => {
                        map.next_value_seed(SemanticsSeed { nodes: self.nodes })?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
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

    let file = std::fs::File::open(bundle_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);

    let out: std::rc::Rc<std::cell::RefCell<Option<Vec<Value>>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));

    let res = RootSeed {
        window_id,
        window_snapshot_seq,
        out: out.clone(),
    }
    .deserialize(&mut de);

    if let Err(err) = res {
        let msg = err.to_string();
        if !msg.starts_with(FOUND_MARKER) {
            return Err(err.to_string());
        }
    }

    Ok(out.borrow_mut().take())
}
