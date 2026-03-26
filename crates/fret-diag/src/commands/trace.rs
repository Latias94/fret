use std::path::{Path, PathBuf};

pub(crate) fn cmd_trace(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    trace_out: Option<PathBuf>,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle artifact path (try: fretboard diag trace <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let resolved = crate::commands::resolve::resolve_bundle_ref(&src)?;
    let bundle_path = resolved.bundle_artifact;
    let bundle_dir = resolved.bundle_dir;
    let out = trace_out
        .map(|path| crate::resolve_path(workspace_root, path))
        .unwrap_or_else(|| bundle_dir.join("trace.chrome.json"));
    crate::trace::write_chrome_trace_from_bundle_path(&bundle_path, &out)?;
    println!("{}", out.display());
    Ok(())
}
