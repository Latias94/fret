use std::collections::HashMap;
use std::path::Path;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

use super::types::{SnapshotMeta, WindowWheelMeta};

pub(super) fn read_window_before_after_metas(
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
            deserializer.deserialize_any(DebugHitAndVlistVisitor {
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

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok((None, None))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok((None, None))
        }
    }

    struct HitTestHitSeed;

    impl<'de> DeserializeSeed<'de> for HitTestHitSeed {
        type Value = Option<u64>;

        fn deserialize<D>(self, deserializer: D) -> Result<Option<u64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(HitTestHitVisitor { hit: None })
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

        fn visit_unit<E>(self) -> Result<Option<u64>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_none<E>(self) -> Result<Option<u64>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
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
