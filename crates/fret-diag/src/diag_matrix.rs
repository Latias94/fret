use super::*;

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

    let scripts: Vec<PathBuf> = diag_suite_scripts::ui_gallery_suite_scripts()
        .into_iter()
        .map(|p| resolve_path(&workspace_root, PathBuf::from(p)))
        .collect();

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
        viewport_input_gate
            .map(|_| ui_gallery_script_requires_viewport_input_gate as fn(&Path) -> bool),
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
        overlay_synthesis_gate
            .map(|_| ui_gallery_script_requires_overlay_synthesis_gate as fn(&Path) -> bool),
        viewport_input_gate,
        viewport_input_gate
            .map(|_| ui_gallery_script_requires_viewport_input_gate as fn(&Path) -> bool),
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

    if stats_json {
        let payload = serde_json::json!({
            "schema_version": 1,
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
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
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
