use std::path::{Path, PathBuf};

fn try_read_bundle_index_json(path: &Path) -> Option<serde_json::Value> {
    let bytes = std::fs::read(path).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if v.get("kind").and_then(|v| v.as_str()) != Some("bundle_index") {
        return None;
    }
    if v.get("schema_version").and_then(|v| v.as_u64()) != Some(1) {
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
        let v = try_read_bundle_index_json(&src)
            .ok_or_else(|| format!("invalid bundle.index.json: {}", src.display()))?;
        if v.get("warmup_frames").and_then(|v| v.as_u64()) == Some(warmup_frames) {
            (src.clone(), src.clone())
        } else {
            // Recover by regenerating from an adjacent bundle.json.
            let mut candidates: Vec<PathBuf> = Vec::new();
            if let Some(parent) = src.parent() {
                candidates.push(parent.to_path_buf());
                if parent.file_name().and_then(|s| s.to_str()) == Some("_root") {
                    if let Some(grandparent) = parent.parent() {
                        candidates.push(grandparent.to_path_buf());
                    }
                }
            }
            let mut regenerated: Option<(PathBuf, PathBuf)> = None;
            for candidate in candidates {
                let bundle_path = crate::resolve_bundle_json_path(&candidate);
                if !bundle_path.is_file() {
                    continue;
                }
                let canonical =
                    crate::bundle_index::ensure_bundle_index_json(&bundle_path, warmup_frames)?;
                let out = crate::bundle_index::default_bundle_index_path(&bundle_path);
                regenerated = Some((canonical, out));
                break;
            }
            regenerated.ok_or_else(|| {
                format!(
                    "bundle.index.json warmup_frames mismatch and no adjacent bundle.json was found to regenerate it (tip: run `fretboard diag index <bundle_dir|bundle.json>`)\n  index: {}",
                    src.display()
                )
            })?
        }
    } else if src.is_dir() {
        let direct = src.join("bundle.index.json");
        if direct.is_file()
            && let Some(v) = try_read_bundle_index_json(&direct)
            && v.get("warmup_frames").and_then(|v| v.as_u64()) == Some(warmup_frames)
        {
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
