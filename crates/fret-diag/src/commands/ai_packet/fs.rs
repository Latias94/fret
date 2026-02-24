use std::path::Path;

pub(super) fn copy_file_named(src: &Path, dir: &Path, name: &str) -> Result<(), String> {
    let dst = dir.join(name);
    std::fs::copy(src, dst).map_err(|e| e.to_string())?;
    Ok(())
}

pub(super) fn copy_if_present(src: &Path, dir: &Path, name: &str) -> Result<(), String> {
    if src.is_file() {
        copy_file_named(src, dir, name)?;
    }
    Ok(())
}

pub(super) fn copy_bundle_schema2_if_present(
    bundle_path: &Path,
    bundle_dir: &Path,
    packet_dir: &Path,
) -> Result<(), String> {
    let schema2_name = std::ffi::OsStr::new("bundle.schema2.json");

    let parent_schema2 = bundle_path
        .parent()
        .unwrap_or(bundle_dir)
        .join("bundle.schema2.json");

    let candidates = [
        bundle_path.to_path_buf(),
        parent_schema2,
        bundle_dir.join("bundle.schema2.json"),
        bundle_dir.join("_root").join("bundle.schema2.json"),
    ];

    for c in candidates {
        if c.is_file() && c.file_name() == Some(schema2_name) {
            copy_file_named(&c, packet_dir, "bundle.schema2.json")?;
            break;
        }
    }

    Ok(())
}
