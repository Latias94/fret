use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::LaunchedDemo;
use super::stats::BundleStatsSort;
use super::util::{now_unix_ms, touch};

#[derive(Debug, Clone, Copy)]
pub(super) struct CompareOptions {
    pub(super) warmup_frames: u64,
    pub(super) eps_px: f32,
    pub(super) ignore_bounds: bool,
    pub(super) ignore_scene_fingerprint: bool,
}

#[derive(Debug, Clone)]
pub(super) struct CompareReport {
    pub(super) ok: bool,
    a_path: PathBuf,
    b_path: PathBuf,
    a_frame_id: Option<u64>,
    b_frame_id: Option<u64>,
    a_scene_fingerprint: Option<u64>,
    b_scene_fingerprint: Option<u64>,
    opts: CompareOptions,
    pub(super) diffs: Vec<CompareDiff>,
}

#[derive(Debug, Clone)]
pub(super) struct CompareDiff {
    pub(super) kind: &'static str,
    pub(super) key: Option<String>,
    pub(super) field: Option<&'static str>,
    pub(super) a: Option<serde_json::Value>,
    pub(super) b: Option<serde_json::Value>,
}

impl CompareReport {
    pub(super) fn print_human(&self) {
        println!("bundle_a: {}", self.a_path.display());
        println!("bundle_b: {}", self.b_path.display());
        if let (Some(a), Some(b)) = (self.a_frame_id, self.b_frame_id) {
            println!("frame_id: a={a} b={b}");
        }
        if let (Some(a), Some(b)) = (self.a_scene_fingerprint, self.b_scene_fingerprint) {
            println!("scene_fingerprint: a=0x{a:016x} b=0x{b:016x}");
        }
        if self.ok {
            println!(
                "compare: ok (diffs=0, warmup_frames={}, eps_px={}, ignore_bounds={}, ignore_scene_fingerprint={})",
                self.opts.warmup_frames,
                self.opts.eps_px,
                self.opts.ignore_bounds,
                self.opts.ignore_scene_fingerprint
            );
            return;
        }
        println!(
            "compare: failed (diffs={}, warmup_frames={}, eps_px={}, ignore_bounds={}, ignore_scene_fingerprint={})",
            self.diffs.len(),
            self.opts.warmup_frames,
            self.opts.eps_px,
            self.opts.ignore_bounds,
            self.opts.ignore_scene_fingerprint
        );
        for d in self.diffs.iter().take(20) {
            let key = d.key.as_deref().unwrap_or("<none>");
            let field = d.field.unwrap_or("<none>");
            println!("  - {} key={} field={}", d.kind, key, field);
        }
        if self.diffs.len() > 20 {
            println!("  ... ({} more)", self.diffs.len() - 20);
        }
    }

    pub(super) fn to_human_error(&self) -> String {
        let mut msg = String::new();
        msg.push_str("bundle compare failed\n");
        msg.push_str(&format!("bundle_a: {}\n", self.a_path.display()));
        msg.push_str(&format!("bundle_b: {}\n", self.b_path.display()));
        if let (Some(a), Some(b)) = (self.a_frame_id, self.b_frame_id) {
            msg.push_str(&format!("frame_id: a={a} b={b}\n"));
        }
        if let (Some(a), Some(b)) = (self.a_scene_fingerprint, self.b_scene_fingerprint) {
            msg.push_str(&format!("scene_fingerprint: a=0x{a:016x} b=0x{b:016x}\n"));
        }
        msg.push_str(&format!(
            "diffs: {} (warmup_frames={}, eps_px={}, ignore_bounds={}, ignore_scene_fingerprint={})\n",
            self.diffs.len(),
            self.opts.warmup_frames,
            self.opts.eps_px,
            self.opts.ignore_bounds,
            self.opts.ignore_scene_fingerprint
        ));
        for d in self.diffs.iter().take(20) {
            let key = d.key.as_deref().unwrap_or("<none>");
            let field = d.field.unwrap_or("<none>");
            msg.push_str(&format!("  - {} key={} field={}\n", d.kind, key, field));
        }
        if self.diffs.len() > 20 {
            msg.push_str(&format!("  ... ({} more)\n", self.diffs.len() - 20));
        }
        msg
    }

    pub(super) fn to_json(&self) -> serde_json::Value {
        let diffs = self
            .diffs
            .iter()
            .map(|d| {
                serde_json::json!({
                    "kind": d.kind,
                    "key": d.key,
                    "field": d.field,
                    "a": d.a,
                    "b": d.b,
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({
            "schema_version": 1,
            "ok": self.ok,
            "bundle_a": self.a_path.display().to_string(),
            "bundle_b": self.b_path.display().to_string(),
            "a_frame_id": self.a_frame_id,
            "b_frame_id": self.b_frame_id,
            "a_scene_fingerprint": self.a_scene_fingerprint,
            "b_scene_fingerprint": self.b_scene_fingerprint,
            "options": {
                "warmup_frames": self.opts.warmup_frames,
                "eps_px": self.opts.eps_px,
                "ignore_bounds": self.opts.ignore_bounds,
                "ignore_scene_fingerprint": self.opts.ignore_scene_fingerprint,
            },
            "diffs": diffs,
        })
    }
}

pub(super) fn compare_bundles(
    a_bundle_path: &Path,
    b_bundle_path: &Path,
    opts: CompareOptions,
) -> Result<CompareReport, String> {
    let a_bytes = std::fs::read(a_bundle_path).map_err(|e| e.to_string())?;
    let b_bytes = std::fs::read(b_bundle_path).map_err(|e| e.to_string())?;
    let a_bundle: serde_json::Value =
        serde_json::from_slice(&a_bytes).map_err(|e| e.to_string())?;
    let b_bundle: serde_json::Value =
        serde_json::from_slice(&b_bytes).map_err(|e| e.to_string())?;
    compare_bundles_json(&a_bundle, a_bundle_path, &b_bundle, b_bundle_path, opts)
}

pub(super) fn compare_bundles_json(
    a_bundle: &serde_json::Value,
    a_bundle_path: &Path,
    b_bundle: &serde_json::Value,
    b_bundle_path: &Path,
    opts: CompareOptions,
) -> Result<CompareReport, String> {
    let a_window = first_window_from_bundle(a_bundle)?;
    let b_window = first_window_from_bundle(b_bundle)?;

    let (a_snapshot, a_selected) = select_snapshot_for_compare(a_window, opts.warmup_frames);
    let (b_snapshot, b_selected) = select_snapshot_for_compare(b_window, opts.warmup_frames);

    let mut diffs: Vec<CompareDiff> = Vec::new();

    let a_fp = a_snapshot
        .and_then(|s| s.get("scene_fingerprint"))
        .and_then(|v| v.as_u64());
    let b_fp = b_snapshot
        .and_then(|s| s.get("scene_fingerprint"))
        .and_then(|v| v.as_u64());
    if !opts.ignore_scene_fingerprint {
        if let (Some(a_fp), Some(b_fp)) = (a_fp, b_fp) {
            if a_fp != b_fp {
                diffs.push(CompareDiff {
                    kind: "scene_fingerprint_mismatch",
                    key: None,
                    field: Some("scene_fingerprint"),
                    a: Some(serde_json::Value::from(a_fp)),
                    b: Some(serde_json::Value::from(b_fp)),
                });
            }
        }
    }

    if let (Some(a_snapshot), Some(b_snapshot)) = (a_snapshot, b_snapshot) {
        compare_semantics_by_test_id(&mut diffs, a_snapshot, b_snapshot, opts)?;
    }

    Ok(CompareReport {
        ok: diffs.is_empty(),
        a_path: a_bundle_path.to_path_buf(),
        b_path: b_bundle_path.to_path_buf(),
        a_frame_id: a_selected.frame_id,
        b_frame_id: b_selected.frame_id,
        a_scene_fingerprint: a_fp,
        b_scene_fingerprint: b_fp,
        opts,
        diffs,
    })
}

fn first_window_from_bundle(bundle: &serde_json::Value) -> Result<&serde_json::Value, String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    windows
        .first()
        .ok_or_else(|| "bundle.json contains no windows".to_string())
}

#[derive(Debug, Clone, Copy, Default)]
struct SelectedSnapshotInfo {
    frame_id: Option<u64>,
}

fn select_snapshot_for_compare<'a>(
    window: &'a serde_json::Value,
    warmup_frames: u64,
) -> (Option<&'a serde_json::Value>, SelectedSnapshotInfo) {
    let snaps = window
        .get("snapshots")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    if snaps.is_empty() {
        return (None, SelectedSnapshotInfo::default());
    }

    let mut selected: Option<&serde_json::Value> = None;
    for s in snaps {
        let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
        if frame_id >= warmup_frames {
            selected = Some(s);
        }
    }
    let selected = selected.or_else(|| snaps.last());
    let info = SelectedSnapshotInfo {
        frame_id: selected.and_then(|s| s.get("frame_id").and_then(|v| v.as_u64())),
    };
    (selected, info)
}

#[derive(Debug, Clone)]
struct SemanticsNodeSummary {
    role: String,
    flags: serde_json::Value,
    actions: serde_json::Value,
    bounds: Option<(f64, f64, f64, f64)>,
}

fn compare_semantics_by_test_id(
    diffs: &mut Vec<CompareDiff>,
    a_snapshot: &serde_json::Value,
    b_snapshot: &serde_json::Value,
    opts: CompareOptions,
) -> Result<(), String> {
    let a_sem = a_snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .ok_or_else(|| {
            "bundle snapshot missing debug.semantics (ensure FRET_DIAG_SEMANTICS=1)".to_string()
        })?;
    let b_sem = b_snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .ok_or_else(|| {
            "bundle snapshot missing debug.semantics (ensure FRET_DIAG_SEMANTICS=1)".to_string()
        })?;

    let (a_by_test_id, a_id_to_test_id) = semantics_nodes_by_test_id(a_sem);
    let (b_by_test_id, b_id_to_test_id) = semantics_nodes_by_test_id(b_sem);

    for test_id in a_by_test_id.keys() {
        if !b_by_test_id.contains_key(test_id) {
            diffs.push(CompareDiff {
                kind: "missing_test_id",
                key: Some(test_id.clone()),
                field: None,
                a: Some(serde_json::Value::from("present")),
                b: Some(serde_json::Value::Null),
            });
        }
    }
    for test_id in b_by_test_id.keys() {
        if !a_by_test_id.contains_key(test_id) {
            diffs.push(CompareDiff {
                kind: "extra_test_id",
                key: Some(test_id.clone()),
                field: None,
                a: Some(serde_json::Value::Null),
                b: Some(serde_json::Value::from("present")),
            });
        }
    }

    for (test_id, a_node) in a_by_test_id.iter() {
        let Some(b_node) = b_by_test_id.get(test_id) else {
            continue;
        };
        if a_node.role != b_node.role {
            diffs.push(CompareDiff {
                kind: "node_field_mismatch",
                key: Some(test_id.clone()),
                field: Some("role"),
                a: Some(serde_json::Value::from(a_node.role.clone())),
                b: Some(serde_json::Value::from(b_node.role.clone())),
            });
        }
        if a_node.flags != b_node.flags {
            diffs.push(CompareDiff {
                kind: "node_field_mismatch",
                key: Some(test_id.clone()),
                field: Some("flags"),
                a: Some(a_node.flags.clone()),
                b: Some(b_node.flags.clone()),
            });
        }
        if a_node.actions != b_node.actions {
            diffs.push(CompareDiff {
                kind: "node_field_mismatch",
                key: Some(test_id.clone()),
                field: Some("actions"),
                a: Some(a_node.actions.clone()),
                b: Some(b_node.actions.clone()),
            });
        }
        if !opts.ignore_bounds {
            if let (Some(a), Some(b)) = (a_node.bounds, b_node.bounds) {
                if !rect_eq_eps(a, b, opts.eps_px) {
                    diffs.push(CompareDiff {
                        kind: "node_field_mismatch",
                        key: Some(test_id.clone()),
                        field: Some("bounds"),
                        a: Some(serde_json::json!({ "x": a.0, "y": a.1, "w": a.2, "h": a.3 })),
                        b: Some(serde_json::json!({ "x": b.0, "y": b.1, "w": b.2, "h": b.3 })),
                    });
                }
            }
        }
    }

    compare_semantics_root_distribution(diffs, a_sem, b_sem);
    compare_focus_and_capture_by_test_id(diffs, a_sem, b_sem, &a_id_to_test_id, &b_id_to_test_id);

    Ok(())
}

fn semantics_nodes_by_test_id(
    semantics: &serde_json::Value,
) -> (
    std::collections::BTreeMap<String, SemanticsNodeSummary>,
    std::collections::HashMap<u64, String>,
) {
    let mut by_test_id: std::collections::BTreeMap<String, SemanticsNodeSummary> =
        std::collections::BTreeMap::new();
    let mut id_to_test_id: std::collections::HashMap<u64, String> =
        std::collections::HashMap::new();

    let nodes = semantics
        .get("nodes")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for node in nodes {
        let Some(test_id) = node.get("test_id").and_then(|v| v.as_str()) else {
            continue;
        };
        let test_id = test_id.to_string();
        let role = node
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let flags = node
            .get("flags")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let actions = node
            .get("actions")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let bounds = node.get("bounds").and_then(rect_xywh_from_json);
        by_test_id.insert(
            test_id.clone(),
            SemanticsNodeSummary {
                role,
                flags,
                actions,
                bounds,
            },
        );
        if let Some(id) = node.get("id").and_then(|v| v.as_u64()) {
            id_to_test_id.insert(id, test_id);
        }
    }

    (by_test_id, id_to_test_id)
}

fn rect_xywh_from_json(bounds: &serde_json::Value) -> Option<(f64, f64, f64, f64)> {
    let x = bounds.get("x").and_then(|v| v.as_f64())?;
    let y = bounds.get("y").and_then(|v| v.as_f64())?;
    let w = bounds.get("w").and_then(|v| v.as_f64())?;
    let h = bounds.get("h").and_then(|v| v.as_f64())?;
    Some((x, y, w, h))
}

fn rect_eq_eps(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64), eps_px: f32) -> bool {
    let eps = eps_px as f64;
    (a.0 - b.0).abs() <= eps
        && (a.1 - b.1).abs() <= eps
        && (a.2 - b.2).abs() <= eps
        && (a.3 - b.3).abs() <= eps
}

fn compare_semantics_root_distribution(
    diffs: &mut Vec<CompareDiff>,
    a_sem: &serde_json::Value,
    b_sem: &serde_json::Value,
) {
    let a = semantics_root_distribution(a_sem);
    let b = semantics_root_distribution(b_sem);
    if a != b {
        diffs.push(CompareDiff {
            kind: "semantics_roots_mismatch",
            key: None,
            field: Some("roots"),
            a: Some(semantics_root_distribution_to_json(&a)),
            b: Some(semantics_root_distribution_to_json(&b)),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SemanticsRootSummary {
    visible: bool,
    blocks_underlay_input: bool,
    hit_testable: bool,
    z_index: u32,
}

fn semantics_root_distribution(sem: &serde_json::Value) -> Vec<SemanticsRootSummary> {
    let roots = sem
        .get("roots")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    let mut out: Vec<SemanticsRootSummary> = roots
        .iter()
        .map(|r| SemanticsRootSummary {
            visible: r.get("visible").and_then(|v| v.as_bool()).unwrap_or(false),
            blocks_underlay_input: r
                .get("blocks_underlay_input")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            hit_testable: r
                .get("hit_testable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            z_index: r.get("z_index").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        })
        .collect();
    out.sort();
    out
}

fn semantics_root_distribution_to_json(dist: &[SemanticsRootSummary]) -> serde_json::Value {
    serde_json::Value::Array(
        dist.iter()
            .map(|r| {
                serde_json::json!({
                    "visible": r.visible,
                    "blocks_underlay_input": r.blocks_underlay_input,
                    "hit_testable": r.hit_testable,
                    "z_index": r.z_index,
                })
            })
            .collect(),
    )
}

fn compare_focus_and_capture_by_test_id(
    diffs: &mut Vec<CompareDiff>,
    a_sem: &serde_json::Value,
    b_sem: &serde_json::Value,
    a_id_to_test_id: &std::collections::HashMap<u64, String>,
    b_id_to_test_id: &std::collections::HashMap<u64, String>,
) {
    let a_focus = a_sem.get("focus").and_then(|v| v.as_u64());
    let b_focus = b_sem.get("focus").and_then(|v| v.as_u64());
    let a_focus_tid = a_focus.and_then(|id| a_id_to_test_id.get(&id).cloned());
    let b_focus_tid = b_focus.and_then(|id| b_id_to_test_id.get(&id).cloned());
    if a_focus_tid.is_some() || b_focus_tid.is_some() {
        if a_focus_tid != b_focus_tid {
            diffs.push(CompareDiff {
                kind: "focus_mismatch",
                key: None,
                field: Some("focus.test_id"),
                a: a_focus_tid.map(serde_json::Value::from),
                b: b_focus_tid.map(serde_json::Value::from),
            });
        }
    }

    let a_captured = a_sem.get("captured").and_then(|v| v.as_u64());
    let b_captured = b_sem.get("captured").and_then(|v| v.as_u64());
    let a_captured_tid = a_captured.and_then(|id| a_id_to_test_id.get(&id).cloned());
    let b_captured_tid = b_captured.and_then(|id| b_id_to_test_id.get(&id).cloned());
    if a_captured_tid.is_some() || b_captured_tid.is_some() {
        if a_captured_tid != b_captured_tid {
            diffs.push(CompareDiff {
                kind: "captured_mismatch",
                key: None,
                field: Some("captured.test_id"),
                a: a_captured_tid.map(serde_json::Value::from),
                b: b_captured_tid.map(serde_json::Value::from),
            });
        }
    }
}

pub(super) fn read_latest_pointer(out_dir: &Path) -> Option<PathBuf> {
    let s = std::fs::read_to_string(out_dir.join("latest.txt")).ok()?;
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let path = PathBuf::from(s);
    Some(if path.is_absolute() {
        path
    } else {
        out_dir.join(path)
    })
}

pub(super) fn find_latest_export_dir(out_dir: &Path) -> Option<PathBuf> {
    let mut best: Option<(u64, PathBuf)> = None;
    let entries = std::fs::read_dir(out_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(ts) = name.parse::<u64>() else {
            continue;
        };
        match &best {
            Some((prev, _)) if *prev >= ts => {}
            _ => best = Some((ts, path)),
        }
    }
    best.map(|(_, p)| p)
}

pub(super) fn maybe_launch_demo(
    launch: &Option<Vec<String>>,
    launch_env: &[(String, String)],
    workspace_root: &Path,
    out_dir: &Path,
    ready_path: &Path,
    exit_path: &Path,
    wants_screenshots: bool,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<Option<LaunchedDemo>, String> {
    let Some(launch) = launch else {
        return Ok(None);
    };

    let prev_ready_mtime = std::fs::metadata(ready_path)
        .and_then(|m| m.modified())
        .ok();

    let exe = launch
        .first()
        .ok_or_else(|| "missing launch command".to_string())?;

    let mut cmd = Command::new(exe);
    cmd.args(launch.iter().skip(1));
    cmd.current_dir(workspace_root);
    cmd.env("FRET_DIAG", "1");
    cmd.env("FRET_DIAG_DIR", out_dir);
    cmd.env("FRET_DIAG_READY_PATH", ready_path);
    cmd.env("FRET_DIAG_EXIT_PATH", exit_path);
    if wants_screenshots {
        cmd.env("FRET_DIAG_SCREENSHOTS", "1");
    }
    for (key, value) in launch_env {
        match key.as_str() {
            "FRET_DIAG" | "FRET_DIAG_DIR" | "FRET_DIAG_READY_PATH" | "FRET_DIAG_EXIT_PATH" => {
                return Err(format!("--env cannot override reserved var: {key}"));
            }
            _ => cmd.env(key, value),
        };
    }

    // When collecting redraw hitch logs under `diag repro`, default to writing into `FRET_DIAG_DIR`
    // (and thus into `--dir`) so the log can be packed and checked deterministically.
    //
    // The desktop runner resolves relative `FRET_REDRAW_HITCH_LOG_PATH` against `FRET_DIAG_DIR`.
    let is_truthy = |v: &str| !v.trim().is_empty() && v.trim() != "0";
    let redraw_hitch_log_enabled = if let Some((_, v)) = launch_env
        .iter()
        .find(|(k, _)| k == "FRET_REDRAW_HITCH_LOG")
    {
        is_truthy(v)
    } else if let Ok(v) = std::env::var("FRET_REDRAW_HITCH_LOG") {
        is_truthy(&v)
    } else {
        false
    };
    let redraw_hitch_log_path_set = launch_env
        .iter()
        .any(|(k, v)| k == "FRET_REDRAW_HITCH_LOG_PATH" && !v.trim().is_empty())
        || std::env::var_os("FRET_REDRAW_HITCH_LOG_PATH").is_some_and(|v| !v.is_empty());
    if redraw_hitch_log_enabled && !redraw_hitch_log_path_set {
        cmd.env("FRET_REDRAW_HITCH_LOG_PATH", "redraw_hitches.log");
    }

    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR").filter(|v| !v.is_empty()) {
        cmd.env("CARGO_TARGET_DIR", target_dir);
    }

    let launched_unix_ms = now_unix_ms();
    let launched_instant = Instant::now();
    let child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn `{}`: {e}", launch.join(" ")))?;
    let demo = LaunchedDemo {
        child,
        launched_unix_ms,
        launched_instant,
        launch_cmd: launch.clone(),
    };

    // Avoid racing cold-start compilation by waiting for the app to signal readiness.
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(180_000));
    while Instant::now() < deadline {
        let ready_mtime = std::fs::metadata(ready_path)
            .and_then(|m| m.modified())
            .ok();
        let ready = match (prev_ready_mtime, ready_mtime) {
            (Some(prev), Some(now)) => now > prev,
            (None, Some(_)) => true,
            _ => false,
        };
        if ready {
            return Ok(Some(demo));
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }

    Ok(Some(demo))
}

#[cfg_attr(windows, allow(dead_code))]
#[derive(Debug, Clone)]
struct ObservedProcessFootprint {
    collector: String,
    sample_interval_ms: u64,
    samples: u64,
    cpu_usage_percent_avg: f64,
    cpu_usage_percent_max: f64,
    cpu_usage_percent_last: f64,
    working_set_bytes_last: u64,
    working_set_bytes_peak: u64,
    virtual_memory_bytes_last: u64,
    virtual_memory_bytes_peak: u64,
}

#[cfg(not(windows))]
struct SysinfoProcessFootprintSampler {
    sys: sysinfo::System,
    pid: sysinfo::Pid,
    refresh_kind: sysinfo::ProcessRefreshKind,
    refresh_count: u64,
    last_refresh: std::time::Instant,
    samples: u64,
    cpu_usage_percent_sum: f64,
    cpu_usage_percent_max: f64,
    cpu_usage_percent_last: f64,
    working_set_bytes_last: u64,
    working_set_bytes_peak: u64,
    virtual_memory_bytes_last: u64,
    virtual_memory_bytes_peak: u64,
}

#[cfg(not(windows))]
impl SysinfoProcessFootprintSampler {
    fn new(pid_u32: u32) -> Self {
        use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

        let mut sys = System::new();
        let pid = sysinfo::Pid::from_u32(pid_u32);
        let refresh_kind = ProcessRefreshKind::nothing().with_cpu().with_memory();

        let _ =
            sys.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), false, refresh_kind);

        let mut out = Self {
            sys,
            pid,
            refresh_kind,
            refresh_count: 1,
            last_refresh: std::time::Instant::now(),
            samples: 0,
            cpu_usage_percent_sum: 0.0,
            cpu_usage_percent_max: 0.0,
            cpu_usage_percent_last: 0.0,
            working_set_bytes_last: 0,
            working_set_bytes_peak: 0,
            virtual_memory_bytes_last: 0,
            virtual_memory_bytes_peak: 0,
        };

        out.capture_latest();
        out
    }

    fn refresh_force(&mut self) {
        use sysinfo::ProcessesToUpdate;

        let _ = self.sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[self.pid]),
            false,
            self.refresh_kind,
        );
        self.refresh_count = self.refresh_count.saturating_add(1);
        self.last_refresh = std::time::Instant::now();
        self.capture_latest();
    }

    fn refresh_if_due(&mut self) {
        let since = self.last_refresh.elapsed();
        if since < sysinfo::MINIMUM_CPU_UPDATE_INTERVAL {
            return;
        }
        self.refresh_force();
    }

    fn capture_latest(&mut self) {
        let Some(p) = self.sys.process(self.pid) else {
            return;
        };

        let mem = p.memory();
        let vmem = p.virtual_memory();
        self.working_set_bytes_last = mem;
        self.virtual_memory_bytes_last = vmem;
        self.working_set_bytes_peak = self.working_set_bytes_peak.max(mem);
        self.virtual_memory_bytes_peak = self.virtual_memory_bytes_peak.max(vmem);

        let cpu = p.cpu_usage().max(0.0) as f64;
        self.cpu_usage_percent_last = cpu;
        self.cpu_usage_percent_max = self.cpu_usage_percent_max.max(cpu);

        // sysinfo computes CPU usage based on a diff between two refreshes.
        // The first refresh establishes a baseline and produces a (likely) zero usage.
        if self.refresh_count >= 2 {
            self.samples = self.samples.saturating_add(1);
            self.cpu_usage_percent_sum += cpu;
        }
    }

    fn finish(self) -> ObservedProcessFootprint {
        let sample_interval_ms = sysinfo::MINIMUM_CPU_UPDATE_INTERVAL
            .as_millis()
            .min(u64::MAX as u128) as u64;
        let avg = if self.samples > 0 {
            self.cpu_usage_percent_sum / (self.samples as f64)
        } else {
            0.0
        };

        ObservedProcessFootprint {
            collector: "sysinfo".to_string(),
            sample_interval_ms,
            samples: self.samples,
            cpu_usage_percent_avg: avg,
            cpu_usage_percent_max: self.cpu_usage_percent_max,
            cpu_usage_percent_last: self.cpu_usage_percent_last,
            working_set_bytes_last: self.working_set_bytes_last,
            working_set_bytes_peak: self.working_set_bytes_peak,
            virtual_memory_bytes_last: self.virtual_memory_bytes_last,
            virtual_memory_bytes_peak: self.virtual_memory_bytes_peak,
        }
    }
}

fn collect_demo_footprint_json(
    demo: &LaunchedDemo,
    killed: bool,
    observed: Option<ObservedProcessFootprint>,
) -> serde_json::Value {
    #[cfg(windows)]
    let _ = observed;

    let ended_unix_ms = now_unix_ms();
    let ended_instant = Instant::now();
    let wall_time_ms = ended_instant
        .duration_since(demo.launched_instant)
        .as_millis()
        .min(u64::MAX as u128) as u64;
    let logical_cores = std::thread::available_parallelism()
        .map(|n| n.get() as u64)
        .unwrap_or(1)
        .max(1);

    let mut out = serde_json::json!({
        "schema_version": 1,
        "kind": "process_footprint",
        "pid": demo.child.id(),
        "launch_cmd": demo.launch_cmd,
        "launched_unix_ms": demo.launched_unix_ms,
        "ended_unix_ms": ended_unix_ms,
        "wall_time_ms": wall_time_ms,
        "killed": killed,
        "logical_cores": logical_cores,
    });

    #[cfg(windows)]
    {
        use std::mem::size_of;
        use std::os::windows::io::AsRawHandle as _;
        use windows_sys::Win32::Foundation::FILETIME;
        use windows_sys::Win32::System::ProcessStatus::{
            GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
        };
        use windows_sys::Win32::System::Threading::GetProcessTimes;

        let handle = demo.child.as_raw_handle();
        unsafe fn filetime_to_100ns(ft: FILETIME) -> u64 {
            ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
        }

        let mut creation: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut exit: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut kernel: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut user: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };

        let times_ok =
            unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) }
                != 0;
        if times_ok {
            let kernel_100ns = unsafe { filetime_to_100ns(kernel) };
            let user_100ns = unsafe { filetime_to_100ns(user) };
            let total_100ns = kernel_100ns.saturating_add(user_100ns);
            let total_time_ms = total_100ns / 10_000;
            let kernel_time_ms = kernel_100ns / 10_000;
            let user_time_ms = user_100ns / 10_000;

            let avg_cpu_cores = if wall_time_ms > 0 {
                (total_time_ms as f64) / (wall_time_ms as f64)
            } else {
                0.0
            };
            let avg_cpu_percent_total = if logical_cores > 0 {
                (avg_cpu_cores / (logical_cores as f64)) * 100.0
            } else {
                0.0
            };

            if let Some(obj) = out.as_object_mut() {
                obj.insert(
                    "cpu".to_string(),
                    serde_json::json!({
                        "total_time_ms": total_time_ms,
                        "kernel_time_ms": kernel_time_ms,
                        "user_time_ms": user_time_ms,
                        "avg_cpu_cores": avg_cpu_cores,
                        "avg_cpu_percent_total_cores": avg_cpu_percent_total,
                    }),
                );
            }
        } else if let Some(obj) = out.as_object_mut() {
            obj.insert(
                "cpu_error".to_string(),
                serde_json::Value::String("GetProcessTimes failed".to_string()),
            );
        }

        let mut mem: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
        mem.cb = size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        let mem_ok = unsafe { GetProcessMemoryInfo(handle, &mut mem, mem.cb) } != 0;
        if mem_ok {
            if let Some(obj) = out.as_object_mut() {
                obj.insert(
                    "memory".to_string(),
                    serde_json::json!({
                        "working_set_bytes": mem.WorkingSetSize as u64,
                        "peak_working_set_bytes": mem.PeakWorkingSetSize as u64,
                        "pagefile_bytes": mem.PagefileUsage as u64,
                        "peak_pagefile_bytes": mem.PeakPagefileUsage as u64,
                    }),
                );
            }
        } else if let Some(obj) = out.as_object_mut() {
            obj.insert(
                "memory_error".to_string(),
                serde_json::Value::String("GetProcessMemoryInfo failed".to_string()),
            );
        }
    }

    #[cfg(not(windows))]
    {
        if let Some(observed) = observed {
            let avg_cpu_cores = observed.cpu_usage_percent_avg / 100.0;
            let avg_cpu_percent_total_cores = if logical_cores > 0 {
                observed.cpu_usage_percent_avg / (logical_cores as f64)
            } else {
                0.0
            };

            if let Some(obj) = out.as_object_mut() {
                obj.insert(
                    "cpu".to_string(),
                    serde_json::json!({
                        "collector": observed.collector,
                        "sample_interval_ms": observed.sample_interval_ms,
                        "samples": observed.samples,
                        "usage_percent_avg": observed.cpu_usage_percent_avg,
                        "usage_percent_max": observed.cpu_usage_percent_max,
                        "usage_percent_last": observed.cpu_usage_percent_last,
                        "avg_cpu_cores": avg_cpu_cores,
                        "avg_cpu_percent_total_cores": avg_cpu_percent_total_cores,
                    }),
                );
                obj.insert(
                    "memory".to_string(),
                    serde_json::json!({
                        // Align naming with Windows for easier tooling consumption.
                        "working_set_bytes": observed.working_set_bytes_last,
                        "peak_working_set_bytes": observed.working_set_bytes_peak,
                        "virtual_memory_bytes": observed.virtual_memory_bytes_last,
                        "peak_virtual_memory_bytes": observed.virtual_memory_bytes_peak,
                        "collector": "sysinfo",
                    }),
                );
            }
        } else if let Some(obj) = out.as_object_mut() {
            obj.insert(
                "note".to_string(),
                serde_json::Value::String("process footprint sampling unavailable".to_string()),
            );
        }
    }

    out
}

fn kill_launched_demo(child: &mut Option<LaunchedDemo>) {
    let Some(demo) = child.take() else {
        return;
    };
    let mut child_proc = demo.child;

    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &child_proc.id().to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = child_proc.kill();
    }

    #[cfg(not(windows))]
    {
        let _ = child_proc.kill();
    }

    let deadline = Instant::now() + Duration::from_millis(3_000);
    while Instant::now() < deadline {
        if child_proc.try_wait().ok().flatten().is_some() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

pub(super) fn stop_launched_demo(
    child: &mut Option<LaunchedDemo>,
    exit_path: &Path,
    poll_ms: u64,
) -> Option<serde_json::Value> {
    let demo = child.as_mut()?;

    let _ = touch(exit_path);
    #[cfg(not(windows))]
    let mut sampler = SysinfoProcessFootprintSampler::new(demo.child.id());

    let deadline = Instant::now() + Duration::from_millis(20_000);
    while Instant::now() < deadline {
        #[cfg(not(windows))]
        sampler.refresh_if_due();

        let exited = demo.child.try_wait().ok().flatten().is_some();
        if exited {
            #[cfg(not(windows))]
            sampler.refresh_force();

            let footprint = Some(collect_demo_footprint_json(demo, false, {
                #[cfg(not(windows))]
                {
                    Some(sampler.finish())
                }
                #[cfg(windows)]
                {
                    None
                }
            }));
            if let Some(mut c) = child.take().map(|d| d.child) {
                let _ = c.wait();
            }
            return footprint;
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }

    #[cfg(not(windows))]
    sampler.refresh_force();

    let footprint = Some(collect_demo_footprint_json(demo, true, {
        #[cfg(not(windows))]
        {
            Some(sampler.finish())
        }
        #[cfg(windows)]
        {
            None
        }
    }));
    kill_launched_demo(child);
    footprint
}

pub(super) fn ensure_env_var(env: &mut Vec<(String, String)>, key: &str, value: &str) -> bool {
    if env.iter().any(|(k, _)| k == key) {
        return false;
    }
    env.push((key.to_string(), value.to_string()));
    true
}

pub(super) fn cargo_run_inject_feature(cmd: &mut Vec<String>, feature: &str) -> bool {
    if cmd.is_empty() {
        return false;
    }

    let exe = cmd[0].to_ascii_lowercase();
    let is_cargo = exe == "cargo" || exe.ends_with("\\cargo.exe") || exe.ends_with("/cargo");
    if !is_cargo {
        return false;
    }

    let Some(run_idx) = cmd.iter().position(|a| a == "run") else {
        return false;
    };

    // If `--features` already exists, try to extend it.
    for i in 0..cmd.len() {
        if cmd[i] == "--features" || cmd[i] == "-F" {
            if let Some(value) = cmd.get_mut(i + 1) {
                let mut features: Vec<&str> = value
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                if features.iter().any(|f| *f == feature) {
                    return false;
                }
                features.push(feature);
                *value = features.join(",");
                return true;
            }
        }
        if let Some(rest) = cmd[i].strip_prefix("--features=") {
            let mut features: Vec<&str> = rest
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if features.iter().any(|f| *f == feature) {
                return false;
            }
            features.push(feature);
            cmd[i] = format!("--features={}", features.join(","));
            return true;
        }
        if let Some(rest) = cmd[i].strip_prefix("-F=") {
            let mut features: Vec<&str> = rest
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if features.iter().any(|f| *f == feature) {
                return false;
            }
            features.push(feature);
            cmd[i] = format!("-F={}", features.join(","));
            return true;
        }
    }

    // Insert `--features` before the `--` sentinel (if any), otherwise append.
    let sentinel = cmd.iter().position(|a| a == "--").unwrap_or(cmd.len());
    let insert_at = sentinel.max(run_idx + 1);
    cmd.insert(insert_at, "--features".to_string());
    cmd.insert(insert_at + 1, feature.to_string());
    true
}

fn list_files_with_extensions(dir: &Path, exts: &[&str]) -> Vec<PathBuf> {
    fn visit(out: &mut Vec<PathBuf>, dir: &Path, exts: &[&str]) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                visit(out, &path, exts);
                continue;
            }
            if !meta.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            if ext.is_empty() {
                continue;
            }
            if exts
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(ext.as_str()))
            {
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    visit(&mut out, dir, exts);
    out.sort();
    out
}

pub(super) fn wait_for_files_with_extensions(
    dir: &Path,
    exts: &[&str],
    timeout_ms: u64,
    poll_ms: u64,
) -> Vec<PathBuf> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        let files = list_files_with_extensions(dir, exts);
        if !files.is_empty() {
            return files;
        }
        if Instant::now() >= deadline {
            return files;
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }
}

#[derive(Debug, Clone)]
pub(super) struct RenderdocDumpAttempt {
    pub(super) marker: String,
    pub(super) out_dir: PathBuf,
    pub(super) exit_code: Option<i32>,
    pub(super) response_json: Option<PathBuf>,
    pub(super) stdout_file: Option<PathBuf>,
    pub(super) stderr_file: Option<PathBuf>,
    pub(super) error: Option<String>,
}

pub(super) fn run_fret_renderdoc_dump(
    workspace_root: &Path,
    capture: &Path,
    out_dir: &Path,
    basename: &str,
    marker: &str,
    max_results: Option<u32>,
    no_uniform_bytes: bool,
    no_outputs_png: bool,
    print_summary_md_top: Option<u32>,
) -> RenderdocDumpAttempt {
    let _ = std::fs::create_dir_all(out_dir);

    let stdout_path = out_dir.join("stdout.txt");
    let stderr_path = out_dir.join("stderr.txt");

    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root);
    cmd.args(["run", "-p", "fret-renderdoc", "--", "dump"]);
    cmd.args(["--capture", &capture.display().to_string()]);
    cmd.args(["--marker", marker]);
    cmd.args(["--out", &out_dir.display().to_string()]);
    cmd.args(["--basename", basename]);
    if let Some(n) = max_results {
        cmd.args(["--max-results", &n.to_string()]);
    }
    if no_uniform_bytes {
        cmd.arg("--no-uniform-bytes");
    }
    if no_outputs_png {
        cmd.arg("--no-outputs-png");
    }
    if let Some(top) = print_summary_md_top {
        cmd.args(["--print-summary", "md", &top.to_string()]);
    }
    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR").filter(|v| !v.is_empty()) {
        cmd.env("CARGO_TARGET_DIR", target_dir);
    }

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return RenderdocDumpAttempt {
                marker: marker.to_string(),
                out_dir: out_dir.to_path_buf(),
                exit_code: None,
                response_json: None,
                stdout_file: None,
                stderr_file: None,
                error: Some(format!("failed to run fret-renderdoc: {e}")),
            };
        }
    };

    let _ = std::fs::write(&stdout_path, &output.stdout);
    let _ = std::fs::write(&stderr_path, &output.stderr);

    let exit_code = output.status.code();

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let response_json = stdout_str
        .lines()
        .rev()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && l.ends_with(".response.json"))
        .map(|l| {
            let p = PathBuf::from(l);
            if p.is_absolute() {
                p
            } else {
                workspace_root.join(p)
            }
        });

    RenderdocDumpAttempt {
        marker: marker.to_string(),
        out_dir: out_dir.to_path_buf(),
        exit_code,
        response_json,
        stdout_file: Some(stdout_path),
        stderr_file: Some(stderr_path),
        error: None,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PerfThresholds {
    pub(super) max_top_total_us: Option<u64>,
    pub(super) max_top_layout_us: Option<u64>,
    pub(super) max_top_solve_us: Option<u64>,
    pub(super) max_pointer_move_dispatch_us: Option<u64>,
    pub(super) max_pointer_move_hit_test_us: Option<u64>,
    pub(super) max_pointer_move_global_changes: Option<u64>,
}

impl PerfThresholds {
    pub(super) fn any(self) -> bool {
        self.max_top_total_us.is_some()
            || self.max_top_layout_us.is_some()
            || self.max_top_solve_us.is_some()
            || self.max_pointer_move_dispatch_us.is_some()
            || self.max_pointer_move_hit_test_us.is_some()
            || self.max_pointer_move_global_changes.is_some()
    }
}

#[derive(Debug, Clone)]
pub(super) struct PerfBaselineFile {
    pub(super) path: PathBuf,
    pub(super) thresholds_by_script: std::collections::HashMap<String, PerfThresholds>,
}

pub(super) fn normalize_repo_relative_path(workspace_root: &Path, p: &Path) -> String {
    let rel = p.strip_prefix(workspace_root).unwrap_or(p);
    let mut out = String::new();
    for (idx, part) in rel.components().enumerate() {
        let s = part.as_os_str().to_string_lossy();
        if idx > 0 {
            out.push('/');
        }
        out.push_str(&s);
    }
    out
}

pub(super) fn read_perf_baseline_file(
    workspace_root: &Path,
    path: &Path,
) -> Result<PerfBaselineFile, String> {
    use std::collections::HashMap;

    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    };

    let bytes = std::fs::read(&resolved).map_err(|e| {
        format!(
            "failed to read perf baseline file {}: {e}",
            resolved.display()
        )
    })?;
    let root: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
        format!(
            "failed to parse perf baseline JSON {}: {e}",
            resolved.display()
        )
    })?;

    let schema_version = root
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if schema_version != 1 {
        return Err(format!(
            "unsupported perf baseline schema_version={schema_version} (expected 1): {}",
            resolved.display()
        ));
    }
    let kind = root.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    if kind != "perf_baseline" {
        return Err(format!(
            "invalid perf baseline kind={kind:?} (expected \"perf_baseline\"): {}",
            resolved.display()
        ));
    }

    let rows = root.get("rows").and_then(|v| v.as_array()).ok_or_else(|| {
        format!(
            "invalid perf baseline: missing rows array: {}",
            resolved.display()
        )
    })?;

    let mut thresholds_by_script: HashMap<String, PerfThresholds> = HashMap::new();
    for row in rows {
        let Some(script) = row.get("script").and_then(|v| v.as_str()) else {
            continue;
        };
        let t = row.get("thresholds").and_then(|v| v.as_object());
        let thresholds = PerfThresholds {
            max_top_total_us: t
                .and_then(|m| m.get("max_top_total_us"))
                .and_then(|v| v.as_u64()),
            max_top_layout_us: t
                .and_then(|m| m.get("max_top_layout_us"))
                .and_then(|v| v.as_u64()),
            max_top_solve_us: t
                .and_then(|m| m.get("max_top_solve_us"))
                .and_then(|v| v.as_u64()),
            max_pointer_move_dispatch_us: t
                .and_then(|m| m.get("max_pointer_move_dispatch_us"))
                .and_then(|v| v.as_u64()),
            max_pointer_move_hit_test_us: t
                .and_then(|m| m.get("max_pointer_move_hit_test_us"))
                .and_then(|v| v.as_u64()),
            max_pointer_move_global_changes: t
                .and_then(|m| m.get("max_pointer_move_global_changes"))
                .and_then(|v| v.as_u64()),
        };

        thresholds_by_script.insert(script.to_string(), thresholds);
    }

    Ok(PerfBaselineFile {
        path: resolved,
        thresholds_by_script,
    })
}

pub(super) fn apply_perf_baseline_headroom(value_us: u64, headroom_pct: u32) -> u64 {
    let pct = (headroom_pct as u64).min(10_000);
    value_us.saturating_mul(100 + pct).saturating_add(99) / 100
}

pub(super) fn resolve_threshold(
    cli: Option<u64>,
    baseline: Option<u64>,
) -> (Option<u64>, Option<&'static str>) {
    if let Some(v) = cli {
        return (Some(v), Some("cli"));
    }
    if let Some(v) = baseline {
        return (Some(v), Some("baseline"));
    }
    (None, None)
}

pub(super) fn scan_perf_threshold_failures(
    script: &str,
    sort: BundleStatsSort,
    cli: PerfThresholds,
    baseline: PerfThresholds,
    max_total_time_us: u64,
    max_layout_time_us: u64,
    max_layout_engine_solve_time_us: u64,
    max_pointer_move_dispatch_time_us: u64,
    max_pointer_move_hit_test_time_us: u64,
    max_pointer_move_global_changes: u64,
) -> Vec<serde_json::Value> {
    let mut out: Vec<serde_json::Value> = Vec::new();
    let (threshold_total, source_total) =
        resolve_threshold(cli.max_top_total_us, baseline.max_top_total_us);
    let (threshold_layout, source_layout) =
        resolve_threshold(cli.max_top_layout_us, baseline.max_top_layout_us);
    let (threshold_solve, source_solve) =
        resolve_threshold(cli.max_top_solve_us, baseline.max_top_solve_us);
    let (threshold_pointer_move_dispatch, source_pointer_move_dispatch) = resolve_threshold(
        cli.max_pointer_move_dispatch_us,
        baseline.max_pointer_move_dispatch_us,
    );
    let (threshold_pointer_move_hit_test, source_pointer_move_hit_test) = resolve_threshold(
        cli.max_pointer_move_hit_test_us,
        baseline.max_pointer_move_hit_test_us,
    );
    let (threshold_pointer_move_global_changes, source_pointer_move_global_changes) =
        resolve_threshold(
            cli.max_pointer_move_global_changes,
            baseline.max_pointer_move_global_changes,
        );

    if let Some(threshold_us) = threshold_total
        && max_total_time_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "top_total_time_us",
            "threshold_us": threshold_us,
            "threshold_source": source_total,
            "actual_us": max_total_time_us,
            "script": script,
            "sort": sort.as_str(),
        }));
    }
    if let Some(threshold_us) = threshold_layout
        && max_layout_time_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "top_layout_time_us",
            "threshold_us": threshold_us,
            "threshold_source": source_layout,
            "actual_us": max_layout_time_us,
            "script": script,
            "sort": sort.as_str(),
        }));
    }
    if let Some(threshold_us) = threshold_solve
        && max_layout_engine_solve_time_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "top_layout_engine_solve_time_us",
            "threshold_us": threshold_us,
            "threshold_source": source_solve,
            "actual_us": max_layout_engine_solve_time_us,
            "script": script,
            "sort": sort.as_str(),
        }));
    }
    if let Some(threshold_us) = threshold_pointer_move_dispatch
        && max_pointer_move_dispatch_time_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "pointer_move_max_dispatch_time_us",
            "threshold_us": threshold_us,
            "threshold_source": source_pointer_move_dispatch,
            "actual_us": max_pointer_move_dispatch_time_us,
            "script": script,
            "sort": sort.as_str(),
        }));
    }
    if let Some(threshold_us) = threshold_pointer_move_hit_test
        && max_pointer_move_hit_test_time_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "pointer_move_max_hit_test_time_us",
            "threshold_us": threshold_us,
            "threshold_source": source_pointer_move_hit_test,
            "actual_us": max_pointer_move_hit_test_time_us,
            "script": script,
            "sort": sort.as_str(),
        }));
    }
    if let Some(threshold) = threshold_pointer_move_global_changes
        && max_pointer_move_global_changes > threshold
    {
        out.push(serde_json::json!({
            "metric": "pointer_move_snapshots_with_global_changes",
            "threshold": threshold,
            "threshold_source": source_pointer_move_global_changes,
            "actual": max_pointer_move_global_changes,
            "script": script,
            "sort": sort.as_str(),
        }));
    }
    out
}

// (moved to `diag::util`)
