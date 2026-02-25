use std::collections::BTreeMap;
use std::path::Path;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

use crate::util::{now_unix_ms, write_json_value};

#[derive(Debug, Clone, Default)]
struct NotifyOut {
    windows_total: u64,
    examined_snapshots: u64,
    total_notify_requests: u64,
    matched_notify_requests: u64,
    matched_samples: Vec<serde_json::Value>,
    matched_hotspot_counts: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Default)]
struct NotifyReqLite {
    caller_node: Option<u64>,
    target_view: Option<u64>,
    file: String,
    line: u64,
    column: u64,
}

fn file_matches(actual: &str, filter: &str) -> bool {
    if filter.is_empty() {
        return false;
    }
    if actual == filter {
        return true;
    }
    let actual_norm = actual.replace('\\', "/");
    let filter_norm = filter.replace('\\', "/");
    actual_norm.ends_with(&filter_norm) || actual_norm.contains(&filter_norm)
}

pub(crate) fn check_bundle_for_notify_hotspot_file_max_streaming(
    bundle_path: &Path,
    file_filter: &str,
    max_count: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    struct RootSeed {
        cfg: Cfg,
        out: std::rc::Rc<std::cell::RefCell<NotifyOut>>,
    }

    #[derive(Debug, Clone)]
    struct Cfg {
        file_filter: String,
        warmup_frames: u64,
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
        cfg: Cfg,
        out: std::rc::Rc<std::cell::RefCell<NotifyOut>>,
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
                            cfg: self.cfg.clone(),
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
        cfg: Cfg,
        out: std::rc::Rc<std::cell::RefCell<NotifyOut>>,
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
        cfg: Cfg,
        out: std::rc::Rc<std::cell::RefCell<NotifyOut>>,
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
                    cfg: self.cfg.clone(),
                    out: self.out.clone(),
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct WindowSeed {
        cfg: Cfg,
        out: std::rc::Rc<std::cell::RefCell<NotifyOut>>,
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
        cfg: Cfg,
        out: std::rc::Rc<std::cell::RefCell<NotifyOut>>,
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
                            cfg: self.cfg.clone(),
                            out: self.out.clone(),
                            window_id,
                        })?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            let mut out = self.out.borrow_mut();
            out.windows_total = out.windows_total.saturating_add(1);
            Ok(())
        }
    }

    struct SnapshotsSeed {
        cfg: Cfg,
        out: std::rc::Rc<std::cell::RefCell<NotifyOut>>,
        window_id: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotsSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(SnapshotsVisitor {
                cfg: self.cfg,
                out: self.out,
                window_id: self.window_id,
            })
        }
    }

    struct SnapshotsVisitor {
        cfg: Cfg,
        out: std::rc::Rc<std::cell::RefCell<NotifyOut>>,
        window_id: u64,
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
                    cfg: self.cfg.clone(),
                    out: self.out.clone(),
                    window_id: self.window_id,
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct SnapshotSeed {
        cfg: Cfg,
        out: std::rc::Rc<std::cell::RefCell<NotifyOut>>,
        window_id: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotVisitor {
                cfg: self.cfg,
                out: self.out,
                window_id: self.window_id,
            })
        }
    }

    struct SnapshotVisitor {
        cfg: Cfg,
        out: std::rc::Rc<std::cell::RefCell<NotifyOut>>,
        window_id: u64,
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
            let mut frame_id: u64 = 0;
            let mut pending: Vec<NotifyReqLite> = Vec::new();

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "frame_id" | "frameId" => {
                        frame_id = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "debug" => {
                        map.next_value_seed(DebugSeed { pending: &mut pending })?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            if frame_id < self.cfg.warmup_frames {
                return Ok(());
            }

            let mut out = self.out.borrow_mut();
            out.examined_snapshots = out.examined_snapshots.saturating_add(1);

            for req in pending {
                out.total_notify_requests = out.total_notify_requests.saturating_add(1);

                let key = format!("{}:{}:{}", req.file, req.line, req.column);
                *out.matched_hotspot_counts.entry(key).or_insert(0) += 1;

                if file_matches(req.file.as_str(), self.cfg.file_filter.as_str()) {
                    out.matched_notify_requests = out.matched_notify_requests.saturating_add(1);
                    if out.matched_samples.len() < 20 {
                        out.matched_samples.push(serde_json::json!({
                            "window_id": self.window_id,
                            "frame_id": frame_id,
                            "caller_node": req.caller_node,
                            "target_view": req.target_view,
                            "file": req.file,
                            "line": req.line,
                            "column": req.column,
                        }));
                    }
                }
            }

            Ok(())
        }
    }

    struct DebugSeed<'a> {
        pending: &'a mut Vec<NotifyReqLite>,
    }

    impl<'de> DeserializeSeed<'de> for DebugSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugVisitor { pending: self.pending })
        }
    }

    struct DebugVisitor<'a> {
        pending: &'a mut Vec<NotifyReqLite>,
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
                    "notify_requests" | "notifyRequests" => {
                        map.next_value_seed(NotifyRequestsSeed { pending: self.pending })?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(())
        }
    }

    struct NotifyRequestsSeed<'a> {
        pending: &'a mut Vec<NotifyReqLite>,
    }

    impl<'de> DeserializeSeed<'de> for NotifyRequestsSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(NotifyRequestsVisitor { pending: self.pending })
        }
    }

    struct NotifyRequestsVisitor<'a> {
        pending: &'a mut Vec<NotifyReqLite>,
    }

    impl<'de> Visitor<'de> for NotifyRequestsVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "notify_requests array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some(req) = seq.next_element_seed(NotifyReqSeed)? {
                self.pending.push(req);
            }
            Ok(())
        }
    }

    struct NotifyReqSeed;

    impl<'de> DeserializeSeed<'de> for NotifyReqSeed {
        type Value = NotifyReqLite;

        fn deserialize<D>(self, deserializer: D) -> Result<NotifyReqLite, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(NotifyReqVisitor {
                out: NotifyReqLite::default(),
            })
        }
    }

    struct NotifyReqVisitor {
        out: NotifyReqLite,
    }

    impl<'de> Visitor<'de> for NotifyReqVisitor {
        type Value = NotifyReqLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "notify request object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<NotifyReqLite, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "caller_node" | "callerNode" => {
                        self.out.caller_node = map.next_value::<Option<u64>>()?;
                    }
                    "target_view" | "targetView" => {
                        self.out.target_view = map.next_value::<Option<u64>>()?;
                    }
                    "file" => {
                        self.out.file = map.next_value::<Option<String>>()?.unwrap_or_default();
                    }
                    "line" => {
                        self.out.line = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "column" => {
                        self.out.column = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(self.out)
        }
    }

    let file = std::fs::File::open(bundle_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);

    let out = std::rc::Rc::new(std::cell::RefCell::new(NotifyOut::default()));
    RootSeed {
        cfg: Cfg {
            file_filter: file_filter.to_string(),
            warmup_frames,
        },
        out: out.clone(),
    }
    .deserialize(&mut de)
    .map_err(|e| e.to_string())?;

    let out = out.borrow().clone();
    if out.windows_total == 0 {
        return Ok(());
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.notify_hotspots.json");

    let mut top_hotspots: Vec<(String, u64)> = out.matched_hotspot_counts.into_iter().collect();
    top_hotspots.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top_hotspots: Vec<serde_json::Value> = top_hotspots
        .into_iter()
        .take(30)
        .map(|(key, count)| serde_json::json!({ "key": key, "count": count }))
        .collect();

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "notify_hotspots",
        "bundle_artifact": bundle_path.display().to_string(),
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": out.examined_snapshots,
        "file_filter": file_filter,
        "max_count": max_count,
        "total_notify_requests": out.total_notify_requests,
        "matched_notify_requests": out.matched_notify_requests,
        "matched_samples": out.matched_samples,
        "top_hotspots": top_hotspots,
    });
    write_json_value(&evidence_path, &payload)?;

    if out.matched_notify_requests > max_count {
        return Err(format!(
            "notify hotspot file budget exceeded: file_filter={file_filter} matched_notify_requests={} max_count={max_count}\n  bundle: {}\n  evidence: {}",
            out.matched_notify_requests,
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    Ok(())
}
