use std::collections::HashMap;
use std::path::Path;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

pub(super) fn read_wheel_frames_min_by_window(
    bundle_path: &Path,
) -> Result<HashMap<u64, u64>, String> {
    struct RootSeed {
        out: std::rc::Rc<std::cell::RefCell<HashMap<u64, u64>>>,
    }

    impl<'de> DeserializeSeed<'de> for RootSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(RootVisitor { out: self.out })
        }
    }

    struct RootVisitor {
        out: std::rc::Rc<std::cell::RefCell<HashMap<u64, u64>>>,
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
        out: std::rc::Rc<std::cell::RefCell<HashMap<u64, u64>>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowsSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(WindowsVisitor { out: self.out })
        }
    }

    struct WindowsVisitor {
        out: std::rc::Rc<std::cell::RefCell<HashMap<u64, u64>>>,
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
                    out: self.out.clone(),
                })?
                .is_some()
            {}
            Ok(())
        }
    }

    struct WindowSeed {
        out: std::rc::Rc<std::cell::RefCell<HashMap<u64, u64>>>,
    }

    impl<'de> DeserializeSeed<'de> for WindowSeed {
        type Value = ();

        fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(WindowVisitor { out: self.out })
        }
    }

    struct WindowVisitor {
        out: std::rc::Rc<std::cell::RefCell<HashMap<u64, u64>>>,
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
            let mut wheel_min: Option<u64> = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "window" | "window_id" | "windowId" => {
                        window_id = map.next_value::<u64>()?;
                    }
                    "events" => {
                        wheel_min = map.next_value_seed(EventsWheelMinSeed)?;
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            if let Some(fid) = wheel_min {
                self.out
                    .borrow_mut()
                    .entry(window_id)
                    .and_modify(|v| *v = (*v).min(fid))
                    .or_insert(fid);
            }
            Ok(())
        }
    }

    struct EventsWheelMinSeed;

    impl<'de> DeserializeSeed<'de> for EventsWheelMinSeed {
        type Value = Option<u64>;

        fn deserialize<D>(self, deserializer: D) -> Result<Option<u64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(EventsWheelMinVisitor { min: None })
        }
    }

    struct EventsWheelMinVisitor {
        min: Option<u64>,
    }

    impl<'de> Visitor<'de> for EventsWheelMinVisitor {
        type Value = Option<u64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "events array")
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<Option<u64>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            while let Some(fid) = seq.next_element_seed(WheelEventFrameSeed)? {
                let Some(fid) = fid else {
                    continue;
                };
                self.min = Some(self.min.map_or(fid, |m| m.min(fid)));
            }
            Ok(self.min)
        }

        fn visit_map<M>(self, mut map: M) -> Result<Option<u64>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while map.next_key::<IgnoredAny>()?.is_some() {
                map.next_value::<IgnoredAny>()?;
            }
            Ok(None)
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

    struct WheelEventFrameSeed;

    impl<'de> DeserializeSeed<'de> for WheelEventFrameSeed {
        type Value = Option<u64>;

        fn deserialize<D>(self, deserializer: D) -> Result<Option<u64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(WheelEventFrameVisitor {
                kind: None,
                frame_id: None,
            })
        }
    }

    struct WheelEventFrameVisitor {
        kind: Option<String>,
        frame_id: Option<u64>,
    }

    impl<'de> Visitor<'de> for WheelEventFrameVisitor {
        type Value = Option<u64>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "event object")
        }

        fn visit_map<M>(mut self, mut map: M) -> Result<Option<u64>, M::Error>
        where
            M: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "kind" => self.kind = map.next_value::<Option<String>>()?,
                    "frame_id" | "frameId" => self.frame_id = map.next_value::<Option<u64>>()?,
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }
            if self.kind.as_deref() == Some("pointer.wheel") {
                Ok(self.frame_id)
            } else {
                Ok(None)
            }
        }

        fn visit_unit<E>(self) -> Result<Option<u64>, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    let out = std::rc::Rc::new(std::cell::RefCell::new(HashMap::new()));
    crate::json_stream::with_bundle_json_deserializer(bundle_path, |de| {
        RootSeed { out: out.clone() }.deserialize(de)
    })?;
    Ok(out.borrow().clone())
}
