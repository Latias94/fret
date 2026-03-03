use std::path::Path;

use super::args;

pub(crate) fn cmd_layout_perf_summary(
    rest: &[String],
    resolved_out_dir: &Path,
    workspace_root: &Path,
    warmup_frames: u64,
    stats_json: bool,
    out: Option<&Path>,
) -> Result<(), String> {
    let mut top: usize = crate::layout_perf_summary::DEFAULT_LAYOUT_PERF_SUMMARY_TOP;
    let mut target: Option<String> = None;

    let mut i: usize = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        match arg {
            "--top" => {
                i += 1;
                let Some(v) = rest.get(i).cloned() else {
                    return Err("missing value for --top".to_string());
                };
                top = v
                    .parse::<usize>()
                    .map_err(|e| format!("invalid value for --top: {e}"))?;
                if top == 0 {
                    return Err("--top must be >= 1".to_string());
                }
                i += 1;
            }
            "--help" | "-h" => {
                return Err(
                    "usage: fretboard diag layout-perf-summary [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] [--top <n>] [--warmup-frames <n>] [--json] [--out <path>]".to_string(),
                );
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown diag layout-perf-summary flag: {other}"));
            }
            _ => {
                if target.is_some() {
                    return Err(format!("unexpected arguments: {}", rest[i..].join(" ")));
                }
                target = Some(rest[i].clone());
                i += 1;
            }
        }
    }

    let bundle_path = args::resolve_bundle_artifact_path_or_latest(
        target.as_deref(),
        workspace_root,
        resolved_out_dir,
    )?;
    let bundle_dir = crate::resolve_bundle_root_dir(&bundle_path)?;

    let summary = crate::layout_perf_summary::layout_perf_summary_v1_from_bundle_path_strict(
        &bundle_path,
        &bundle_dir,
        warmup_frames,
        top,
    )?;

    let output_bytes: Vec<u8> = if stats_json {
        serde_json::to_vec_pretty(&summary).map_err(|e| e.to_string())?
    } else {
        crate::layout_perf_summary::human_layout_perf_summary_v1(&summary).into_bytes()
    };

    if let Some(out) = out {
        let out = crate::resolve_path(workspace_root, out.to_path_buf());
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&out, output_bytes).map_err(|e| e.to_string())?;
        return Ok(());
    }

    print!("{}", String::from_utf8_lossy(&output_bytes));
    Ok(())
}
