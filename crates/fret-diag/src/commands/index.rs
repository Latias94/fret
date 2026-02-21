use std::path::{Path, PathBuf};

fn try_read_bundle_index_json(path: &Path) -> Option<serde_json::Value> {
    let bytes = std::fs::read(path).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("bundle_index") {
        return None;
    }
    Some(v)
}

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

    let (index_path, default_out) = if src.is_file()
        && src
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s == "bundle.index.json")
    {
        (src.clone(), src.clone())
    } else if src.is_dir() {
        let direct = src.join("bundle.index.json");
        if direct.is_file() && try_read_bundle_index_json(&direct).is_some() {
            (direct.clone(), direct)
        } else {
            let bundle_path = crate::resolve_bundle_json_path(&src);
            let canonical =
                crate::bundle_index::ensure_bundle_index_json(&bundle_path, warmup_frames)?;
            let out = crate::bundle_index::default_bundle_index_path(&bundle_path);
            (canonical, out)
        }
    } else {
        let bundle_path = crate::resolve_bundle_json_path(&src);
        let canonical = crate::bundle_index::ensure_bundle_index_json(&bundle_path, warmup_frames)?;
        let out = crate::bundle_index::default_bundle_index_path(&bundle_path);
        (canonical, out)
    };

    let out = index_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or(default_out);

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

    if out != index_path {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::copy(&index_path, &out).map_err(|e| e.to_string())?;
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
