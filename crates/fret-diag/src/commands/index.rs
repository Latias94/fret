use std::path::{Path, PathBuf};

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_index(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    index_out: Option<PathBuf>,
    warmup_frames: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle path (try: fretboard diag index ./target/fret-diag/1234/bundle.json)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let bundle_path = crate::resolve_bundle_json_path(&src);

    let out = index_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| crate::bundle_index::default_bundle_index_path(&bundle_path));

    if out.is_file() {
        if stats_json {
            println!(
                "{}",
                std::fs::read_to_string(&out).map_err(|e| e.to_string())?
            );
        } else {
            println!("{}", out.display());
        }
        return Ok(());
    }

    let canonical = crate::bundle_index::ensure_bundle_index_json(&bundle_path, warmup_frames)?;
    if out != canonical {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::copy(&canonical, &out).map_err(|e| e.to_string())?;
    }

    if stats_json {
        println!(
            "{}",
            std::fs::read_to_string(&out).map_err(|e| e.to_string())?
        );
    } else {
        println!("{}", out.display());
    }
    Ok(())
}
