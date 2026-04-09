use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(windows)]
use std::process::Stdio;
use std::time::{Duration, Instant};

use fret_diag_protocol::{UiDiagnosticsConfigFileV1, UiDiagnosticsConfigPathsV1};

use crate::launch_env_policy::{TOOL_LAUNCH_SCRUB_ENV_PREFIXES, tool_launch_env_key_is_reserved};

use super::LaunchedDemo;
use super::stats::BundleStatsSort;
use super::util::{now_unix_ms, touch, write_json_value};

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
    let a_semantics = crate::json_bundle::SemanticsResolver::new(a_bundle);
    let b_semantics = crate::json_bundle::SemanticsResolver::new(b_bundle);

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
    if !opts.ignore_scene_fingerprint
        && let (Some(a_fp), Some(b_fp)) = (a_fp, b_fp)
        && a_fp != b_fp
    {
        diffs.push(CompareDiff {
            kind: "scene_fingerprint_mismatch",
            key: None,
            field: Some("scene_fingerprint"),
            a: Some(serde_json::Value::from(a_fp)),
            b: Some(serde_json::Value::from(b_fp)),
        });
    }

    if let (Some(a_snapshot), Some(b_snapshot)) = (a_snapshot, b_snapshot) {
        compare_semantics_by_test_id(
            &mut diffs,
            &a_semantics,
            &b_semantics,
            a_snapshot,
            b_snapshot,
            opts,
        )?;
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
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    windows
        .first()
        .ok_or_else(|| "bundle contains no windows".to_string())
}

#[derive(Debug, Clone, Copy, Default)]
struct SelectedSnapshotInfo {
    frame_id: Option<u64>,
}

fn select_snapshot_for_compare(
    window: &serde_json::Value,
    warmup_frames: u64,
) -> (Option<&serde_json::Value>, SelectedSnapshotInfo) {
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
    a_semantics: &crate::json_bundle::SemanticsResolver<'_>,
    b_semantics: &crate::json_bundle::SemanticsResolver<'_>,
    a_snapshot: &serde_json::Value,
    b_snapshot: &serde_json::Value,
    opts: CompareOptions,
) -> Result<(), String> {
    let a_sem = a_semantics.semantics_snapshot(a_snapshot).ok_or_else(|| {
        "bundle snapshot missing semantics (ensure FRET_DIAG_SEMANTICS=1)".to_string()
    })?;
    let b_sem = b_semantics.semantics_snapshot(b_snapshot).ok_or_else(|| {
        "bundle snapshot missing semantics (ensure FRET_DIAG_SEMANTICS=1)".to_string()
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
        if !opts.ignore_bounds
            && let (Some(a), Some(b)) = (a_node.bounds, b_node.bounds)
            && !rect_eq_eps(a, b, opts.eps_px)
        {
            diffs.push(CompareDiff {
                kind: "node_field_mismatch",
                key: Some(test_id.clone()),
                field: Some("bounds"),
                a: Some(serde_json::json!({ "x": a.0, "y": a.1, "w": a.2, "h": a.3 })),
                b: Some(serde_json::json!({ "x": b.0, "y": b.1, "w": b.2, "h": b.3 })),
            });
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
    if (a_focus_tid.is_some() || b_focus_tid.is_some()) && a_focus_tid != b_focus_tid {
        diffs.push(CompareDiff {
            kind: "focus_mismatch",
            key: None,
            field: Some("focus.test_id"),
            a: a_focus_tid.map(serde_json::Value::from),
            b: b_focus_tid.map(serde_json::Value::from),
        });
    }

    let a_captured = a_sem.get("captured").and_then(|v| v.as_u64());
    let b_captured = b_sem.get("captured").and_then(|v| v.as_u64());
    let a_captured_tid = a_captured.and_then(|id| a_id_to_test_id.get(&id).cloned());
    let b_captured_tid = b_captured.and_then(|id| b_id_to_test_id.get(&id).cloned());
    if (a_captured_tid.is_some() || b_captured_tid.is_some()) && a_captured_tid != b_captured_tid {
        diffs.push(CompareDiff {
            kind: "captured_mismatch",
            key: None,
            field: Some("captured.test_id"),
            a: a_captured_tid.map(serde_json::Value::from),
            b: b_captured_tid.map(serde_json::Value::from),
        });
    }
}

pub(crate) fn read_latest_pointer(out_dir: &Path) -> Option<PathBuf> {
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

pub(crate) fn find_latest_export_dir(out_dir: &Path) -> Option<PathBuf> {
    fn parse_leading_ts(name: &str) -> Option<u64> {
        let digits: String = name.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return None;
        }
        digits.parse::<u64>().ok()
    }

    let mut best: Option<(u64, PathBuf)> = None;
    let entries = std::fs::read_dir(out_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(ts) = parse_leading_ts(&name) else {
            continue;
        };
        match &best {
            Some((prev, _)) if *prev >= ts => {}
            _ => best = Some((ts, path)),
        }
    }
    best.map(|(_, p)| p)
}

fn tool_launched_diag_config(
    out_dir: &Path,
    fs_transport_cfg: &crate::transport::FsDiagTransportConfig,
    ready_path: &Path,
    exit_path: &Path,
    wants_screenshots: bool,
    launch_write_bundle_json: bool,
    launch_env: &[(String, String)],
) -> UiDiagnosticsConfigFileV1 {
    let mut cfg = UiDiagnosticsConfigFileV1 {
        schema_version: 1,
        enabled: Some(true),
        out_dir: Some(out_dir.to_string_lossy().to_string()),
        paths: Some(UiDiagnosticsConfigPathsV1 {
            trigger_path: Some(fs_transport_cfg.trigger_path.to_string_lossy().to_string()),
            ready_path: Some(ready_path.to_string_lossy().to_string()),
            exit_path: Some(exit_path.to_string_lossy().to_string()),
            screenshot_request_path: Some(
                fs_transport_cfg
                    .screenshots_request_path
                    .to_string_lossy()
                    .to_string(),
            ),
            screenshot_trigger_path: Some(
                fs_transport_cfg
                    .screenshots_trigger_path
                    .to_string_lossy()
                    .to_string(),
            ),
            screenshot_result_path: Some(
                fs_transport_cfg
                    .screenshots_result_path
                    .to_string_lossy()
                    .to_string(),
            ),
            screenshot_result_trigger_path: Some(
                fs_transport_cfg
                    .screenshots_result_trigger_path
                    .to_string_lossy()
                    .to_string(),
            ),
            script_path: Some(fs_transport_cfg.script_path.to_string_lossy().to_string()),
            script_trigger_path: Some(
                fs_transport_cfg
                    .script_trigger_path
                    .to_string_lossy()
                    .to_string(),
            ),
            script_result_path: Some(
                fs_transport_cfg
                    .script_result_path
                    .to_string_lossy()
                    .to_string(),
            ),
            script_result_trigger_path: Some(
                fs_transport_cfg
                    .script_result_trigger_path
                    .to_string_lossy()
                    .to_string(),
            ),
            pick_trigger_path: Some(
                fs_transport_cfg
                    .pick_trigger_path
                    .to_string_lossy()
                    .to_string(),
            ),
            pick_result_path: Some(
                fs_transport_cfg
                    .pick_result_path
                    .to_string_lossy()
                    .to_string(),
            ),
            pick_result_trigger_path: Some(
                fs_transport_cfg
                    .pick_result_trigger_path
                    .to_string_lossy()
                    .to_string(),
            ),
            inspect_path: Some(fs_transport_cfg.inspect_path.to_string_lossy().to_string()),
            inspect_trigger_path: Some(
                fs_transport_cfg
                    .inspect_trigger_path
                    .to_string_lossy()
                    .to_string(),
            ),
            ..Default::default()
        }),
        // Tooling upgrades scripts to schema v2 on execution; keep the runtime v2-only in
        // tool-launched runs so compat paths stay removable.
        allow_script_schema_v1: Some(false),
        screenshots_enabled: Some(wants_screenshots),
        // Keep default artifacts small-by-default for tool-launched runs:
        // - sidecars + manifest are sufficient for most triage flows
        // - compact schema2 view is useful for downstream tooling without the raw monolith
        write_bundle_json: Some(launch_write_bundle_json),
        write_bundle_schema2: Some(true),
        // Keep script-driven bundle dumps reasonably small by default.
        //
        // The runtime exports full frame snapshots; large dump windows can easily produce
        // 10s of MB per bundle, which makes sharing/triage harder and increases the chance
        // of accidental output explosions.
        script_dump_max_snapshots: Some(10),
        // Tool-launched runs should be small-by-default; auto-dumping after every injected
        // step is useful during script authoring but is too explosive for suites.
        script_auto_dump: Some(false),
        pick_auto_dump: Some(false),
        // Keep the diagnostics runtime alive between frames so filesystem-triggered scripts
        // can be observed and started even if the app goes idle between runs.
        script_keepalive: Some(true),
        // Bound the length of exported debug strings (paths, etc).
        max_debug_string_bytes: Some(2048),
        // Keep tool-launched scripted runs deterministic even if the user moves/clicks the real
        // mouse while playback is active (especially for cross-window docking/tear-off).
        isolate_external_pointer_input_while_script_running: Some(true),
        // Keep tool-launched scripted runs deterministic even if the user types while playback
        // is active (keyboard/text/IME interference).
        isolate_external_keyboard_input_while_script_running: Some(true),
        ..Default::default()
    };

    if let Some((_, v)) = launch_env
        .iter()
        .find(|(k, _)| k == "FRET_DIAG_FIXED_FRAME_DELTA_MS")
        && let Ok(parsed) = v.trim().parse::<u64>()
    {
        cfg.frame_clock_fixed_delta_ms = Some(parsed);
    }
    if let Some((_, v)) = launch_env
        .iter()
        .find(|(k, _)| k == "FRET_DIAG_REDACT_TEXT")
    {
        let raw = v.trim();
        if !raw.is_empty() {
            cfg.redact_text = Some(raw != "0");
        }
    }

    cfg
}

fn validate_tool_launched_diag_config(
    cfg: &UiDiagnosticsConfigFileV1,
    wants_screenshots: bool,
    launch_write_bundle_json: bool,
) -> Result<(), String> {
    // Keep these invariants explicit so future edits can't accidentally regress tool-launched runs
    // back to "huge raw bundle.json by default" or other output-explosion behaviors.
    if cfg.enabled != Some(true) {
        return Err("tool launch config invariant failed: enabled must be true".to_string());
    }
    if cfg.allow_script_schema_v1 != Some(false) {
        return Err(
            "tool launch config invariant failed: allow_script_schema_v1 must be false".to_string(),
        );
    }
    if cfg.write_bundle_schema2 != Some(true) {
        return Err(
            "tool launch config invariant failed: write_bundle_schema2 must be true".to_string(),
        );
    }
    if cfg.write_bundle_json != Some(launch_write_bundle_json) {
        return Err(format!(
            "tool launch config invariant failed: write_bundle_json must equal --launch-write-bundle-json ({})",
            launch_write_bundle_json
        ));
    }
    if cfg.script_auto_dump != Some(false) || cfg.pick_auto_dump != Some(false) {
        return Err("tool launch config invariant failed: script_auto_dump and pick_auto_dump must be false".to_string());
    }
    if cfg.screenshots_enabled != Some(wants_screenshots) {
        return Err(format!(
            "tool launch config invariant failed: screenshots_enabled must equal wants_screenshots ({})",
            wants_screenshots
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn maybe_launch_demo(
    launch: &Option<Vec<String>>,
    launch_env: &[(String, String)],
    workspace_root: &Path,
    ready_path: &Path,
    exit_path: &Path,
    fs_transport_cfg: &crate::transport::FsDiagTransportConfig,
    wants_screenshots: bool,
    launch_write_bundle_json: bool,
    timeout_ms: u64,
    poll_ms: u64,
    launch_high_priority: bool,
) -> Result<Option<LaunchedDemo>, String> {
    let Some(launch) = launch else {
        return Ok(None);
    };

    // Tool-launched runs should be deterministic and small-by-default. The runtime's config
    // resolution is "env overrides config", so inherited shell env vars can silently override the
    // per-run `diag.config.json` that tooling writes. Scrub known diagnostics env keys from the
    // inherited environment, then re-apply any explicit `--env KEY=VALUE` overrides below.
    //
    // This avoids "works on my machine" drift and reduces the chance of accidental output
    // explosions (e.g. large snapshot caps or pretty-printed raw bundles) during tool-launched
    // runs.
    let scrub_env_prefixes = TOOL_LAUNCH_SCRUB_ENV_PREFIXES;

    let prev_ready_mtime = std::fs::metadata(ready_path)
        .and_then(|m| m.modified())
        .ok();

    let exe = launch
        .first()
        .ok_or_else(|| "missing launch command".to_string())?;

    let out_dir = &fs_transport_cfg.out_dir;

    let mut cmd = Command::new(exe);
    cmd.args(launch.iter().skip(1));
    cmd.current_dir(workspace_root);

    let mut inherited_keys_to_remove: Vec<String> = std::env::vars()
        .map(|(k, _)| k)
        .filter(|k| scrub_env_prefixes.iter().any(|p| k.starts_with(p)))
        .collect();
    inherited_keys_to_remove.sort();
    inherited_keys_to_remove.dedup();

    let mut scrubbed_inherited_nonreserved_keys: Vec<String> = Vec::new();
    for key in &inherited_keys_to_remove {
        if !tool_launch_env_key_is_reserved(key) {
            scrubbed_inherited_nonreserved_keys.push(key.clone());
        }
        cmd.env_remove(key);
    }

    let mut explicit_override_fret_keys: Vec<String> = Vec::new();
    let mut explicit_override_other_keys_total: u64 = 0;
    for (key, _) in launch_env {
        if key.starts_with("FRET_") {
            explicit_override_fret_keys.push(key.clone());
        } else {
            explicit_override_other_keys_total =
                explicit_override_other_keys_total.saturating_add(1);
        }
    }
    explicit_override_fret_keys.sort();
    explicit_override_fret_keys.dedup();

    if !scrubbed_inherited_nonreserved_keys.is_empty()
        || !explicit_override_fret_keys.is_empty()
        || explicit_override_other_keys_total > 0
    {
        fn format_keys(keys: &[String], max: usize) -> String {
            let mut out: Vec<&str> = keys.iter().take(max).map(|s| s.as_str()).collect();
            if keys.len() > max {
                out.push("...");
            }
            out.join(",")
        }

        scrubbed_inherited_nonreserved_keys.sort();
        scrubbed_inherited_nonreserved_keys.dedup();

        let scrubbed = format_keys(&scrubbed_inherited_nonreserved_keys, 8);
        let overrides = format_keys(&explicit_override_fret_keys, 8);
        eprintln!(
            "diag --launch env: scrubbed_inherited_nonreserved=[{}] explicit_overrides_fret=[{}] explicit_overrides_other_total={} (see: fretboard-dev diag config doctor --mode launch --report-json)",
            scrubbed, overrides, explicit_override_other_keys_total,
        );
    }
    cmd.env("FRET_DIAG", "1");
    cmd.env("FRET_DIAG_DIR", out_dir);
    cmd.env("FRET_DIAG_TRIGGER_PATH", &fs_transport_cfg.trigger_path);
    cmd.env("FRET_DIAG_READY_PATH", ready_path);
    cmd.env("FRET_DIAG_EXIT_PATH", exit_path);
    cmd.env("FRET_DIAG_SCRIPT_PATH", &fs_transport_cfg.script_path);
    cmd.env(
        "FRET_DIAG_SCRIPT_TRIGGER_PATH",
        &fs_transport_cfg.script_trigger_path,
    );
    cmd.env(
        "FRET_DIAG_SCRIPT_RESULT_PATH",
        &fs_transport_cfg.script_result_path,
    );
    cmd.env(
        "FRET_DIAG_SCRIPT_RESULT_TRIGGER_PATH",
        &fs_transport_cfg.script_result_trigger_path,
    );
    cmd.env(
        "FRET_DIAG_PICK_TRIGGER_PATH",
        &fs_transport_cfg.pick_trigger_path,
    );
    cmd.env(
        "FRET_DIAG_PICK_RESULT_PATH",
        &fs_transport_cfg.pick_result_path,
    );
    cmd.env(
        "FRET_DIAG_PICK_RESULT_TRIGGER_PATH",
        &fs_transport_cfg.pick_result_trigger_path,
    );
    cmd.env("FRET_DIAG_INSPECT_PATH", &fs_transport_cfg.inspect_path);
    cmd.env(
        "FRET_DIAG_INSPECT_TRIGGER_PATH",
        &fs_transport_cfg.inspect_trigger_path,
    );
    cmd.env(
        "FRET_DIAG_SCREENSHOT_REQUEST_PATH",
        &fs_transport_cfg.screenshots_request_path,
    );
    cmd.env(
        "FRET_DIAG_SCREENSHOT_TRIGGER_PATH",
        &fs_transport_cfg.screenshots_trigger_path,
    );
    cmd.env(
        "FRET_DIAG_SCREENSHOT_RESULT_PATH",
        &fs_transport_cfg.screenshots_result_path,
    );
    cmd.env(
        "FRET_DIAG_SCREENSHOT_RESULT_TRIGGER_PATH",
        &fs_transport_cfg.screenshots_result_trigger_path,
    );
    // Runner-visible knob: used to best-effort isolate OS cursor/device events during scripted
    // docking drags. The diagnostics runtime also reads this as an env override.
    cmd.env("FRET_DIAG_ISOLATE_POINTER_INPUT", "1");
    // Keep tool-launched scripted runs deterministic even if the user types while playback is
    // active. The diagnostics runtime reads this as an env override.
    cmd.env("FRET_DIAG_ISOLATE_KEYBOARD_INPUT", "1");

    // Config file is the compat-first consolidation path for diagnostics runtime config.
    //
    // For tool-launched runs we treat this as required: if it can't be written, abort rather than
    // silently falling back to runtime defaults (which can re-enable large `bundle.json` writing).
    let config_path = out_dir.join("diag.config.json");
    cmd.env("FRET_DIAG_CONFIG_PATH", &config_path);
    let cfg = tool_launched_diag_config(
        out_dir,
        fs_transport_cfg,
        ready_path,
        exit_path,
        wants_screenshots,
        launch_write_bundle_json,
        launch_env,
    );
    validate_tool_launched_diag_config(&cfg, wants_screenshots, launch_write_bundle_json)?;
    let bytes = serde_json::to_vec_pretty(&cfg)
        .map_err(|e| format!("failed to serialize diag config: {e}"))?;
    std::fs::create_dir_all(out_dir).map_err(|e| {
        format!(
            "failed to create diagnostics out_dir (required for --launch): {} ({e})",
            out_dir.display()
        )
    })?;
    std::fs::write(&config_path, bytes).map_err(|e| {
        format!(
            "failed to write tool launch diag config (required for --launch): {} ({e})",
            config_path.display()
        )
    })?;

    for (key, value) in launch_env {
        if tool_launch_env_key_is_reserved(key) {
            return Err(format!("--env cannot override reserved var: {key}"));
        }
        cmd.env(key, value);
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
    let pid = child.id();

    #[cfg(not(windows))]
    let _ = launch_high_priority;

    #[cfg(windows)]
    if launch_high_priority {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::Threading::{HIGH_PRIORITY_CLASS, SetPriorityClass};

        let handle = child.as_raw_handle();
        let ok = unsafe { SetPriorityClass(handle, HIGH_PRIORITY_CLASS) } != 0;
        if !ok {
            eprintln!(
                "WARN: failed to set HIGH_PRIORITY_CLASS on launched demo: {}",
                std::io::Error::last_os_error()
            );
        }
    }

    // Write the PID for external profilers (ETW/WPA) and post-run triage.
    //
    // Best-effort: avoid failing the run due to filesystem issues.
    let _ = std::fs::write(out_dir.join("launched.pid"), pid.to_string());
    let _ = std::fs::write(
        out_dir.join("launched.demo.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": 1,
            "pid": pid,
            "launched_unix_ms": launched_unix_ms,
            "launch_cmd": launch,
        }))
        .unwrap_or_else(|_| b"{}".to_vec()),
    );

    let mut demo = LaunchedDemo {
        child,
        launched_unix_ms,
        launched_instant,
        launch_cmd: launch.clone(),
        #[cfg(not(windows))]
        process_footprint_sampler: Some(ProcessFootprintSamplerHandle::spawn(pid)),
    };

    // Avoid racing cold-start compilation by waiting for the app to signal readiness.
    // `--launch` commonly runs `cargo run`, which may require a cold build and can take several
    // minutes in large workspaces. Waiting long enough here avoids racing the diagnostics trigger
    // protocol (the in-app side treats the first observed stamp as a baseline).
    let exe_lower = exe.to_ascii_lowercase();
    let is_cargo = exe_lower == "cargo"
        || exe_lower.ends_with("\\cargo.exe")
        || exe_lower.ends_with("/cargo.exe");
    let min_timeout_ms = if is_cargo { 600_000 } else { 30_000 };
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(min_timeout_ms));
    while Instant::now() < deadline {
        match demo.child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!(
                    "launched demo exited before signaling readiness (ready.touch): {status}"
                ));
            }
            Ok(None) => {}
            Err(e) => {
                return Err(format!(
                    "failed to query launched demo status while waiting for readiness: {e}"
                ));
            }
        }

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

pub(crate) fn maybe_launch_demo_without_diagnostics(
    launch: &Option<Vec<String>>,
    launch_env: &[(String, String)],
    workspace_root: &Path,
    out_dir: &Path,
    _poll_ms: u64,
    _launch_high_priority: bool,
) -> Result<Option<LaunchedDemo>, String> {
    let Some(launch) = launch else {
        return Ok(None);
    };

    let exe = launch
        .first()
        .ok_or_else(|| "missing launch command".to_string())?;

    let mut cmd = Command::new(exe);
    cmd.args(launch.iter().skip(1));
    cmd.current_dir(workspace_root);

    let scrub_env_prefixes = TOOL_LAUNCH_SCRUB_ENV_PREFIXES;
    let mut inherited_keys_to_remove: Vec<String> = std::env::vars()
        .map(|(k, _)| k)
        .filter(|k| scrub_env_prefixes.iter().any(|p| k.starts_with(p)))
        .collect();
    inherited_keys_to_remove.sort();
    inherited_keys_to_remove.dedup();

    let mut scrubbed_inherited_nonreserved_keys: Vec<String> = Vec::new();
    for key in &inherited_keys_to_remove {
        if !tool_launch_env_key_is_reserved(key) {
            scrubbed_inherited_nonreserved_keys.push(key.clone());
        }
        cmd.env_remove(key);
    }

    let mut explicit_override_fret_keys: Vec<String> = Vec::new();
    let mut explicit_override_other_keys_total: u64 = 0;
    for (key, _) in launch_env {
        if key.starts_with("FRET_") {
            explicit_override_fret_keys.push(key.clone());
        } else {
            explicit_override_other_keys_total =
                explicit_override_other_keys_total.saturating_add(1);
        }
    }
    explicit_override_fret_keys.sort();
    explicit_override_fret_keys.dedup();

    if !scrubbed_inherited_nonreserved_keys.is_empty()
        || !explicit_override_fret_keys.is_empty()
        || explicit_override_other_keys_total > 0
    {
        fn format_keys(keys: &[String], max: usize) -> String {
            let mut out: Vec<&str> = keys.iter().take(max).map(|s| s.as_str()).collect();
            if keys.len() > max {
                out.push("...");
            }
            out.join(",")
        }

        scrubbed_inherited_nonreserved_keys.sort();
        scrubbed_inherited_nonreserved_keys.dedup();

        let scrubbed = format_keys(&scrubbed_inherited_nonreserved_keys, 8);
        let overrides = format_keys(&explicit_override_fret_keys, 8);
        eprintln!(
            "diag --launch(no-diagnostics) env: scrubbed_inherited_nonreserved=[{}] explicit_overrides_fret=[{}] explicit_overrides_other_total={}",
            scrubbed, overrides, explicit_override_other_keys_total,
        );
    }

    std::fs::create_dir_all(out_dir).map_err(|e| {
        format!(
            "failed to create diagnostics out_dir (required for --launch): {} ({e})",
            out_dir.display()
        )
    })?;

    for (key, value) in launch_env {
        if tool_launch_env_key_is_reserved(key) {
            return Err(format!(
                "--env cannot override reserved diagnostics var in no-diagnostics mode: {key}"
            ));
        }
        cmd.env(key, value);
    }

    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR").filter(|v| !v.is_empty()) {
        cmd.env("CARGO_TARGET_DIR", target_dir);
    }

    let launched_unix_ms = now_unix_ms();
    let launched_instant = Instant::now();
    let child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn `{}`: {e}", launch.join(" ")))?;
    let pid = child.id();

    let _ = std::fs::write(out_dir.join("launched.pid"), pid.to_string());
    let _ = std::fs::write(
        out_dir.join("launched.demo.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": 1,
            "pid": pid,
            "launched_unix_ms": launched_unix_ms,
            "launch_cmd": launch,
            "mode": "external_no_diagnostics",
        }))
        .unwrap_or_else(|_| b"{}".to_vec()),
    );

    let demo = LaunchedDemo {
        child,
        launched_unix_ms,
        launched_instant,
        launch_cmd: launch.clone(),
        #[cfg(not(windows))]
        process_footprint_sampler: Some(ProcessFootprintSamplerHandle::spawn(pid)),
    };

    Ok(Some(demo))
}

#[cfg(test)]
mod tool_launch_config_tests {
    use std::path::PathBuf;

    use crate::transport::FsDiagTransportConfig;

    use super::tool_launched_diag_config;

    #[test]
    fn tool_launch_config_defaults_are_small_by_default() {
        let out_dir = PathBuf::from("target/fret-diag/test-launch-config");
        let fs_transport_cfg = FsDiagTransportConfig::from_out_dir(out_dir.clone());
        let ready_path = PathBuf::from("target/fret-diag/test-launch-config/ready.touch");
        let exit_path = PathBuf::from("target/fret-diag/test-launch-config/exit.touch");

        let cfg = tool_launched_diag_config(
            &out_dir,
            &fs_transport_cfg,
            &ready_path,
            &exit_path,
            true,
            false,
            &[("FRET_DIAG_REDACT_TEXT".to_string(), "0".to_string())],
        );

        assert_eq!(cfg.schema_version, 1);
        assert_eq!(cfg.enabled, Some(true));
        assert_eq!(cfg.allow_script_schema_v1, Some(false));
        assert_eq!(cfg.screenshots_enabled, Some(true));
        assert_eq!(cfg.write_bundle_json, Some(false));
        assert_eq!(cfg.write_bundle_schema2, Some(true));
        assert_eq!(cfg.script_dump_max_snapshots, Some(10));
        assert_eq!(cfg.script_auto_dump, Some(false));
        assert_eq!(cfg.pick_auto_dump, Some(false));
        assert_eq!(cfg.max_debug_string_bytes, Some(2048));
        assert_eq!(
            cfg.isolate_external_pointer_input_while_script_running,
            Some(true)
        );
        assert_eq!(
            cfg.isolate_external_keyboard_input_while_script_running,
            Some(true)
        );
        assert_eq!(cfg.redact_text, Some(false));
    }

    #[test]
    fn tool_launch_config_allows_raw_bundle_json_when_requested() {
        let out_dir = PathBuf::from("target/fret-diag/test-launch-config-raw");
        let fs_transport_cfg = FsDiagTransportConfig::from_out_dir(out_dir.clone());
        let ready_path = PathBuf::from("target/fret-diag/test-launch-config-raw/ready.touch");
        let exit_path = PathBuf::from("target/fret-diag/test-launch-config-raw/exit.touch");

        let cfg = tool_launched_diag_config(
            &out_dir,
            &fs_transport_cfg,
            &ready_path,
            &exit_path,
            false,
            true,
            &[],
        );

        assert_eq!(cfg.write_bundle_json, Some(true));
        assert_eq!(cfg.write_bundle_schema2, Some(true));
    }
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
            sys.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), true, refresh_kind);

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
            true,
            self.refresh_kind,
        );
        self.refresh_count = self.refresh_count.saturating_add(1);
        self.last_refresh = std::time::Instant::now();
        self.capture_latest();
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

#[cfg(not(windows))]
#[derive(Debug)]
pub(crate) struct ProcessFootprintSamplerHandle {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    rx: std::sync::mpsc::Receiver<ObservedProcessFootprint>,
    join: Option<std::thread::JoinHandle<()>>,
}

#[cfg(not(windows))]
impl ProcessFootprintSamplerHandle {
    fn spawn(pid: u32) -> Self {
        use std::sync::atomic::Ordering;

        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (tx, rx) = std::sync::mpsc::channel::<ObservedProcessFootprint>();
        let stop_for_thread = stop.clone();

        let join = std::thread::spawn(move || {
            let mut sampler = SysinfoProcessFootprintSampler::new(pid);
            let interval = sysinfo::MINIMUM_CPU_UPDATE_INTERVAL;

            loop {
                if stop_for_thread.load(Ordering::Relaxed) {
                    break;
                }

                std::thread::sleep(interval);
                sampler.refresh_force();

                // Stop once the process disappears from sysinfo.
                if sampler.sys.process(sampler.pid).is_none() {
                    break;
                }
            }

            let _ = tx.send(sampler.finish());
        });

        Self {
            stop,
            rx,
            join: Some(join),
        }
    }

    fn finish_best_effort(mut self) -> Option<ObservedProcessFootprint> {
        use std::sync::atomic::Ordering;

        self.stop.store(true, Ordering::Relaxed);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        self.rx.try_recv().ok()
    }
}

#[cfg(not(windows))]
impl Drop for ProcessFootprintSamplerHandle {
    fn drop(&mut self) {
        use std::sync::atomic::Ordering;

        self.stop.store(true, Ordering::Relaxed);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        let _ = self.rx.try_recv();
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

    if demo
        .launch_cmd
        .first()
        .is_some_and(|exe| exe.trim().eq_ignore_ascii_case("cargo"))
        && let Some(obj) = out.as_object_mut()
    {
        obj.insert(
            "notes".to_string(),
            serde_json::json!([
                "process_footprint tracks the launched PID. If you use `--launch -- cargo run ...`, this samples the cargo process (and can include compilation time). For app-only footprint, build first and launch the binary directly (e.g. `--launch -- target/release/<bin>`)."
            ]),
        );
    }

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

pub(crate) fn stop_launched_demo(
    child: &mut Option<LaunchedDemo>,
    exit_path: &Path,
    poll_ms: u64,
) -> Option<serde_json::Value> {
    let demo = child.as_mut()?;
    let out_dir = exit_path.parent().unwrap_or_else(|| Path::new("."));
    let footprint_path = out_dir.join("resource.footprint.json");

    #[cfg(target_os = "macos")]
    let macos_vmmap_steady = crate::macos_vmmap::collect_macos_vmmap_summary_best_effort(
        demo.child.id(),
        out_dir,
        "resource.vmmap_summary.steady.txt",
    );
    #[cfg(target_os = "macos")]
    let macos_footprint_tool_steady =
        crate::macos_footprint_tool::collect_macos_footprint_tool_best_effort(
            demo.child.id(),
            out_dir,
            "resource.macos_footprint.steady.json",
        );
    #[cfg(target_os = "macos")]
    let macos_vmmap_regions_sorted_steady =
        crate::macos_vmmap::collect_macos_vmmap_regions_sorted_best_effort(
            demo.child.id(),
            out_dir,
            "resource.vmmap_regions_sorted.steady.txt",
            18,
        );

    let _ = touch(exit_path);

    #[cfg(target_os = "macos")]
    let macos_vmmap = crate::macos_vmmap::collect_macos_vmmap_summary_best_effort(
        demo.child.id(),
        out_dir,
        "resource.vmmap_summary.txt",
    );

    let deadline = Instant::now() + Duration::from_millis(20_000);
    while Instant::now() < deadline {
        let exited = demo.child.try_wait().ok().flatten().is_some();
        if exited {
            #[cfg(not(windows))]
            let observed = demo
                .process_footprint_sampler
                .take()
                .and_then(|h| h.finish_best_effort());
            let mut footprint = collect_demo_footprint_json(demo, false, {
                #[cfg(not(windows))]
                {
                    observed
                }
                #[cfg(windows)]
                {
                    None
                }
            });

            #[cfg(target_os = "macos")]
            if let Some(v) = macos_vmmap.as_ref()
                && let Some(obj) = footprint.as_object_mut()
            {
                obj.insert("macos_vmmap".to_string(), v.clone());
            }
            #[cfg(target_os = "macos")]
            if let Some(v) = macos_vmmap_steady.as_ref()
                && let Some(obj) = footprint.as_object_mut()
            {
                obj.insert("macos_vmmap_steady".to_string(), v.clone());
            }
            #[cfg(target_os = "macos")]
            if let Some(v) = macos_footprint_tool_steady.as_ref()
                && let Some(obj) = footprint.as_object_mut()
            {
                obj.insert("macos_footprint_tool_steady".to_string(), v.clone());
            }
            #[cfg(target_os = "macos")]
            if let Some(v) = macos_vmmap_regions_sorted_steady.as_ref()
                && let Some(obj) = footprint.as_object_mut()
            {
                obj.insert("macos_vmmap_regions_sorted_steady".to_string(), v.clone());
            }

            let footprint = Some(footprint);
            if let Some(footprint) = &footprint {
                let _ = write_json_value(&footprint_path, footprint);
            }
            if let Some(mut c) = child.take().map(|d| d.child) {
                let _ = c.wait();
            }
            return footprint;
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(10)));
    }

    #[cfg(not(windows))]
    let observed = demo
        .process_footprint_sampler
        .take()
        .and_then(|h| h.finish_best_effort());
    let mut footprint = collect_demo_footprint_json(demo, true, {
        #[cfg(not(windows))]
        {
            observed
        }
        #[cfg(windows)]
        {
            None
        }
    });

    #[cfg(target_os = "macos")]
    if let Some(v) = macos_vmmap.as_ref()
        && let Some(obj) = footprint.as_object_mut()
    {
        obj.insert("macos_vmmap".to_string(), v.clone());
    }
    #[cfg(target_os = "macos")]
    if let Some(v) = macos_vmmap_steady.as_ref()
        && let Some(obj) = footprint.as_object_mut()
    {
        obj.insert("macos_vmmap_steady".to_string(), v.clone());
    }
    #[cfg(target_os = "macos")]
    if let Some(v) = macos_footprint_tool_steady.as_ref()
        && let Some(obj) = footprint.as_object_mut()
    {
        obj.insert("macos_footprint_tool_steady".to_string(), v.clone());
    }
    #[cfg(target_os = "macos")]
    if let Some(v) = macos_vmmap_regions_sorted_steady.as_ref()
        && let Some(obj) = footprint.as_object_mut()
    {
        obj.insert("macos_vmmap_regions_sorted_steady".to_string(), v.clone());
    }

    let footprint = Some(footprint);
    if let Some(footprint) = &footprint {
        let _ = write_json_value(&footprint_path, footprint);
    }
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
        if (cmd[i] == "--features" || cmd[i] == "-F")
            && let Some(value) = cmd.get_mut(i + 1)
        {
            let mut features: Vec<&str> = value
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if features.contains(&feature) {
                return false;
            }
            features.push(feature);
            *value = features.join(",");
            return true;
        }
        if let Some(rest) = cmd[i].strip_prefix("--features=") {
            let mut features: Vec<&str> = rest
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if features.contains(&feature) {
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
            if features.contains(&feature) {
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

#[allow(clippy::too_many_arguments)]
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
    pub(super) max_frame_p95_total_us: Option<u64>,
    pub(super) max_frame_p95_layout_us: Option<u64>,
    pub(super) max_frame_p95_solve_us: Option<u64>,
    pub(super) max_pointer_move_dispatch_us: Option<u64>,
    pub(super) max_pointer_move_hit_test_us: Option<u64>,
    pub(super) max_pointer_move_global_changes: Option<u64>,
    pub(super) min_run_paint_cache_hit_test_only_replay_allowed_max: Option<u64>,
    pub(super) max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: Option<u64>,
    pub(super) max_renderer_encode_scene_us: Option<u64>,
    pub(super) max_renderer_upload_us: Option<u64>,
    pub(super) max_renderer_record_passes_us: Option<u64>,
    pub(super) max_renderer_encoder_finish_us: Option<u64>,
    pub(super) max_renderer_prepare_text_us: Option<u64>,
    pub(super) max_renderer_prepare_svg_us: Option<u64>,
}

impl PerfThresholds {
    pub(super) fn any(self) -> bool {
        self.max_top_total_us.is_some()
            || self.max_top_layout_us.is_some()
            || self.max_top_solve_us.is_some()
            || self.max_frame_p95_total_us.is_some()
            || self.max_frame_p95_layout_us.is_some()
            || self.max_frame_p95_solve_us.is_some()
            || self.max_pointer_move_dispatch_us.is_some()
            || self.max_pointer_move_hit_test_us.is_some()
            || self.max_pointer_move_global_changes.is_some()
            || self
                .min_run_paint_cache_hit_test_only_replay_allowed_max
                .is_some()
            || self
                .max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max
                .is_some()
            || self.max_renderer_encode_scene_us.is_some()
            || self.max_renderer_upload_us.is_some()
            || self.max_renderer_record_passes_us.is_some()
            || self.max_renderer_encoder_finish_us.is_some()
            || self.max_renderer_prepare_text_us.is_some()
            || self.max_renderer_prepare_svg_us.is_some()
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

    if let Ok(meta) = std::fs::metadata(&resolved)
        && meta.is_dir()
    {
        return Err(format!(
            "invalid --perf-baseline path (expected a JSON file, got a directory): {}",
            resolved.display()
        ));
    }

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
            max_frame_p95_total_us: t
                .and_then(|m| m.get("max_frame_p95_total_us"))
                .and_then(|v| v.as_u64()),
            max_frame_p95_layout_us: t
                .and_then(|m| m.get("max_frame_p95_layout_us"))
                .and_then(|v| v.as_u64()),
            max_frame_p95_solve_us: t
                .and_then(|m| m.get("max_frame_p95_solve_us"))
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
            min_run_paint_cache_hit_test_only_replay_allowed_max: t
                .and_then(|m| m.get("min_run_paint_cache_hit_test_only_replay_allowed_max"))
                .and_then(|v| v.as_u64()),
            max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: t
                .and_then(|m| {
                    m.get("max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max")
                })
                .and_then(|v| v.as_u64()),
            max_renderer_encode_scene_us: t
                .and_then(|m| m.get("max_renderer_encode_scene_us"))
                .and_then(|v| v.as_u64()),
            max_renderer_upload_us: t
                .and_then(|m| m.get("max_renderer_upload_us"))
                .and_then(|v| v.as_u64()),
            max_renderer_record_passes_us: t
                .and_then(|m| m.get("max_renderer_record_passes_us"))
                .and_then(|v| v.as_u64()),
            max_renderer_encoder_finish_us: t
                .and_then(|m| m.get("max_renderer_encoder_finish_us"))
                .and_then(|v| v.as_u64()),
            max_renderer_prepare_text_us: t
                .and_then(|m| m.get("max_renderer_prepare_text_us"))
                .and_then(|v| v.as_u64()),
            max_renderer_prepare_svg_us: t
                .and_then(|m| m.get("max_renderer_prepare_svg_us"))
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

pub(super) fn apply_perf_baseline_floor(value: u64, headroom_pct: u32) -> u64 {
    if value == 0 {
        return 0;
    }
    let pct = (headroom_pct as u64).min(95);
    let floored = value.saturating_mul(100 - pct) / 100;
    floored.max(1)
}

#[cfg(test)]
pub(super) fn apply_perf_baseline_headroom_with_slack_and_quantum(
    value_us: u64,
    headroom_pct: u32,
    min_slack_us: u64,
    quantum_us: u64,
) -> u64 {
    if value_us == 0 {
        return 0;
    }

    let threshold = apply_perf_baseline_headroom(value_us, headroom_pct)
        .max(value_us.saturating_add(min_slack_us));
    if quantum_us <= 1 {
        return threshold;
    }
    threshold.saturating_add(quantum_us - 1) / quantum_us * quantum_us
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

#[cfg(test)]
mod tests {
    use super::{apply_perf_baseline_floor, apply_perf_baseline_headroom_with_slack_and_quantum};

    #[test]
    fn perf_baseline_headroom_with_slack_and_quantum_is_stable_for_small_values() {
        // Baseline "26us @ +20%" would yield 32us without slack; we want extra headroom to avoid
        // 1–2us jitter causing flaky gates.
        assert_eq!(
            apply_perf_baseline_headroom_with_slack_and_quantum(26, 20, 8, 4),
            36
        );

        // Quantum rounding is a no-op for quantum<=1.
        assert_eq!(
            apply_perf_baseline_headroom_with_slack_and_quantum(26, 20, 8, 1),
            34
        );

        // Zero stays zero so scripts without pointer-move frames do not accidentally gain a gate.
        assert_eq!(
            apply_perf_baseline_headroom_with_slack_and_quantum(0, 20, 8, 4),
            0
        );
    }

    #[test]
    fn perf_baseline_floor_preserves_non_zero_signal() {
        assert_eq!(apply_perf_baseline_floor(17, 20), 13);
        assert_eq!(apply_perf_baseline_floor(1, 20), 1);
        assert_eq!(apply_perf_baseline_floor(0, 20), 0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PerfThresholdAggregate {
    Max,
    P90,
    P95,
}

impl PerfThresholdAggregate {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            PerfThresholdAggregate::Max => "max",
            PerfThresholdAggregate::P90 => "p90",
            PerfThresholdAggregate::P95 => "p95",
        }
    }
}

impl std::str::FromStr for PerfThresholdAggregate {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "max" => Ok(PerfThresholdAggregate::Max),
            "p90" => Ok(PerfThresholdAggregate::P90),
            "p95" => Ok(PerfThresholdAggregate::P95),
            _ => Err(format!("invalid aggregate (expected max|p90|p95): {s:?}")),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn scan_perf_threshold_failures(
    script: &str,
    sort: BundleStatsSort,
    observed_agg: PerfThresholdAggregate,
    cli: PerfThresholds,
    baseline: PerfThresholds,
    observed_total_time_us: u64,
    max_total_time_us: u64,
    p95_total_time_us: u64,
    observed_layout_time_us: u64,
    max_layout_time_us: u64,
    p95_layout_time_us: u64,
    observed_layout_engine_solve_time_us: u64,
    max_layout_engine_solve_time_us: u64,
    p95_layout_engine_solve_time_us: u64,
    observed_frame_p95_total_time_us: u64,
    max_frame_p95_total_time_us: u64,
    p95_frame_p95_total_time_us: u64,
    observed_frame_p95_layout_time_us: u64,
    max_frame_p95_layout_time_us: u64,
    p95_frame_p95_layout_time_us: u64,
    observed_frame_p95_layout_engine_solve_time_us: u64,
    max_frame_p95_layout_engine_solve_time_us: u64,
    p95_frame_p95_layout_engine_solve_time_us: u64,
    pointer_move_frames_present: bool,
    max_pointer_move_dispatch_time_us: u64,
    max_pointer_move_hit_test_time_us: u64,
    max_pointer_move_global_changes: u64,
    max_run_paint_cache_hit_test_only_replay_allowed: u64,
    max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch: u64,
    observed_renderer_encode_scene_us: u64,
    max_renderer_encode_scene_us: u64,
    p95_renderer_encode_scene_us: u64,
    observed_renderer_upload_us: u64,
    max_renderer_upload_us: u64,
    p95_renderer_upload_us: u64,
    observed_renderer_record_passes_us: u64,
    max_renderer_record_passes_us: u64,
    p95_renderer_record_passes_us: u64,
    observed_renderer_encoder_finish_us: u64,
    max_renderer_encoder_finish_us: u64,
    p95_renderer_encoder_finish_us: u64,
    observed_renderer_prepare_text_us: u64,
    max_renderer_prepare_text_us: u64,
    p95_renderer_prepare_text_us: u64,
    observed_renderer_prepare_svg_us: u64,
    max_renderer_prepare_svg_us: u64,
    p95_renderer_prepare_svg_us: u64,
    evidence_bundle_total: Option<&Path>,
    evidence_run_index_total: Option<u64>,
    evidence_bundle_layout: Option<&Path>,
    evidence_run_index_layout: Option<u64>,
    evidence_bundle_solve: Option<&Path>,
    evidence_run_index_solve: Option<u64>,
) -> Vec<serde_json::Value> {
    let mut out: Vec<serde_json::Value> = Vec::new();
    let evidence_bundle_total = evidence_bundle_total.map(|p| p.display().to_string());
    let evidence_bundle_layout = evidence_bundle_layout
        .map(|p| p.display().to_string())
        .or_else(|| evidence_bundle_total.clone());
    let evidence_bundle_solve = evidence_bundle_solve
        .map(|p| p.display().to_string())
        .or_else(|| evidence_bundle_total.clone());
    let evidence_bundle = evidence_bundle_total.clone();
    let evidence_run_index_layout = evidence_run_index_layout.or(evidence_run_index_total);
    let evidence_run_index_solve = evidence_run_index_solve.or(evidence_run_index_total);
    let evidence_run_index = evidence_run_index_total;
    let (threshold_total, source_total) =
        resolve_threshold(cli.max_top_total_us, baseline.max_top_total_us);
    let (threshold_layout, source_layout) =
        resolve_threshold(cli.max_top_layout_us, baseline.max_top_layout_us);
    let (threshold_solve, source_solve) =
        resolve_threshold(cli.max_top_solve_us, baseline.max_top_solve_us);
    let (threshold_frame_p95_total, source_frame_p95_total) =
        resolve_threshold(cli.max_frame_p95_total_us, baseline.max_frame_p95_total_us);
    let (threshold_frame_p95_layout, source_frame_p95_layout) = resolve_threshold(
        cli.max_frame_p95_layout_us,
        baseline.max_frame_p95_layout_us,
    );
    let (threshold_frame_p95_solve, source_frame_p95_solve) =
        resolve_threshold(cli.max_frame_p95_solve_us, baseline.max_frame_p95_solve_us);
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
    let (
        threshold_min_run_paint_cache_hit_test_only_replay_allowed,
        source_min_run_paint_cache_hit_test_only_replay_allowed,
    ) = resolve_threshold(
        cli.min_run_paint_cache_hit_test_only_replay_allowed_max,
        baseline.min_run_paint_cache_hit_test_only_replay_allowed_max,
    );
    let (
        threshold_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch,
        source_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch,
    ) = resolve_threshold(
        cli.max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
        baseline.max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max,
    );
    let (threshold_renderer_encode_scene, source_renderer_encode_scene) = resolve_threshold(
        cli.max_renderer_encode_scene_us,
        baseline.max_renderer_encode_scene_us,
    );
    let (threshold_renderer_upload, source_renderer_upload) =
        resolve_threshold(cli.max_renderer_upload_us, baseline.max_renderer_upload_us);
    let (threshold_renderer_record_passes, source_renderer_record_passes) = resolve_threshold(
        cli.max_renderer_record_passes_us,
        baseline.max_renderer_record_passes_us,
    );
    let (threshold_renderer_encoder_finish, source_renderer_encoder_finish) = resolve_threshold(
        cli.max_renderer_encoder_finish_us,
        baseline.max_renderer_encoder_finish_us,
    );
    let (threshold_renderer_prepare_text, source_renderer_prepare_text) = resolve_threshold(
        cli.max_renderer_prepare_text_us,
        baseline.max_renderer_prepare_text_us,
    );
    let (threshold_renderer_prepare_svg, source_renderer_prepare_svg) = resolve_threshold(
        cli.max_renderer_prepare_svg_us,
        baseline.max_renderer_prepare_svg_us,
    );

    if let Some(threshold_us) = threshold_total
        && observed_total_time_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "top_total_time_us",
            "threshold_us": threshold_us,
            "threshold_source": source_total,
            "actual_us": observed_total_time_us,
            "actual_aggregate": observed_agg.as_str(),
            "actual_max_us": max_total_time_us,
            "actual_p95_us": p95_total_time_us,
            "outlier_suspected": p95_total_time_us <= threshold_us,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle_total,
            "evidence_run_index": evidence_run_index_total,
        }));
    }
    if let Some(threshold_us) = threshold_layout
        && observed_layout_time_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "top_layout_time_us",
            "threshold_us": threshold_us,
            "threshold_source": source_layout,
            "actual_us": observed_layout_time_us,
            "actual_aggregate": observed_agg.as_str(),
            "actual_max_us": max_layout_time_us,
            "actual_p95_us": p95_layout_time_us,
            "outlier_suspected": p95_layout_time_us <= threshold_us,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle_layout,
            "evidence_run_index": evidence_run_index_layout,
        }));
    }
    if let Some(threshold_us) = threshold_solve
        && observed_layout_engine_solve_time_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "top_layout_engine_solve_time_us",
            "threshold_us": threshold_us,
            "threshold_source": source_solve,
            "actual_us": observed_layout_engine_solve_time_us,
            "actual_aggregate": observed_agg.as_str(),
            "actual_max_us": max_layout_engine_solve_time_us,
            "actual_p95_us": p95_layout_engine_solve_time_us,
            "outlier_suspected": p95_layout_engine_solve_time_us <= threshold_us,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle_solve,
            "evidence_run_index": evidence_run_index_solve,
        }));
    }
    const FRAME_P95_TOTAL_EPS_US: u64 = 100;
    const FRAME_P95_LAYOUT_EPS_US: u64 = 100;
    const FRAME_P95_SOLVE_EPS_US: u64 = 8;

    if let Some(threshold_us) = threshold_frame_p95_total {
        let threshold_effective_us = threshold_us.saturating_add(FRAME_P95_TOTAL_EPS_US);
        if observed_frame_p95_total_time_us > threshold_effective_us {
            out.push(serde_json::json!({
                "metric": "frame_p95_total_time_us",
                "threshold_us": threshold_us,
                "threshold_effective_us": threshold_effective_us,
                "threshold_eps_us": FRAME_P95_TOTAL_EPS_US,
                "threshold_source": source_frame_p95_total,
                "actual_us": observed_frame_p95_total_time_us,
                "actual_aggregate": observed_agg.as_str(),
                "actual_max_us": max_frame_p95_total_time_us,
                "actual_p95_us": p95_frame_p95_total_time_us,
                "outlier_suspected": p95_frame_p95_total_time_us <= threshold_effective_us,
                "script": script,
                "sort": sort.as_str(),
                "evidence_bundle": evidence_bundle,
                "evidence_run_index": evidence_run_index,
            }));
        }
    }
    if let Some(threshold_us) = threshold_frame_p95_layout {
        let threshold_effective_us = threshold_us.saturating_add(FRAME_P95_LAYOUT_EPS_US);
        if observed_frame_p95_layout_time_us > threshold_effective_us {
            out.push(serde_json::json!({
                "metric": "frame_p95_layout_time_us",
                "threshold_us": threshold_us,
                "threshold_effective_us": threshold_effective_us,
                "threshold_eps_us": FRAME_P95_LAYOUT_EPS_US,
                "threshold_source": source_frame_p95_layout,
                "actual_us": observed_frame_p95_layout_time_us,
                "actual_aggregate": observed_agg.as_str(),
                "actual_max_us": max_frame_p95_layout_time_us,
                "actual_p95_us": p95_frame_p95_layout_time_us,
                "outlier_suspected": p95_frame_p95_layout_time_us <= threshold_effective_us,
                "script": script,
                "sort": sort.as_str(),
                "evidence_bundle": evidence_bundle,
                "evidence_run_index": evidence_run_index,
            }));
        }
    }
    if let Some(threshold_us) = threshold_frame_p95_solve {
        let threshold_effective_us = threshold_us.saturating_add(FRAME_P95_SOLVE_EPS_US);
        if observed_frame_p95_layout_engine_solve_time_us > threshold_effective_us {
            out.push(serde_json::json!({
            "metric": "frame_p95_layout_engine_solve_time_us",
            "threshold_us": threshold_us,
            "threshold_effective_us": threshold_effective_us,
            "threshold_eps_us": FRAME_P95_SOLVE_EPS_US,
            "threshold_source": source_frame_p95_solve,
            "actual_us": observed_frame_p95_layout_engine_solve_time_us,
            "actual_aggregate": observed_agg.as_str(),
            "actual_max_us": max_frame_p95_layout_engine_solve_time_us,
            "actual_p95_us": p95_frame_p95_layout_engine_solve_time_us,
            "outlier_suspected": p95_frame_p95_layout_engine_solve_time_us <= threshold_effective_us,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle,
            "evidence_run_index": evidence_run_index,
        }));
        }
    }
    if pointer_move_frames_present {
        if let Some(threshold_us) = threshold_pointer_move_dispatch
            && threshold_us > 0
            && max_pointer_move_dispatch_time_us > threshold_us
        {
            out.push(serde_json::json!({
                "metric": "pointer_move_max_dispatch_time_us",
                "threshold_us": threshold_us,
                "threshold_source": source_pointer_move_dispatch,
                "actual_us": max_pointer_move_dispatch_time_us,
                "script": script,
                "sort": sort.as_str(),
                "evidence_bundle": evidence_bundle,
                "evidence_run_index": evidence_run_index,
            }));
        }
        if let Some(threshold_us) = threshold_pointer_move_hit_test
            && threshold_us > 0
            && max_pointer_move_hit_test_time_us > threshold_us
        {
            out.push(serde_json::json!({
                "metric": "pointer_move_max_hit_test_time_us",
                "threshold_us": threshold_us,
                "threshold_source": source_pointer_move_hit_test,
                "actual_us": max_pointer_move_hit_test_time_us,
                "script": script,
                "sort": sort.as_str(),
                "evidence_bundle": evidence_bundle,
                "evidence_run_index": evidence_run_index,
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
                "evidence_bundle": evidence_bundle,
                "evidence_run_index": evidence_run_index,
            }));
        }
    }

    if let Some(threshold_us) = threshold_renderer_encode_scene
        && observed_renderer_encode_scene_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "renderer_encode_scene_us",
            "threshold_us": threshold_us,
            "threshold_source": source_renderer_encode_scene,
            "actual_us": observed_renderer_encode_scene_us,
            "actual_aggregate": observed_agg.as_str(),
            "actual_max_us": max_renderer_encode_scene_us,
            "actual_p95_us": p95_renderer_encode_scene_us,
            "outlier_suspected": p95_renderer_encode_scene_us <= threshold_us,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle.clone(),
            "evidence_run_index": evidence_run_index,
        }));
    }
    if let Some(threshold_us) = threshold_renderer_upload
        && observed_renderer_upload_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "renderer_upload_us",
            "threshold_us": threshold_us,
            "threshold_source": source_renderer_upload,
            "actual_us": observed_renderer_upload_us,
            "actual_aggregate": observed_agg.as_str(),
            "actual_max_us": max_renderer_upload_us,
            "actual_p95_us": p95_renderer_upload_us,
            "outlier_suspected": p95_renderer_upload_us <= threshold_us,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle.clone(),
            "evidence_run_index": evidence_run_index,
        }));
    }
    if let Some(threshold_us) = threshold_renderer_record_passes
        && observed_renderer_record_passes_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "renderer_record_passes_us",
            "threshold_us": threshold_us,
            "threshold_source": source_renderer_record_passes,
            "actual_us": observed_renderer_record_passes_us,
            "actual_aggregate": observed_agg.as_str(),
            "actual_max_us": max_renderer_record_passes_us,
            "actual_p95_us": p95_renderer_record_passes_us,
            "outlier_suspected": p95_renderer_record_passes_us <= threshold_us,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle.clone(),
            "evidence_run_index": evidence_run_index,
        }));
    }
    if let Some(threshold_us) = threshold_renderer_encoder_finish
        && observed_renderer_encoder_finish_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "renderer_encoder_finish_us",
            "threshold_us": threshold_us,
            "threshold_source": source_renderer_encoder_finish,
            "actual_us": observed_renderer_encoder_finish_us,
            "actual_aggregate": observed_agg.as_str(),
            "actual_max_us": max_renderer_encoder_finish_us,
            "actual_p95_us": p95_renderer_encoder_finish_us,
            "outlier_suspected": p95_renderer_encoder_finish_us <= threshold_us,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle.clone(),
            "evidence_run_index": evidence_run_index,
        }));
    }
    if let Some(threshold_us) = threshold_renderer_prepare_text
        && observed_renderer_prepare_text_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "renderer_prepare_text_us",
            "threshold_us": threshold_us,
            "threshold_source": source_renderer_prepare_text,
            "actual_us": observed_renderer_prepare_text_us,
            "actual_aggregate": observed_agg.as_str(),
            "actual_max_us": max_renderer_prepare_text_us,
            "actual_p95_us": p95_renderer_prepare_text_us,
            "outlier_suspected": p95_renderer_prepare_text_us <= threshold_us,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle.clone(),
            "evidence_run_index": evidence_run_index,
        }));
    }
    if let Some(threshold_us) = threshold_renderer_prepare_svg
        && observed_renderer_prepare_svg_us > threshold_us
    {
        out.push(serde_json::json!({
            "metric": "renderer_prepare_svg_us",
            "threshold_us": threshold_us,
            "threshold_source": source_renderer_prepare_svg,
            "actual_us": observed_renderer_prepare_svg_us,
            "actual_aggregate": observed_agg.as_str(),
            "actual_max_us": max_renderer_prepare_svg_us,
            "actual_p95_us": p95_renderer_prepare_svg_us,
            "outlier_suspected": p95_renderer_prepare_svg_us <= threshold_us,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle.clone(),
            "evidence_run_index": evidence_run_index,
        }));
    }

    if let Some(threshold) = threshold_min_run_paint_cache_hit_test_only_replay_allowed
        && max_run_paint_cache_hit_test_only_replay_allowed < threshold
    {
        out.push(serde_json::json!({
            "metric": "run_paint_cache_hit_test_only_replay_allowed_max",
            "threshold_min": threshold,
            "threshold_source": source_min_run_paint_cache_hit_test_only_replay_allowed,
            "actual": max_run_paint_cache_hit_test_only_replay_allowed,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle,
            "evidence_run_index": evidence_run_index,
        }));
    }
    if let Some(threshold) =
        threshold_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch
        && max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch > threshold
    {
        out.push(serde_json::json!({
            "metric": "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max",
            "threshold": threshold,
            "threshold_source": source_max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch,
            "actual": max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch,
            "script": script,
            "sort": sort.as_str(),
            "evidence_bundle": evidence_bundle,
            "evidence_run_index": evidence_run_index,
        }));
    }
    out
}

// (moved to `diag::util`)
