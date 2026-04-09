mod contracts;
mod fs;
mod iconify;
mod naming;
mod presentation_defaults;
mod semantic_aliases;
mod svg_dir;
mod templates;

use std::collections::{BTreeMap, BTreeSet};

pub use contracts::{
    DependencySpec, GeneratePackRequest, GeneratedPackReport, IconifyCollectionSource,
    PresentationDefaults, PresentationDefaultsConfigFileV1, PresentationOverride,
    PresentationRenderMode, SemanticAlias, SemanticAliasConfigFileV1, SourceSpec,
    SvgDirectorySource,
};
pub use naming::normalize_svg_directory_icon_name;
pub use presentation_defaults::load_presentation_defaults_json_file;
pub use semantic_aliases::load_semantic_aliases_json_file;

use fs::{
    crate_module_name, ensure_dir_is_new_or_empty, sanitize_package_name,
    validate_vendor_namespace, write_new_bytes, write_new_file,
};
use iconify::collect_iconify_collection;
use presentation_defaults::sanitize_presentation_defaults;
use semantic_aliases::sanitize_semantic_aliases;
use svg_dir::collect_svg_directory;
use templates::{
    render_advanced_rs, render_app_rs, render_cargo_toml, render_generated_ids, render_icon_list,
    render_lib_rs, render_provenance_json, render_readme_md,
};

#[derive(Debug, thiserror::Error)]
pub enum GeneratePackError {
    #[error("{0}")]
    InvalidPackageName(String),
    #[error("{0}")]
    InvalidVendorNamespace(String),
    #[error("{0}")]
    InvalidIconName(String),
    #[error("output path exists but is not a directory: {0}")]
    OutputPathNotDirectory(String),
    #[error("{0}")]
    OutputDirectoryNotEmpty(String),
    #[error("refusing to overwrite existing file: {0}")]
    RefusingToOverwrite(String),
    #[error("missing source directory: {0}")]
    MissingSourceDirectory(String),
    #[error("source path is not a directory: {0}")]
    SourcePathNotDirectory(String),
    #[error("missing source file: {0}")]
    MissingSourceFile(String),
    #[error("source path is not a file: {0}")]
    SourcePathNotFile(String),
    #[error("missing semantic alias config file: {0}")]
    MissingSemanticAliasConfigFile(String),
    #[error("semantic alias config path is not a file: {0}")]
    SemanticAliasConfigPathNotFile(String),
    #[error("unsupported semantic alias config schema version: expected {expected}, got {actual}")]
    UnsupportedSemanticAliasConfigSchemaVersion { expected: u32, actual: u32 },
    #[error("missing presentation defaults config file: {0}")]
    MissingPresentationDefaultsConfigFile(String),
    #[error("presentation defaults config path is not a file: {0}")]
    PresentationDefaultsConfigPathNotFile(String),
    #[error(
        "unsupported presentation defaults config schema version: expected {expected}, got {actual}"
    )]
    UnsupportedPresentationDefaultsConfigSchemaVersion { expected: u32, actual: u32 },
    #[error("no SVG icons found in {0}")]
    NoSvgIconsFound(String),
    #[error("iconify collection has no icons or aliases: {0}")]
    EmptyIconifyCollection(String),
    #[error("icon name collision after normalization: `{icon_name}` from `{first}` and `{second}`")]
    IconNameCollision {
        icon_name: String,
        first: String,
        second: String,
    },
    #[error("missing iconify parent icon or alias: `{icon_name}`")]
    MissingIconifyParent { icon_name: String },
    #[error("iconify alias loop detected: {chain}")]
    IconifyAliasLoop { chain: String },
    #[error("semantic alias id `{semantic_id}` must use the `ui.*` namespace")]
    SemanticAliasMustUseUiNamespace { semantic_id: String },
    #[error("duplicate semantic alias id `{semantic_id}`")]
    DuplicateSemanticAliasId { semantic_id: String },
    #[error("semantic alias target `{target_icon}` does not exist in the generated icon list")]
    MissingSemanticAliasTarget { target_icon: String },
    #[error("semantic alias id cannot be empty")]
    EmptySemanticAliasId,
    #[error("duplicate presentation override for `{icon_name}`")]
    DuplicatePresentationOverride { icon_name: String },
    #[error("presentation override target `{icon_name}` does not exist in the generated icon list")]
    MissingPresentationOverrideTarget { icon_name: String },
    #[error("presentation override icon name cannot be empty")]
    EmptyPresentationOverrideIconName,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn generate_pack_crate(
    request: GeneratePackRequest,
) -> Result<GeneratedPackReport, GeneratePackError> {
    let package_name = sanitize_package_name(&request.package_name)?;
    let pack_id = sanitize_package_name(&request.pack_id)?;
    let vendor_namespace = validate_vendor_namespace(&request.vendor_namespace)?;

    let request = GeneratePackRequest {
        package_name,
        pack_id,
        vendor_namespace,
        semantic_aliases: sanitize_semantic_aliases(request.semantic_aliases)?,
        presentation_defaults: sanitize_presentation_defaults(request.presentation_defaults)?,
        ..request
    };

    ensure_dir_is_new_or_empty(&request.output_dir)?;
    let mut icons = match &request.source {
        SourceSpec::SvgDirectory(source) => collect_svg_directory(source)?,
        SourceSpec::IconifyCollection(source) => collect_iconify_collection(source)?,
    };
    validate_semantic_aliases(&request.semantic_aliases, &icons)?;
    validate_presentation_defaults(&request.presentation_defaults, &icons)?;
    apply_presentation_defaults(&request.presentation_defaults, &mut icons);

    let src_dir = request.output_dir.join("src");
    let assets_icons_dir = request.output_dir.join("assets").join("icons");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::create_dir_all(&assets_icons_dir)?;

    write_new_file(
        &request.output_dir.join("Cargo.toml"),
        &render_cargo_toml(&request, !request.semantic_aliases.is_empty()),
    )?;
    write_new_file(
        &request.output_dir.join("README.md"),
        &render_readme_md(&request, &icons),
    )?;
    write_new_file(
        &request.output_dir.join("icon-list.txt"),
        &render_icon_list(&request, &icons),
    )?;
    write_new_file(
        &request.output_dir.join("pack-provenance.json"),
        &render_provenance_json(&request, &icons)?,
    )?;
    write_new_file(
        &src_dir.join("generated_ids.rs"),
        &render_generated_ids(&request, &icons),
    )?;
    write_new_file(&src_dir.join("lib.rs"), &render_lib_rs(&request, &icons))?;
    write_new_file(
        &src_dir.join("app.rs"),
        &render_app_rs(&crate_module_name(&request.package_name)),
    )?;
    write_new_file(&src_dir.join("advanced.rs"), render_advanced_rs())?;

    for icon in &icons {
        write_new_bytes(
            &assets_icons_dir.join(format!("{}.svg", icon.icon_name)),
            &icon.svg_bytes,
        )?;
    }

    Ok(GeneratedPackReport {
        output_dir: request.output_dir,
        package_name: request.package_name,
        pack_id: request.pack_id,
        vendor_namespace: request.vendor_namespace,
        icon_count: icons.len(),
    })
}

fn validate_semantic_aliases(
    aliases: &[SemanticAlias],
    icons: &[svg_dir::CollectedSvg],
) -> Result<(), GeneratePackError> {
    let icon_names = icons
        .iter()
        .map(|icon| icon.icon_name.as_str())
        .collect::<BTreeSet<_>>();

    for alias in aliases {
        if alias.semantic_id.trim().is_empty() {
            return Err(GeneratePackError::EmptySemanticAliasId);
        }
        if !icon_names.contains(alias.target_icon.as_str()) {
            return Err(GeneratePackError::MissingSemanticAliasTarget {
                target_icon: alias.target_icon.clone(),
            });
        }
    }

    Ok(())
}

fn validate_presentation_defaults(
    defaults: &PresentationDefaults,
    icons: &[svg_dir::CollectedSvg],
) -> Result<(), GeneratePackError> {
    let icon_names = icons
        .iter()
        .map(|icon| icon.icon_name.as_str())
        .collect::<BTreeSet<_>>();

    for override_entry in &defaults.icon_overrides {
        if !icon_names.contains(override_entry.icon_name.as_str()) {
            return Err(GeneratePackError::MissingPresentationOverrideTarget {
                icon_name: override_entry.icon_name.clone(),
            });
        }
    }

    Ok(())
}

fn apply_presentation_defaults(
    defaults: &PresentationDefaults,
    icons: &mut [svg_dir::CollectedSvg],
) {
    let default_render_mode = defaults
        .default_render_mode
        .unwrap_or(PresentationRenderMode::Mask);
    let overrides = defaults
        .icon_overrides
        .iter()
        .map(|override_entry| {
            (
                override_entry.icon_name.as_str(),
                override_entry.render_mode,
            )
        })
        .collect::<BTreeMap<_, _>>();

    for icon in icons {
        icon.render_mode = overrides
            .get(icon.icon_name.as_str())
            .copied()
            .unwrap_or(default_render_mode);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DependencySpec, GeneratePackError, GeneratePackRequest, PresentationDefaults,
        PresentationOverride, PresentationRenderMode, SemanticAlias,
    };
    use crate::{IconifyCollectionSource, SourceSpec, SvgDirectorySource};
    use serde_json::Value;
    use std::path::PathBuf;
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

    fn write_demo_svgs(source_dir: &std::path::Path) {
        std::fs::create_dir_all(source_dir.join("actions")).expect("create nested source dir");
        std::fs::write(
            source_dir.join("actions").join("search.svg"),
            r#"<svg viewBox="0 0 24 24"><path d="M10 10h4"/></svg>"#,
        )
        .expect("write search svg");
        std::fs::write(
            source_dir.join("close.svg"),
            r#"<svg viewBox="0 0 24 24"><path d="M3 3l18 18"/></svg>"#,
        )
        .expect("write close svg");
    }

    #[test]
    fn svg_directory_generation_emits_complete_pack_surface() {
        let root = make_temp_dir("fret-icons-generator");
        let source_dir = root.join("source");
        let output_dir = root.join("generated-pack");
        std::fs::create_dir_all(&source_dir).expect("create source dir");
        write_demo_svgs(&source_dir);

        let report = super::generate_pack_crate(GeneratePackRequest {
            package_name: "demo-icons".to_string(),
            pack_id: "demo-icons".to_string(),
            vendor_namespace: "demo".to_string(),
            output_dir: output_dir.clone(),
            source: SourceSpec::SvgDirectory(SvgDirectorySource {
                dir: source_dir,
                label: "demo-source".to_string(),
            }),
            dependency_spec: DependencySpec::Published {
                fret_version: "0.1.0".to_string(),
                rust_embed_version: "8.9.0".to_string(),
            },
            generator_label: "fretboard icons import svg-dir".to_string(),
            semantic_aliases: vec![SemanticAlias {
                semantic_id: "ui.search".to_string(),
                target_icon: "actions-search".to_string(),
            }],
            presentation_defaults: PresentationDefaults::default(),
        })
        .expect("pack generation should succeed");

        assert_eq!(report.icon_count, 2);
        assert!(output_dir.join("Cargo.toml").exists());
        assert!(output_dir.join("README.md").exists());
        assert!(output_dir.join("icon-list.txt").exists());
        assert!(output_dir.join("pack-provenance.json").exists());
        assert!(output_dir.join("src/lib.rs").exists());
        assert!(output_dir.join("src/app.rs").exists());
        assert!(output_dir.join("src/advanced.rs").exists());
        assert!(output_dir.join("src/generated_ids.rs").exists());
        assert!(output_dir.join("assets/icons/actions-search.svg").exists());
        assert!(output_dir.join("assets/icons/close.svg").exists());

        let lib_rs =
            std::fs::read_to_string(output_dir.join("src/lib.rs")).expect("generated lib.rs");
        assert!(lib_rs.contains("IconPackImportModel::Generated"));
        assert!(lib_rs.contains("pub const PACK_METADATA: IconPackMetadata"));
        assert!(lib_rs.contains("pub const PACK: IconPackRegistration"));
        assert!(lib_rs.contains("pub const UI_SEMANTIC_ALIAS_PACK"));

        let readme =
            std::fs::read_to_string(output_dir.join("README.md")).expect("generated README.md");
        assert!(readme.contains("`demo_icons::app::install(...)`"));
        assert!(
            readme.contains("`BootstrapBuilder::register_icon_pack_contract(demo_icons::PACK)`")
        );
        let app_rs =
            std::fs::read_to_string(output_dir.join("src/app.rs")).expect("generated app.rs");
        assert!(app_rs.contains("let frozen = icons.freeze().unwrap_or_else(|errors|"));
        assert!(app_rs.contains("installed.record(crate::PACK_METADATA).unwrap_or_else(|err|"));

        let provenance = std::fs::read_to_string(output_dir.join("pack-provenance.json"))
            .expect("generated provenance json");
        let provenance: Value = serde_json::from_str(&provenance).expect("valid provenance json");
        assert_eq!(provenance["pack"]["import_model"], "Generated");
        assert_eq!(provenance["source"]["label"], "demo-source");
        assert_eq!(provenance["icons"][0]["icon_name"], "actions-search");
        assert_eq!(provenance["icons"][0]["render_mode"], "mask");
        assert_eq!(
            provenance["icons"][0]["source_relative_path"],
            "actions/search.svg"
        );
    }

    #[test]
    fn semantic_aliases_must_target_existing_icons() {
        let root = make_temp_dir("fret-icons-generator-alias");
        let source_dir = root.join("source");
        let output_dir = root.join("generated-pack");
        std::fs::create_dir_all(&source_dir).expect("create source dir");
        write_demo_svgs(&source_dir);

        let err = super::generate_pack_crate(GeneratePackRequest {
            package_name: "demo-icons".to_string(),
            pack_id: "demo-icons".to_string(),
            vendor_namespace: "demo".to_string(),
            output_dir,
            source: SourceSpec::SvgDirectory(SvgDirectorySource {
                dir: source_dir,
                label: "demo-source".to_string(),
            }),
            dependency_spec: DependencySpec::Published {
                fret_version: "0.1.0".to_string(),
                rust_embed_version: "8.9.0".to_string(),
            },
            generator_label: "fretboard icons import svg-dir".to_string(),
            semantic_aliases: vec![SemanticAlias {
                semantic_id: "ui.search".to_string(),
                target_icon: "missing".to_string(),
            }],
            presentation_defaults: PresentationDefaults::default(),
        })
        .expect_err("missing alias target should fail");

        assert!(matches!(
            err,
            GeneratePackError::MissingSemanticAliasTarget { .. }
        ));
    }

    #[test]
    fn semantic_aliases_must_use_ui_namespace() {
        let root = make_temp_dir("fret-icons-generator-alias-ui");
        let source_dir = root.join("source");
        let output_dir = root.join("generated-pack");
        std::fs::create_dir_all(&source_dir).expect("create source dir");
        write_demo_svgs(&source_dir);

        let err = super::generate_pack_crate(GeneratePackRequest {
            package_name: "demo-icons".to_string(),
            pack_id: "demo-icons".to_string(),
            vendor_namespace: "demo".to_string(),
            output_dir,
            source: SourceSpec::SvgDirectory(SvgDirectorySource {
                dir: source_dir,
                label: "demo-source".to_string(),
            }),
            dependency_spec: DependencySpec::Published {
                fret_version: "0.1.0".to_string(),
                rust_embed_version: "8.9.0".to_string(),
            },
            generator_label: "fretboard icons import svg-dir".to_string(),
            semantic_aliases: vec![SemanticAlias {
                semantic_id: "search".to_string(),
                target_icon: "actions-search".to_string(),
            }],
            presentation_defaults: PresentationDefaults::default(),
        })
        .expect_err("non-ui semantic alias should fail");

        assert!(matches!(
            err,
            GeneratePackError::SemanticAliasMustUseUiNamespace { .. }
        ));
    }

    #[test]
    fn iconify_collection_generation_emits_complete_pack_surface() {
        let root = make_temp_dir("fret-icons-generator-iconify-pack");
        let source_file = root.join("demo-iconify.json");
        let output_dir = root.join("generated-pack");
        std::fs::write(
            &source_file,
            r#"{
  "prefix": "demo",
  "width": 24,
  "height": 24,
  "icons": {
    "search": { "body": "<path d='M10 10h4'/>" }
  },
  "aliases": {
    "search-rotated": { "parent": "search", "rotate": 1 }
  }
}"#,
        )
        .expect("write iconify collection snapshot");

        let report = super::generate_pack_crate(GeneratePackRequest {
            package_name: "demo-icons".to_string(),
            pack_id: "demo-icons".to_string(),
            vendor_namespace: "demo".to_string(),
            output_dir: output_dir.clone(),
            source: SourceSpec::IconifyCollection(IconifyCollectionSource {
                file: source_file,
                label: "demo-iconify".to_string(),
            }),
            dependency_spec: DependencySpec::Published {
                fret_version: "0.1.0".to_string(),
                rust_embed_version: "8.9.0".to_string(),
            },
            generator_label: "fretboard icons import iconify-collection".to_string(),
            semantic_aliases: Vec::new(),
            presentation_defaults: PresentationDefaults::default(),
        })
        .expect("iconify pack generation should succeed");

        assert_eq!(report.icon_count, 2);
        let icon_list =
            std::fs::read_to_string(output_dir.join("icon-list.txt")).expect("generated icon list");
        assert!(icon_list.contains("search.svg"));
        assert!(icon_list.contains("search-rotated.svg"));

        let provenance = std::fs::read_to_string(output_dir.join("pack-provenance.json"))
            .expect("generated provenance json");
        let provenance: Value = serde_json::from_str(&provenance).expect("valid provenance json");
        assert_eq!(provenance["source"]["kind"], "iconify-collection");
        assert_eq!(provenance["icons"][1]["icon_name"], "search-rotated");
        assert_eq!(provenance["icons"][1]["render_mode"], "mask");
    }

    #[test]
    fn presentation_defaults_must_target_existing_icons() {
        let root = make_temp_dir("fret-icons-generator-presentation-target");
        let source_dir = root.join("source");
        let output_dir = root.join("generated-pack");
        std::fs::create_dir_all(&source_dir).expect("create source dir");
        write_demo_svgs(&source_dir);

        let err = super::generate_pack_crate(GeneratePackRequest {
            package_name: "demo-icons".to_string(),
            pack_id: "demo-icons".to_string(),
            vendor_namespace: "demo".to_string(),
            output_dir,
            source: SourceSpec::SvgDirectory(SvgDirectorySource {
                dir: source_dir,
                label: "demo-source".to_string(),
            }),
            dependency_spec: DependencySpec::Published {
                fret_version: "0.1.0".to_string(),
                rust_embed_version: "8.9.0".to_string(),
            },
            generator_label: "fretboard icons import svg-dir".to_string(),
            semantic_aliases: Vec::new(),
            presentation_defaults: PresentationDefaults {
                default_render_mode: None,
                icon_overrides: vec![PresentationOverride {
                    icon_name: "missing".to_string(),
                    render_mode: PresentationRenderMode::OriginalColors,
                }],
            },
        })
        .expect_err("missing presentation override target should fail");

        assert!(matches!(
            err,
            GeneratePackError::MissingPresentationOverrideTarget { .. }
        ));
    }

    #[test]
    fn generation_emits_original_colors_registration_when_configured() {
        let root = make_temp_dir("fret-icons-generator-presentation-proof");
        let source_dir = root.join("source");
        let output_dir = root.join("generated-pack");
        std::fs::create_dir_all(&source_dir).expect("create source dir");
        std::fs::write(
            source_dir.join("brand-logo.svg"),
            r##"<svg viewBox="0 0 24 24"><path fill="#ff0000" d="M0 0h24v24H0z"/></svg>"##,
        )
        .expect("write brand logo svg");
        std::fs::write(
            source_dir.join("close.svg"),
            r#"<svg viewBox="0 0 24 24"><path d="M3 3l18 18"/></svg>"#,
        )
        .expect("write close svg");

        super::generate_pack_crate(GeneratePackRequest {
            package_name: "demo-icons".to_string(),
            pack_id: "demo-icons".to_string(),
            vendor_namespace: "demo".to_string(),
            output_dir: output_dir.clone(),
            source: SourceSpec::SvgDirectory(SvgDirectorySource {
                dir: source_dir,
                label: "demo-source".to_string(),
            }),
            dependency_spec: DependencySpec::Published {
                fret_version: "0.1.0".to_string(),
                rust_embed_version: "8.9.0".to_string(),
            },
            generator_label: "fretboard icons import svg-dir".to_string(),
            semantic_aliases: Vec::new(),
            presentation_defaults: PresentationDefaults {
                default_render_mode: Some(PresentationRenderMode::Mask),
                icon_overrides: vec![PresentationOverride {
                    icon_name: "brand-logo".to_string(),
                    render_mode: PresentationRenderMode::OriginalColors,
                }],
            },
        })
        .expect("pack generation should succeed");

        let lib_rs =
            std::fs::read_to_string(output_dir.join("src/lib.rs")).expect("generated lib.rs");
        assert!(lib_rs.contains("IconRenderMode::OriginalColors"));
        assert!(lib_rs.contains("IconRenderMode::Mask"));

        let provenance = std::fs::read_to_string(output_dir.join("pack-provenance.json"))
            .expect("generated provenance json");
        let provenance: Value = serde_json::from_str(&provenance).expect("valid provenance json");
        assert_eq!(
            provenance["presentation_defaults"]["default_render_mode"],
            "mask"
        );
        assert_eq!(
            provenance["presentation_defaults"]["icon_overrides"][0]["render_mode"],
            "original-colors"
        );
        assert_eq!(provenance["icons"][0]["render_mode"], "original-colors");
        assert_eq!(provenance["icons"][1]["render_mode"], "mask");
    }
}
