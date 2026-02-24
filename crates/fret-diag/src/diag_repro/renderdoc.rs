use super::super::*;

fn renderdoc_markers_or_default(renderdoc_markers: &[String]) -> Vec<String> {
    if renderdoc_markers.is_empty() {
        vec![
            "fret clip mask pass".to_string(),
            "fret downsample-nearest pass".to_string(),
            "fret upscale-nearest pass".to_string(),
        ]
    } else {
        renderdoc_markers.to_vec()
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn stop_demo_and_collect_renderdoc_captures(
    workspace_root: &Path,
    resolved_out_dir: &Path,
    resolved_exit_path: &Path,
    poll_ms: u64,
    child: &mut Option<LaunchedDemo>,
    with_renderdoc: bool,
    renderdoc_capture_dir: Option<&PathBuf>,
    renderdoc_autocapture_after_frames: Option<u32>,
    renderdoc_markers: &[String],
    renderdoc_no_outputs_png: bool,
) -> (Option<serde_json::Value>, Option<serde_json::Value>) {
    if !with_renderdoc {
        let repro_process_footprint = stop_launched_demo(child, resolved_exit_path, poll_ms);
        return (repro_process_footprint, None);
    }

    let Some(dir) = renderdoc_capture_dir else {
        let repro_process_footprint = stop_launched_demo(child, resolved_exit_path, poll_ms);
        return (repro_process_footprint, None);
    };

    let markers = renderdoc_markers_or_default(renderdoc_markers);
    let captures = wait_for_files_with_extensions(dir, &["rdc"], 10_000, poll_ms);

    let repro_process_footprint = stop_launched_demo(child, resolved_exit_path, poll_ms);

    let mut capture_rows: Vec<serde_json::Value> = Vec::new();
    for (cap_idx, capture) in captures.iter().enumerate() {
        let stem = capture
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("capture");
        let safe_stem = format!(
            "{:02}-{}",
            cap_idx.saturating_add(1),
            zip_safe_component(stem)
        );
        let inspect_root = dir.join("inspect").join(&safe_stem);

        let summary_dir = inspect_root.join("summary");
        let summary_attempt = run_fret_renderdoc_dump(
            workspace_root,
            capture,
            &summary_dir,
            "summary",
            "",
            Some(200_000),
            true,
            true,
            Some(30),
        );

        let mut attempts: Vec<RenderdocDumpAttempt> = Vec::new();
        attempts.push(summary_attempt);

        for (idx, marker) in markers.iter().enumerate() {
            let safe_marker = zip_safe_component(marker);
            let out_dir =
                inspect_root.join(format!("marker_{:02}_{safe_marker}", idx.saturating_add(1)));
            let attempt = run_fret_renderdoc_dump(
                workspace_root,
                capture,
                &out_dir,
                "dump",
                marker,
                Some(2_000),
                true,
                renderdoc_no_outputs_png,
                None,
            );
            attempts.push(attempt);
        }

        let attempt_rows = attempts
            .into_iter()
            .map(|a| {
                let out_dir = a
                    .out_dir
                    .strip_prefix(resolved_out_dir)
                    .unwrap_or(&a.out_dir)
                    .display()
                    .to_string();
                let stdout_file = a.stdout_file.as_ref().map(|p| {
                    p.strip_prefix(resolved_out_dir)
                        .unwrap_or(p)
                        .display()
                        .to_string()
                });
                let stderr_file = a.stderr_file.as_ref().map(|p| {
                    p.strip_prefix(resolved_out_dir)
                        .unwrap_or(p)
                        .display()
                        .to_string()
                });
                let response_json = a.response_json.as_ref().map(|p| {
                    p.strip_prefix(resolved_out_dir)
                        .unwrap_or(p)
                        .display()
                        .to_string()
                });

                serde_json::json!({
                    "marker": a.marker,
                    "out_dir": out_dir,
                    "stdout_file": stdout_file,
                    "stderr_file": stderr_file,
                    "response_json": response_json,
                    "exit_code": a.exit_code,
                    "error": a.error,
                })
            })
            .collect::<Vec<_>>();

        let capture_rel = capture
            .strip_prefix(resolved_out_dir)
            .unwrap_or(capture)
            .display()
            .to_string();
        let inspect_rel = inspect_root
            .strip_prefix(resolved_out_dir)
            .unwrap_or(&inspect_root)
            .display()
            .to_string();

        capture_rows.push(serde_json::json!({
            "capture": capture_rel,
            "inspect_dir": inspect_rel,
            "dumps": attempt_rows,
        }));
    }

    let payload = serde_json::json!({
        "schema_version": 2,
        "generated_unix_ms": now_unix_ms(),
        "capture_dir": "renderdoc",
        "autocapture_after_frames": renderdoc_autocapture_after_frames,
        "captures": capture_rows,
    });
    let _ = write_json_value(&resolved_out_dir.join("renderdoc.captures.json"), &payload);

    (repro_process_footprint, Some(payload))
}
