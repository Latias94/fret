use std::collections::HashMap;
use std::path::Path;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};

#[derive(Debug, Clone)]
struct WheelEventsWindowSummary {
    window_id: u64,
    total_wheel_events: u64,
    max_wheel_events_per_frame: u64,
    sample_frames_over_1: Vec<(u64, u64)>,
}

pub(crate) fn check_bundle_for_wheel_events_max_per_frame(
    bundle_path: &Path,
    out_dir: &Path,
    max_per_frame: u64,
) -> Result<(), String> {
    let windows = read_wheel_events_window_summaries(bundle_path)?;

    let mut total_wheel_events: u64 = 0;
    let mut global_max: u64 = 0;
    let mut offenders: Vec<WheelEventsWindowSummary> = Vec::new();
    for w in windows {
        total_wheel_events = total_wheel_events.saturating_add(w.total_wheel_events);
        global_max = global_max.max(w.max_wheel_events_per_frame);
        if w.max_wheel_events_per_frame > max_per_frame {
            offenders.push(w);
        }
    }

    let evidence_path = out_dir.join("check.wheel_events_max_per_frame.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "bundle": bundle_path.display().to_string(),
        "max_per_frame": max_per_frame,
        "observed": {
            "total_wheel_events": total_wheel_events,
            "global_max_wheel_events_per_frame": global_max,
            "windows_over_max": offenders.len(),
        },
        "failures": offenders.iter().map(|w| {
            serde_json::json!({
                "window_id": w.window_id,
                "total_wheel_events": w.total_wheel_events,
                "max_wheel_events_per_frame": w.max_wheel_events_per_frame,
                "sample_frames_over_1": w.sample_frames_over_1.iter().map(|(frame_id, count)| {
                    serde_json::json!({
                        "frame_id": frame_id,
                        "count": count,
                    })
                }).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    });
    let _ = std::fs::create_dir_all(out_dir);
    let _ = std::fs::write(
        &evidence_path,
        serde_json::to_string_pretty(&payload).unwrap_or_default(),
    );

    if total_wheel_events == 0 {
        return Err(format!(
            "wheel coalescing gate requires at least one pointer.wheel event in the bundle: {}",
            bundle_path.display()
        ));
    }
    if global_max <= max_per_frame {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("wheel events per frame gate failed\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "observed global_max_wheel_events_per_frame={global_max} (threshold={max_per_frame})\n"
    ));
    msg.push_str(&format!("evidence: {}\n", evidence_path.display()));
    for w in &payload["failures"].as_array().cloned().unwrap_or_default() {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let observed = w
            .get("max_wheel_events_per_frame")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        msg.push_str(&format!(
            "  window={window_id} max_wheel_events_per_frame={observed}\n"
        ));
    }
    Err(msg)
}

fn read_wheel_events_window_summaries(
    bundle_path: &Path,
) -> Result<Vec<WheelEventsWindowSummary>, String> {
    struct RootSeed {
        out: std::rc::Rc<std::cell::RefCell<Vec<WheelEventsWindowSummary>>>,
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
        out: std::rc::Rc<std::cell::RefCell<Vec<WheelEventsWindowSummary>>>,
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
        out: std::rc::Rc<std::cell::RefCell<Vec<WheelEventsWindowSummary>>>,
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
        out: std::rc::Rc<std::cell::RefCell<Vec<WheelEventsWindowSummary>>>,
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
        out: std::rc::Rc<std::cell::RefCell<Vec<WheelEventsWindowSummary>>>,
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
        out: std::rc::Rc<std::cell::RefCell<Vec<WheelEventsWindowSummary>>>,
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
            let mut summary: Option<WheelEventsWindowSummary> = None;

            while let Some(key) = map.next_key::<String>()? {
                match key.as_str() {
                    "window" | "window_id" | "windowId" => {
                        window_id = map.next_value::<u64>()?;
                    }
                    "events" => {
                        summary = Some(map.next_value_seed(EventsWheelCountsSeed { window_id })?);
                    }
                    _ => {
                        map.next_value::<IgnoredAny>()?;
                    }
                }
            }

            if let Some(mut s) = summary {
                s.window_id = window_id;
                self.out.borrow_mut().push(s);
            }
            Ok(())
        }
    }

    struct EventsWheelCountsSeed {
        window_id: u64,
    }

    impl<'de> DeserializeSeed<'de> for EventsWheelCountsSeed {
        type Value = WheelEventsWindowSummary;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(EventsWheelCountsVisitor {
                window_id: self.window_id,
            })
        }
    }

    struct EventsWheelCountsVisitor {
        window_id: u64,
    }

    impl<'de> Visitor<'de> for EventsWheelCountsVisitor {
        type Value = WheelEventsWindowSummary;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "events array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut counts: HashMap<u64, u64> = HashMap::new();
            let mut total: u64 = 0;
            let mut max_per_frame: u64 = 0;

            while let Some(frame_id) = seq.next_element_seed(WheelEventFrameSeed)? {
                let Some(frame_id) = frame_id else {
                    continue;
                };
                total = total.saturating_add(1);
                let next = counts.entry(frame_id).or_insert(0);
                *next = next.saturating_add(1);
                max_per_frame = max_per_frame.max(*next);
            }

            let mut sample_frames_over_1: Vec<(u64, u64)> = Vec::new();
            for (frame_id, count) in counts {
                if count > 1 {
                    sample_frames_over_1.push((frame_id, count));
                    if sample_frames_over_1.len() >= 10 {
                        break;
                    }
                }
            }

            Ok(WheelEventsWindowSummary {
                window_id: self.window_id,
                total_wheel_events: total,
                max_wheel_events_per_frame: max_per_frame,
                sample_frames_over_1,
            })
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            while map.next_key::<IgnoredAny>()?.is_some() {
                map.next_value::<IgnoredAny>()?;
            }
            Ok(WheelEventsWindowSummary {
                window_id: self.window_id,
                total_wheel_events: 0,
                max_wheel_events_per_frame: 0,
                sample_frames_over_1: Vec::new(),
            })
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(WheelEventsWindowSummary {
                window_id: self.window_id,
                total_wheel_events: 0,
                max_wheel_events_per_frame: 0,
                sample_frames_over_1: Vec::new(),
            })
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(WheelEventsWindowSummary {
                window_id: self.window_id,
                total_wheel_events: 0,
                max_wheel_events_per_frame: 0,
                sample_frames_over_1: Vec::new(),
            })
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
    }

    let out = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let file = std::fs::File::open(bundle_path).map_err(|e| {
        format!(
            "failed to read bundle artifact: {} ({e})",
            bundle_path.display()
        )
    })?;
    let reader = std::io::BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);
    RootSeed { out: out.clone() }
        .deserialize(&mut de)
        .map_err(|e| {
            format!(
                "failed to parse bundle artifact: {} ({e})",
                bundle_path.display()
            )
        })?;
    Ok(out.take())
}
