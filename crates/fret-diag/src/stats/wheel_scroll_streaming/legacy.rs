// Streaming wheel-scroll gates.
//
// These checks intentionally avoid materializing the full bundle artifact in memory so they can
// run on huge `bundle.json` / `bundle.schema2.json` inputs.
//
// Note: this module is being gradually split into smaller, testable pieces under
// `crates/fret-diag/src/stats/wheel_scroll_streaming/`.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

use super::super::semantics::is_descendant;
use super::types::{SemanticsLite, SnapshotMeta, WindowWheelMeta, resolve_semantics_lite};
use super::wheel_frames_min::read_wheel_frames_min_by_window;

fn read_window_before_after_metas(
    bundle_path: &Path,
    wheel_frames: &HashMap<u64, u64>,
    warmup_frames: u64,
) -> Result<Vec<WindowWheelMeta>, String> {
    struct RootSeed {
        wheel_frames: HashMap<u64, u64>,
        warmup_frames: u64,
        out: std::rc::Rc<std::cell::RefCell<Vec<WindowWheelMeta>>>,
    }

    impl<'de> DeserializeSeed<'de> for RootSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(RootVisitor {
                wheel_frames: self.wheel_frames,
                warmup_frames: self.warmup_frames,
                out: self.out,
            })
        }
    }

    struct RootVisitor {
        wheel_frames: HashMap<u64, u64>,
        warmup_frames: u64,
        out: std::rc::Rc<std::cell::RefCell<Vec<WindowWheelMeta>>>,
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
                            wheel_frames: self.wheel_frames.clone(),
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
        wheel_frames: HashMap<u64, u64>,
        warmup_frames: u64,
        out: std::rc::Rc<std::cell::RefCell<Vec<WindowWheelMeta>>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowsSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(WindowsVisitor {
                wheel_frames: self.wheel_frames,
                warmup_frames: self.warmup_frames,
                out: self.out,
            })
        }
    }

    struct WindowsVisitor {
        wheel_frames: HashMap<u64, u64>,
        warmup_frames: u64,
        out: std::rc::Rc<std::cell::RefCell<Vec<WindowWheelMeta>>>,
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
                    wheel_frames: self.wheel_frames.clone(),
                    warmup_frames: self.warmup_frames,
                    out: self.out.clone(),
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct WindowSeed {
        wheel_frames: HashMap<u64, u64>,
        warmup_frames: u64,
        out: std::rc::Rc<std::cell::RefCell<Vec<WindowWheelMeta>>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(WindowVisitor {
                wheel_frames: self.wheel_frames,
                warmup_frames: self.warmup_frames,
                out: self.out,
            })
        }
    }

    struct WindowVisitor {
        wheel_frames: HashMap<u64, u64>,
        warmup_frames: u64,
        out: std::rc::Rc<std::cell::RefCell<Vec<WindowWheelMeta>>>,
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
            let mut wheel_frame: Option<u64> = None;
            let mut before: Option<SnapshotMeta> = None;
            let mut after: Option<SnapshotMeta> = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "window" | "window_id" | "windowId" => {
                        window_id = map.next_value::<u64>()?;
                        wheel_frame = self.wheel_frames.get(&window_id).copied();
                    }
                    "snapshots" => {
                        let Some(wheel_frame) = wheel_frame else {
                            map.next_value::<IgnoredAny>()?;
                            continue;
                        };
                        let after_frame = wheel_frame.max(self.warmup_frames);
                        let (b, a) = map.next_value_seed(SnapshotsBeforeAfterSeed {
                            window_id,
                            after_frame,
                        })?;
                        before = b;
                        after = a;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            let Some(wheel_frame) = wheel_frame else {
                return Ok(());
            };
            self.out.borrow_mut().push(WindowWheelMeta {
                window_id,
                wheel_frame,
                before,
                after,
            });
            Ok(())
        }
    }

    struct SnapshotsBeforeAfterSeed {
        window_id: u64,
        after_frame: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotsBeforeAfterSeed {
        type Value = (Option<SnapshotMeta>, Option<SnapshotMeta>);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(SnapshotsBeforeAfterVisitor {
                window_id: self.window_id,
                after_frame: self.after_frame,
                before: None,
                after: None,
                before_frame: 0,
                done: false,
            })
        }
    }

    struct SnapshotsBeforeAfterVisitor {
        window_id: u64,
        after_frame: u64,
        before: Option<SnapshotMeta>,
        after: Option<SnapshotMeta>,
        before_frame: u64,
        done: bool,
    }

    impl<'de> Visitor<'de> for SnapshotsBeforeAfterVisitor {
        type Value = (Option<SnapshotMeta>, Option<SnapshotMeta>);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "snapshots array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            loop {
                if self.done {
                    if seq.next_element::<IgnoredAny>()?.is_none() {
                        break;
                    }
                    continue;
                }
                let Some(meta) = seq.next_element_seed(SnapshotMetaSeed {
                    window_id: self.window_id,
                })?
                else {
                    break;
                };
                if meta.frame_id < self.after_frame {
                    if meta.frame_id >= self.before_frame {
                        self.before_frame = meta.frame_id;
                        self.before = Some(meta);
                    }
                } else if self.after.is_none() {
                    self.after = Some(meta);
                    self.done = true;
                }
            }
            Ok((self.before, self.after))
        }
    }

    struct SnapshotMetaSeed {
        window_id: u64,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotMetaSeed {
        type Value = SnapshotMeta;

        fn deserialize<D>(self, deserializer: D) -> Result<SnapshotMeta, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotMetaVisitor {
                window_id: self.window_id,
            })
        }
    }

    struct SnapshotMetaVisitor {
        window_id: u64,
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
            let mut semantics_window_id: Option<u64> = None;
            let mut hit: Option<u64> = None;
            let mut vlist_offset: Option<f64> = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "frame_id" | "frameId" => {
                        frame_id = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "window" | "window_id" | "windowId" => {
                        semantics_window_id = map.next_value::<Option<u64>>()?;
                    }
                    "semantics_fingerprint" | "semanticsFingerprint" => {
                        semantics_fingerprint = map.next_value::<Option<u64>>()?;
                    }
                    "debug" => {
                        let (h, off) = map.next_value_seed(DebugHitAndVlistSeed)?;
                        hit = h;
                        vlist_offset = off;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            Ok(SnapshotMeta {
                frame_id,
                semantics_fingerprint,
                semantics_window_id: semantics_window_id.unwrap_or(self.window_id),
                hit,
                vlist_offset,
            })
        }
    }

    struct DebugHitAndVlistSeed;

    impl<'de> DeserializeSeed<'de> for DebugHitAndVlistSeed {
        type Value = (Option<u64>, Option<f64>);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugHitAndVlistVisitor {
                hit: None,
                vlist_offset: None,
            })
        }
    }

    struct DebugHitAndVlistVisitor {
        hit: Option<u64>,
        vlist_offset: Option<f64>,
    }

    impl<'de> Visitor<'de> for DebugHitAndVlistVisitor {
        type Value = (Option<u64>, Option<f64>);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "hit_test" | "hitTest" => {
                        self.hit = map.next_value_seed(HitTestHitSeed)?;
                    }
                    "virtual_list_windows" | "virtualListWindows" => {
                        self.vlist_offset = map.next_value_seed(VlistOffsetSeed)?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((self.hit, self.vlist_offset))
        }
    }

    struct HitTestHitSeed;

    impl<'de> DeserializeSeed<'de> for HitTestHitSeed {
        type Value = Option<u64>;

        fn deserialize<D>(self, deserializer: D) -> Result<Option<u64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(HitTestHitVisitor { hit: None })
        }
    }

    struct HitTestHitVisitor {
        hit: Option<u64>,
    }

    impl<'de> Visitor<'de> for HitTestHitVisitor {
        type Value = Option<u64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "hit_test object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<u64>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "hit" {
                    self.hit = map.next_value::<Option<u64>>()?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(self.hit)
        }
    }

    struct VlistOffsetSeed;

    impl<'de> DeserializeSeed<'de> for VlistOffsetSeed {
        type Value = Option<f64>;

        fn deserialize<D>(self, deserializer: D) -> Result<Option<f64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(VlistOffsetVisitor { offset: None })
        }
    }

    struct VlistOffsetVisitor {
        offset: Option<f64>,
    }

    impl<'de> Visitor<'de> for VlistOffsetVisitor {
        type Value = Option<f64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "virtual_list_windows array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<Option<f64>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            if let Some(off) = seq.next_element_seed(FirstVlistWindowOffsetSeed)? {
                let Some(off) = off else {
                    while seq.next_element::<IgnoredAny>()?.is_some() {}
                    return Ok(None);
                };
                self.offset = Some(off);
            }
            while seq.next_element::<IgnoredAny>()?.is_some() {}
            Ok(self.offset)
        }

        fn visit_map<M>(self, mut map: M) -> Result<Option<f64>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while map.next_key::<IgnoredAny>()?.is_some() {
                map.next_value::<IgnoredAny>()?;
            }
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Option<f64>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_none<E>(self) -> Result<Option<f64>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    struct FirstVlistWindowOffsetSeed;

    impl<'de> DeserializeSeed<'de> for FirstVlistWindowOffsetSeed {
        type Value = Option<f64>;

        fn deserialize<D>(self, deserializer: D) -> Result<Option<f64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(FirstVlistWindowOffsetVisitor { offset: None })
        }
    }

    struct FirstVlistWindowOffsetVisitor {
        offset: Option<f64>,
    }

    impl<'de> Visitor<'de> for FirstVlistWindowOffsetVisitor {
        type Value = Option<f64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "vlist window object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<f64>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "offset" {
                    self.offset = map.next_value::<Option<f64>>()?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(self.offset)
        }
    }

    let out = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    crate::json_stream::with_bundle_json_deserializer(bundle_path, |de| {
        RootSeed {
            wheel_frames: wheel_frames.clone(),
            warmup_frames,
            out: out.clone(),
        }
        .deserialize(de)
    })?;
    Ok(out.borrow().clone())
}

fn stream_read_inline_semantics_lite_for_pairs(
    bundle_path: &Path,
    wanted: &HashMap<u64, HashSet<u64>>,
    test_id: &str,
) -> Result<HashMap<(u64, u64), Option<SemanticsLite>>, String> {
    const FOUND_MARKER: &str = "__FRET_DIAG_FOUND_SEM_LITE__";

    #[derive(Debug, Clone)]
    struct State {
        wanted: HashMap<u64, HashSet<u64>>,
        found: HashMap<(u64, u64), Option<SemanticsLite>>,
        total: usize,
        test_id: String,
    }

    impl State {
        fn is_done(&self) -> bool {
            self.found.len() >= self.total
        }
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
                        let wanted_frames = {
                            let st = self.state.borrow();
                            st.wanted.get(&window_id).cloned().unwrap_or_default()
                        };
                        if wanted_frames.is_empty() {
                            map.next_value::<IgnoredAny>()?;
                            continue;
                        }
                        map.next_value_seed(SnapshotsSeed {
                            window_id,
                            wanted_frames,
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
        wanted_frames: HashSet<u64>,
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
                wanted_frames: self.wanted_frames,
                state: self.state,
            })
        }
    }

    struct SnapshotsVisitor {
        window_id: u64,
        wanted_frames: HashSet<u64>,
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
            let test_id = { self.state.borrow().test_id.clone() };
            while let Some(v) = seq.next_element_seed(SnapshotSemLiteSeed {
                wanted_frames: self.wanted_frames.clone(),
                test_id: test_id.clone(),
            })? {
                let Some((frame_id, sem)) = v else {
                    continue;
                };
                self.state
                    .borrow_mut()
                    .found
                    .insert((self.window_id, frame_id), sem);
                let done = { self.state.borrow().is_done() };
                if done {
                    return Err(serde::de::Error::custom(FOUND_MARKER));
                }
            }
            Ok(())
        }
    }

    struct SnapshotSemLiteSeed {
        wanted_frames: HashSet<u64>,
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for SnapshotSemLiteSeed {
        type Value = Option<(u64, Option<SemanticsLite>)>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(SnapshotSemLiteVisitor {
                wanted_frames: self.wanted_frames,
                test_id: self.test_id,
                frame_id: 0,
                sem: None,
            })
        }
    }

    struct SnapshotSemLiteVisitor {
        wanted_frames: HashSet<u64>,
        test_id: String,
        frame_id: u64,
        sem: Option<Option<SemanticsLite>>,
    }

    impl<'de> Visitor<'de> for SnapshotSemLiteVisitor {
        type Value = Option<(u64, Option<SemanticsLite>)>;

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
                        self.frame_id = map.next_value::<Option<u64>>()?.unwrap_or(0);
                    }
                    "debug" => {
                        if self.wanted_frames.contains(&self.frame_id) {
                            self.sem = Some(map.next_value_seed(DebugSemLiteSeed {
                                test_id: self.test_id.clone(),
                            })?);
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    "semantics" | "semantic_tree" | "semanticTree" | "tree" => {
                        if self.wanted_frames.contains(&self.frame_id) && self.sem.is_none() {
                            self.sem = Some(map.next_value_seed(SemanticsLiteSeed {
                                test_id: self.test_id.clone(),
                            })?);
                        } else {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            if !self.wanted_frames.contains(&self.frame_id) {
                return Ok(None);
            }

            Ok(Some((self.frame_id, self.sem.unwrap_or(None))))
        }
    }

    struct DebugSemLiteSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for DebugSemLiteSeed {
        type Value = Option<SemanticsLite>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(DebugSemLiteVisitor {
                test_id: self.test_id,
                sem: None,
            })
        }
    }

    struct DebugSemLiteVisitor {
        test_id: String,
        sem: Option<Option<SemanticsLite>>,
    }

    impl<'de> Visitor<'de> for DebugSemLiteVisitor {
        type Value = Option<SemanticsLite>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "debug object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<SemanticsLite>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "semantics" => {
                        self.sem = Some(map.next_value_seed(SemanticsLiteSeed {
                            test_id: self.test_id.clone(),
                        })?);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok(self.sem.unwrap_or(None))
        }
    }

    struct SemanticsLiteSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for SemanticsLiteSeed {
        type Value = Option<SemanticsLite>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(SemanticsLiteVisitor {
                test_id: self.test_id,
                out: SemanticsLite::default(),
                has_nodes: false,
            })
        }
    }

    struct SemanticsLiteVisitor {
        test_id: String,
        out: SemanticsLite,
        has_nodes: bool,
    }

    impl<'de> Visitor<'de> for SemanticsLiteVisitor {
        type Value = Option<SemanticsLite>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "a semantics object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<SemanticsLite>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                if key == "nodes" {
                    self.has_nodes = true;
                    self.out = map.next_value_seed(SemanticsNodesLiteSeed {
                        test_id: self.test_id.clone(),
                    })?;
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            if self.has_nodes {
                Ok(Some(self.out))
            } else {
                Ok(None)
            }
        }

        fn visit_unit<E>(self) -> Result<Option<SemanticsLite>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_none<E>(self) -> Result<Option<SemanticsLite>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    struct SemanticsNodesLiteSeed {
        test_id: String,
    }

    impl<'de> DeserializeSeed<'de> for SemanticsNodesLiteSeed {
        type Value = SemanticsLite;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(SemanticsNodesLiteVisitor {
                test_id: self.test_id,
                out: SemanticsLite::default(),
            })
        }
    }

    struct SemanticsNodesLiteVisitor {
        test_id: String,
        out: SemanticsLite,
    }

    impl<'de> Visitor<'de> for SemanticsNodesLiteVisitor {
        type Value = SemanticsLite;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "nodes array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<SemanticsLite, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some((id, parent, test_id)) = seq.next_element_seed(NodeLiteSeed)? {
                if let (Some(id), Some(parent)) = (id, parent) {
                    self.out.parents.insert(id, parent);
                }
                if self.out.target_node_id.is_none()
                    && test_id.as_deref() == Some(self.test_id.as_str())
                {
                    self.out.target_node_id = id;
                }
            }
            Ok(self.out)
        }
    }

    struct NodeLiteSeed;

    impl<'de> DeserializeSeed<'de> for NodeLiteSeed {
        type Value = (Option<u64>, Option<u64>, Option<String>);

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(NodeLiteVisitor {
                id: None,
                parent: None,
                test_id: None,
            })
        }
    }

    struct NodeLiteVisitor {
        id: Option<u64>,
        parent: Option<u64>,
        test_id: Option<String>,
    }

    impl<'de> Visitor<'de> for NodeLiteVisitor {
        type Value = (Option<u64>, Option<u64>, Option<String>);

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "node object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "id" => self.id = map.next_value::<Option<u64>>()?,
                    "parent" => self.parent = map.next_value::<Option<u64>>()?,
                    "test_id" => self.test_id = map.next_value::<Option<String>>()?,
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            Ok((self.id, self.parent, self.test_id))
        }
    }

    let total = wanted.values().map(|s| s.len()).sum::<usize>();
    let state = std::rc::Rc::new(std::cell::RefCell::new(State {
        wanted: wanted.clone(),
        found: HashMap::new(),
        total,
        test_id: test_id.to_string(),
    }));

    crate::json_stream::with_bundle_json_deserializer_allow_stop(
        bundle_path,
        FOUND_MARKER,
        |de| {
            RootSeed {
                state: state.clone(),
            }
            .deserialize(de)
        },
    )?;

    Ok(state.borrow().found.clone())
}

pub(crate) fn check_bundle_for_wheel_scroll_streaming(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let wheel_frames = read_wheel_frames_min_by_window(bundle_path)?;
    if wheel_frames.is_empty() {
        return Err(format!(
            "wheel scroll check requires at least one pointer.wheel event in the bundle: {}",
            bundle_path.display()
        ));
    }

    let windows = read_window_before_after_metas(bundle_path, &wheel_frames, warmup_frames)?;

    let mut wanted: HashMap<u64, HashSet<u64>> = HashMap::new();
    for w in &windows {
        if let (Some(b), Some(a)) = (w.before.as_ref(), w.after.as_ref()) {
            wanted.entry(w.window_id).or_default().insert(b.frame_id);
            wanted.entry(w.window_id).or_default().insert(a.frame_id);
        }
    }
    let inline_sem = stream_read_inline_semantics_lite_for_pairs(bundle_path, &wanted, test_id)?;

    let mut failures: Vec<String> = Vec::new();
    for w in windows {
        let window_id = w.window_id;
        let wheel_frame = w.wheel_frame;
        let after_frame_id = w.after.as_ref().map(|m| m.frame_id).unwrap_or(0);

        let (Some(before), Some(after)) = (w.before.as_ref(), w.after.as_ref()) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_before_or_after_snapshot"
            ));
            continue;
        };

        let Some(hit_before) = before.hit else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_hit_before"
            ));
            continue;
        };
        let Some(hit_after) = after.hit else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} error=missing_hit_after"
            ));
            continue;
        };

        let Some(before_sem) =
            resolve_semantics_lite(bundle_path, &inline_sem, before, window_id, test_id)?
        else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(after_sem) =
            resolve_semantics_lite(bundle_path, &inline_sem, after, window_id, test_id)?
        else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        let Some(target_before) = before_sem.target_node_id else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(target_after) = after_sem.target_node_id else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        if !is_descendant(hit_before, target_before, &before_sem.parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=hit_not_within_target_before hit={hit_before} target={target_before}"
            ));
            continue;
        }

        if is_descendant(hit_after, target_after, &after_sem.parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=hit_still_within_target_after hit={hit_after} target={target_after}"
            ));
        }
    }

    if failures.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("wheel scroll check failed (expected hit-test result to move after wheel)\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in failures {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(crate) fn check_bundle_for_wheel_scroll_hit_changes_streaming(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let wheel_frames = read_wheel_frames_min_by_window(bundle_path)?;
    if wheel_frames.is_empty() {
        return Err(format!(
            "wheel scroll hit-change check requires at least one pointer.wheel event in the bundle: {}",
            bundle_path.display()
        ));
    }

    let windows = read_window_before_after_metas(bundle_path, &wheel_frames, warmup_frames)?;

    let mut wanted: HashMap<u64, HashSet<u64>> = HashMap::new();
    for w in &windows {
        if let (Some(b), Some(a)) = (w.before.as_ref(), w.after.as_ref()) {
            wanted.entry(w.window_id).or_default().insert(b.frame_id);
            wanted.entry(w.window_id).or_default().insert(a.frame_id);
        }
    }
    let inline_sem = stream_read_inline_semantics_lite_for_pairs(bundle_path, &wanted, test_id)?;

    let mut failures: Vec<String> = Vec::new();
    for w in windows {
        let window_id = w.window_id;
        let wheel_frame = w.wheel_frame;
        let after_frame_id = w.after.as_ref().map(|m| m.frame_id).unwrap_or(0);

        let (Some(before), Some(after)) = (w.before.as_ref(), w.after.as_ref()) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_before_or_after_snapshot"
            ));
            continue;
        };

        let Some(before_sem) =
            resolve_semantics_lite(bundle_path, &inline_sem, before, window_id, test_id)?
        else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(after_sem) =
            resolve_semantics_lite(bundle_path, &inline_sem, after, window_id, test_id)?
        else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        let Some(target_before) = before_sem.target_node_id else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(target_after) = after_sem.target_node_id else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        let Some(hit_before) = before.hit else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_hit_before"
            ));
            continue;
        };
        let Some(hit_after) = after.hit else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} error=missing_hit_after"
            ));
            continue;
        };

        if !is_descendant(hit_before, target_before, &before_sem.parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=hit_not_within_target_before hit={hit_before} target={target_before}"
            ));
            continue;
        }
        if !is_descendant(hit_after, target_after, &after_sem.parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=hit_not_within_target_after hit={hit_after} target={target_after}"
            ));
            continue;
        }

        if let (Some(a), Some(b)) = (before.vlist_offset, after.vlist_offset)
            && (a - b).abs() > 0.1
        {
            continue;
        }

        if hit_before == hit_after {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=hit_did_not_change hit={hit_after}"
            ));
        }
    }

    if failures.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str(
        "wheel scroll hit-change check failed (expected wheel to affect the scrolled content)\n",
    );
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in failures {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_scroll_streaming_passes_when_hit_moves_outside_target() {
        let mut dir = std::env::temp_dir();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        dir.push(format!(
            "fret-diag-wheel-scroll-streaming-test-{}-{}",
            std::process::id(),
            ts
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let bundle_path = crate::resolve_bundle_artifact_path(&dir);
        std::fs::write(
            &bundle_path,
            r#"{
  "schema_version": 1,
  "windows": [{
    "window": 1,
    "events": [{ "kind": "pointer.wheel", "frame_id": 1 }],
    "snapshots": [
      {
        "frame_id": 0,
        "debug": {
          "hit_test": { "hit": 2 },
          "semantics": { "nodes": [
            { "id": 1, "test_id": "root" },
            { "id": 2, "parent": 1 }
          ]}
        }
      },
      {
        "frame_id": 1,
        "debug": {
          "hit_test": { "hit": 3 },
          "semantics": { "nodes": [
            { "id": 1, "test_id": "root" },
            { "id": 2, "parent": 1 },
            { "id": 3, "parent": 99 }
          ]}
        }
      }
    ]
  }]
}"#,
        )
        .expect("write bundle");

        check_bundle_for_wheel_scroll_streaming(&bundle_path, "root", 0)
            .expect("expected wheel scroll check to pass");
    }

    #[test]
    fn wheel_scroll_hit_changes_streaming_passes_when_offset_changes() {
        let mut dir = std::env::temp_dir();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        dir.push(format!(
            "fret-diag-wheel-scroll-hit-changes-streaming-test-{}-{}",
            std::process::id(),
            ts
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let bundle_path = crate::resolve_bundle_artifact_path(&dir);
        std::fs::write(
            &bundle_path,
            r#"{
  "schema_version": 1,
  "windows": [{
    "window": 1,
    "events": [{ "kind": "pointer.wheel", "frame_id": 1 }],
    "snapshots": [
      {
        "frame_id": 0,
        "debug": {
          "hit_test": { "hit": 2 },
          "semantics": { "nodes": [
            { "id": 1, "test_id": "root" },
            { "id": 2, "parent": 1 }
          ]},
          "virtual_list_windows": [{ "offset": 0.0 }]
        }
      },
      {
        "frame_id": 1,
        "debug": {
          "hit_test": { "hit": 2 },
          "semantics": { "nodes": [
            { "id": 1, "test_id": "root" },
            { "id": 2, "parent": 1 }
          ]},
          "virtual_list_windows": [{ "offset": 12.0 }]
        }
      }
    ]
  }]
}"#,
        )
        .expect("write bundle");

        check_bundle_for_wheel_scroll_hit_changes_streaming(&bundle_path, "root", 0)
            .expect("expected wheel scroll hit-change check to pass");
    }
}
