use std::path::Path;

use crate::util::{now_unix_ms, write_json_value};

pub(crate) fn check_bundle_for_view_cache_reuse_min(
    bundle_path: &Path,
    min_reuse_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_view_cache_reuse_min_json(
        &bundle,
        bundle_path,
        min_reuse_events,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_view_cache_reuse_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_reuse_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reuse_events: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut any_view_cache_active = false;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let view_cache_active = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if !view_cache_active {
                continue;
            }

            let roots = s
                .get("debug")
                .and_then(|v| v.get("cache_roots"))
                .and_then(|v| v.as_array());
            let Some(roots) = roots else {
                continue;
            };

            for r in roots {
                if r.get("reused").and_then(|v| v.as_bool()) == Some(true) {
                    reuse_events = reuse_events.saturating_add(1);
                    if reuse_events >= min_reuse_events {
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(format!(
        "expected at least {min_reuse_events} view-cache reuse events, got {reuse_events} \
 (any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) \
 in bundle: {}",
        bundle_path.display()
    ))
}

#[derive(Debug, Clone)]
struct ViewCacheReuseStableWindowReport {
    window: u64,
    examined_snapshots: u64,
    view_cache_active_snapshots: u64,
    non_reuse_cache_inactive_snapshots: u64,
    non_reuse_active_no_signal_snapshots: u64,
    reuse_snapshots: u64,
    reuse_streak_max: u64,
    reuse_streak_tail: u64,
    last_non_reuse: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy)]
struct ViewCacheReuseSignal {
    view_cache_active: bool,
    has_reuse_signal: bool,
    reused_roots: u64,
    paint_cache_replayed_ops: u64,
    cache_roots_present: bool,
}

impl ViewCacheReuseSignal {
    fn no_signal_reason(self) -> &'static str {
        if !self.view_cache_active {
            return "view_cache_inactive";
        }
        "active_no_signal"
    }
}

fn snapshot_view_cache_reuse_signal(snapshot: &serde_json::Value) -> ViewCacheReuseSignal {
    let stats = snapshot.get("debug").and_then(|v| v.get("stats"));
    let view_cache_active = stats
        .and_then(|v| v.get("view_cache_active"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let replayed_ops = stats
        .and_then(|v| v.get("paint_cache_replayed_ops"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let mut reused_roots: u64 = 0;
    let mut cache_roots_present = false;
    if let Some(roots) = snapshot
        .get("debug")
        .and_then(|v| v.get("cache_roots"))
        .and_then(|v| v.as_array())
    {
        cache_roots_present = true;
        for r in roots {
            if r.get("reused").and_then(|v| v.as_bool()) == Some(true) {
                reused_roots = reused_roots.saturating_add(1);
            }
        }
    }

    let has_signal = view_cache_active && (reused_roots > 0 || replayed_ops > 0);
    ViewCacheReuseSignal {
        view_cache_active,
        has_reuse_signal: has_signal,
        reused_roots,
        paint_cache_replayed_ops: replayed_ops,
        cache_roots_present,
    }
}

pub(crate) fn check_bundle_for_view_cache_reuse_stable_min(
    bundle_path: &Path,
    out_dir: &Path,
    min_tail_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reports: Vec<ViewCacheReuseStableWindowReport> = Vec::new();
    let mut failures: Vec<serde_json::Value> = Vec::new();

    let mut any_view_cache_active = false;
    let mut best_tail: u64 = 0;

    for w in windows {
        let window = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut examined_snapshots: u64 = 0;
        let mut view_cache_active_snapshots: u64 = 0;
        let mut non_reuse_cache_inactive_snapshots: u64 = 0;
        let mut non_reuse_active_no_signal_snapshots: u64 = 0;
        let mut reuse_snapshots: u64 = 0;
        let mut reuse_streak: u64 = 0;
        let mut reuse_streak_max: u64 = 0;
        let mut last_non_reuse: Option<serde_json::Value> = None;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let stats = s.get("debug").and_then(|v| v.get("stats"));
            let view_cache_active = stats
                .and_then(|v| v.get("view_cache_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            any_view_cache_active |= view_cache_active;
            if view_cache_active {
                view_cache_active_snapshots = view_cache_active_snapshots.saturating_add(1);
            }

            let signal = snapshot_view_cache_reuse_signal(s);
            if signal.has_reuse_signal {
                reuse_snapshots = reuse_snapshots.saturating_add(1);
                reuse_streak = reuse_streak.saturating_add(1);
                reuse_streak_max = reuse_streak_max.max(reuse_streak);
            } else {
                reuse_streak = 0;
                match signal.no_signal_reason() {
                    "view_cache_inactive" => {
                        non_reuse_cache_inactive_snapshots =
                            non_reuse_cache_inactive_snapshots.saturating_add(1);
                    }
                    _ => {
                        non_reuse_active_no_signal_snapshots =
                            non_reuse_active_no_signal_snapshots.saturating_add(1);
                    }
                }
                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                last_non_reuse = Some(serde_json::json!({
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "reason": signal.no_signal_reason(),
                    "view_cache_active": signal.view_cache_active,
                    "cache_roots_present": signal.cache_roots_present,
                    "reused_roots": signal.reused_roots,
                    "paint_cache_replayed_ops": signal.paint_cache_replayed_ops,
                }));
            }
        }

        best_tail = best_tail.max(reuse_streak);

        reports.push(ViewCacheReuseStableWindowReport {
            window,
            examined_snapshots,
            view_cache_active_snapshots,
            non_reuse_cache_inactive_snapshots,
            non_reuse_active_no_signal_snapshots,
            reuse_snapshots,
            reuse_streak_max,
            reuse_streak_tail: reuse_streak,
            last_non_reuse: last_non_reuse.clone(),
        });

        if min_tail_frames > 0 && examined_snapshots < min_tail_frames {
            failures.push(serde_json::json!({
                "window": window,
                "reason": "insufficient_snapshots",
                "examined_snapshots": examined_snapshots,
            }));
        } else if min_tail_frames > 0 && reuse_streak < min_tail_frames {
            failures.push(serde_json::json!({
                "window": window,
                "reason": "reuse_tail_streak_too_small",
                "examined_snapshots": examined_snapshots,
                "view_cache_active_snapshots": view_cache_active_snapshots,
                "non_reuse_cache_inactive_snapshots": non_reuse_cache_inactive_snapshots,
                "non_reuse_active_no_signal_snapshots": non_reuse_active_no_signal_snapshots,
                "reuse_streak_tail": reuse_streak,
                "reuse_streak_max": reuse_streak_max,
                "reuse_snapshots": reuse_snapshots,
                "last_non_reuse": last_non_reuse,
            }));
        }
    }

    let out_path = out_dir.join("check.view_cache_reuse_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "view_cache_reuse_stable",
        "bundle_artifact": bundle_path.display().to_string(),
        "bundle_json": bundle_path.display().to_string(),
        "out_dir": out_dir.display().to_string(),
        "warmup_frames": warmup_frames,
        "min_tail_frames": min_tail_frames,
        "any_view_cache_active": any_view_cache_active,
        "best_reuse_streak_tail": best_tail,
        "windows": reports.iter().map(|r| serde_json::json!({
            "window": r.window,
            "examined_snapshots": r.examined_snapshots,
            "view_cache_active_snapshots": r.view_cache_active_snapshots,
            "non_reuse_cache_inactive_snapshots": r.non_reuse_cache_inactive_snapshots,
            "non_reuse_active_no_signal_snapshots": r.non_reuse_active_no_signal_snapshots,
            "reuse_snapshots": r.reuse_snapshots,
            "reuse_streak_max": r.reuse_streak_max,
            "reuse_streak_tail": r.reuse_streak_tail,
            "last_non_reuse": r.last_non_reuse,
        })).collect::<Vec<_>>(),
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    if min_tail_frames == 0 {
        return Ok(());
    }
    if !any_view_cache_active {
        return Err(format!(
            "view-cache reuse stable gate requires view_cache_active snapshots, but none were observed (warmup_frames={warmup_frames})\n  hint: enable view-cache for the target demo if applicable (e.g. UI gallery: FRET_UI_GALLERY_VIEW_CACHE=1)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            out_path.display()
        ));
    }
    if best_tail >= min_tail_frames {
        return Ok(());
    }

    Err(format!(
        "view-cache reuse stable gate failed (min_tail_frames={min_tail_frames}, best_tail={best_tail}, warmup_frames={warmup_frames})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        out_path.display()
    ))
}
