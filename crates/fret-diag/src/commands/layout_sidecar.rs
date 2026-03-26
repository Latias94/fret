use std::path::{Path, PathBuf};

use serde_json::json;

use super::args;

const LAYOUT_SIDECAR_TAFFY_V1_FILENAME: &str = "layout.taffy.v1.json";

pub(crate) fn cmd_layout_sidecar(
    rest: &[String],
    resolved_out_dir: &Path,
    workspace_root: &Path,
    stats_json: bool,
    out: Option<&Path>,
) -> Result<(), String> {
    let mut print: bool = false;
    let mut target: Option<String> = None;

    let mut i: usize = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        match arg {
            "--print" => {
                print = true;
                i += 1;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown diag layout-sidecar flag: {other}"));
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

    let Some(sidecar_path) = resolve_layout_sidecar_path(&bundle_dir) else {
        return Err(format!(
            "layout sidecar not found under: {}\n  expected: {}/{LAYOUT_SIDECAR_TAFFY_V1_FILENAME}\n  hint: ensure a script runs `capture_layout_sidecar` (example: tools/diag-scripts/ui-gallery/layout/ui-gallery-empty-outline-layout-sidecar.json)\n  hint: if this is a packed zip extraction, pass the extracted bundle dir (the folder that contains `_root/`) or the `_root/` folder itself",
            bundle_dir.display(),
            bundle_dir.display()
        ));
    };

    let bytes = std::fs::read(&sidecar_path).map_err(|e| e.to_string())?;

    let output_bytes: Vec<u8> = if stats_json {
        serde_json::to_vec_pretty(&json!({
            "found": true,
            "bundle_artifact": bundle_path.display().to_string(),
            "bundle_dir": bundle_dir.display().to_string(),
            "sidecar": {
                "name": LAYOUT_SIDECAR_TAFFY_V1_FILENAME,
                "path": sidecar_path.display().to_string(),
                "bytes": bytes.len(),
            },
        }))
        .map_err(|e| e.to_string())?
    } else if print {
        // Bounded stdout: allow large payloads only when writing to a file.
        if out.is_none() && bytes.len() > 2 * 1024 * 1024 {
            return Err(format!(
                "layout sidecar is too large to print to stdout (bytes={}).\n  hint: use `--out <path>` to write it to a file, or omit `--print` to print only the path.",
                bytes.len()
            ));
        }
        bytes
    } else {
        format!("{}\n", sidecar_path.display()).into_bytes()
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

fn resolve_layout_sidecar_path(bundle_dir: &Path) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    candidates.push(bundle_dir.join(LAYOUT_SIDECAR_TAFFY_V1_FILENAME));
    candidates.push(
        bundle_dir
            .join("_root")
            .join(LAYOUT_SIDECAR_TAFFY_V1_FILENAME),
    );

    // If `bundle_dir` is already `_root/`, also check the parent dir (runtime layout).
    if bundle_dir
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s == "_root")
    {
        if let Some(parent) = bundle_dir.parent() {
            candidates.push(parent.join(LAYOUT_SIDECAR_TAFFY_V1_FILENAME));
        }
    }

    for c in candidates {
        if c.is_file() {
            return Some(c);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("{prefix}-{}", crate::util::now_unix_ms()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn resolves_sidecar_in_direct_bundle_dir() {
        let base = make_temp_dir("fret-diag-layout-sidecar-direct");
        let bundle_dir = base.join("123-bundle");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(bundle_dir.join("bundle.schema2.json"), b"{}").expect("write bundle");
        std::fs::write(
            bundle_dir.join(LAYOUT_SIDECAR_TAFFY_V1_FILENAME),
            br#"{"schema_version":"v1"}"#,
        )
        .expect("write sidecar");

        let got = resolve_layout_sidecar_path(&bundle_dir).expect("resolve sidecar");
        assert_eq!(got, bundle_dir.join(LAYOUT_SIDECAR_TAFFY_V1_FILENAME));
    }

    #[test]
    fn resolves_sidecar_under_root_dir_for_packed_extraction_layout() {
        let base = make_temp_dir("fret-diag-layout-sidecar-packed");
        let outer = base.join("123-bundle");
        let root = outer.join("_root");
        std::fs::create_dir_all(&root).expect("create _root dir");
        std::fs::write(root.join("bundle.schema2.json"), b"{}").expect("write bundle");
        std::fs::write(
            root.join(LAYOUT_SIDECAR_TAFFY_V1_FILENAME),
            br#"{"schema_version":"v1"}"#,
        )
        .expect("write sidecar");

        // When pointed at the outer dir, bundle artifact resolution prefers `_root/bundle.schema2.json`.
        let bundle_path = crate::resolve_bundle_artifact_path(&outer);
        assert_eq!(bundle_path, root.join("bundle.schema2.json"));
        let bundle_dir = crate::resolve_bundle_root_dir(&bundle_path).expect("bundle dir");
        assert_eq!(bundle_dir, root);

        let got = resolve_layout_sidecar_path(&bundle_dir).expect("resolve sidecar");
        assert_eq!(got, root.join(LAYOUT_SIDECAR_TAFFY_V1_FILENAME));
    }
}
