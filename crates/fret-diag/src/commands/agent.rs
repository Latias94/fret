use std::path::{Path, PathBuf};

use serde_json::json;

fn looks_like_path(s: &str) -> bool {
    s.contains('/') || s.contains('\\') || s.ends_with(".json")
}

fn resolve_bundle_artifact_path_or_latest(
    bundle_arg: Option<&str>,
    workspace_root: &Path,
    out_dir: &Path,
) -> Result<PathBuf, String> {
    if let Some(s) = bundle_arg {
        let src = crate::resolve_path(workspace_root, PathBuf::from(s));
        return Ok(crate::resolve_bundle_artifact_path(&src));
    }
    let latest = crate::read_latest_pointer(out_dir)
        .or_else(|| crate::find_latest_export_dir(out_dir))
        .ok_or_else(|| format!("no diagnostics bundle found under {}", out_dir.display()))?;
    Ok(crate::resolve_bundle_artifact_path(&latest))
}

pub(crate) fn cmd_agent(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let mut bundle_arg: Option<String> = None;
    let mut out: Option<PathBuf> = None;

    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--out" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --out".to_string());
                };
                out = Some(PathBuf::from(v));
                i += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for agent: {other}"));
            }
            other => {
                if bundle_arg.is_none() && looks_like_path(other) {
                    bundle_arg = Some(other.to_string());
                } else if bundle_arg.is_none() {
                    let p = crate::resolve_path(workspace_root, PathBuf::from(other));
                    if p.is_file() || p.is_dir() {
                        bundle_arg = Some(other.to_string());
                    } else {
                        return Err(format!("unexpected argument: {other}"));
                    }
                } else {
                    return Err(format!("unexpected argument: {other}"));
                }
                i += 1;
            }
        }
    }

    let bundle_path =
        resolve_bundle_artifact_path_or_latest(bundle_arg.as_deref(), workspace_root, out_dir)?;
    let bundle_dir = crate::resolve_bundle_root_dir(&bundle_path)?;

    let out = out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| bundle_dir.join("agent.plan.json"));

    let bundle_label = bundle_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("bundle");

    let payload = json!({
        "schema_version": 1,
        "kind": "diag.agent_plan",
        "generated_unix_ms": crate::util::now_unix_ms(),
        "bundle": bundle_path.display().to_string(),
        "bundle_dir": bundle_dir.display().to_string(),
        "bundle_label": bundle_label,
        "warmup_frames": warmup_frames,
        "notes": [
            "This plan prioritizes sidecars (frames.index.json) over materializing bundle artifacts in memory.",
            "Run doctor first; it is safe to run repeatedly.",
        ],
        "steps": [
            {
                "id": "doctor_check",
                "command": format!("fretboard diag doctor --check {} --warmup-frames {}", bundle_dir.display(), warmup_frames),
                "why": "Verify that required sidecars exist and match warmup_frames.",
            },
            {
                "id": "doctor_fix",
                "command": format!("fretboard diag doctor --fix {} --warmup-frames {}", bundle_dir.display(), warmup_frames),
                "why": "Self-heal: regenerate missing/invalid sidecars (including frames.index.json).",
            },
            {
                "id": "doctor_fix_schema2_optional",
                "command": format!("fretboard diag doctor --fix-schema2 {} --warmup-frames {}", bundle_dir.display(), warmup_frames),
                "why": "Optional: write bundle.schema2.json for large bundles to keep tooling and AI loops fast.",
            },
            {
                "id": "pack_schema2_only_optional",
                "command": format!("fretboard diag pack {} --include-all --pack-schema2-only --warmup-frames {}", bundle_dir.display(), warmup_frames),
                "why": "Optional: pack a small shareable zip (schema2-only; avoids shipping a huge bundle.json).",
            },
            {
                "id": "triage_lite_total",
                "command": format!("fretboard diag triage --lite {} --warmup-frames {} --metric total", bundle_dir.display(), warmup_frames),
                "why": "First-pass perf triage (slowest frames) without materializing bundle artifacts.",
            },
            {
                "id": "triage_lite_layout",
                "command": format!("fretboard diag triage --lite {} --warmup-frames {} --metric layout", bundle_dir.display(), warmup_frames),
                "why": "Identify layout-heavy worst frames.",
            },
            {
                "id": "triage_lite_paint",
                "command": format!("fretboard diag triage --lite {} --warmup-frames {} --metric paint", bundle_dir.display(), warmup_frames),
                "why": "Identify paint-heavy worst frames.",
            },
            {
                "id": "hotspots_lite_total",
                "command": format!("fretboard diag hotspots --lite {} --warmup-frames {} --metric total", bundle_dir.display(), warmup_frames),
                "why": "Perf hotspots (slow frames) fallback when a bundle artifact is too large for JSON-size hotspots.",
            },
            {
                "id": "slice_targeted",
                "command": format!("fretboard diag slice {} --test-id <test_id> --window <window> --frame-id <frame_id> --warmup-frames {}", bundle_dir.display(), warmup_frames),
                "why": "Extract a small, shareable slice for the candidate window/frame.",
            },
            {
                "id": "ai_packet",
                "command": format!("fretboard diag ai-packet {} --warmup-frames {}", bundle_dir.display(), warmup_frames),
                "why": "Generate a compact packet for an AI agent (includes doctor + lite reports).",
            },
            {
                "id": "hotspots_json_size_optional",
                "command": format!("fretboard diag hotspots {} --force", bundle_dir.display()),
                "why": "Optional: JSON subtree size hotspots (may be expensive; avoid for very large bundles).",
            },
        ],
    });

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

    if stats_json {
        println!("{pretty}");
    } else {
        println!("{}", out.display());
    }
    Ok(())
}
