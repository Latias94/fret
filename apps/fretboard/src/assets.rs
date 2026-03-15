use std::fmt::Write as _;
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
        "rust" => rust_cmd(it.collect()),
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

fn rust_cmd(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let Some(target) = it.next() else {
        return Err(
            "missing rust action (try: fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle my-app)"
                .to_string(),
        );
    };

    match target.as_str() {
        "--help" | "-h" => crate::cli::help(),
        "write" => rust_write_cmd(it.collect()),
        other => Err(format!("unknown assets rust action: {other}")),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AssetBundleSelector {
    Raw(String),
    App(String),
    Package(String),
}

impl AssetBundleSelector {
    fn asset_bundle_id(&self) -> AssetBundleId {
        match self {
            Self::Raw(value) => AssetBundleId::new(value.clone()),
            Self::App(name) => AssetBundleId::app(name.clone()),
            Self::Package(name) => AssetBundleId::package(name.clone()),
        }
    }

    fn expression(&self) -> String {
        match self {
            Self::Raw(value) => format!("AssetBundleId::new({value:?})"),
            Self::App(name) => format!("AssetBundleId::app({name:?})"),
            Self::Package(name) => format!("AssetBundleId::package({name:?})"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RustSurface {
    Fret,
    Framework,
}

impl RustSurface {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "fret" => Ok(Self::Fret),
            "framework" => Ok(Self::Framework),
            other => Err(format!(
                "unknown assets rust surface: {other} (expected `fret` or `framework`)"
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Fret => "fret",
            Self::Framework => "framework",
        }
    }
}

fn manifest_write_cmd(args: Vec<String>) -> Result<(), String> {
    let mut dir: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut bundle: Option<AssetBundleSelector> = None;
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
                set_bundle_arg(&mut bundle, "--bundle", AssetBundleSelector::Raw(raw))?;
            }
            "--app-bundle" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--app-bundle requires a value".to_string())?;
                set_bundle_arg(&mut bundle, "--app-bundle", AssetBundleSelector::App(raw))?;
            }
            "--package-bundle" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--package-bundle requires a value".to_string())?;
                set_bundle_arg(
                    &mut bundle,
                    "--package-bundle",
                    AssetBundleSelector::Package(raw),
                )?;
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

    reject_out_path_inside_bundle_dir(&dir, &out)?;

    if out.exists() && !force {
        return Err(format!(
            "refusing to overwrite existing file: {} (use --force)",
            out.display()
        ));
    }

    let bundle_id = bundle.asset_bundle_id();
    let manifest = FileAssetManifestV1::new([FileAssetManifestBundleV1::scan_dir(
        bundle_id.clone(),
        &dir,
    )
    .map_err(|e| e.to_string())?]);
    let entry_count = manifest
        .bundles
        .first()
        .map(|bundle| bundle.entries.len())
        .unwrap_or(0);

    manifest.write_json_path(&out).map_err(|e| e.to_string())?;
    println!(
        "wrote {} (bundle {}, {} entr{})",
        out.display(),
        bundle_id.as_str(),
        entry_count,
        if entry_count == 1 { "y" } else { "ies" }
    );
    Ok(())
}

fn rust_write_cmd(args: Vec<String>) -> Result<(), String> {
    let mut dir: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut crate_root: Option<PathBuf> = None;
    let mut bundle: Option<AssetBundleSelector> = None;
    let mut surface = RustSurface::Fret;
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
            "--crate-root" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--crate-root requires a value".to_string())?;
                crate_root = Some(PathBuf::from(raw));
            }
            "--surface" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--surface requires a value".to_string())?;
                surface = RustSurface::parse(&raw)?;
            }
            "--bundle" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--bundle requires a value".to_string())?;
                set_bundle_arg(&mut bundle, "--bundle", AssetBundleSelector::Raw(raw))?;
            }
            "--app-bundle" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--app-bundle requires a value".to_string())?;
                set_bundle_arg(&mut bundle, "--app-bundle", AssetBundleSelector::App(raw))?;
            }
            "--package-bundle" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--package-bundle requires a value".to_string())?;
                set_bundle_arg(
                    &mut bundle,
                    "--package-bundle",
                    AssetBundleSelector::Package(raw),
                )?;
            }
            "--force" => force = true,
            "--help" | "-h" => return crate::cli::help(),
            other => {
                return Err(format!("unknown argument for assets rust write: {other}"));
            }
        }
    }

    let dir = dir.ok_or_else(|| "--dir is required".to_string())?;
    let out = out.ok_or_else(|| "--out is required".to_string())?;
    let bundle = bundle.ok_or_else(|| {
        "one bundle selector is required: --app-bundle <name> | --package-bundle <name> | --bundle <id>"
            .to_string()
    })?;

    reject_out_path_inside_bundle_dir(&dir, &out)?;

    if out.exists() && !force {
        return Err(format!(
            "refusing to overwrite existing file: {} (use --force)",
            out.display()
        ));
    }

    let crate_root = match crate_root {
        Some(path) => path,
        None => {
            std::env::current_dir().map_err(|e| format!("failed to read current directory: {e}"))?
        }
    };
    let crate_root_abs = crate_root.canonicalize().map_err(|e| {
        format!(
            "failed to resolve crate root `{}`: {e}",
            crate_root.display()
        )
    })?;
    let dir_abs = dir
        .canonicalize()
        .map_err(|e| format!("failed to resolve bundle dir `{}`: {e}", dir.display()))?;

    let bundle_id = bundle.asset_bundle_id();
    let scanned_bundle = FileAssetManifestBundleV1::scan_dir(bundle_id.clone(), &dir_abs)
        .map_err(|e| e.to_string())?;
    let generated =
        render_rust_asset_module(&bundle, &scanned_bundle, &crate_root_abs, &dir_abs, surface)?;

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create output parent `{}`: {e}", parent.display()))?;
    }
    std::fs::write(&out, generated).map_err(|e| {
        format!(
            "failed to write generated Rust asset module `{}`: {e}",
            out.display()
        )
    })?;

    println!(
        "wrote {} (surface {}, bundle {}, {} entr{})",
        out.display(),
        surface.as_str(),
        bundle_id.as_str(),
        scanned_bundle.entries.len(),
        if scanned_bundle.entries.len() == 1 {
            "y"
        } else {
            "ies"
        }
    );
    Ok(())
}

fn render_rust_asset_module(
    bundle_selector: &AssetBundleSelector,
    bundle: &FileAssetManifestBundleV1,
    crate_root_abs: &Path,
    dir_abs: &Path,
    surface: RustSurface,
) -> Result<String, String> {
    let mut out = String::new();
    writeln!(
        out,
        "#![allow(dead_code)]\n\
         \n\
         // Generated by `fretboard assets rust write`; do not edit by hand.\n\
         // Regenerate this file from the asset source directory instead of editing it manually.\n\
         // Surface: {}\n\
         // Bundle: {}\n",
        surface.as_str(),
        bundle.id.as_str()
    )
    .expect("write to string");

    match surface {
        RustSurface::Fret => {
            writeln!(
                out,
                "use fret::assets::{{self, AssetBundleId, AssetKey, AssetLocator, AssetRevision, StaticAssetEntry}};\n"
            )
            .expect("write to string");
            writeln!(
                out,
                "// `--surface fret` modules expose both `register(app)` and `mount(builder)`.\n"
            )
            .expect("write to string");
        }
        RustSurface::Framework => {
            writeln!(
                out,
                "use fret_assets::{{AssetBundleId, AssetKey, AssetLocator, AssetRevision, StaticAssetEntry}};\nuse fret_runtime::GlobalsHost;\n"
            )
            .expect("write to string");
            writeln!(
                out,
                "// `--surface framework` modules expose `register(host)` for direct runtime mounting.\n"
            )
            .expect("write to string");
        }
    }

    writeln!(
        out,
        "pub fn bundle_id() -> AssetBundleId {{\n    {}\n}}\n",
        bundle_selector.expression()
    )
    .expect("write to string");
    writeln!(
        out,
        "pub fn locator(key: impl Into<AssetKey>) -> AssetLocator {{\n    AssetLocator::bundle(bundle_id(), key)\n}}\n"
    )
    .expect("write to string");
    writeln!(out, "pub const ENTRIES: &[StaticAssetEntry] = &[").expect("write to string");

    for entry in &bundle.entries {
        let asset_path = asset_path_from_key(dir_abs, entry.key.as_str());
        let bytes = std::fs::read(&asset_path).map_err(|e| {
            format!(
                "failed to read asset file `{}` while generating Rust module: {e}",
                asset_path.display()
            )
        })?;
        let rel_path = asset_path.strip_prefix(crate_root_abs).map_err(|_| {
            format!(
                "asset file `{}` must live under crate root `{}` when generating a Rust module",
                asset_path.display(),
                crate_root_abs.display()
            )
        })?;
        let rel_path = path_to_forward_slashes(rel_path);
        let media_type = guess_media_type(&asset_path);
        let revision = stable_asset_revision(&bytes);
        let key_literal = format!("{:?}", entry.key.as_str());
        let path_literal = format!("{:?}", rel_path);

        writeln!(
            out,
            "    StaticAssetEntry::new(\n        {key_literal},\n        AssetRevision({revision}),\n        include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/\", {path_literal})),\n    )"
        )
        .expect("write to string");

        if let Some(media_type) = media_type {
            writeln!(out, "    .with_media_type({media_type:?}),").expect("write to string");
        } else {
            writeln!(out, "    ,").expect("write to string");
        }
    }

    writeln!(out, "];\n").expect("write to string");

    match surface {
        RustSurface::Fret => {
            writeln!(
                out,
                "pub fn register(app: &mut fret::app::App) {{\n    assets::register_bundle_entries(app, bundle_id(), ENTRIES.iter().copied());\n}}\n"
            )
            .expect("write to string");
            writeln!(
                out,
                "pub fn mount<S: 'static>(builder: fret::UiAppBuilder<S>) -> fret::UiAppBuilder<S> {{\n    builder.with_bundle_asset_entries(bundle_id(), ENTRIES.iter().copied())\n}}"
            )
            .expect("write to string");
        }
        RustSurface::Framework => {
            writeln!(
                out,
                "pub fn register(host: &mut impl GlobalsHost) {{\n    fret_runtime::register_bundle_asset_entries(host, bundle_id(), ENTRIES.iter().copied());\n}}"
            )
            .expect("write to string");
        }
    }

    Ok(out)
}

fn asset_path_from_key(root: &Path, key: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    for part in key.split('/') {
        path.push(part);
    }
    path
}

fn path_to_forward_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn guess_media_type(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "svg" => Some("image/svg+xml"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        "bmp" => Some("image/bmp"),
        "ico" => Some("image/x-icon"),
        "avif" => Some("image/avif"),
        "ttf" => Some("font/ttf"),
        "otf" => Some("font/otf"),
        "woff" => Some("font/woff"),
        "woff2" => Some("font/woff2"),
        "json" => Some("application/json"),
        "txt" => Some("text/plain"),
        "md" => Some("text/markdown"),
        "css" => Some("text/css"),
        "js" | "mjs" => Some("text/javascript"),
        _ => None,
    }
}

fn stable_asset_revision(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn set_bundle_arg(
    slot: &mut Option<AssetBundleSelector>,
    flag: &'static str,
    value: AssetBundleSelector,
) -> Result<(), String> {
    if let Some(existing) = slot {
        return Err(format!(
            "bundle selector already set to {} (cannot also use {flag})",
            existing.asset_bundle_id().as_str()
        ));
    }
    *slot = Some(value);
    Ok(())
}

fn reject_out_path_inside_bundle_dir(dir: &Path, out: &Path) -> Result<(), String> {
    let dir_abs = dir
        .canonicalize()
        .map_err(|e| format!("failed to resolve bundle dir `{}`: {e}", dir.display()))?;
    let cwd =
        std::env::current_dir().map_err(|e| format!("failed to read current directory: {e}"))?;
    let out_abs = resolve_output_path(&cwd, out)?;

    if out_abs.starts_with(&dir_abs) {
        return Err(format!(
            "--out must live outside --dir to avoid asset self-inclusion\n  dir: {}\n  out: {}",
            dir_abs.display(),
            out_abs.display()
        ));
    }

    Ok(())
}

fn resolve_output_path(cwd: &Path, out: &Path) -> Result<PathBuf, String> {
    let out_abs = if out.is_absolute() {
        out.to_path_buf()
    } else {
        cwd.join(out)
    };

    let mut existing = out_abs.as_path();
    let mut suffix = Vec::new();
    while !existing.exists() {
        let name = existing.file_name().ok_or_else(|| {
            format!(
                "failed to resolve output path `{}` to an existing ancestor",
                out.display()
            )
        })?;
        suffix.push(name.to_os_string());
        existing = existing.parent().ok_or_else(|| {
            format!(
                "failed to resolve output path `{}` to an existing ancestor",
                out.display()
            )
        })?;
    }

    let mut resolved = existing.canonicalize().map_err(|e| {
        format!(
            "failed to resolve output path ancestor `{}`: {e}",
            existing.display()
        )
    })?;
    for part in suffix.iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
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
        std::fs::write(asset_dir.join("logo.png"), b"png").expect("write asset");

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

    #[test]
    fn rust_write_emits_fret_surface_module_with_embedded_entries() {
        let root = make_temp_dir("fretboard-assets-rust-write-fret");
        let asset_dir = root.join("assets").join("icons");
        let out = root.join("src").join("generated_assets.rs");
        std::fs::create_dir_all(&asset_dir).expect("create asset dir");
        std::fs::write(asset_dir.join("search.svg"), br#"<svg></svg>"#).expect("write asset");

        rust_write_cmd(vec![
            "--dir".into(),
            root.join("assets").display().to_string(),
            "--out".into(),
            out.display().to_string(),
            "--crate-root".into(),
            root.display().to_string(),
            "--app-bundle".into(),
            "demo-app".into(),
        ])
        .expect("rust write should succeed");

        let generated = std::fs::read_to_string(&out).expect("read generated module");
        assert!(generated.contains("use fret::assets::{self, AssetBundleId, AssetKey, AssetLocator, AssetRevision, StaticAssetEntry};"));
        assert!(generated.contains(
            "// `--surface fret` modules expose both `register(app)` and `mount(builder)`."
        ));
        assert!(generated.contains("AssetBundleId::app(\"demo-app\")"));
        assert!(generated.contains("include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/\", \"assets/icons/search.svg\"))"));
        assert!(generated.contains(".with_media_type(\"image/svg+xml\")"));
        assert!(generated.contains("pub fn register(app: &mut fret::app::App)"));
        assert!(generated.contains(
            "assets::register_bundle_entries(app, bundle_id(), ENTRIES.iter().copied());"
        ));
        assert!(generated.contains(
            "pub fn mount<S: 'static>(builder: fret::UiAppBuilder<S>) -> fret::UiAppBuilder<S>"
        ));
        assert!(
            generated.contains(
                "builder.with_bundle_asset_entries(bundle_id(), ENTRIES.iter().copied())"
            )
        );
    }

    #[test]
    fn rust_write_supports_framework_surface() {
        let root = make_temp_dir("fretboard-assets-rust-write-framework");
        let asset_dir = root.join("assets").join("fonts");
        let out = root.join("generated_assets.rs");
        std::fs::create_dir_all(&asset_dir).expect("create asset dir");
        std::fs::write(asset_dir.join("ui.ttf"), b"font-bytes").expect("write asset");

        rust_write_cmd(vec![
            "--dir".into(),
            root.join("assets").display().to_string(),
            "--out".into(),
            out.display().to_string(),
            "--crate-root".into(),
            root.display().to_string(),
            "--surface".into(),
            "framework".into(),
            "--package-bundle".into(),
            "demo-kit".into(),
        ])
        .expect("framework surface rust write should succeed");

        let generated = std::fs::read_to_string(&out).expect("read generated module");
        assert!(generated.contains("use fret_assets::{AssetBundleId, AssetKey, AssetLocator, AssetRevision, StaticAssetEntry};"));
        assert!(generated.contains("use fret_runtime::GlobalsHost;"));
        assert!(generated.contains(
            "// `--surface framework` modules expose `register(host)` for direct runtime mounting."
        ));
        assert!(generated.contains("AssetBundleId::package(\"demo-kit\")"));
        assert!(generated.contains(".with_media_type(\"font/ttf\")"));
        assert!(generated.contains("pub fn register(host: &mut impl GlobalsHost)"));
        assert!(generated.contains("fret_runtime::register_bundle_asset_entries(host, bundle_id(), ENTRIES.iter().copied());"));
    }

    #[test]
    fn rust_write_rejects_assets_outside_crate_root() {
        let root = make_temp_dir("fretboard-assets-rust-write-crate-root");
        let external_assets = make_temp_dir("fretboard-assets-rust-write-external-assets");
        let out = root.join("generated_assets.rs");

        std::fs::create_dir_all(external_assets.join("images")).expect("create asset dir");
        std::fs::write(external_assets.join("images/logo.png"), b"png").expect("write asset");

        let err = rust_write_cmd(vec![
            "--dir".into(),
            external_assets.display().to_string(),
            "--out".into(),
            out.display().to_string(),
            "--crate-root".into(),
            root.display().to_string(),
            "--app-bundle".into(),
            "demo-app".into(),
        ])
        .expect_err("rust write should reject assets outside crate root");

        assert!(err.contains("must live under crate root"));
    }

    #[test]
    fn rust_write_rejects_output_inside_scanned_dir() {
        let root = make_temp_dir("fretboard-assets-rust-self-include");
        let asset_dir = root.join("assets");
        std::fs::create_dir_all(asset_dir.join("images")).expect("create asset dir");
        std::fs::write(asset_dir.join("images/logo.png"), b"png").expect("write asset");

        let err = rust_write_cmd(vec![
            "--dir".into(),
            asset_dir.display().to_string(),
            "--out".into(),
            asset_dir.join("generated_assets.rs").display().to_string(),
            "--crate-root".into(),
            root.display().to_string(),
            "--app-bundle".into(),
            "demo-app".into(),
        ])
        .expect_err("rust write should reject out path under dir");

        assert!(err.contains("--out must live outside --dir"));
    }
}
