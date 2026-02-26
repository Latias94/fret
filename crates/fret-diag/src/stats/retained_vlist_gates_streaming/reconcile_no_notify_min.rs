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

    let out: Rc<std::cell::RefCell<Out>> = Rc::new(std::cell::RefCell::new(Out::default()));
    crate::json_stream::with_bundle_json_deserializer_allow_stop(bundle_path, STOP_MARKER, |de| {
        RootSeed {
            cfg: Cfg { warmup_frames },
            out: out.clone(),
        }
        .deserialize(de)
    })?;

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
