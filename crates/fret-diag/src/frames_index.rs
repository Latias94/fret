use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde_json::{Value, json};

const FRAMES_INDEX_KIND: &str = "frames_index";
const FRAMES_INDEX_SCHEMA_VERSION: u64 = 1;

// Guardrail: building a frames index that is larger than this is unlikely to be useful for agentic
// triage. Keep the tail to avoid unbounded memory usage.
const FRAMES_INDEX_MAX_ROWS_PER_WINDOW: usize = 200_000;

pub(crate) fn default_frames_index_path(bundle_path: &Path) -> PathBuf {
    let dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    dir.join("frames.index.json")
}

#[derive(Debug, Clone, Default)]
struct FrameRow {
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
    timestamp_unix_ms: Option<u64>,

    total_time_us: Option<u64>,
    layout_time_us: Option<u64>,
    prepaint_time_us: Option<u64>,
    paint_time_us: Option<u64>,
    invalidation_walk_calls: Option<u64>,
    invalidation_walk_nodes: Option<u64>,

    semantics_fingerprint: Option<u64>,
    semantics_source_tag: u64,
}

impl FrameRow {
    fn with_semantics_source_tag(mut self, has_semantics_table: bool) -> Self {
        // 0 = none, 1 = inline, 2 = table.
        if self.semantics_source_tag == 1 {
            return self;
        }
        if self.semantics_fingerprint.is_some() && has_semantics_table {
            self.semantics_source_tag = 2;
        }
        self
    }

    fn as_row_values(self) -> Vec<Value> {
        vec![
            self.frame_id.map(Value::from).unwrap_or(Value::Null),
            self.window_snapshot_seq
                .map(Value::from)
                .unwrap_or(Value::Null),
            self.timestamp_unix_ms
                .map(Value::from)
                .unwrap_or(Value::Null),
            self.total_time_us.map(Value::from).unwrap_or(Value::Null),
            self.layout_time_us.map(Value::from).unwrap_or(Value::Null),
            self.prepaint_time_us
                .map(Value::from)
                .unwrap_or(Value::Null),
            self.paint_time_us.map(Value::from).unwrap_or(Value::Null),
            self.invalidation_walk_calls
                .map(Value::from)
                .unwrap_or(Value::Null),
            self.invalidation_walk_nodes
                .map(Value::from)
                .unwrap_or(Value::Null),
            self.semantics_fingerprint
                .map(Value::from)
                .unwrap_or(Value::Null),
            Value::from(self.semantics_source_tag),
        ]
    }
}

#[derive(Debug, Clone, Default)]
struct WindowOut {
    window: u64,
    snapshots_total: u64,
    rows_total: u64,
    warmup_fallback: bool,
    clipped_rows_dropped: u64,
    rows: Vec<FrameRow>,
}

pub(crate) fn ensure_frames_index_json(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<PathBuf, String> {
    let out = default_frames_index_path(bundle_path);
    if out.is_file() {
        if let Some(existing) = crate::util::read_json_value(&out) {
            let kind_ok = existing.get("kind").and_then(|v| v.as_str()) == Some(FRAMES_INDEX_KIND);
            let schema_ok = existing.get("schema_version").and_then(|v| v.as_u64())
                == Some(FRAMES_INDEX_SCHEMA_VERSION);
            let warmup_ok =
                existing.get("warmup_frames").and_then(|v| v.as_u64()) == Some(warmup_frames);
            if kind_ok && schema_ok && warmup_ok {
                return Ok(out);
            }
        }
    }

    let payload = build_frames_index_payload_streaming(bundle_path, warmup_frames)?;
    crate::util::write_json_value(&out, &payload)?;
    Ok(out)
}

fn build_frames_index_payload_streaming(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<Value, String> {
    #[derive(Debug, Clone)]
    struct RootOut {
        has_semantics_table: bool,
        windows: Vec<WindowOut>,
    }

    #[derive(Debug, Clone, Copy)]
    struct RootCfg {
        warmup_frames: u64,
    }

    struct RootSeed {
        cfg: RootCfg,
        out: std::rc::Rc<std::cell::RefCell<RootOut>>,
    }

    impl<'de> DeserializeSeed<'de> for RootSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(RootVisitor {
                cfg: self.cfg,
                out: self.out,
            })
        }
    }

    struct RootVisitor {
        cfg: RootCfg,
        out: std::rc::Rc<std::cell::RefCell<RootOut>>,
    }

    impl<'de> Visitor<'de> for RootVisitor {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "bundle.json object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "tables" => {
                        map.next_value_seed(TablesSeed {
                            out: self.out.clone(),
                        })?;
                    }
                    "windows" => {
                        map.next_value_seed(WindowsSeed {
                            cfg: self.cfg,
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
        out: std::rc::Rc<std::cell::RefCell<RootOut>>,
    }

    impl<'de> DeserializeSeed<'de> for TablesSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(TablesVisitor { out: self.out })
        }
    }

    struct TablesVisitor {
        out: std::rc::Rc<std::cell::RefCell<RootOut>>,
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
                if key == "semantics" {
                    self.out.borrow_mut().has_semantics_table = true;
                    map.next_value::<IgnoredAny>()?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(())
        }
    }

    struct WindowsSeed {
        cfg: RootCfg,
        out: std::rc::Rc<std::cell::RefCell<RootOut>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowsSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(WindowsVisitor {
                cfg: self.cfg,
                out: self.out,
            })
        }
    }

    struct WindowsVisitor {
        cfg: RootCfg,
        out: std::rc::Rc<std::cell::RefCell<RootOut>>,
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
                    cfg: self.cfg,
                    out: self.out.clone(),
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct WindowSeed {
        cfg: RootCfg,
        out: std::rc::Rc<std::cell::RefCell<RootOut>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(WindowVisitor {
                cfg: self.cfg,
                out: self.out,
            })
        }
    }

    struct WindowVisitor {
        cfg: RootCfg,
        out: std::rc::Rc<std::cell::RefCell<RootOut>>,
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
            let mut snapshots_total: u64 = 0;
            let mut rows: VecDeque<FrameRow> = VecDeque::new();
            let mut clipped_rows_dropped: u64 = 0;
            let mut last_seen: Option<FrameRow> = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "window" | "window_id" | "windowId" => {
                        window_id = map.next_value::<u64>()?;
                    }
                    "snapshots" => {
                        let has_semantics_table = self.out.borrow().has_semantics_table;
                        map.next_value_seed(SnapshotsSeed {
                            warmup_frames: self.cfg.warmup_frames,
                            has_semantics_table,
                            snapshots_total: &mut snapshots_total,
                            rows: &mut rows,
                            clipped_rows_dropped: &mut clipped_rows_dropped,
                            last_seen: &mut last_seen,
                        })?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            let mut out_rows: Vec<FrameRow> = rows.into_iter().collect();
            let mut warmup_fallback: bool = false;
            if out_rows.is_empty() && snapshots_total > 0 {
                if let Some(last) = last_seen {
                    out_rows.push(last);
                    warmup_fallback = true;
                }
            }

            let rows_total = out_rows.len() as u64;
            self.out.borrow_mut().windows.push(WindowOut {
                window: window_id,
                snapshots_total,
                rows_total,
                warmup_fallback,
                clipped_rows_dropped,
                rows: out_rows,
            });
            Ok(())
        }
    }

    struct SnapshotsSeed<'a> {
        warmup_frames: u64,
        has_semantics_table: bool,
        snapshots_total: &'a mut u64,
        rows: &'a mut VecDeque<FrameRow>,
        clipped_rows_dropped: &'a mut u64,
        last_seen: &'a mut Option<FrameRow>,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotsSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(SnapshotsVisitor {
                warmup_frames: self.warmup_frames,
                has_semantics_table: self.has_semantics_table,
                snapshots_total: self.snapshots_total,
                rows: self.rows,
                clipped_rows_dropped: self.clipped_rows_dropped,
                last_seen: self.last_seen,
            })
        }
    }

    struct SnapshotsVisitor<'a> {
        warmup_frames: u64,
        has_semantics_table: bool,
        snapshots_total: &'a mut u64,
        rows: &'a mut VecDeque<FrameRow>,
        clipped_rows_dropped: &'a mut u64,
        last_seen: &'a mut Option<FrameRow>,
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
            while let Some(row) = seq.next_element_seed(SnapshotSeed {
                has_semantics_table: self.has_semantics_table,
            })? {
                *self.snapshots_total = self.snapshots_total.saturating_add(1);
                *self.last_seen = Some(row.clone());

                let frame_id = row.frame_id.unwrap_or(0);
                let keep = self.warmup_frames == 0 || frame_id >= self.warmup_frames;
                if keep {
                    self.rows.push_back(row);
                    if self.rows.len() > FRAMES_INDEX_MAX_ROWS_PER_WINDOW {
                        let _ = self.rows.pop_front();
                        *self.clipped_rows_dropped = self.clipped_rows_dropped.saturating_add(1);
                    }
                }
            }
            Ok(())
        }
    }

    struct SnapshotSeed {
        has_semantics_table: bool,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotSeed {
        type Value = FrameRow;

        fn deserialize<D>(self, deserializer: D) -> Result<FrameRow, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotVisitor {
                has_semantics_table: self.has_semantics_table,
            })
        }
    }

    struct SnapshotVisitor {
        has_semantics_table: bool,
    }

    impl<'de> Visitor<'de> for SnapshotVisitor {
        type Value = FrameRow;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshot object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<FrameRow, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut out = FrameRow::default();

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "frame_id" | "frameId" => {
                        out.frame_id = map.next_value::<Option<u64>>()?;
                    }
                    "window_snapshot_seq" | "windowSnapshotSeq" => {
                        out.window_snapshot_seq = map.next_value::<Option<u64>>()?;
                    }
                    "timestamp_unix_ms" | "timestamp_ms" => {
                        // Some bundles use `timestamp_ms`. Normalize onto the same column.
                        out.timestamp_unix_ms = map.next_value::<Option<u64>>()?;
                    }
                    "semantics_fingerprint" | "semanticsFingerprint" => {
                        out.semantics_fingerprint = map.next_value::<Option<u64>>()?;
                    }
                    "debug" => {
                        map.next_value_seed(DebugSeed { out: &mut out })?;
                    }
                    // Older/alternate layouts: treat these as potential semantics containers.
                    "semantics" | "semantic_tree" | "semanticTree" | "tree" => {
                        let has_nodes = map.next_value_seed(SemanticsSeed)?;
                        if has_nodes {
                            out.semantics_source_tag = 1;
                        }
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            Ok(out.with_semantics_source_tag(self.has_semantics_table))
        }
    }

    struct DebugSeed<'a> {
        out: &'a mut FrameRow,
    }

    impl<'de> DeserializeSeed<'de> for DebugSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugVisitor { out: self.out })
        }
    }

    struct DebugVisitor<'a> {
        out: &'a mut FrameRow,
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
                    "stats" => {
                        map.next_value_seed(StatsSeed { out: self.out })?;
                    }
                    "semantics" => {
                        let has_nodes = map.next_value_seed(SemanticsSeed)?;
                        if has_nodes {
                            self.out.semantics_source_tag = 1;
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

    struct StatsSeed<'a> {
        out: &'a mut FrameRow,
    }

    impl<'de> DeserializeSeed<'de> for StatsSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(StatsVisitor { out: self.out })
        }
    }

    struct StatsVisitor<'a> {
        out: &'a mut FrameRow,
    }

    impl<'de> Visitor<'de> for StatsVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "stats object")
        }

        fn visit_map<M>(self, mut map: M) -> Result<(), M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "total_time_us" => self.out.total_time_us = map.next_value::<Option<u64>>()?,
                    "layout_time_us" => {
                        self.out.layout_time_us = map.next_value::<Option<u64>>()?
                    }
                    "prepaint_time_us" => {
                        self.out.prepaint_time_us = map.next_value::<Option<u64>>()?
                    }
                    "paint_time_us" => self.out.paint_time_us = map.next_value::<Option<u64>>()?,
                    "invalidation_walk_calls" => {
                        self.out.invalidation_walk_calls = map.next_value::<Option<u64>>()?
                    }
                    "invalidation_walk_nodes" => {
                        self.out.invalidation_walk_nodes = map.next_value::<Option<u64>>()?
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(())
        }
    }

    struct SemanticsSeed;

    impl<'de> DeserializeSeed<'de> for SemanticsSeed {
        type Value = bool;

        fn deserialize<D>(self, deserializer: D) -> Result<bool, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SemanticsVisitor { has_nodes: false })
        }
    }

    struct SemanticsVisitor {
        has_nodes: bool,
    }

    impl<'de> Visitor<'de> for SemanticsVisitor {
        type Value = bool;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "semantics object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<bool, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "nodes" {
                    self.has_nodes = true;
                    map.next_value::<IgnoredAny>()?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(self.has_nodes)
        }
    }

    let file = std::fs::File::open(bundle_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);

    let out: std::rc::Rc<std::cell::RefCell<RootOut>> =
        std::rc::Rc::new(std::cell::RefCell::new(RootOut {
            has_semantics_table: false,
            windows: Vec::new(),
        }));

    RootSeed {
        cfg: RootCfg { warmup_frames },
        out: out.clone(),
    }
    .deserialize(&mut de)
    .map_err(|e| e.to_string())?;

    let out = out.borrow().clone();

    let columns: Vec<Value> = vec![
        Value::from("frame_id"),
        Value::from("window_snapshot_seq"),
        Value::from("timestamp_unix_ms"),
        Value::from("total_time_us"),
        Value::from("layout_time_us"),
        Value::from("prepaint_time_us"),
        Value::from("paint_time_us"),
        Value::from("invalidation_walk_calls"),
        Value::from("invalidation_walk_nodes"),
        Value::from("semantics_fingerprint"),
        Value::from("semantics_source_tag"),
    ];

    let mut windows_out: Vec<Value> = Vec::new();
    let mut windows_total: u64 = 0;
    let mut frames_total: u64 = 0;
    let mut snapshots_total: u64 = 0;

    for w in &out.windows {
        windows_total = windows_total.saturating_add(1);
        frames_total = frames_total.saturating_add(w.rows_total);
        snapshots_total = snapshots_total.saturating_add(w.snapshots_total);

        let mut rows: Vec<Value> = Vec::with_capacity(w.rows.len());
        let mut first_frame_id: Option<u64> = None;
        let mut last_frame_id: Option<u64> = None;
        let mut first_ts: Option<u64> = None;
        let mut last_ts: Option<u64> = None;
        for r in w.rows.iter().cloned() {
            if first_frame_id.is_none() {
                first_frame_id = r.frame_id;
            }
            if let Some(fid) = r.frame_id {
                last_frame_id = Some(fid);
            }
            if first_ts.is_none() {
                first_ts = r.timestamp_unix_ms;
            }
            if let Some(ts) = r.timestamp_unix_ms {
                last_ts = Some(ts);
            }
            rows.push(Value::Array(r.as_row_values()));
        }

        let clipped = if w.clipped_rows_dropped > 0 {
            Some(json!({
                "schema_version": 1,
                "max_rows_per_window": FRAMES_INDEX_MAX_ROWS_PER_WINDOW,
                "dropped_early_rows": w.clipped_rows_dropped,
            }))
        } else {
            None
        };

        windows_out.push(json!({
            "window": w.window,
            "snapshots_total": w.snapshots_total,
            "frames_total": w.rows_total,
            "first_frame_id": first_frame_id,
            "last_frame_id": last_frame_id,
            "first_timestamp_unix_ms": first_ts,
            "last_timestamp_unix_ms": last_ts,
            "warmup_fallback": w.warmup_fallback,
            "clipped": clipped,
            "rows": rows,
        }));
    }

    Ok(json!({
        "schema_version": FRAMES_INDEX_SCHEMA_VERSION,
        "kind": FRAMES_INDEX_KIND,
        "bundle": bundle_path.display().to_string(),
        "generated_unix_ms": crate::util::now_unix_ms(),
        "warmup_frames": warmup_frames,
        "has_semantics_table": out.has_semantics_table,
        "columns": columns,
        "windows_total": windows_total,
        "snapshots_total": snapshots_total,
        "frames_total": frames_total,
        "windows": windows_out,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_index_streaming_reads_debug_stats_and_semantics() {
        let mut dir = std::env::temp_dir();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        dir.push(format!(
            "fret-diag-frames-index-test-{}-{}",
            std::process::id(),
            ts
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let bundle_path = dir.join("bundle.json");
        std::fs::write(
            &bundle_path,
            r#"{
  "schema_version": 1,
  "tables": { "semantics": { "entries": [] } },
  "windows": [{
    "window": 1,
    "snapshots": [
      { "frame_id": 0, "window_snapshot_seq": 1, "timestamp_unix_ms": 1, "debug": { "stats": { "total_time_us": 10 } } },
      { "frame_id": 5, "window_snapshot_seq": 2, "timestamp_unix_ms": 2, "semantics_fingerprint": 42, "debug": { "stats": { "total_time_us": 20, "layout_time_us": 3 }, "semantics": { "nodes": [] } } },
      { "frame_id": 6, "window_snapshot_seq": 3, "timestamp_unix_ms": 3, "semantics_fingerprint": 43, "debug": { "stats": { "total_time_us": 30 } } }
    ]
  }]
}"#,
        )
        .expect("write bundle");

        let payload = build_frames_index_payload_streaming(&bundle_path, 5).expect("payload");
        assert_eq!(payload["kind"].as_str(), Some("frames_index"));
        assert_eq!(payload["schema_version"].as_u64(), Some(1));
        assert_eq!(payload["warmup_frames"].as_u64(), Some(5));
        assert_eq!(payload["has_semantics_table"].as_bool(), Some(true));
        assert_eq!(payload["windows_total"].as_u64(), Some(1));
        assert_eq!(payload["snapshots_total"].as_u64(), Some(3));
        assert_eq!(payload["frames_total"].as_u64(), Some(2));

        let windows = payload["windows"].as_array().expect("windows");
        let w = windows[0].as_object().expect("window");
        assert_eq!(w.get("window").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(w.get("frames_total").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(w.get("first_frame_id").and_then(|v| v.as_u64()), Some(5));

        let rows = w.get("rows").and_then(|v| v.as_array()).expect("rows");
        assert_eq!(rows.len(), 2);
        // columns: frame_id, seq, ts, total, layout, prepaint, paint, calls, nodes, fp, tag
        assert_eq!(rows[0][0].as_u64(), Some(5));
        assert_eq!(rows[0][3].as_u64(), Some(20));
        assert_eq!(rows[0][4].as_u64(), Some(3));
        assert_eq!(rows[0][9].as_u64(), Some(42));
        assert_eq!(rows[0][10].as_u64(), Some(1)); // inline

        assert_eq!(rows[1][0].as_u64(), Some(6));
        assert_eq!(rows[1][3].as_u64(), Some(30));
        assert_eq!(rows[1][9].as_u64(), Some(43));
        assert_eq!(rows[1][10].as_u64(), Some(2)); // table (fp + tables present)
    }
}
