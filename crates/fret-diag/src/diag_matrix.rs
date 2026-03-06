use super::*;

use crate::regression_summary::{
    RegressionArtifactsV1, RegressionCampaignSummaryV1, RegressionEvidenceV1,
    RegressionHighlightsV1, RegressionItemKindV1, RegressionItemSummaryV1, RegressionLaneV1,
    RegressionNotesV1, RegressionRunSummaryV1, RegressionSourceV1, RegressionStatusV1,
    RegressionSummaryV1, RegressionTotalsV1,
};

fn matrix_comparison_to_regression_item(
    workspace_root: &Path,
    comparison: &serde_json::Value,
    matrix_summary_path: &Path,
    target: &str,
) -> RegressionItemSummaryV1 {
    let raw_script = comparison
        .get("script")
        .and_then(|v| v.as_str())
        .unwrap_or("matrix");
    let script = normalize_repo_relative_path(workspace_root, Path::new(raw_script));
    let report = comparison
        .get("report")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let ok = report.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    let diff_count = report
        .get("diffs")
        .and_then(|v| v.as_array())
        .map(|diffs| diffs.len())
        .unwrap_or(0);
    let bundle_a = report
        .get("bundle_a")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());
    let bundle_b = report
        .get("bundle_b")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    RegressionItemSummaryV1 {
        item_id: format!("matrix:{script}"),
        kind: RegressionItemKindV1::MatrixCase,
        name: script.clone(),
        status: if ok {
            RegressionStatusV1::Passed
        } else {
            RegressionStatusV1::FailedDeterministic
        },
        reason_code: (!ok).then(|| "diag.matrix.compare_failed".to_string()),
        lane: RegressionLaneV1::Matrix,
        owner: None,
        feature_tags: Vec::new(),
        timing: None,
        attempts: None,
        evidence: Some(RegressionEvidenceV1 {
            bundle_artifact: bundle_b.clone().or_else(|| bundle_a.clone()),
            bundle_dir: None,
            triage_json: None,
            script_result_json: None,
            ai_packet_dir: None,
            pack_path: None,
            screenshots_manifest: None,
            perf_summary_json: None,
            compare_json: Some(matrix_summary_path.display().to_string()),
            extra: Some(serde_json::json!({
                "target": target,
                "bundle_a": bundle_a,
                "bundle_b": bundle_b,
                "report": report,
            })),
        }),
        source: Some(RegressionSourceV1 {
            script: Some(script.clone()),
            suite: Some(target.to_string()),
            campaign_case: Some("cached_vs_uncached".to_string()),
            metadata: None,
        }),
        notes: Some(RegressionNotesV1 {
            summary: Some(format!("diffs={diff_count}")),
            details: Vec::new(),
        }),
    }
}

fn write_regression_summary_for_matrix(
    workspace_root: &Path,
    resolved_out_dir: &Path,
    generated_unix_ms: u64,
    target: &str,
    payload: &serde_json::Value,
    matrix_summary_path: &Path,
) {
    let mut items = payload
        .get("comparisons")
        .and_then(|v| v.as_array())
        .map(|comparisons| {
            comparisons
                .iter()
                .map(|comparison| {
                    matrix_comparison_to_regression_item(
                        workspace_root,
                        comparison,
                        matrix_summary_path,
                        target,
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if items.is_empty() {
        items.push(RegressionItemSummaryV1 {
            item_id: format!("matrix:{target}"),
            kind: RegressionItemKindV1::CampaignStep,
            name: target.to_string(),
            status: RegressionStatusV1::FailedTooling,
            reason_code: Some("tooling.diag_matrix.no_rows".to_string()),
            lane: RegressionLaneV1::Matrix,
            owner: None,
            feature_tags: Vec::new(),
            timing: None,
            attempts: None,
            evidence: Some(RegressionEvidenceV1 {
                bundle_artifact: None,
                bundle_dir: None,
                triage_json: None,
                script_result_json: None,
                ai_packet_dir: None,
                pack_path: None,
                screenshots_manifest: None,
                perf_summary_json: None,
                compare_json: Some(matrix_summary_path.display().to_string()),
                extra: Some(payload.clone()),
            }),
            source: Some(RegressionSourceV1 {
                script: None,
                suite: Some(target.to_string()),
                campaign_case: Some("matrix".to_string()),
                metadata: None,
            }),
            notes: Some(RegressionNotesV1 {
                summary: Some("matrix completed without comparison rows".to_string()),
                details: Vec::new(),
            }),
        });
    }

    let mut totals = RegressionTotalsV1::default();
    for item in &items {
        totals.record_status(item.status);
    }

    let mut summary = RegressionSummaryV1::new(
        RegressionCampaignSummaryV1 {
            name: target.to_string(),
            lane: RegressionLaneV1::Matrix,
            profile: Some("matrix".to_string()),
            schema_version: Some(1),
            requested_by: Some("diag matrix".to_string()),
            filters: payload.get("options").cloned(),
        },
        RegressionRunSummaryV1 {
            run_id: generated_unix_ms.to_string(),
            created_unix_ms: generated_unix_ms,
            started_unix_ms: None,
            finished_unix_ms: None,
            duration_ms: None,
            workspace_root: Some(workspace_root.display().to_string()),
            out_dir: Some(resolved_out_dir.display().to_string()),
            tool: "fretboard diag matrix".to_string(),
            tool_version: None,
            git_commit: None,
            git_branch: None,
            host: None,
        },
        totals,
    );
    summary.items = items;
    summary.highlights = RegressionHighlightsV1::from_items(&summary.items);
    summary.artifacts = Some(RegressionArtifactsV1 {
        summary_dir: Some(resolved_out_dir.display().to_string()),
        packed_report: None,
        index_json: Some(matrix_summary_path.display().to_string()),
        html_report: None,
    });

    let regression_summary_path = resolved_out_dir.join("regression.summary.json");
    if let Err(err) = write_json_value(
        &regression_summary_path,
        &serde_json::to_value(&summary).unwrap_or_else(|_| serde_json::json!({})),
    ) {
        eprintln!(
            "warning: failed to write regression summary {}: {}",
            regression_summary_path.display(),
            err
        );
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MatrixCmdContext {
    pub rest: Vec<String>,
    pub launch: Option<Vec<String>>,
    pub launch_env: Vec<(String, String)>,
    pub launch_high_priority: bool,
    pub workspace_root: PathBuf,
    pub resolved_out_dir: PathBuf,
    pub timeout_ms: u64,
    pub poll_ms: u64,
    pub warmup_frames: u64,
    pub compare_eps_px: f32,
    pub compare_ignore_bounds: bool,
    pub compare_ignore_scene_fingerprint: bool,
    pub check_view_cache_reuse_min: Option<u64>,
    pub check_view_cache_reuse_stable_min: Option<u64>,
    pub check_overlay_synthesis_min: Option<u64>,
    pub check_viewport_input_min: Option<u64>,
    pub stats_json: bool,
}

pub(crate) fn cmd_matrix(ctx: MatrixCmdContext) -> Result<(), String> {
    let MatrixCmdContext {
        rest,
        launch,
        launch_env,
        launch_high_priority,
        workspace_root,
        resolved_out_dir,
        timeout_ms,
        poll_ms,
        warmup_frames,
        compare_eps_px,
        compare_ignore_bounds,
        compare_ignore_scene_fingerprint,
        check_view_cache_reuse_min,
        check_view_cache_reuse_stable_min,
        check_overlay_synthesis_min,
        check_viewport_input_min,
        stats_json,
    } = ctx;

    let Some(target) = rest.first().cloned() else {
        return Err("missing matrix target (try: fretboard diag matrix ui-gallery)".to_string());
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }
    if target != "ui-gallery" {
        return Err(format!("unknown matrix target: {target}"));
    }

    let Some(launch) = &launch else {
        return Err(
            "diag matrix requires --launch to run uncached/cached variants (for env control)"
                .to_string(),
        );
    };

    let inputs = diag_suite_scripts::ui_gallery_suite_scripts();
    let scripts: Vec<PathBuf> = expand_script_inputs(&workspace_root, &inputs)?;

    let compare_opts = CompareOptions {
        warmup_frames,
        eps_px: compare_eps_px,
        ignore_bounds: compare_ignore_bounds,
        ignore_scene_fingerprint: compare_ignore_scene_fingerprint,
    };

    // In matrix mode, treat `--check-view-cache-reuse-min 0` as "disabled".
    let reuse_gate = match check_view_cache_reuse_min {
        Some(0) => None,
        Some(v) => Some(v),
        None => Some(1),
    };

    // In matrix mode, treat `--check-view-cache-reuse-stable-min 0` as "disabled".
    let reuse_stable_gate = match check_view_cache_reuse_stable_min {
        Some(0) => None,
        Some(v) => Some(v),
        None => None,
    };

    // In matrix mode, treat `--check-overlay-synthesis-min 0` as "disabled".
    //
    // Default behavior:
    //
    // - If the caller enables shell reuse (`FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`), also
    //   enable a minimal overlay synthesis gate by default. This helps ensure the
    //   cached-synthesis seam is actually exercised (rather than "view cache enabled but
    //   overlay producers always rerendered").
    // - Otherwise, leave the gate off by default to avoid forcing overlay-specific
    //   assumptions onto non-overlay scripts (e.g. virtual-list torture).
    let mut matrix_base_env = launch_env.clone();
    let _ = ensure_env_var(&mut matrix_base_env, "FRET_DIAG_RENDERER_PERF", "1");
    if reuse_gate.is_some() || reuse_stable_gate.is_some() {
        // View-cache reuse gates depend on cache-root debug records, which are only produced when
        // the app enables UiTree debug collection. UI gallery disables debug in perf mode unless
        // `FRET_UI_DEBUG_STATS` is set.
        let _ = ensure_env_var(&mut matrix_base_env, "FRET_UI_DEBUG_STATS", "1");
    }

    let shell_reuse_enabled = matrix_base_env.iter().any(|(k, v)| {
        (k.as_str() == "FRET_UI_GALLERY_VIEW_CACHE_SHELL")
            && !v.trim().is_empty()
            && (v.as_str() != "0")
    });
    let overlay_synthesis_gate = match check_overlay_synthesis_min {
        Some(0) => None,
        Some(v) => Some(v),
        None => shell_reuse_enabled.then_some(1),
    };

    // In matrix mode, treat `--check-viewport-input-min 0` as "disabled".
    let viewport_input_gate = match check_viewport_input_min {
        Some(0) => None,
        Some(v) => Some(v),
        None => None,
    };

    let uncached_out_dir = resolved_out_dir.join("uncached");
    let cached_out_dir = resolved_out_dir.join("cached");

    let uncached_paths = ResolvedScriptPaths::for_out_dir(&workspace_root, &uncached_out_dir);
    let cached_paths = ResolvedScriptPaths::for_out_dir(&workspace_root, &cached_out_dir);

    let uncached_env = matrix_launch_env(&matrix_base_env, false)?;
    let cached_env = matrix_launch_env(&matrix_base_env, true)?;

    let uncached_bundles = run_script_suite_collect_bundles(
        &scripts,
        &uncached_paths,
        launch,
        &uncached_env,
        launch_high_priority,
        &workspace_root,
        timeout_ms,
        poll_ms,
        warmup_frames,
        None,
        None,
        None,
        None,
        viewport_input_gate,
        viewport_input_gate.map(|_| {
            diag_policy::ui_gallery_script_requires_viewport_input_gate as fn(&Path) -> bool
        }),
        None,
        None,
    )?;
    let cached_bundles = run_script_suite_collect_bundles(
        &scripts,
        &cached_paths,
        launch,
        &cached_env,
        launch_high_priority,
        &workspace_root,
        timeout_ms,
        poll_ms,
        warmup_frames,
        reuse_stable_gate,
        reuse_gate,
        overlay_synthesis_gate,
        overlay_synthesis_gate.map(|_| {
            diag_policy::ui_gallery_script_requires_overlay_synthesis_gate as fn(&Path) -> bool
        }),
        viewport_input_gate,
        viewport_input_gate.map(|_| {
            diag_policy::ui_gallery_script_requires_viewport_input_gate as fn(&Path) -> bool
        }),
        None,
        None,
    )?;

    let mut ok = true;
    let mut comparisons: Vec<(PathBuf, CompareReport)> = Vec::new();
    for (idx, script) in scripts.iter().enumerate() {
        let a = uncached_bundles
            .get(idx)
            .cloned()
            .ok_or_else(|| format!("missing uncached bundle for script: {}", script.display()))?;
        let b = cached_bundles
            .get(idx)
            .cloned()
            .ok_or_else(|| format!("missing cached bundle for script: {}", script.display()))?;
        let report = compare_bundles(&a, &b, compare_opts)?;
        ok &= report.ok;
        comparisons.push((script.clone(), report));
    }

    let generated_unix_ms = now_unix_ms();
    let matrix_payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": generated_unix_ms,
        "kind": "diag_matrix_summary",
        "target": target,
        "ok": ok,
        "out_dir_uncached": uncached_paths.out_dir.display().to_string(),
        "out_dir_cached": cached_paths.out_dir.display().to_string(),
        "options": {
            "warmup_frames": compare_opts.warmup_frames,
            "eps_px": compare_opts.eps_px,
            "ignore_bounds": compare_opts.ignore_bounds,
            "ignore_scene_fingerprint": compare_opts.ignore_scene_fingerprint,
            "check_view_cache_reuse_min": reuse_gate,
            "check_view_cache_reuse_stable_min": reuse_stable_gate,
            "check_overlay_synthesis_min": overlay_synthesis_gate,
            "check_viewport_input_min": viewport_input_gate,
        },
        "comparisons": comparisons.iter().map(|(script, report)| serde_json::json!({
            "script": script.display().to_string(),
            "report": report.to_json(),
        })).collect::<Vec<_>>(),
    });
    let matrix_summary_path = resolved_out_dir.join("matrix.summary.json");
    if let Err(err) = write_json_value(&matrix_summary_path, &matrix_payload) {
        eprintln!(
            "warning: failed to write matrix summary {}: {}",
            matrix_summary_path.display(),
            err
        );
    }
    write_regression_summary_for_matrix(
        &workspace_root,
        &resolved_out_dir,
        generated_unix_ms,
        &target,
        &matrix_payload,
        &matrix_summary_path,
    );

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&matrix_payload).unwrap_or_else(|_| "{}".to_string())
        );
        if !ok {
            std::process::exit(1);
        }
        Ok(())
    } else if ok {
        println!(
            "matrix: ok (scripts={}, warmup_frames={}, check_view_cache_reuse_min={:?}, check_view_cache_reuse_stable_min={:?}, check_overlay_synthesis_min={:?}, check_viewport_input_min={:?})",
            scripts.len(),
            warmup_frames,
            reuse_gate,
            reuse_stable_gate,
            overlay_synthesis_gate,
            viewport_input_gate
        );
        Ok(())
    } else {
        println!(
            "matrix: failed (scripts={}, warmup_frames={}, check_view_cache_reuse_min={:?}, check_view_cache_reuse_stable_min={:?}, check_overlay_synthesis_min={:?}, check_viewport_input_min={:?})",
            scripts.len(),
            warmup_frames,
            reuse_gate,
            reuse_stable_gate,
            overlay_synthesis_gate,
            viewport_input_gate
        );
        for (script, report) in comparisons {
            if report.ok {
                continue;
            }
            println!("\nscript: {}", script.display());
            report.print_human();
        }
        Err("matrix compare failed".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_comparison_to_regression_item_marks_failed_compare() {
        let workspace_root = Path::new("F:/repo");
        let comparison = serde_json::json!({
            "script": "F:/repo/apps/ui-gallery/button.diag.ron",
            "report": {
                "ok": false,
                "bundle_a": "F:/repo/out/uncached/bundle.json",
                "bundle_b": "F:/repo/out/cached/bundle.json",
                "diffs": [
                    { "kind": "scene_mismatch", "key": "root", "field": "bounds", "a": 1, "b": 2 }
                ],
                "options": { "warmup_frames": 8 }
            }
        });

        let item = matrix_comparison_to_regression_item(
            workspace_root,
            &comparison,
            Path::new("F:/repo/out/matrix.summary.json"),
            "ui-gallery",
        );

        assert_eq!(item.status, RegressionStatusV1::FailedDeterministic);
        assert_eq!(
            item.reason_code.as_deref(),
            Some("diag.matrix.compare_failed")
        );
        assert_eq!(item.name, "apps/ui-gallery/button.diag.ron");
        assert_eq!(
            item.evidence
                .as_ref()
                .and_then(|e| e.compare_json.as_deref()),
            Some("F:/repo/out/matrix.summary.json")
        );
    }
}
