// Streaming retained virtual-list gates.
//
// These checks intentionally avoid materializing the full bundle artifact in memory so they can
// run on huge `bundle.json` / `bundle.schema2.json` inputs.

use std::path::Path;
use std::rc::Rc;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

pub(crate) fn check_bundle_for_retained_vlist_reconcile_no_notify_min_streaming(
    bundle_path: &Path,
    min_reconcile_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    const STOP_MARKER: &str = "__FRET_DIAG_RETAINED_VLIST_STOP__";

    #[derive(Debug, Default)]
    struct Out {
        examined_snapshots: u64,
        reconcile_events: u64,
        reconcile_frames: u64,
        offenders: Vec<String>,
    }

    #[derive(Debug, Clone)]
    struct Cfg {
        warmup_frames: u64,
    }

    #[derive(Debug, Clone, Default)]
    struct DirtyViewsLite {
        has_notify: bool,
        notify_root_node: u64,
        notify_source: String,
        notify_detail: String,
    }

    #[derive(Debug, Clone, Default)]
    struct DebugLite {
        reconcile_list_count: u64,
        reconcile_stats_count: u64,
        dirty_views: DirtyViewsLite,
    }

    #[derive(Debug, Clone, Default)]
    struct SnapshotLite {
        frame_id: u64,
        debug: DebugLite,
    }

    fn is_notify_dirty_view(source: &str, detail: &str) -> bool {
        source == "notify" || detail.contains("notify")
    }

    struct RootSeed {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
            {
                let done = { self.out.borrow().offenders.len() >= 10 };
                if done {
                    return Err(serde::de::Error::custom(STOP_MARKER));
                }
            }
            Ok(())
        }
    }

    struct WindowSeed {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "snapshots" => {
                        map.next_value_seed(SnapshotsSeed {
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

    struct SnapshotsSeed {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
            })
        }
    }

    struct SnapshotsVisitor {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
                if snapshot.frame_id < self.cfg.warmup_frames {
                    continue;
                }
                {
                    let mut out = self.out.borrow_mut();
                    out.examined_snapshots = out.examined_snapshots.saturating_add(1);
                }

                let count = snapshot
                    .debug
                    .reconcile_list_count
                    .max(snapshot.debug.reconcile_stats_count);
                if count == 0 {
                    continue;
                }

                {
                    let mut out = self.out.borrow_mut();
                    out.reconcile_frames = out.reconcile_frames.saturating_add(1);
                    out.reconcile_events = out.reconcile_events.saturating_add(count);
                }

                if snapshot.debug.dirty_views.has_notify {
                    let mut out = self.out.borrow_mut();
                    out.offenders.push(format!(
                        "frame_id={} dirty_view_root_node={} source={} detail={}",
                        snapshot.frame_id,
                        snapshot.debug.dirty_views.notify_root_node,
                        snapshot.debug.dirty_views.notify_source,
                        snapshot.debug.dirty_views.notify_detail
                    ));
                }

                let done = { self.out.borrow().offenders.len() >= 10 };
                if done {
                    return Err(serde::de::Error::custom(STOP_MARKER));
                }
            }
            Ok(())
        }
    }

    struct SnapshotSeed;

    impl<'de> DeserializeSeed<'de> for SnapshotSeed {
        type Value = SnapshotLite;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotVisitor {
                out: SnapshotLite::default(),
            })
        }
    }

    struct SnapshotVisitor {
        out: SnapshotLite,
    }

    impl<'de> Visitor<'de> for SnapshotVisitor {
        type Value = SnapshotLite;

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
                        self.out.frame_id = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "debug" => {
                        self.out.debug = map.next_value_seed(DebugSeed)?;
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
        type Value = DebugLite;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugVisitor {
                out: DebugLite::default(),
            })
        }
    }

    struct DebugVisitor {
        out: DebugLite,
    }

    impl<'de> Visitor<'de> for DebugVisitor {
        type Value = DebugLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "retained_virtual_list_reconciles" => {
                        self.out.reconcile_list_count = map.next_value_seed(SeqLenSeed)?;
                    }
                    "dirty_views" => {
                        self.out.dirty_views = map.next_value_seed(DirtyViewsSeed)?;
                    }
                    "stats" => {
                        map.next_value_seed(StatsSeed { out: &mut self.out })?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(self.out)
        }
    }

    struct StatsSeed<'a> {
        out: &'a mut DebugLite,
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
        out: &'a mut DebugLite,
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
                    "retained_virtual_list_reconciles" => {
                        self.out.reconcile_stats_count =
                            map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(())
        }
    }

    struct DirtyViewsSeed;

    impl<'de> DeserializeSeed<'de> for DirtyViewsSeed {
        type Value = DirtyViewsLite;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(DirtyViewsVisitor {
                out: DirtyViewsLite::default(),
            })
        }
    }

    struct DirtyViewsVisitor {
        out: DirtyViewsLite,
    }

    impl<'de> Visitor<'de> for DirtyViewsVisitor {
        type Value = DirtyViewsLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "dirty_views array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<DirtyViewsLite, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some((root_node, source, detail)) = seq.next_element_seed(DirtyViewSeed)? {
                let source = source.unwrap_or_default();
                let detail = detail.unwrap_or_default();
                if is_notify_dirty_view(source.as_str(), detail.as_str()) {
                    self.out.has_notify = true;
                    self.out.notify_root_node = root_node.unwrap_or(0);
                    self.out.notify_source = source;
                    self.out.notify_detail = detail;
                    while seq.next_element::<IgnoredAny>()?.is_some() {}
                    break;
                }
            }
            Ok(self.out)
        }
    }

    struct DirtyViewSeed;

    impl<'de> DeserializeSeed<'de> for DirtyViewSeed {
        type Value = (Option<u64>, Option<String>, Option<String>);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DirtyViewVisitor {
                root_node: None,
                source: None,
                detail: None,
            })
        }
    }

    struct DirtyViewVisitor {
        root_node: Option<u64>,
        source: Option<String>,
        detail: Option<String>,
    }

    impl<'de> Visitor<'de> for DirtyViewVisitor {
        type Value = (Option<u64>, Option<String>, Option<String>);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "dirty view object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "root_node" | "rootNode" => {
                        self.root_node = map.next_value::<Option<u64>>()?;
                    }
                    "source" => {
                        self.source = map.next_value::<Option<String>>()?;
                    }
                    "detail" => {
                        self.detail = map.next_value::<Option<String>>()?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((self.root_node, self.source, self.detail))
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
    }

    let file = std::fs::File::open(bundle_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);

    let out: Rc<std::cell::RefCell<Out>> = Rc::new(std::cell::RefCell::new(Out::default()));
    let res = RootSeed {
        cfg: Cfg { warmup_frames },
        out: out.clone(),
    }
    .deserialize(&mut de);

    if let Err(err) = res {
        let msg = err.to_string();
        if !msg.starts_with(STOP_MARKER) {
            return Err(msg);
        }
    }

    let out = out.borrow();
    if !out.offenders.is_empty() {
        let mut msg = String::new();
        msg.push_str(
            "retained virtual-list reconcile should not require notify-based dirty views\n",
        );
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!(
            "min_reconcile_events={min_reconcile_events} reconcile_events={} reconcile_frames={} warmup_frames={warmup_frames} examined_snapshots={}\n",
            out.reconcile_events, out.reconcile_frames, out.examined_snapshots
        ));
        for line in out.offenders.iter().take(10) {
            msg.push_str("  ");
            msg.push_str(line);
            msg.push('\n');
        }
        return Err(msg);
    }

    if out.reconcile_events < min_reconcile_events {
        return Err(format!(
            "expected at least {min_reconcile_events} retained virtual-list reconcile events, got {} \
(warmup_frames={warmup_frames}, examined_snapshots={}) bundle: {}",
            out.reconcile_events,
            out.examined_snapshots,
            bundle_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_retained_vlist_keep_alive_reuse_min_streaming(
    bundle_path: &Path,
    min_keep_alive_reuse_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    const STOP_MARKER: &str = "__FRET_DIAG_RETAINED_VLIST_STOP__";

    #[derive(Debug, Default)]
    struct Out {
        examined_snapshots: u64,
        keep_alive_reuse_frames: u64,
        offenders: Vec<String>,
    }

    #[derive(Debug, Clone)]
    struct Cfg {
        warmup_frames: u64,
    }

    #[derive(Debug, Clone, Default)]
    struct ReconcileRecordsLite {
        len: u64,
        kept_alive_items_sum: u64,
        any_keep_alive_reuse: bool,
    }

    #[derive(Debug, Clone, Default)]
    struct DebugLite {
        records: Option<ReconcileRecordsLite>,
    }

    #[derive(Debug, Clone, Default)]
    struct SnapshotLite {
        frame_id: u64,
        debug: DebugLite,
    }

    struct RootSeed {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
            {
                let done = { self.out.borrow().offenders.len() >= 10 };
                if done {
                    return Err(serde::de::Error::custom(STOP_MARKER));
                }
            }
            Ok(())
        }
    }

    struct WindowSeed {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "snapshots" => {
                        map.next_value_seed(SnapshotsSeed {
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

    struct SnapshotsSeed {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
            })
        }
    }

    struct SnapshotsVisitor {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
                if snapshot.frame_id < self.cfg.warmup_frames {
                    continue;
                }
                {
                    let mut out = self.out.borrow_mut();
                    out.examined_snapshots = out.examined_snapshots.saturating_add(1);
                }

                let Some(records) = snapshot.debug.records.as_ref() else {
                    continue;
                };
                if records.len == 0 {
                    continue;
                }

                if records.any_keep_alive_reuse {
                    let mut out = self.out.borrow_mut();
                    out.keep_alive_reuse_frames = out.keep_alive_reuse_frames.saturating_add(1);
                } else {
                    let mut out = self.out.borrow_mut();
                    out.offenders.push(format!(
                        "frame_id={} reconciles={} kept_alive_sum={}",
                        snapshot.frame_id, records.len, records.kept_alive_items_sum
                    ));
                }

                let done = { self.out.borrow().offenders.len() >= 10 };
                if done {
                    return Err(serde::de::Error::custom(STOP_MARKER));
                }
            }
            Ok(())
        }
    }

    struct SnapshotSeed;

    impl<'de> DeserializeSeed<'de> for SnapshotSeed {
        type Value = SnapshotLite;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotVisitor {
                out: SnapshotLite::default(),
            })
        }
    }

    struct SnapshotVisitor {
        out: SnapshotLite,
    }

    impl<'de> Visitor<'de> for SnapshotVisitor {
        type Value = SnapshotLite;

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
                        self.out.frame_id = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "debug" => {
                        self.out.debug = map.next_value_seed(DebugSeed)?;
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
        type Value = DebugLite;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugVisitor {
                out: DebugLite::default(),
            })
        }
    }

    struct DebugVisitor {
        out: DebugLite,
    }

    impl<'de> Visitor<'de> for DebugVisitor {
        type Value = DebugLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "retained_virtual_list_reconciles" => {
                        self.out.records = Some(map.next_value_seed(ReconcileRecordsSeed)?);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(self.out)
        }
    }

    struct ReconcileRecordsSeed;

    impl<'de> DeserializeSeed<'de> for ReconcileRecordsSeed {
        type Value = ReconcileRecordsLite;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(ReconcileRecordsVisitor {
                out: ReconcileRecordsLite::default(),
            })
        }
    }

    struct ReconcileRecordsVisitor {
        out: ReconcileRecordsLite,
    }

    impl<'de> Visitor<'de> for ReconcileRecordsVisitor {
        type Value = ReconcileRecordsLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "reconcile records array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<ReconcileRecordsLite, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some((kept_alive, reused_keep_alive)) =
                seq.next_element_seed(ReconcileRecordSeed)?
            {
                self.out.len = self.out.len.saturating_add(1);
                self.out.kept_alive_items_sum =
                    self.out.kept_alive_items_sum.saturating_add(kept_alive);
                if reused_keep_alive > 0 {
                    self.out.any_keep_alive_reuse = true;
                }
            }
            Ok(self.out)
        }
    }

    struct ReconcileRecordSeed;

    impl<'de> DeserializeSeed<'de> for ReconcileRecordSeed {
        type Value = (u64, u64);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(ReconcileRecordVisitor {
                kept_alive: 0,
                reused_keep_alive: 0,
            })
        }
    }

    struct ReconcileRecordVisitor {
        kept_alive: u64,
        reused_keep_alive: u64,
    }

    impl<'de> Visitor<'de> for ReconcileRecordVisitor {
        type Value = (u64, u64);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "reconcile record object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "kept_alive_items" | "keptAliveItems" => {
                        self.kept_alive = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "reused_from_keep_alive_items" | "reusedFromKeepAliveItems" => {
                        self.reused_keep_alive = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((self.kept_alive, self.reused_keep_alive))
        }
    }

    let file = std::fs::File::open(bundle_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);

    let out: Rc<std::cell::RefCell<Out>> = Rc::new(std::cell::RefCell::new(Out::default()));
    let res = RootSeed {
        cfg: Cfg { warmup_frames },
        out: out.clone(),
    }
    .deserialize(&mut de);

    if let Err(err) = res {
        let msg = err.to_string();
        if !msg.starts_with(STOP_MARKER) {
            return Err(msg);
        }
    }

    let out = out.borrow();
    if out.keep_alive_reuse_frames < min_keep_alive_reuse_frames {
        let mut msg = String::new();
        msg.push_str("expected retained virtual-list to reuse keep-alive items\n");
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!(
            "min_keep_alive_reuse_frames={min_keep_alive_reuse_frames} keep_alive_reuse_frames={} warmup_frames={warmup_frames} examined_snapshots={}\n",
            out.keep_alive_reuse_frames, out.examined_snapshots
        ));
        for line in out.offenders.iter().take(10) {
            msg.push_str("  ");
            msg.push_str(line);
            msg.push('\n');
        }
        return Err(msg);
    }

    Ok(())
}

pub(crate) fn check_bundle_for_retained_vlist_attach_detach_max_streaming(
    bundle_path: &Path,
    max_delta: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    const STOP_MARKER: &str = "__FRET_DIAG_RETAINED_VLIST_STOP__";

    #[derive(Debug, Default)]
    struct Out {
        examined_snapshots: u64,
        reconcile_events: u64,
        reconcile_frames: u64,
        offenders: Vec<String>,
    }

    #[derive(Debug, Clone)]
    struct Cfg {
        warmup_frames: u64,
        max_delta: u64,
    }

    #[derive(Debug, Clone, Default)]
    struct ReconcileRecordsLite {
        len: u64,
        attached_items_sum: u64,
        detached_items_sum: u64,
    }

    #[derive(Debug, Clone, Default)]
    struct DebugLite {
        records: Option<ReconcileRecordsLite>,
        reconciles_stats_count: u64,
        attached_items_stats: u64,
        detached_items_stats: u64,
    }

    #[derive(Debug, Clone, Default)]
    struct SnapshotLite {
        frame_id: u64,
        debug: DebugLite,
    }

    struct RootSeed {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
            {
                let done = { self.out.borrow().offenders.len() >= 10 };
                if done {
                    return Err(serde::de::Error::custom(STOP_MARKER));
                }
            }
            Ok(())
        }
    }

    struct WindowSeed {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
        out: Rc<std::cell::RefCell<Out>>,
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
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "snapshots" => {
                        map.next_value_seed(SnapshotsSeed {
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

    struct SnapshotsSeed {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
            })
        }
    }

    struct SnapshotsVisitor {
        cfg: Cfg,
        out: Rc<std::cell::RefCell<Out>>,
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
                if snapshot.frame_id < self.cfg.warmup_frames {
                    continue;
                }
                {
                    let mut out = self.out.borrow_mut();
                    out.examined_snapshots = out.examined_snapshots.saturating_add(1);
                }

                let list_count = snapshot.debug.records.as_ref().map(|r| r.len).unwrap_or(0);
                let stats_count = snapshot.debug.reconciles_stats_count;
                let count = list_count.max(stats_count);
                if count == 0 {
                    continue;
                }

                {
                    let mut out = self.out.borrow_mut();
                    out.reconcile_frames = out.reconcile_frames.saturating_add(1);
                    out.reconcile_events = out.reconcile_events.saturating_add(count);
                }

                let (attached, detached) = match snapshot.debug.records.as_ref() {
                    Some(records) if records.len > 0 => {
                        (records.attached_items_sum, records.detached_items_sum)
                    }
                    _ => (
                        snapshot.debug.attached_items_stats,
                        snapshot.debug.detached_items_stats,
                    ),
                };

                let delta = attached.saturating_add(detached);
                if delta > self.cfg.max_delta {
                    let mut out = self.out.borrow_mut();
                    out.offenders.push(format!(
                        "frame_id={} attached={} detached={} delta={} max={}",
                        snapshot.frame_id, attached, detached, delta, self.cfg.max_delta
                    ));
                }

                let done = { self.out.borrow().offenders.len() >= 10 };
                if done {
                    return Err(serde::de::Error::custom(STOP_MARKER));
                }
            }
            Ok(())
        }
    }

    struct SnapshotSeed;

    impl<'de> DeserializeSeed<'de> for SnapshotSeed {
        type Value = SnapshotLite;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotVisitor {
                out: SnapshotLite::default(),
            })
        }
    }

    struct SnapshotVisitor {
        out: SnapshotLite,
    }

    impl<'de> Visitor<'de> for SnapshotVisitor {
        type Value = SnapshotLite;

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
                        self.out.frame_id = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "debug" => {
                        self.out.debug = map.next_value_seed(DebugSeed)?;
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
        type Value = DebugLite;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugVisitor {
                out: DebugLite::default(),
            })
        }
    }

    struct DebugVisitor {
        out: DebugLite,
    }

    impl<'de> Visitor<'de> for DebugVisitor {
        type Value = DebugLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "retained_virtual_list_reconciles" => {
                        self.out.records = Some(map.next_value_seed(ReconcileRecordsSeed)?);
                    }
                    "stats" => {
                        map.next_value_seed(StatsSeed { out: &mut self.out })?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(self.out)
        }
    }

    struct StatsSeed<'a> {
        out: &'a mut DebugLite,
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
        out: &'a mut DebugLite,
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
                    "retained_virtual_list_reconciles" => {
                        self.out.reconciles_stats_count =
                            map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "retained_virtual_list_attached_items" => {
                        self.out.attached_items_stats =
                            map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "retained_virtual_list_detached_items" => {
                        self.out.detached_items_stats =
                            map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(())
        }
    }

    struct ReconcileRecordsSeed;

    impl<'de> DeserializeSeed<'de> for ReconcileRecordsSeed {
        type Value = ReconcileRecordsLite;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(ReconcileRecordsVisitor {
                out: ReconcileRecordsLite::default(),
            })
        }
    }

    struct ReconcileRecordsVisitor {
        out: ReconcileRecordsLite,
    }

    impl<'de> Visitor<'de> for ReconcileRecordsVisitor {
        type Value = ReconcileRecordsLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "reconcile records array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<ReconcileRecordsLite, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some((attached, detached)) = seq.next_element_seed(ReconcileRecordSeed)? {
                self.out.len = self.out.len.saturating_add(1);
                self.out.attached_items_sum = self.out.attached_items_sum.saturating_add(attached);
                self.out.detached_items_sum = self.out.detached_items_sum.saturating_add(detached);
            }
            Ok(self.out)
        }
    }

    struct ReconcileRecordSeed;

    impl<'de> DeserializeSeed<'de> for ReconcileRecordSeed {
        type Value = (u64, u64);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(ReconcileRecordVisitor {
                attached: 0,
                detached: 0,
            })
        }
    }

    struct ReconcileRecordVisitor {
        attached: u64,
        detached: u64,
    }

    impl<'de> Visitor<'de> for ReconcileRecordVisitor {
        type Value = (u64, u64);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "reconcile record object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "attached_items" | "attachedItems" => {
                        self.attached = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "detached_items" | "detachedItems" => {
                        self.detached = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((self.attached, self.detached))
        }
    }

    let file = std::fs::File::open(bundle_path).map_err(|e| e.to_string())?;
    let reader = std::io::BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);

    let out: Rc<std::cell::RefCell<Out>> = Rc::new(std::cell::RefCell::new(Out::default()));
    let res = RootSeed {
        cfg: Cfg {
            warmup_frames,
            max_delta,
        },
        out: out.clone(),
    }
    .deserialize(&mut de);

    if let Err(err) = res {
        let msg = err.to_string();
        if !msg.starts_with(STOP_MARKER) {
            return Err(msg);
        }
    }

    let out = out.borrow();
    if out.reconcile_events == 0 {
        return Err(format!(
            "expected at least 1 retained virtual-list reconcile event (required for attach/detach max check), got 0 \
(warmup_frames={warmup_frames}, examined_snapshots={}) bundle: {}",
            out.examined_snapshots,
            bundle_path.display()
        ));
    }

    if out.offenders.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("retained virtual-list attach/detach delta exceeded the configured maximum\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "max_delta={max_delta} reconcile_events={} reconcile_frames={} warmup_frames={warmup_frames} examined_snapshots={}\n",
        out.reconcile_events, out.reconcile_frames, out.examined_snapshots
    ));
    for line in out.offenders.iter().take(10) {
        msg.push_str("  ");
        msg.push_str(line);
        msg.push('\n');
    }
    Err(msg)
}
