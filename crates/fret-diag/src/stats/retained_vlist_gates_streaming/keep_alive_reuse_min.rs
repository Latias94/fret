use std::path::Path;
use std::rc::Rc;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

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
                let done = { self.out.borrow().keep_alive_reuse_frames >= 3 };
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

                let Some(records) = snapshot.debug.records else {
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
                    if out.offenders.len() < 10 {
                        out.offenders.push(format!(
                            "frame_id={} records={} kept_alive_items_sum={} any_keep_alive_reuse=false",
                            snapshot.frame_id, records.len, records.kept_alive_items_sum
                        ));
                    }
                }

                let done =
                    { self.out.borrow().keep_alive_reuse_frames >= min_keep_alive_reuse_frames };
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
            while let Some((kept_alive, reused_keep_alive)) =
                seq.next_element_seed(ReconcileRecordSeed)?
            {
                self.out.len = self.out.len.saturating_add(1);
                self.out.kept_alive_items_sum =
                    self.out.kept_alive_items_sum.saturating_add(kept_alive);
                self.out.any_keep_alive_reuse |= reused_keep_alive;

                if self.out.any_keep_alive_reuse && self.out.len >= 2 {
                    while seq.next_element::<IgnoredAny>()?.is_some() {}
                    break;
                }
            }
            Ok(self.out)
        }
    }

    struct ReconcileRecordSeed;

    impl<'de> DeserializeSeed<'de> for ReconcileRecordSeed {
        type Value = (u64, bool);

        fn deserialize<D>(self, deserializer: D) -> Result<(u64, bool), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(ReconcileRecordVisitor {
                kept_alive: 0,
                reused_keep_alive: false,
            })
        }
    }

    struct ReconcileRecordVisitor {
        kept_alive: u64,
        reused_keep_alive: bool,
    }

    impl<'de> Visitor<'de> for ReconcileRecordVisitor {
        type Value = (u64, bool);

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
                    "reused_keep_alive_items" | "reusedKeepAliveItems" => {
                        self.reused_keep_alive = map.next_value::<Option<bool>>()?.unwrap_or(false);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((self.kept_alive, self.reused_keep_alive))
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
