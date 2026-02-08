use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use zip::write::FileOptions;

/// Returns the default repo-local exports root used by tooling when materializing in-memory bundles.
///
/// This path is intended for the **DevTools GUI** (and other tooling) to bridge web-runner bundles
/// (which cannot write to the host filesystem) into a packable directory structure.
pub fn diag_exports_root(repo_root: &Path) -> PathBuf {
    repo_root.join(".fret").join("diag").join("exports")
}

#[derive(Debug, Clone)]
pub struct MaterializedBundle {
    pub exports_root: PathBuf,
    pub export_dir: PathBuf,
}

/// Materializes an in-memory `bundle.json` payload into:
/// `{export_root}/{exported_unix_ms}/bundle.json`
pub fn materialize_bundle_json(
    export_root: &Path,
    exported_unix_ms: u64,
    bundle_json: &str,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(export_root).map_err(|e| e.to_string())?;

    let export_dir = export_root.join(exported_unix_ms.to_string());
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    std::fs::write(export_dir.join("bundle.json"), bundle_json.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(export_dir)
}

/// Materializes an in-memory `bundle.json` payload into:
/// `{repo_root}/.fret/diag/exports/{exported_unix_ms}/bundle.json`
pub fn materialize_bundle_json_to_exports(
    repo_root: &Path,
    exported_unix_ms: u64,
    bundle_json: &str,
) -> Result<MaterializedBundle, String> {
    let exports_root = diag_exports_root(repo_root);
    let export_dir = materialize_bundle_json(&exports_root, exported_unix_ms, bundle_json)?;

    Ok(MaterializedBundle {
        exports_root,
        export_dir,
    })
}

/// Packs an in-memory `bundle.json` into a zip (in bytes) with `bundle.json` at the zip root.
///
/// This is intended for future web-only DevTools UIs (download/export) and for MCP-style surfaces
/// that may want to return zip bytes directly.
pub fn pack_bundle_json_to_zip_bytes(bundle_json: &str) -> Result<Vec<u8>, String> {
    let mut cursor = Cursor::new(Vec::<u8>::new());
    {
        let mut zip = zip::ZipWriter::new(&mut cursor);
        let opts = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("bundle.json", opts)
            .map_err(|e| e.to_string())?;
        zip.write_all(bundle_json.as_bytes())
            .map_err(|e| e.to_string())?;
        zip.finish().map_err(|e| e.to_string())?;
    }
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[test]
    fn pack_bundle_json_to_zip_bytes_includes_bundle_json() {
        let bundle_json = r#"{ "schema_version": 1, "hello": "world" }"#;
        let bytes = pack_bundle_json_to_zip_bytes(bundle_json).expect("pack zip bytes");

        let reader = Cursor::new(bytes);
        let mut zip = zip::ZipArchive::new(reader).expect("open zip");
        assert_eq!(zip.len(), 1);

        let mut file = zip.by_name("bundle.json").expect("bundle.json exists");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("read bundle.json");
        assert_eq!(contents, bundle_json);
    }

    #[test]
    fn materialize_bundle_json_to_exports_writes_bundle_json() {
        let repo_root = std::env::temp_dir().join(format!(
            "fret-diag-artifacts-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));
        let _ = std::fs::create_dir_all(&repo_root);

        let bundle_json = r#"{ "schema_version": 1 }"#;
        let ts = 1234567890u64;
        let mat = materialize_bundle_json_to_exports(&repo_root, ts, bundle_json)
            .expect("materialize bundle");

        let path = mat.export_dir.join("bundle.json");
        let on_disk = std::fs::read_to_string(path).expect("bundle.json readable");
        assert_eq!(on_disk, bundle_json);

        let _ = std::fs::remove_dir_all(&repo_root);
    }
}
