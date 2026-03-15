use std::path::{Path, PathBuf};

use fret_assets::{AssetBundleId, FileAssetManifestBundleV1, FileAssetManifestV1};

pub(crate) fn assets_cmd(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let Some(target) = it.next() else {
        return Err(
            "missing assets target (try: fretboard assets manifest write --dir assets --out assets.manifest.json --app-bundle my-app)"
                .to_string(),
        );
    };

    match target.as_str() {
        "--help" | "-h" => crate::cli::help(),
        "manifest" => manifest_cmd(it.collect()),
        other => Err(format!("unknown assets target: {other}")),
    }
}

fn manifest_cmd(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let Some(target) = it.next() else {
        return Err(
            "missing manifest action (try: fretboard assets manifest write --dir assets --out assets.manifest.json --app-bundle my-app)"
                .to_string(),
        );
    };

    match target.as_str() {
        "--help" | "-h" => crate::cli::help(),
        "write" => manifest_write_cmd(it.collect()),
        other => Err(format!("unknown assets manifest action: {other}")),
    }
}

fn manifest_write_cmd(args: Vec<String>) -> Result<(), String> {
    let mut dir: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut bundle: Option<AssetBundleId> = None;
    let mut force = false;

    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--dir" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--dir requires a value".to_string())?;
                dir = Some(PathBuf::from(raw));
            }
            "--out" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--out requires a value".to_string())?;
                out = Some(PathBuf::from(raw));
            }
            "--bundle" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--bundle requires a value".to_string())?;
                set_bundle_arg(&mut bundle, "--bundle", AssetBundleId::new(raw))?;
            }
            "--app-bundle" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--app-bundle requires a value".to_string())?;
                set_bundle_arg(&mut bundle, "--app-bundle", AssetBundleId::app(raw))?;
            }
            "--package-bundle" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--package-bundle requires a value".to_string())?;
                set_bundle_arg(&mut bundle, "--package-bundle", AssetBundleId::package(raw))?;
            }
            "--force" => force = true,
            "--help" | "-h" => return crate::cli::help(),
            other => {
                return Err(format!(
                    "unknown argument for assets manifest write: {other}"
                ));
            }
        }
    }

    let dir = dir.ok_or_else(|| "--dir is required".to_string())?;
    let out = out.ok_or_else(|| "--out is required".to_string())?;
    let bundle = bundle.ok_or_else(|| {
        "one bundle selector is required: --app-bundle <name> | --package-bundle <name> | --bundle <id>"
            .to_string()
    })?;

    reject_manifest_path_inside_bundle_dir(&dir, &out)?;

    if out.exists() && !force {
        return Err(format!(
            "refusing to overwrite existing file: {} (use --force)",
            out.display()
        ));
    }

    let manifest =
        FileAssetManifestV1::new([
            FileAssetManifestBundleV1::scan_dir(bundle.clone(), &dir).map_err(|e| e.to_string())?
        ]);
    let entry_count = manifest
        .bundles
        .first()
        .map(|bundle| bundle.entries.len())
        .unwrap_or(0);

    manifest.write_json_path(&out).map_err(|e| e.to_string())?;
    println!(
        "wrote {} (bundle {}, {} entr{})",
        out.display(),
        bundle.as_str(),
        entry_count,
        if entry_count == 1 { "y" } else { "ies" }
    );
    Ok(())
}

fn set_bundle_arg(
    slot: &mut Option<AssetBundleId>,
    flag: &'static str,
    value: AssetBundleId,
) -> Result<(), String> {
    if let Some(existing) = slot {
        return Err(format!(
            "bundle selector already set to {} (cannot also use {flag})",
            existing.as_str()
        ));
    }
    *slot = Some(value);
    Ok(())
}

fn reject_manifest_path_inside_bundle_dir(dir: &Path, out: &Path) -> Result<(), String> {
    let dir_abs = dir
        .canonicalize()
        .map_err(|e| format!("failed to resolve bundle dir `{}`: {e}", dir.display()))?;
    let cwd =
        std::env::current_dir().map_err(|e| format!("failed to read current directory: {e}"))?;
    let out_parent = out.parent().unwrap_or_else(|| Path::new("."));
    let out_parent_abs = if out_parent.is_absolute() {
        out_parent.to_path_buf()
    } else {
        cwd.join(out_parent)
    }
    .canonicalize()
    .map_err(|e| {
        format!(
            "failed to resolve output parent `{}`: {e}",
            out_parent.display()
        )
    })?;
    let out_file_name = out
        .file_name()
        .ok_or_else(|| format!("--out must point to a file path: {}", out.display()))?;
    let out_abs = out_parent_abs.join(out_file_name);

    if out_abs.starts_with(&dir_abs) {
        return Err(format!(
            "--out must live outside --dir to avoid manifest self-inclusion\n  dir: {}\n  out: {}",
            dir_abs.display(),
            out_abs.display()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn manifest_write_emits_scanned_bundle_manifest() {
        let root = make_temp_dir("fretboard-assets-manifest-write");
        let asset_dir = root.join("assets").join("images");
        std::fs::create_dir_all(&asset_dir).expect("create asset dir");
        std::fs::write(asset_dir.join("logo.png"), b"png").expect("write asset");

        let out = root.join("assets.manifest.json");
        manifest_write_cmd(vec![
            "--dir".into(),
            root.join("assets").display().to_string(),
            "--out".into(),
            out.display().to_string(),
            "--app-bundle".into(),
            "demo-app".into(),
        ])
        .expect("manifest write should succeed");

        let manifest =
            FileAssetManifestV1::load_json_path(&out).expect("written manifest should parse");
        assert_eq!(manifest.bundles.len(), 1);
        assert_eq!(manifest.bundles[0].id, AssetBundleId::app("demo-app"));
        assert_eq!(manifest.bundles[0].entries.len(), 1);
        assert_eq!(
            manifest.bundles[0].entries[0].key.as_str(),
            "images/logo.png"
        );
    }

    #[test]
    fn manifest_write_rejects_output_inside_scanned_dir() {
        let root = make_temp_dir("fretboard-assets-manifest-self-include");
        let asset_dir = root.join("assets");
        std::fs::create_dir_all(&asset_dir).expect("create asset dir");

        let err = manifest_write_cmd(vec![
            "--dir".into(),
            asset_dir.display().to_string(),
            "--out".into(),
            asset_dir.join("assets.manifest.json").display().to_string(),
            "--app-bundle".into(),
            "demo-app".into(),
        ])
        .expect_err("manifest write should reject out path under dir");

        assert!(err.contains("--out must live outside --dir"));
    }

    #[test]
    fn manifest_write_rejects_multiple_bundle_selectors() {
        let root = make_temp_dir("fretboard-assets-manifest-bundle-conflict");
        std::fs::create_dir_all(root.join("assets")).expect("create asset dir");

        let err = manifest_write_cmd(vec![
            "--dir".into(),
            root.join("assets").display().to_string(),
            "--out".into(),
            root.join("assets.manifest.json").display().to_string(),
            "--app-bundle".into(),
            "demo-app".into(),
            "--bundle".into(),
            "legacy".into(),
        ])
        .expect_err("manifest write should reject conflicting bundle selectors");

        assert!(err.contains("bundle selector already set"));
    }
}
