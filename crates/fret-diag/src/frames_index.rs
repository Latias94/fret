use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde_json::{Value, json};

const FRAMES_INDEX_KIND: &str = "frames_index";
const FRAMES_INDEX_SCHEMA_VERSION: u64 = 1;

const FRAMES_INDEX_FEATURE_WINDOW_AGG_V1: &str = "window_aggregates.v1";
const FRAMES_INDEX_REQUIRED_FEATURES: &[&str] = &[FRAMES_INDEX_FEATURE_WINDOW_AGG_V1];

// Guardrail: building a frames index that is larger than this is unlikely to be useful for agentic
// triage. Keep the tail to avoid unbounded memory usage.
const FRAMES_INDEX_MAX_ROWS_PER_WINDOW: usize = 50_000;

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

    // Parsed for per-window aggregates (not encoded into the frame row columns).
    viewport_input_events: u64,
    dock_drag_active: bool,
    viewport_capture_active: bool,
    view_cache_active: bool,
    view_cache_roots_reused: Option<u64>,
    cache_roots_reused: Option<u64>,
    paint_cache_replayed_ops: Option<u64>,
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
    aggregates: WindowAggregates,
    rows: Vec<FrameRow>,
}

#[derive(Debug, Clone, Default)]
struct WindowAggregates {
    // These are post-warmup (frame_id >= warmup_frames) and are computed over the full stream,
    // even when per-window rows are tail-clipped.
    examined_snapshots_post_warmup: u64,
    viewport_input_events_post_warmup: u64,
    dock_drag_active_frames_post_warmup: u64,
    viewport_capture_active_frames_post_warmup: u64,
    view_cache_active_snapshots_post_warmup: u64,
    view_cache_reuse_events_post_warmup: u64,
    paint_cache_replayed_ops_post_warmup: u64,
}

pub(crate) fn ensure_frames_index_json(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<PathBuf, String> {
    fn has_required_features(v: &Value, required: &[&str]) -> bool {
        if required.is_empty() {
            return true;
        }
        let Some(features) = v.get("features").and_then(|v| v.as_array()) else {
            return false;
        };
        required.iter().all(|req| {
            features
                .iter()
                .any(|f| f.as_str().is_some_and(|s| s == *req))
        })
    }

    let out = default_frames_index_path(bundle_path);
    let expected_bundle = bundle_path.display().to_string();
    if out.is_file() {
        if let Some(existing) = read_frames_index_json_v1(&out, warmup_frames) {
            let kind_ok = existing.get("kind").and_then(|v| v.as_str()) == Some(FRAMES_INDEX_KIND);
            let schema_ok = existing.get("schema_version").and_then(|v| v.as_u64())
                == Some(FRAMES_INDEX_SCHEMA_VERSION);
            let warmup_ok =
                existing.get("warmup_frames").and_then(|v| v.as_u64()) == Some(warmup_frames);
            let bundle_ok =
                existing.get("bundle").and_then(|v| v.as_str()) == Some(&expected_bundle);
            let features_ok = has_required_features(&existing, FRAMES_INDEX_REQUIRED_FEATURES);
            if kind_ok && schema_ok && warmup_ok && bundle_ok && features_ok {
                return Ok(out);
            }
        }
    }

    let payload = build_frames_index_payload_streaming(bundle_path, warmup_frames)?;
    write_compact_json(&out, &payload)?;
    Ok(out)
}

pub(crate) fn read_frames_index_json_v1(path: &Path, warmup_frames: u64) -> Option<Value> {
    let bytes = std::fs::read(path).ok()?;
    let v: Value = serde_json::from_slice(&bytes).ok()?;
    if v.get("kind").and_then(|v| v.as_str()) != Some(FRAMES_INDEX_KIND) {
        return None;
    }
    if v.get("schema_version").and_then(|v| v.as_u64()) != Some(FRAMES_INDEX_SCHEMA_VERSION) {
        return None;
    }
    if v.get("warmup_frames").and_then(|v| v.as_u64()) != Some(warmup_frames) {
        return None;
    }
    Some(v)
}

fn write_compact_json(path: &Path, v: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = serde_json::to_vec(v).map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())
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
            let mut aggregates: WindowAggregates = WindowAggregates::default();

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
                            aggregates: &mut aggregates,
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
                aggregates,
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
        aggregates: &'a mut WindowAggregates,
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
                aggregates: self.aggregates,
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
        aggregates: &'a mut WindowAggregates,
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
                    self.aggregates.examined_snapshots_post_warmup =
                        self.aggregates.examined_snapshots_post_warmup.saturating_add(1);
                    self.aggregates.viewport_input_events_post_warmup = self
                        .aggregates
                        .viewport_input_events_post_warmup
                        .saturating_add(row.viewport_input_events);
                    if row.dock_drag_active {
                        self.aggregates.dock_drag_active_frames_post_warmup = self
                            .aggregates
                            .dock_drag_active_frames_post_warmup
                            .saturating_add(1);
                    }
                    if row.viewport_capture_active {
                        self.aggregates.viewport_capture_active_frames_post_warmup = self
                            .aggregates
                            .viewport_capture_active_frames_post_warmup
                            .saturating_add(1);
                    }
                    if row.view_cache_active {
                        self.aggregates.view_cache_active_snapshots_post_warmup = self
                            .aggregates
                            .view_cache_active_snapshots_post_warmup
                            .saturating_add(1);
                    }
                    let reuse_events = row
                        .view_cache_roots_reused
                        .or(row.cache_roots_reused)
                        .unwrap_or(0);
                    self.aggregates.view_cache_reuse_events_post_warmup = self
                        .aggregates
                        .view_cache_reuse_events_post_warmup
                        .saturating_add(reuse_events);
                    self.aggregates.paint_cache_replayed_ops_post_warmup = self
                        .aggregates
                        .paint_cache_replayed_ops_post_warmup
                        .saturating_add(row.paint_cache_replayed_ops.unwrap_or(0));

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
                    "viewport_input" | "viewportInput" => {
                        self.out.viewport_input_events = map.next_value_seed(SeqLenSeed)?;
                    }
                    "docking_interaction" | "dockingInteraction" => {
                        let flags = map.next_value_seed(DockingInteractionSeed)?;
                        self.out.dock_drag_active |= flags.dock_drag_active;
                        self.out.viewport_capture_active |= flags.viewport_capture_active;
                    }
                    "cache_roots" | "cacheRoots" => {
                        self.out.cache_roots_reused = Some(map.next_value_seed(CacheRootsSeed)?);
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
                    "view_cache_active" => {
                        self.out.view_cache_active =
                            map.next_value::<Option<bool>>()?.unwrap_or(false);
                    }
                    "view_cache_roots_reused" => {
                        self.out.view_cache_roots_reused = map.next_value::<Option<u64>>()?;
                    }
                    "paint_cache_replayed_ops" => {
                        self.out.paint_cache_replayed_ops = map.next_value::<Option<u64>>()?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(())
        }
    }

    struct SeqLenSeed;

    impl<'de> DeserializeSeed<'de> for SeqLenSeed {
        type Value = u64;

        fn deserialize<D>(self, deserializer: D) -> Result<u64, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(SeqLenVisitor { len: 0 })
        }
    }

    struct SeqLenVisitor {
        len: u64,
    }

    impl<'de> Visitor<'de> for SeqLenVisitor {
        type Value = u64;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "an array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<u64, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while seq.next_element::<IgnoredAny>()?.is_some() {
                self.len = self.len.saturating_add(1);
            }
            Ok(self.len)
        }

        fn visit_map<M>(self, mut map: M) -> Result<u64, M::Error>
        where
            M: MapAccess<'de>,
        {
            while map.next_key::<IgnoredAny>()?.is_some() {
                map.next_value::<IgnoredAny>()?;
            }
            Ok(0)
        }

        fn visit_unit<E>(self) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_none<E>(self) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_bool<E>(self, _v: bool) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_i64<E>(self, _v: i64) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_u64<E>(self, _v: u64) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_f64<E>(self, _v: f64) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_str<E>(self, _v: &str) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    struct DockingInteractionFlags {
        dock_drag_active: bool,
        viewport_capture_active: bool,
    }

    struct DockingInteractionSeed;

    impl<'de> DeserializeSeed<'de> for DockingInteractionSeed {
        type Value = DockingInteractionFlags;

        fn deserialize<D>(self, deserializer: D) -> Result<DockingInteractionFlags, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(DockingInteractionVisitor {
                flags: DockingInteractionFlags::default(),
            })
        }
    }

    struct DockingInteractionVisitor {
        flags: DockingInteractionFlags,
    }

    impl<'de> Visitor<'de> for DockingInteractionVisitor {
        type Value = DockingInteractionFlags;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "a docking_interaction object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<DockingInteractionFlags, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "dock_drag" | "dockDrag" => {
                        self.flags.dock_drag_active = map.next_value_seed(IsObjectSeed)?;
                    }
                    "viewport_capture" | "viewportCapture" => {
                        self.flags.viewport_capture_active = map.next_value_seed(IsObjectSeed)?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(self.flags)
        }

        fn visit_unit<E>(self) -> Result<DockingInteractionFlags, E>
        where
            E: serde::de::Error,
        {
            Ok(self.flags)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<DockingInteractionFlags, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while seq.next_element::<IgnoredAny>()?.is_some() {}
            Ok(self.flags)
        }

        fn visit_bool<E>(self, _v: bool) -> Result<DockingInteractionFlags, E>
        where
            E: serde::de::Error,
        {
            Ok(self.flags)
        }

        fn visit_i64<E>(self, _v: i64) -> Result<DockingInteractionFlags, E>
        where
            E: serde::de::Error,
        {
            Ok(self.flags)
        }

        fn visit_u64<E>(self, _v: u64) -> Result<DockingInteractionFlags, E>
        where
            E: serde::de::Error,
        {
            Ok(self.flags)
        }

        fn visit_f64<E>(self, _v: f64) -> Result<DockingInteractionFlags, E>
        where
            E: serde::de::Error,
        {
            Ok(self.flags)
        }

        fn visit_str<E>(self, _v: &str) -> Result<DockingInteractionFlags, E>
        where
            E: serde::de::Error,
        {
            Ok(self.flags)
        }
    }

    struct IsObjectSeed;

    impl<'de> DeserializeSeed<'de> for IsObjectSeed {
        type Value = bool;

        fn deserialize<D>(self, deserializer: D) -> Result<bool, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(IsObjectVisitor)
        }
    }

    struct IsObjectVisitor;

    impl<'de> Visitor<'de> for IsObjectVisitor {
        type Value = bool;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "any value")
        }

        fn visit_map<M>(self, mut map: M) -> Result<bool, M::Error>
        where
            M: MapAccess<'de>,
        {
            while map.next_key::<IgnoredAny>()?.is_some() {
                map.next_value::<IgnoredAny>()?;
            }
            Ok(true)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<bool, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while seq.next_element::<IgnoredAny>()?.is_some() {}
            Ok(false)
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

        fn visit_bool<E>(self, _v: bool) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_i64<E>(self, _v: i64) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_u64<E>(self, _v: u64) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_f64<E>(self, _v: f64) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_str<E>(self, _v: &str) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }
    }

    struct CacheRootsSeed;

    impl<'de> DeserializeSeed<'de> for CacheRootsSeed {
        type Value = u64;

        fn deserialize<D>(self, deserializer: D) -> Result<u64, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(CacheRootsVisitor { reused: 0 })
        }
    }

    struct CacheRootsVisitor {
        reused: u64,
    }

    impl<'de> Visitor<'de> for CacheRootsVisitor {
        type Value = u64;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "a cache_roots array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<u64, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some(reused) = seq.next_element_seed(CacheRootSeed)? {
                if reused {
                    self.reused = self.reused.saturating_add(1);
                }
            }
            Ok(self.reused)
        }

        fn visit_map<M>(self, mut map: M) -> Result<u64, M::Error>
        where
            M: MapAccess<'de>,
        {
            while map.next_key::<IgnoredAny>()?.is_some() {
                map.next_value::<IgnoredAny>()?;
            }
            Ok(0)
        }

        fn visit_unit<E>(self) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_none<E>(self) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_bool<E>(self, _v: bool) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_i64<E>(self, _v: i64) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_u64<E>(self, _v: u64) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_f64<E>(self, _v: f64) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }

        fn visit_str<E>(self, _v: &str) -> Result<u64, E>
        where
            E: serde::de::Error,
        {
            Ok(0)
        }
    }

    struct CacheRootSeed;

    impl<'de> DeserializeSeed<'de> for CacheRootSeed {
        type Value = bool;

        fn deserialize<D>(self, deserializer: D) -> Result<bool, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(CacheRootVisitor { reused: false })
        }
    }

    struct CacheRootVisitor {
        reused: bool,
    }

    impl<'de> Visitor<'de> for CacheRootVisitor {
        type Value = bool;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "a cache root object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<bool, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "reused" {
                    self.reused = map.next_value::<Option<bool>>()?.unwrap_or(false);
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(self.reused)
        }

        fn visit_unit<E>(self) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<bool, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while seq.next_element::<IgnoredAny>()?.is_some() {}
            Ok(false)
        }

        fn visit_none<E>(self) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_bool<E>(self, _v: bool) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_i64<E>(self, _v: i64) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_u64<E>(self, _v: u64) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_f64<E>(self, _v: f64) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
        }

        fn visit_str<E>(self, _v: &str) -> Result<bool, E>
        where
            E: serde::de::Error,
        {
            Ok(false)
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

        let aggregates = json!({
            "schema_version": 1,
            "examined_snapshots_post_warmup": w.aggregates.examined_snapshots_post_warmup,
            "viewport_input_events_post_warmup": w.aggregates.viewport_input_events_post_warmup,
            "dock_drag_active_frames_post_warmup": w.aggregates.dock_drag_active_frames_post_warmup,
            "viewport_capture_active_frames_post_warmup": w.aggregates.viewport_capture_active_frames_post_warmup,
            "view_cache_active_snapshots_post_warmup": w.aggregates.view_cache_active_snapshots_post_warmup,
            "view_cache_reuse_events_post_warmup": w.aggregates.view_cache_reuse_events_post_warmup,
            "paint_cache_replayed_ops_post_warmup": w.aggregates.paint_cache_replayed_ops_post_warmup,
        });

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
            "aggregates": aggregates,
            "rows": rows,
        }));
    }

    Ok(json!({
        "schema_version": FRAMES_INDEX_SCHEMA_VERSION,
        "kind": FRAMES_INDEX_KIND,
        "features": [FRAMES_INDEX_FEATURE_WINDOW_AGG_V1],
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum TriageLiteMetric {
    TotalTimeUs,
    LayoutTimeUs,
    PaintTimeUs,
}

pub(crate) fn triage_lite_json_from_frames_index(
    bundle_path: &Path,
    frames_index_path: &Path,
    frames_index: &Value,
    warmup_frames: u64,
    top: usize,
    metric: TriageLiteMetric,
) -> Result<Value, String> {
    fn col_index(columns: &[Value], name: &str) -> Option<usize> {
        columns
            .iter()
            .position(|c| c.as_str().is_some_and(|s| s == name))
    }

    fn row_u64(row: &[Value], idx: Option<usize>) -> Option<u64> {
        let idx = idx?;
        row.get(idx)?.as_u64()
    }

    fn row_metric(
        row: &[Value],
        metric: TriageLiteMetric,
        idx_total: Option<usize>,
        idx_layout: Option<usize>,
        idx_paint: Option<usize>,
    ) -> u64 {
        match metric {
            TriageLiteMetric::TotalTimeUs => row_u64(row, idx_total).unwrap_or(0),
            TriageLiteMetric::LayoutTimeUs => row_u64(row, idx_layout).unwrap_or(0),
            TriageLiteMetric::PaintTimeUs => row_u64(row, idx_paint).unwrap_or(0),
        }
    }

    if frames_index.get("kind").and_then(|v| v.as_str()) != Some(FRAMES_INDEX_KIND) {
        return Err("invalid frames.index.json: kind mismatch".to_string());
    }
    if frames_index.get("schema_version").and_then(|v| v.as_u64())
        != Some(FRAMES_INDEX_SCHEMA_VERSION)
    {
        return Err("invalid frames.index.json: schema_version mismatch".to_string());
    }
    if frames_index.get("warmup_frames").and_then(|v| v.as_u64()) != Some(warmup_frames) {
        return Err("invalid frames.index.json: warmup_frames mismatch".to_string());
    }

    let columns = frames_index
        .get("columns")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid frames.index.json: missing columns".to_string())?;

    let idx_frame_id = col_index(columns, "frame_id");
    let idx_seq = col_index(columns, "window_snapshot_seq");
    let idx_ts = col_index(columns, "timestamp_unix_ms");
    let idx_total = col_index(columns, "total_time_us");
    let idx_layout = col_index(columns, "layout_time_us");
    let idx_paint = col_index(columns, "paint_time_us");
    let idx_fp = col_index(columns, "semantics_fingerprint");
    let idx_sem_tag = col_index(columns, "semantics_source_tag");

    let windows = frames_index
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid frames.index.json: missing windows".to_string())?;

    let top = top.max(1).min(1000);
    let metric_name = match metric {
        TriageLiteMetric::TotalTimeUs => "total_time_us",
        TriageLiteMetric::LayoutTimeUs => "layout_time_us",
        TriageLiteMetric::PaintTimeUs => "paint_time_us",
    };

    #[derive(Debug, Clone)]
    struct Entry {
        metric: u64,
        frame_id: Option<u64>,
        window_snapshot_seq: Option<u64>,
        timestamp_unix_ms: Option<u64>,
        total_time_us: Option<u64>,
        layout_time_us: Option<u64>,
        paint_time_us: Option<u64>,
        semantics_fingerprint: Option<u64>,
        semantics_source_tag: Option<u64>,
    }

    let mut windows_out: Vec<Value> = Vec::new();
    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let frames_total = w.get("frames_total").and_then(|v| v.as_u64()).unwrap_or(0);
        let snapshots_total = w
            .get("snapshots_total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let rows = w
            .get("rows")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let mut sum_total: u64 = 0;
        let mut sum_layout: u64 = 0;
        let mut sum_paint: u64 = 0;
        let mut count_total: u64 = 0;
        let mut count_layout: u64 = 0;
        let mut count_paint: u64 = 0;

        let mut best: Vec<Entry> = Vec::new();

        for row in rows {
            let Some(row) = row.as_array() else {
                continue;
            };
            let total = row_u64(row, idx_total);
            let layout = row_u64(row, idx_layout);
            let paint = row_u64(row, idx_paint);

            if let Some(v) = total {
                sum_total = sum_total.saturating_add(v);
                count_total = count_total.saturating_add(1);
            }
            if let Some(v) = layout {
                sum_layout = sum_layout.saturating_add(v);
                count_layout = count_layout.saturating_add(1);
            }
            if let Some(v) = paint {
                sum_paint = sum_paint.saturating_add(v);
                count_paint = count_paint.saturating_add(1);
            }

            let score = row_metric(row, metric, idx_total, idx_layout, idx_paint);
            if score == 0 {
                continue;
            }

            let e = Entry {
                metric: score,
                frame_id: row_u64(row, idx_frame_id),
                window_snapshot_seq: row_u64(row, idx_seq),
                timestamp_unix_ms: row_u64(row, idx_ts),
                total_time_us: total,
                layout_time_us: layout,
                paint_time_us: paint,
                semantics_fingerprint: row_u64(row, idx_fp),
                semantics_source_tag: row_u64(row, idx_sem_tag),
            };

            if best.len() < top {
                best.push(e);
                continue;
            }
            let mut min_idx: usize = 0;
            for i in 1..best.len() {
                if best[i].metric < best[min_idx].metric {
                    min_idx = i;
                }
            }
            if e.metric > best[min_idx].metric {
                best[min_idx] = e;
            }
        }

        best.sort_by(|a, b| b.metric.cmp(&a.metric));
        let worst_frames: Vec<Value> = best
            .into_iter()
            .map(|e| {
                let mut suggestions: Vec<String> = Vec::new();
                if let Some(fid) = e.frame_id {
                    suggestions.push(format!(
                        "fretboard diag slice {} --test-id <test_id> --window {} --frame-id {} --warmup-frames {}",
                        bundle_path.display(),
                        window_id,
                        fid,
                        warmup_frames
                    ));
                } else if let Some(seq) = e.window_snapshot_seq {
                    suggestions.push(format!(
                        "fretboard diag slice {} --test-id <test_id> --window {} --snapshot-seq {} --warmup-frames {}",
                        bundle_path.display(),
                        window_id,
                        seq,
                        warmup_frames
                    ));
                }
                json!({
                    "window": window_id,
                    "frame_id": e.frame_id,
                    "window_snapshot_seq": e.window_snapshot_seq,
                    "timestamp_unix_ms": e.timestamp_unix_ms,
                    "metric": { metric_name: e.metric },
                    "stats": {
                        "total_time_us": e.total_time_us,
                        "layout_time_us": e.layout_time_us,
                        "paint_time_us": e.paint_time_us,
                    },
                    "semantics": {
                        "fingerprint": e.semantics_fingerprint,
                        "source_tag": e.semantics_source_tag,
                    },
                    "suggestions": suggestions,
                })
            })
            .collect();

        let avg_total = if count_total == 0 {
            None
        } else {
            Some(sum_total / count_total)
        };
        let avg_layout = if count_layout == 0 {
            None
        } else {
            Some(sum_layout / count_layout)
        };
        let avg_paint = if count_paint == 0 {
            None
        } else {
            Some(sum_paint / count_paint)
        };

        windows_out.push(json!({
            "window": window_id,
            "snapshots_total": snapshots_total,
            "frames_total": frames_total,
            "frames_index_rows_total": rows.len() as u64,
            "metric": metric_name,
            "stats_avg_us": {
                "total_time_us": avg_total,
                "layout_time_us": avg_layout,
                "paint_time_us": avg_paint,
            },
            "worst_frames": worst_frames,
            "clipped": w.get("clipped").cloned(),
            "warmup_fallback": w.get("warmup_fallback").cloned(),
        }));
    }

    Ok(json!({
        "schema_version": 1,
        "kind": "triage_lite",
        "bundle": bundle_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "source": {
            "kind": FRAMES_INDEX_KIND,
            "schema_version": FRAMES_INDEX_SCHEMA_VERSION,
            "path": frames_index_path.display().to_string(),
        },
        "metric": {
            "name": metric_name,
            "top": top,
        },
        "notes": [
            "triage_lite is generated from frames.index.json; it is intended for agent-friendly first-pass triage.",
            "semantics_source_tag: 0=none 1=inline 2=table",
        ],
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
        let bundle_path = crate::resolve_bundle_artifact_path(&dir);
        std::fs::write(
            &bundle_path,
            r#"{
  "schema_version": 1,
  "tables": { "semantics": { "entries": [] } },
  "windows": [{
    "window": 1,
    "snapshots": [
      { "frame_id": 0, "window_snapshot_seq": 1, "timestamp_unix_ms": 1, "debug": { "stats": { "total_time_us": 10 } } },
      { "frame_id": 5, "window_snapshot_seq": 2, "timestamp_unix_ms": 2, "semantics_fingerprint": 42, "debug": { "viewport_input": [1,2], "docking_interaction": { "dock_drag": {} }, "stats": { "total_time_us": 20, "layout_time_us": 3, "view_cache_active": true, "view_cache_roots_reused": 1, "paint_cache_replayed_ops": 4 }, "semantics": { "nodes": [] } } },
      { "frame_id": 6, "window_snapshot_seq": 3, "timestamp_unix_ms": 3, "semantics_fingerprint": 43, "debug": { "viewport_input": [1], "docking_interaction": { "viewport_capture": {} }, "stats": { "total_time_us": 30, "view_cache_active": true, "view_cache_roots_reused": 2, "paint_cache_replayed_ops": 1 } } }
    ]
  }]
}"#,
        )
        .expect("write bundle");

        let payload = build_frames_index_payload_streaming(&bundle_path, 5).expect("payload");
        assert_eq!(payload["kind"].as_str(), Some("frames_index"));
        assert_eq!(payload["schema_version"].as_u64(), Some(1));
        assert!(payload["features"]
            .as_array()
            .is_some_and(|v| v.iter().any(|f| f.as_str() == Some("window_aggregates.v1"))));
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

        let aggs = w.get("aggregates").and_then(|v| v.as_object()).expect("aggregates");
        assert_eq!(
            aggs.get("examined_snapshots_post_warmup")
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            aggs.get("viewport_input_events_post_warmup")
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            aggs.get("dock_drag_active_frames_post_warmup")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            aggs.get("viewport_capture_active_frames_post_warmup")
                .and_then(|v| v.as_u64()),
            Some(1)
        );
        assert_eq!(
            aggs.get("view_cache_active_snapshots_post_warmup")
                .and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            aggs.get("view_cache_reuse_events_post_warmup")
                .and_then(|v| v.as_u64()),
            Some(3)
        );
        assert_eq!(
            aggs.get("paint_cache_replayed_ops_post_warmup")
                .and_then(|v| v.as_u64()),
            Some(5)
        );

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
