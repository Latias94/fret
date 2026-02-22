use super::*;

#[derive(Debug, Clone)]
pub(crate) struct CompareCmdContext {
    pub rest: Vec<String>,
    pub workspace_root: PathBuf,
    pub warmup_frames: u64,
    pub compare_eps_px: f32,
    pub compare_ignore_bounds: bool,
    pub compare_ignore_scene_fingerprint: bool,
    pub stats_json: bool,
}

pub(crate) fn cmd_compare(ctx: CompareCmdContext) -> Result<(), String> {
    let CompareCmdContext {
        rest,
        workspace_root,
        warmup_frames,
        compare_eps_px,
        compare_ignore_bounds,
        compare_ignore_scene_fingerprint,
        stats_json,
    } = ctx;

    let Some(a_src) = rest.first().cloned() else {
        return Err(
            "missing bundle A path (try: fretboard diag compare ./a/bundle.json ./b/bundle.json)"
                .to_string(),
        );
    };
    let Some(b_src) = rest.get(1).cloned() else {
        return Err(
            "missing bundle B path (try: fretboard diag compare ./a/bundle.json ./b/bundle.json)"
                .to_string(),
        );
    };
    if rest.len() != 2 {
        return Err(format!("unexpected arguments: {}", rest[2..].join(" ")));
    }

    let a_src = resolve_path(&workspace_root, PathBuf::from(a_src));
    let b_src = resolve_path(&workspace_root, PathBuf::from(b_src));
    let a_bundle_path = resolve_bundle_json_path(&a_src);
    let b_bundle_path = resolve_bundle_json_path(&b_src);

    let report = compare_bundles(
        &a_bundle_path,
        &b_bundle_path,
        CompareOptions {
            warmup_frames,
            eps_px: compare_eps_px,
            ignore_bounds: compare_ignore_bounds,
            ignore_scene_fingerprint: compare_ignore_scene_fingerprint,
        },
    )?;

    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report.to_json()).unwrap_or_else(|_| "{}".to_string())
        );
        if !report.ok {
            std::process::exit(1);
        }
        Ok(())
    } else if report.ok {
        report.print_human();
        Ok(())
    } else {
        Err(report.to_human_error())
    }
}

