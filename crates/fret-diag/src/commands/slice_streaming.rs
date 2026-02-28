use std::path::Path;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

use super::slice_payload::build_test_id_slice_payload_from_snapshot_and_nodes;

pub(super) fn try_build_test_id_slice_payload_streaming_inline(
    bundle_path: &Path,
    warmup_frames: u64,
    test_id: &str,
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
    window_id: Option<u64>,
    max_matches: usize,
    max_ancestors: usize,
) -> Result<Option<serde_json::Value>, String> {
    if frame_id.is_none() && window_snapshot_seq.is_none() {
        return Ok(None);
    }

    const FOUND_MARKER: &str = "__FRET_DIAG_FOUND_SNAPSHOT__";

    #[derive(Debug, Clone)]
    struct Found {
        window: u64,
        snapshot: serde_json::Value,
        nodes: Vec<serde_json::Value>,
    }

    #[derive(Debug, Clone, Copy)]
    struct Criteria {
        window_id: Option<u64>,
        frame_id: Option<u64>,
        window_snapshot_seq: Option<u64>,
    }

    impl Criteria {
        fn matches_snapshot(
            self,
            snapshot_frame_id: Option<u64>,
            snapshot_window_snapshot_seq: Option<u64>,
        ) -> bool {
            if let Some(req_frame) = self.frame_id {
                return snapshot_frame_id == Some(req_frame);
            }
            if let Some(req_seq) = self.window_snapshot_seq {
                return snapshot_window_snapshot_seq == Some(req_seq);
            }
            false
        }
    }

    struct BundleSeed {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<Found>>>,
    }

    impl<'de> DeserializeSeed<'de> for BundleSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(BundleVisitor {
                crit: self.crit,
                found: self.found,
            })
        }
    }

    struct BundleVisitor {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<Found>>>,
    }

    impl<'de> Visitor<'de> for BundleVisitor {
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
                            crit: self.crit,
                            found: self.found.clone(),
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
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<Found>>>,
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
                    crit: self.crit,
                    found: self.found.clone(),
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct WindowSeed {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<Found>>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(WindowVisitor {
                crit: self.crit,
                found: self.found,
            })
        }
    }

    struct WindowVisitor {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<Found>>>,
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
                        if let Some(req) = self.crit.window_id
                            && req != w
                        {
                            map.next_value::<IgnoredAny>()?;
                            continue;
                        }
                        map.next_value_seed(SnapshotsSeed {
                            crit: self.crit,
                            found: self.found.clone(),
                            window: w,
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
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<Found>>>,
        window: u64,
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
            while seq
                .next_element_seed(SnapshotSeed {
                    crit: self.crit,
                    found: self.found.clone(),
                    window: self.window,
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct SnapshotSeed {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<Found>>>,
        window: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotVisitor {
                crit: self.crit,
                found: self.found,
                window: self.window,
            })
        }
    }

    struct SnapshotVisitor {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<Found>>>,
        window: u64,
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
            let mut frame_id: Option<u64> = None;
            let mut window_snapshot_seq: Option<u64> = None;
            let mut ts: Option<u64> = None;
            let mut window_bounds: Option<serde_json::Value> = None;
            let mut stats: Option<serde_json::Value> = None;
            let mut nodes: Option<Vec<serde_json::Value>> = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "frame_id" | "frameId" => {
                        frame_id = Some(map.next_value::<u64>()?);
                    }
                    "window_snapshot_seq" | "windowSnapshotSeq" => {
                        window_snapshot_seq = Some(map.next_value::<u64>()?);
                    }
                    "timestamp_unix_ms" | "timestamp_ms" => {
                        ts = Some(map.next_value::<u64>()?);
                    }
                    "window_bounds" => {
                        window_bounds = Some(map.next_value::<serde_json::Value>()?);
                    }
                    "debug" => {
                        let is_match = self.crit.matches_snapshot(frame_id, window_snapshot_seq);
                        map.next_value_seed(DebugSeed {
                            want_nodes: is_match,
                            stats: &mut stats,
                            nodes: &mut nodes,
                        })?;
                        if is_match && nodes.is_some() {
                            let mut snapshot = serde_json::Map::new();
                            if let Some(v) = frame_id {
                                snapshot.insert("frame_id".to_string(), v.into());
                            }
                            if let Some(v) = window_snapshot_seq {
                                snapshot.insert("window_snapshot_seq".to_string(), v.into());
                            }
                            if let Some(v) = ts {
                                snapshot.insert("timestamp_unix_ms".to_string(), v.into());
                            }
                            if let Some(v) = window_bounds {
                                snapshot.insert("window_bounds".to_string(), v);
                            }
                            if let Some(stats) = stats {
                                snapshot.insert(
                                    "debug".to_string(),
                                    serde_json::json!({ "stats": stats }),
                                );
                            }

                            let out = Found {
                                window: self.window,
                                snapshot: serde_json::Value::Object(snapshot),
                                nodes: nodes.unwrap_or_default(),
                            };
                            self.found.borrow_mut().replace(out);
                            return Err(serde::de::Error::custom(FOUND_MARKER));
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

    struct DebugSeed<'a> {
        want_nodes: bool,
        stats: &'a mut Option<serde_json::Value>,
        nodes: &'a mut Option<Vec<serde_json::Value>>,
    }

    impl<'de> DeserializeSeed<'de> for DebugSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugVisitor {
                want_nodes: self.want_nodes,
                stats: self.stats,
                nodes: self.nodes,
            })
        }
    }

    struct DebugVisitor<'a> {
        want_nodes: bool,
        stats: &'a mut Option<serde_json::Value>,
        nodes: &'a mut Option<Vec<serde_json::Value>>,
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
                        *self.stats = Some(map.next_value::<serde_json::Value>()?);
                    }
                    "semantics" if self.want_nodes => {
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
        nodes: &'a mut Option<Vec<serde_json::Value>>,
    }

    impl<'de> DeserializeSeed<'de> for SemanticsSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_option(SemanticsOptVisitor { nodes: self.nodes })
        }
    }

    struct SemanticsOptVisitor<'a> {
        nodes: &'a mut Option<Vec<serde_json::Value>>,
    }

    impl<'de> Visitor<'de> for SemanticsOptVisitor<'_> {
        type Value = ();

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "semantics object or null")
        }

        fn visit_none<E>(self) -> Result<(), E>
        where
            E: serde::de::Error,
        {
            Ok(())
        }

        fn visit_unit<E>(self) -> Result<(), E>
        where
            E: serde::de::Error,
        {
            Ok(())
        }

        fn visit_some<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SemanticsVisitor { nodes: self.nodes })
        }
    }

    struct SemanticsVisitor<'a> {
        nodes: &'a mut Option<Vec<serde_json::Value>>,
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
                        *self.nodes = Some(map.next_value::<Vec<serde_json::Value>>()?);
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

    let found: std::rc::Rc<std::cell::RefCell<Option<Found>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));

    let crit = Criteria {
        window_id,
        frame_id,
        window_snapshot_seq,
    };

    let res = BundleSeed {
        crit,
        found: found.clone(),
    }
    .deserialize(&mut de);

    if let Err(err) = res {
        let msg = err.to_string();
        if !msg.starts_with(FOUND_MARKER) {
            return Err(err.to_string());
        }
    }

    let Some(found) = found.borrow_mut().take() else {
        return Ok(None);
    };

    Ok(Some(build_test_id_slice_payload_from_snapshot_and_nodes(
        bundle_path,
        warmup_frames,
        found.window,
        &found.snapshot,
        found.nodes.as_slice(),
        test_id,
        max_matches,
        max_ancestors,
    )?))
}

pub(super) fn try_build_test_id_slice_payload_streaming_table(
    bundle_path: &Path,
    warmup_frames: u64,
    test_id: &str,
    frame_id: Option<u64>,
    window_snapshot_seq: Option<u64>,
    window_id: Option<u64>,
    expected_semantics_fingerprint: Option<u64>,
    max_matches: usize,
    max_ancestors: usize,
) -> Result<Option<serde_json::Value>, String> {
    if frame_id.is_none() && window_snapshot_seq.is_none() {
        return Ok(None);
    }

    let Some(expected_fp) = expected_semantics_fingerprint else {
        return Ok(None);
    };

    const FOUND_MARKER: &str = "__FRET_DIAG_FOUND_SNAPSHOT__";

    #[derive(Debug, Clone)]
    struct FoundSnapshot {
        window: u64,
        snapshot: serde_json::Value,
    }

    #[derive(Debug, Clone, Copy)]
    struct Criteria {
        window_id: Option<u64>,
        frame_id: Option<u64>,
        window_snapshot_seq: Option<u64>,
    }

    impl Criteria {
        fn matches_snapshot(
            self,
            snapshot_frame_id: Option<u64>,
            snapshot_window_snapshot_seq: Option<u64>,
        ) -> bool {
            if let Some(req_frame) = self.frame_id {
                return snapshot_frame_id == Some(req_frame);
            }
            if let Some(req_seq) = self.window_snapshot_seq {
                return snapshot_window_snapshot_seq == Some(req_seq);
            }
            false
        }
    }

    struct BundleSeed {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<FoundSnapshot>>>,
    }

    impl<'de> DeserializeSeed<'de> for BundleSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(BundleVisitor {
                crit: self.crit,
                found: self.found,
            })
        }
    }

    struct BundleVisitor {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<FoundSnapshot>>>,
    }

    impl<'de> Visitor<'de> for BundleVisitor {
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
                            crit: self.crit,
                            found: self.found.clone(),
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
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<FoundSnapshot>>>,
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
                    crit: self.crit,
                    found: self.found.clone(),
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct WindowSeed {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<FoundSnapshot>>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(WindowVisitor {
                crit: self.crit,
                found: self.found,
            })
        }
    }

    struct WindowVisitor {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<FoundSnapshot>>>,
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
                        if let Some(req) = self.crit.window_id
                            && req != w
                        {
                            map.next_value::<IgnoredAny>()?;
                            continue;
                        }
                        map.next_value_seed(SnapshotsSeed {
                            crit: self.crit,
                            found: self.found.clone(),
                            window: w,
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
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<FoundSnapshot>>>,
        window: u64,
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
            while seq
                .next_element_seed(SnapshotSeed {
                    crit: self.crit,
                    found: self.found.clone(),
                    window: self.window,
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct SnapshotSeed {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<FoundSnapshot>>>,
        window: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotVisitor {
                crit: self.crit,
                found: self.found,
                window: self.window,
            })
        }
    }

    struct SnapshotVisitor {
        crit: Criteria,
        found: std::rc::Rc<std::cell::RefCell<Option<FoundSnapshot>>>,
        window: u64,
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
            let mut frame_id: Option<u64> = None;
            let mut window_snapshot_seq: Option<u64> = None;
            let mut ts: Option<u64> = None;
            let mut window_bounds: Option<serde_json::Value> = None;
            let mut stats: Option<serde_json::Value> = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "frame_id" | "frameId" => {
                        frame_id = Some(map.next_value::<u64>()?);
                    }
                    "window_snapshot_seq" | "windowSnapshotSeq" => {
                        window_snapshot_seq = Some(map.next_value::<u64>()?);
                    }
                    "timestamp_unix_ms" | "timestamp_ms" => {
                        ts = Some(map.next_value::<u64>()?);
                    }
                    "window_bounds" => {
                        window_bounds = Some(map.next_value::<serde_json::Value>()?);
                    }
                    "debug" => {
                        let is_match = self.crit.matches_snapshot(frame_id, window_snapshot_seq);
                        if is_match {
                            map.next_value_seed(DebugStatsSeed { stats: &mut stats })?;
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            let is_match = self.crit.matches_snapshot(frame_id, window_snapshot_seq);
            if is_match {
                let mut snapshot = serde_json::Map::new();
                if let Some(v) = frame_id {
                    snapshot.insert("frame_id".to_string(), v.into());
                }
                if let Some(v) = window_snapshot_seq {
                    snapshot.insert("window_snapshot_seq".to_string(), v.into());
                }
                if let Some(v) = ts {
                    snapshot.insert("timestamp_unix_ms".to_string(), v.into());
                }
                if let Some(v) = window_bounds {
                    snapshot.insert("window_bounds".to_string(), v);
                }
                if let Some(stats) = stats {
                    snapshot.insert("debug".to_string(), serde_json::json!({ "stats": stats }));
                }

                let out = FoundSnapshot {
                    window: self.window,
                    snapshot: serde_json::Value::Object(snapshot),
                };
                self.found.borrow_mut().replace(out);
                return Err(serde::de::Error::custom(FOUND_MARKER));
            }

            Ok(())
        }
    }

    struct DebugStatsSeed<'a> {
        stats: &'a mut Option<serde_json::Value>,
    }

    impl<'de> DeserializeSeed<'de> for DebugStatsSeed<'_> {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugStatsVisitor { stats: self.stats })
        }
    }

    struct DebugStatsVisitor<'a> {
        stats: &'a mut Option<serde_json::Value>,
    }

    impl<'de> Visitor<'de> for DebugStatsVisitor<'_> {
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
                        *self.stats = Some(map.next_value::<serde_json::Value>()?);
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

    let found: std::rc::Rc<std::cell::RefCell<Option<FoundSnapshot>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));

    let crit = Criteria {
        window_id,
        frame_id,
        window_snapshot_seq,
    };

    let res = BundleSeed {
        crit,
        found: found.clone(),
    }
    .deserialize(&mut de);

    if let Err(err) = res {
        let msg = err.to_string();
        if !msg.starts_with(FOUND_MARKER) {
            return Err(err.to_string());
        }
    }

    let Some(found_snapshot) = found.borrow_mut().take() else {
        return Ok(None);
    };

    let nodes = crate::json_bundle::stream_read_semantics_table_nodes(
        bundle_path,
        found_snapshot.window,
        expected_fp,
    )?;
    let Some(nodes) = nodes else {
        let bundle_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
        return Err(format!(
            "bundle.index.json indicates semantics_source=table, but the bundle artifact has no matching semantics table entry \
(window={} semantics_fingerprint={expected_fp}).\n\
  bundle: {}\n\
  hint: try regenerating schema2 and sidecars:\n\
    - fretboard diag doctor --fix-schema2 {} --warmup-frames {warmup_frames}\n\
    - fretboard diag index {} --warmup-frames {warmup_frames}\n\
  if this came from a share zip, re-extract/re-capture to ensure the schema2 bundle includes tables.semantics.entries.",
            found_snapshot.window,
            bundle_path.display(),
            bundle_dir.display(),
            bundle_dir.display(),
        ));
    };

    Ok(Some(build_test_id_slice_payload_from_snapshot_and_nodes(
        bundle_path,
        warmup_frames,
        found_snapshot.window,
        &found_snapshot.snapshot,
        nodes.as_slice(),
        test_id,
        max_matches,
        max_ancestors,
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_streaming_inline_can_extract_nodes_for_frame_id() {
        let bundle = r#"{
  "schema_version": 1,
  "windows": [
    {
      "window": 1,
      "snapshots": [
        {
          "frame_id": 5,
          "window_snapshot_seq": 2,
          "timestamp_unix_ms": 123,
          "window_bounds": { "x": 0, "y": 0, "w": 10, "h": 10 },
          "debug": {
            "stats": {
              "total_time_us": 1,
              "layout_time_us": 2,
              "prepaint_time_us": 3,
              "paint_time_us": 4,
              "invalidation_walk_calls": 5,
              "invalidation_walk_nodes": 6
            },
            "semantics": {
              "nodes": [
                { "id": 10, "test_id": "x", "role": "button", "children": [11] },
                { "id": 11, "parent": 10, "role": "label", "test_id": "y" }
              ]
            }
          }
        }
      ]
    }
  ]
}"#;

        let tmp = std::env::temp_dir().join(format!(
            "fret-diag-slice-streaming-inline-{}.bundle.json",
            crate::util::now_unix_ms()
        ));
        std::fs::write(&tmp, bundle.as_bytes()).unwrap();

        let out = try_build_test_id_slice_payload_streaming_inline(
            &tmp,
            0,
            "x",
            Some(5),
            None,
            Some(1),
            20,
            64,
        )
        .unwrap()
        .expect("payload");

        assert_eq!(
            out.get("kind").and_then(|v| v.as_str()),
            Some("slice.test_id")
        );
        assert_eq!(out.get("window").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(out.get("frame_id").and_then(|v| v.as_u64()), Some(5));
        assert_eq!(out.get("test_id").and_then(|v| v.as_str()), Some("x"));
        assert!(
            out.get("matches")
                .and_then(|v| v.as_array())
                .is_some_and(|v| !v.is_empty())
        );

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn slice_streaming_table_can_extract_nodes_for_frame_id() {
        let bundle = r#"{
  "schema_version": 2,
  "windows": [
    {
      "window": 1,
      "snapshots": [
        {
          "frame_id": 7,
          "window_snapshot_seq": 9,
          "timestamp_unix_ms": 123,
          "window_bounds": { "x": 0, "y": 0, "w": 10, "h": 10 },
          "semantics_fingerprint": 42,
          "debug": {
            "stats": { "total_time_us": 1 }
          }
        }
      ]
    }
  ],
  "tables": {
    "semantics": {
      "entries": [
        {
          "window": 1,
          "semantics_fingerprint": 42,
          "semantics": {
            "nodes": [
              { "id": 10, "test_id": "x", "role": "button", "children": [11] },
              { "id": 11, "parent": 10, "role": "label", "test_id": "y" }
            ]
          }
        }
      ]
    }
  }
}"#;

        let tmp = std::env::temp_dir().join(format!(
            "fret-diag-slice-streaming-table-{}.bundle.json",
            crate::util::now_unix_ms()
        ));
        std::fs::write(&tmp, bundle.as_bytes()).unwrap();

        let out = try_build_test_id_slice_payload_streaming_table(
            &tmp,
            0,
            "x",
            Some(7),
            None,
            Some(1),
            Some(42),
            20,
            64,
        )
        .unwrap()
        .expect("payload");

        assert_eq!(
            out.get("kind").and_then(|v| v.as_str()),
            Some("slice.test_id")
        );
        assert_eq!(out.get("window").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(out.get("frame_id").and_then(|v| v.as_u64()), Some(7));
        assert_eq!(out.get("test_id").and_then(|v| v.as_str()), Some("x"));
        assert!(
            out.get("matches")
                .and_then(|v| v.as_array())
                .is_some_and(|v| !v.is_empty())
        );

        let _ = std::fs::remove_file(&tmp);
    }
}
