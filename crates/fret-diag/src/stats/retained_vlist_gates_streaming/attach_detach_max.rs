use std::path::Path;
use std::rc::Rc;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

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

                let (reconcile_count, attached_sum, detached_sum) =
                    snapshot.debug.records.map_or_else(
                        || {
                            (
                                snapshot.debug.reconciles_stats_count,
                                snapshot.debug.attached_items_stats,
                                snapshot.debug.detached_items_stats,
                            )
                        },
                        |r| (r.len, r.attached_items_sum, r.detached_items_sum),
                    );
                if reconcile_count == 0 {
                    continue;
                }

                {
                    let mut out = self.out.borrow_mut();
                    out.reconcile_frames = out.reconcile_frames.saturating_add(1);
                    out.reconcile_events = out.reconcile_events.saturating_add(reconcile_count);
                }

                let delta = attached_sum.max(detached_sum);
                if delta > self.cfg.max_delta {
                    let mut out = self.out.borrow_mut();
                    out.offenders.push(format!(
                        "frame_id={} reconciles={} attached_sum={} detached_sum={} delta={} max_delta={}",
                        snapshot.frame_id,
                        reconcile_count,
                        attached_sum,
                        detached_sum,
                        delta,
                        self.cfg.max_delta
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
                    "retained_virtual_list_reconcile_records" => {
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

        fn deserialize<D>(self, deserializer: D) -> Result<ReconcileRecordsLite, D::Error>
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

                if self.out.len >= 2
                    && (self.out.attached_items_sum >= 2 || self.out.detached_items_sum >= 2)
                {
                    while seq.next_element::<IgnoredAny>()?.is_some() {}
                    break;
                }
            }
            Ok(self.out)
        }
    }

    struct ReconcileRecordSeed;

    impl<'de> DeserializeSeed<'de> for ReconcileRecordSeed {
        type Value = (u64, u64);

        fn deserialize<D>(self, deserializer: D) -> Result<(u64, u64), D::Error>
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

    let out: Rc<std::cell::RefCell<Out>> = Rc::new(std::cell::RefCell::new(Out::default()));
    crate::json_stream::with_bundle_json_deserializer_allow_stop(bundle_path, STOP_MARKER, |de| {
        RootSeed {
            cfg: Cfg {
                warmup_frames,
                max_delta,
            },
            out: out.clone(),
        }
        .deserialize(de)
    })?;

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
