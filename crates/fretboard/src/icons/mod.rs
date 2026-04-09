mod acquire;
mod suggest;
mod suggest_svg;

use std::path::{Path, PathBuf};
use std::process::Command;

use fret_icons_generator::{
    DependencySpec, GeneratePackRequest, IconifyCollectionSource, PresentationDefaults,
    SemanticAlias, SourceSpec, SvgDirectorySource, generate_pack_crate,
    load_presentation_defaults_json_file, load_semantic_aliases_json_file,
};

use crate::scaffold::fs::{sanitize_package_name, workspace_prefix_from_out_dir};

pub mod contracts;

use self::contracts::{
    IconAcquireCommandArgs, IconAcquireSourceContract, IconImportCommandArgs,
    IconImportSourceContract, IconSuggestCommandArgs, IconSuggestKindContract, IconsCommandArgs,
    IconsCommandContract, ImportIconifyCollectionArgs, ImportSvgDirArgs,
};

const PUBLIC_RUST_EMBED_VERSION: &str = "8.9.0";

pub fn run_public_icons_contract(args: IconsCommandArgs) -> Result<(), String> {
    let cwd = std::env::current_dir()
        .map_err(|err| format!("failed to read current directory: {err}"))?;
    run_icons_contract_with_mode(&IconsMode::Public { cwd }, args)
}

pub fn run_repo_icons_contract(
    args: IconsCommandArgs,
    workspace_root: &Path,
) -> Result<(), String> {
    run_icons_contract_with_mode(
        &IconsMode::Repo {
            workspace_root: workspace_root.to_path_buf(),
        },
        args,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IconsMode {
    Public { cwd: PathBuf },
    Repo { workspace_root: PathBuf },
}

impl IconsMode {
    fn default_out_dir(&self, package_name: &str) -> PathBuf {
        match self {
            Self::Public { cwd } => cwd.join(package_name),
            Self::Repo { workspace_root } => workspace_root.join("local").join(package_name),
        }
    }

    fn dependency_spec(&self, out_dir: &Path) -> Result<DependencySpec, String> {
        match self {
            Self::Public { .. } => Ok(DependencySpec::Published {
                fret_version: env!("CARGO_PKG_VERSION").to_string(),
                rust_embed_version: PUBLIC_RUST_EMBED_VERSION.to_string(),
            }),
            Self::Repo { workspace_root } => {
                if !out_dir.exists() {
                    std::fs::create_dir_all(out_dir).map_err(|err| {
                        format!(
                            "failed to create output directory `{}`: {err}",
                            out_dir.display()
                        )
                    })?;
                }
                Ok(DependencySpec::WorkspacePath {
                    workspace_prefix: workspace_prefix_from_out_dir(workspace_root, out_dir)?,
                    rust_embed_version: PUBLIC_RUST_EMBED_VERSION.to_string(),
                })
            }
        }
    }
}

fn run_icons_contract_with_mode(mode: &IconsMode, args: IconsCommandArgs) -> Result<(), String> {
    match args.command {
        IconsCommandContract::Acquire(args) => run_acquire_contract(args),
        IconsCommandContract::Import(args) => run_import_contract(mode, args),
        IconsCommandContract::Suggest(args) => run_suggest_contract(args),
    }
}

fn run_acquire_contract(args: IconAcquireCommandArgs) -> Result<(), String> {
    match args.source {
        IconAcquireSourceContract::IconifyCollection(args) => {
            let report = acquire::run_iconify_collection_acquire_contract(args)?;
            println!("Acquired Iconify snapshot:");
            println!("  snapshot        : {}", report.snapshot_path.display());
            println!("  provenance      : {}", report.provenance_path.display());
            println!("  collection      : {}", report.collection);
            println!("  icons           : {}", report.icon_count);
            println!("  aliases         : {}", report.alias_count);
            Ok(())
        }
    }
}

fn run_import_contract(mode: &IconsMode, args: IconImportCommandArgs) -> Result<(), String> {
    match args.source {
        IconImportSourceContract::SvgDir(args) => run_svg_dir_import_contract(mode, args),
        IconImportSourceContract::IconifyCollection(args) => {
            run_iconify_collection_import_contract(mode, args)
        }
    }
}

fn run_suggest_contract(args: IconSuggestCommandArgs) -> Result<(), String> {
    match args.kind {
        IconSuggestKindContract::PresentationDefaults(args) => {
            let report = suggest::run_presentation_defaults_suggestion_contract(args)?;
            println!("Suggested presentation-defaults config:");
            println!("  output          : {}", report.out_path.display());
            if let Some(report_path) = &report.report_path {
                println!("  report          : {}", report_path.display());
            }
            println!("  collection      : {}", report.collection);
            println!("  palette         : {}", report.palette);
            println!(
                "  default mode    : {}",
                match report.default_render_mode {
                    fret_icons_generator::PresentationRenderMode::Mask => "mask",
                    fret_icons_generator::PresentationRenderMode::OriginalColors => {
                        "original-colors"
                    }
                }
            );
            Ok(())
        }
        IconSuggestKindContract::SvgDirPresentationOverrides(args) => {
            let report = suggest_svg::run_svg_dir_presentation_overrides_contract(args)?;
            println!("Suggested svg-dir presentation overrides:");
            println!("  output          : {}", report.out_path.display());
            if let Some(report_path) = &report.report_path {
                println!("  report          : {}", report_path.display());
            }
            println!("  icons analyzed  : {}", report.analyzed_icon_count);
            println!("  overrides       : {}", report.suggested_override_count);
            println!("  parse failures  : {}", report.parse_failure_count);
            Ok(())
        }
    }
}

fn run_svg_dir_import_contract(mode: &IconsMode, args: ImportSvgDirArgs) -> Result<(), String> {
    let source_label = args
        .common
        .source_label
        .clone()
        .unwrap_or_else(|| default_source_label(&args.source, "svg-directory"));
    let semantic_aliases = load_semantic_aliases_for_cli(args.common.semantic_aliases.as_deref())?;
    let presentation_defaults =
        load_presentation_defaults_for_cli(args.common.presentation_defaults.as_deref())?;

    run_generated_pack_contract(
        mode,
        args.common.crate_name,
        args.common.pack_id,
        args.common.vendor_namespace,
        args.common.path,
        SourceSpec::SvgDirectory(SvgDirectorySource {
            dir: args.source.clone(),
            label: source_label,
        }),
        "fretboard icons import svg-dir",
        semantic_aliases,
        presentation_defaults,
        args.common.no_check,
    )
}

fn run_iconify_collection_import_contract(
    mode: &IconsMode,
    args: ImportIconifyCollectionArgs,
) -> Result<(), String> {
    let source_label = args
        .common
        .source_label
        .clone()
        .unwrap_or_else(|| default_source_label(&args.source, "iconify-collection"));
    let semantic_aliases = load_semantic_aliases_for_cli(args.common.semantic_aliases.as_deref())?;
    let presentation_defaults =
        load_presentation_defaults_for_cli(args.common.presentation_defaults.as_deref())?;

    run_generated_pack_contract(
        mode,
        args.common.crate_name,
        args.common.pack_id,
        args.common.vendor_namespace,
        args.common.path,
        SourceSpec::IconifyCollection(IconifyCollectionSource {
            file: args.source.clone(),
            label: source_label,
        }),
        "fretboard icons import iconify-collection",
        semantic_aliases,
        presentation_defaults,
        args.common.no_check,
    )
}

fn run_generated_pack_contract(
    mode: &IconsMode,
    crate_name: String,
    pack_id: Option<String>,
    vendor_namespace: String,
    path: Option<PathBuf>,
    source: SourceSpec,
    generator_label: &str,
    semantic_aliases: Vec<SemanticAlias>,
    presentation_defaults: PresentationDefaults,
    no_check: bool,
) -> Result<(), String> {
    let crate_name = sanitize_package_name(&crate_name)?;
    let out_dir = path.unwrap_or_else(|| mode.default_out_dir(&crate_name));
    let pack_id = match pack_id.as_deref() {
        Some(pack_id) => sanitize_package_name(pack_id)?,
        None => crate_name.clone(),
    };

    let report = generate_pack_crate(GeneratePackRequest {
        package_name: crate_name,
        pack_id,
        vendor_namespace,
        output_dir: out_dir.clone(),
        source,
        dependency_spec: mode.dependency_spec(&out_dir)?,
        generator_label: generator_label.to_string(),
        semantic_aliases,
        presentation_defaults,
    })
    .map_err(|err| err.to_string())?;

    maybe_cargo_check(&out_dir, !no_check)?;

    println!("Generated Fret icon pack crate:");
    println!("  path            : {}", report.output_dir.display());
    println!("  package         : {}", report.package_name);
    println!("  pack id         : {}", report.pack_id);
    println!("  vendor namespace: {}", report.vendor_namespace);
    println!("  icons           : {}", report.icon_count);

    Ok(())
}

fn load_semantic_aliases_for_cli(path: Option<&Path>) -> Result<Vec<SemanticAlias>, String> {
    match path {
        Some(path) => load_semantic_aliases_json_file(path).map_err(|err| err.to_string()),
        None => Ok(Vec::new()),
    }
}

fn load_presentation_defaults_for_cli(path: Option<&Path>) -> Result<PresentationDefaults, String> {
    match path {
        Some(path) => load_presentation_defaults_json_file(path).map_err(|err| err.to_string()),
        None => Ok(PresentationDefaults::default()),
    }
}

fn default_source_label(source: &Path, fallback: &str) -> String {
    source
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
        .unwrap_or_else(|| fallback.to_string())
}

fn maybe_cargo_check(out_dir: &Path, run_check: bool) -> Result<(), String> {
    if !run_check {
        return Ok(());
    }

    println!("Running cargo check...");
    let status = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .arg("--features")
        .arg("app-integration")
        .current_dir(out_dir)
        .status()
        .map_err(|err| format!("failed to spawn cargo check: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo check failed with status: {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::contracts::{
        IconImportCommandArgs, IconImportSourceContract, IconSuggestCommandArgs,
        IconSuggestKindContract, IconsCommandArgs, IconsCommandContract, ImportCommonArgs,
        ImportIconifyCollectionArgs, ImportSvgDirArgs, SuggestPresentationDefaultsArgs,
        SuggestSvgDirPresentationOverridesArgs,
    };
    use super::run_repo_icons_contract;
    use serde_json::Value;
    use std::ffi::OsStr;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn repo_workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo workspace root should resolve")
    }

    fn make_repo_local_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = repo_workspace_root()
            .join("local")
            .join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create repo-local test dir");
        dir
    }

    fn write_demo_svgs(source_dir: &Path) {
        std::fs::create_dir_all(source_dir.join("actions")).expect("create nested source dir");
        std::fs::write(
            source_dir.join("actions").join("search.svg"),
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M10 10h4"/></svg>"#,
        )
        .expect("write search svg");
        std::fs::write(
            source_dir.join("close.svg"),
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M3 3l18 18"/></svg>"#,
        )
        .expect("write close svg");
    }

    fn write_demo_svg_dir_for_analysis(source_dir: &Path) {
        std::fs::create_dir_all(source_dir).expect("create source dir");
        std::fs::write(
            source_dir.join("brand-logo.svg"),
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#ff0000" d="M0 0h12v24H0z"/><path fill="#0000ff" d="M12 0h12v24H12z"/></svg>"##,
        )
        .expect("write multicolor svg");
        std::fs::write(
            source_dir.join("close.svg"),
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M3 3l18 18"/></svg>"#,
        )
        .expect("write monochrome svg");
    }

    fn write_demo_iconify_collection(source_file: &Path) {
        std::fs::write(
            source_file,
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
        .expect("write iconify snapshot");
    }

    fn write_demo_semantic_alias_config(config_file: &Path) {
        std::fs::write(
            config_file,
            r#"{
  "schema_version": 1,
  "semantic_aliases": [
    { "semantic_id": "ui.search", "target_icon": "actions-search" }
  ]
}"#,
        )
        .expect("write semantic alias config");
    }

    fn write_demo_presentation_defaults_config(config_file: &Path) {
        std::fs::write(
            config_file,
            r#"{
  "schema_version": 1,
  "default_render_mode": "mask",
  "icon_overrides": [
    { "icon_name": "brand-logo", "render_mode": "original-colors" }
  ]
}"#,
        )
        .expect("write presentation defaults config");
    }

    fn write_demo_iconify_acquisition_provenance(config_file: &Path, palette: bool) {
        std::fs::write(
            config_file,
            format!(
                r#"{{
  "schema_version": 1,
  "acquisition_kind": "iconify-collection",
  "collection": "demo",
  "request": {{ "mode": "subset", "requested_icons": ["brand-logo"] }},
  "source": {{
    "api_base_url": "https://api.iconify.design",
    "collection_info_url": "https://api.iconify.design/collection?prefix=demo&info=true",
    "icons_url": "https://api.iconify.design/demo.json?icons=brand-logo"
  }},
  "upstream": {{
    "title": "Demo Icons",
    "total": 2,
    "collection_info": {{ "palette": {palette} }}
  }},
  "snapshot": {{
    "digest_algorithm": "blake3",
    "digest_hex": "deadbeef",
    "icon_count": 1,
    "alias_count": 0
  }}
}}"#
            ),
        )
        .expect("write iconify acquisition provenance");
    }

    fn cargo_check_generated_pack(out_dir: &Path) {
        let repo_root = repo_workspace_root();
        let target_name = out_dir
            .file_name()
            .and_then(OsStr::to_str)
            .expect("generated pack dir should have a final path segment");
        let target_dir = repo_root
            .join("target")
            .join("fretboard-generated-icon-pack-checks")
            .join(target_name);

        let status = Command::new("cargo")
            .arg("check")
            .arg("--quiet")
            .arg("--features")
            .arg("app-integration")
            .current_dir(out_dir)
            .env("CARGO_TARGET_DIR", &target_dir)
            .status()
            .expect("spawn cargo check for generated pack");

        assert!(
            status.success(),
            "generated pack cargo check failed for {} with status {status}",
            out_dir.display()
        );
    }

    #[test]
    fn repo_svg_dir_import_generates_pack_that_compiles() {
        let workspace_root = repo_workspace_root();
        let suite_root = make_repo_local_dir("fretboard-icon-pack-proof");
        let source_dir = suite_root.join("source");
        let out_dir = suite_root.join("demo-icons-pack");
        std::fs::create_dir_all(&source_dir).expect("create source dir");
        write_demo_svgs(&source_dir);

        run_repo_icons_contract(
            IconsCommandArgs {
                command: IconsCommandContract::Import(IconImportCommandArgs {
                    source: IconImportSourceContract::SvgDir(ImportSvgDirArgs {
                        source: source_dir,
                        common: ImportCommonArgs {
                            crate_name: "demo-icons-pack".to_string(),
                            vendor_namespace: "demo".to_string(),
                            pack_id: None,
                            path: Some(out_dir.clone()),
                            source_label: Some("demo-source".to_string()),
                            semantic_aliases: None,
                            presentation_defaults: None,
                            no_check: true,
                        },
                    }),
                }),
            },
            &workspace_root,
        )
        .expect("repo svg-dir import should succeed");

        cargo_check_generated_pack(&out_dir);

        let cargo_toml =
            std::fs::read_to_string(out_dir.join("Cargo.toml")).expect("generated Cargo.toml");
        assert!(cargo_toml.contains("fret-icons = { path = "));
        assert!(cargo_toml.contains("fret-app = { path = "));
        assert!(cargo_toml.contains("fret-core = { path = "));

        let readme =
            std::fs::read_to_string(out_dir.join("README.md")).expect("generated README.md");
        assert!(readme.contains("`demo_icons_pack::app::install(...)`"));
        assert!(
            readme
                .contains("`BootstrapBuilder::register_icon_pack_contract(demo_icons_pack::PACK)`")
        );
    }

    #[test]
    fn repo_iconify_collection_import_generates_pack_that_compiles() {
        let workspace_root = repo_workspace_root();
        let suite_root = make_repo_local_dir("fretboard-iconify-pack-proof");
        let source_file = suite_root.join("demo-iconify.json");
        let out_dir = suite_root.join("demo-iconify-pack");
        write_demo_iconify_collection(&source_file);

        run_repo_icons_contract(
            IconsCommandArgs {
                command: IconsCommandContract::Import(IconImportCommandArgs {
                    source: IconImportSourceContract::IconifyCollection(
                        ImportIconifyCollectionArgs {
                            source: source_file,
                            common: ImportCommonArgs {
                                crate_name: "demo-iconify-pack".to_string(),
                                vendor_namespace: "demo".to_string(),
                                pack_id: None,
                                path: Some(out_dir.clone()),
                                source_label: Some("demo-iconify".to_string()),
                                semantic_aliases: None,
                                presentation_defaults: None,
                                no_check: true,
                            },
                        },
                    ),
                }),
            },
            &workspace_root,
        )
        .expect("repo iconify-collection import should succeed");

        cargo_check_generated_pack(&out_dir);

        let icon_list =
            std::fs::read_to_string(out_dir.join("icon-list.txt")).expect("generated icon list");
        assert!(icon_list.contains("search.svg"));
        assert!(icon_list.contains("search-rotated.svg"));

        let provenance = std::fs::read_to_string(out_dir.join("pack-provenance.json"))
            .expect("generated provenance json");
        let provenance: Value = serde_json::from_str(&provenance).expect("valid provenance json");
        assert_eq!(provenance["source"]["kind"], "iconify-collection");
        assert_eq!(
            provenance["icons"][1]["source_relative_path"],
            "aliases/search-rotated"
        );
    }

    #[test]
    fn repo_svg_dir_import_with_semantic_alias_config_generates_semantic_pack() {
        let workspace_root = repo_workspace_root();
        let suite_root = make_repo_local_dir("fretboard-icon-pack-alias-proof");
        let source_dir = suite_root.join("source");
        let out_dir = suite_root.join("demo-icons-pack");
        let alias_config = suite_root.join("semantic-aliases.json");
        std::fs::create_dir_all(&source_dir).expect("create source dir");
        write_demo_svgs(&source_dir);
        write_demo_semantic_alias_config(&alias_config);

        run_repo_icons_contract(
            IconsCommandArgs {
                command: IconsCommandContract::Import(IconImportCommandArgs {
                    source: IconImportSourceContract::SvgDir(ImportSvgDirArgs {
                        source: source_dir,
                        common: ImportCommonArgs {
                            crate_name: "demo-icons-pack".to_string(),
                            vendor_namespace: "demo".to_string(),
                            pack_id: None,
                            path: Some(out_dir.clone()),
                            source_label: Some("demo-source".to_string()),
                            semantic_aliases: Some(alias_config),
                            presentation_defaults: None,
                            no_check: true,
                        },
                    }),
                }),
            },
            &workspace_root,
        )
        .expect("repo svg-dir import with aliases should succeed");

        cargo_check_generated_pack(&out_dir);

        let cargo_toml =
            std::fs::read_to_string(out_dir.join("Cargo.toml")).expect("generated Cargo.toml");
        assert!(cargo_toml.contains("default = [\"semantic-ui\"]"));

        let lib_rs = std::fs::read_to_string(out_dir.join("src/lib.rs")).expect("generated lib.rs");
        assert!(lib_rs.contains("pub const UI_SEMANTIC_ALIAS_PACK"));
        assert!(lib_rs.contains("register_ui_semantic_aliases"));
        assert!(lib_rs.contains("IconId::new_static(\"ui.search\")"));

        let provenance = std::fs::read_to_string(out_dir.join("pack-provenance.json"))
            .expect("generated provenance json");
        let provenance: Value = serde_json::from_str(&provenance).expect("valid provenance json");
        assert_eq!(
            provenance["semantic_aliases"][0]["semantic_id"],
            "ui.search"
        );
        assert_eq!(
            provenance["semantic_aliases"][0]["target_icon"],
            "actions-search"
        );
    }

    #[test]
    fn repo_svg_dir_import_with_presentation_defaults_generates_explicit_render_modes() {
        let workspace_root = repo_workspace_root();
        let suite_root = make_repo_local_dir("fretboard-icon-pack-presentation-proof");
        let source_dir = suite_root.join("source");
        let out_dir = suite_root.join("demo-icons-pack");
        let presentation_config = suite_root.join("presentation-defaults.json");
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
        write_demo_presentation_defaults_config(&presentation_config);

        run_repo_icons_contract(
            IconsCommandArgs {
                command: IconsCommandContract::Import(IconImportCommandArgs {
                    source: IconImportSourceContract::SvgDir(ImportSvgDirArgs {
                        source: source_dir,
                        common: ImportCommonArgs {
                            crate_name: "demo-icons-pack".to_string(),
                            vendor_namespace: "demo".to_string(),
                            pack_id: None,
                            path: Some(out_dir.clone()),
                            source_label: Some("demo-source".to_string()),
                            semantic_aliases: None,
                            presentation_defaults: Some(presentation_config),
                            no_check: true,
                        },
                    }),
                }),
            },
            &workspace_root,
        )
        .expect("repo svg-dir import with presentation defaults should succeed");

        cargo_check_generated_pack(&out_dir);

        let lib_rs = std::fs::read_to_string(out_dir.join("src/lib.rs")).expect("generated lib.rs");
        assert!(lib_rs.contains("IconRenderMode::OriginalColors"));
        assert!(lib_rs.contains("IconRenderMode::Mask"));

        let provenance = std::fs::read_to_string(out_dir.join("pack-provenance.json"))
            .expect("generated provenance json");
        let provenance: Value = serde_json::from_str(&provenance).expect("valid provenance json");
        assert_eq!(
            provenance["presentation_defaults"]["default_render_mode"],
            "mask"
        );
        assert_eq!(
            provenance["presentation_defaults"]["icon_overrides"][0]["icon_name"],
            "brand-logo"
        );
    }

    #[test]
    fn repo_iconify_provenance_suggestion_flows_into_imported_pack_defaults() {
        let workspace_root = repo_workspace_root();
        let suite_root = make_repo_local_dir("fretboard-icon-presentation-suggestion-proof");
        let source_file = suite_root.join("demo.json");
        let provenance_file = suite_root.join("demo.provenance.json");
        let suggestion_file = suite_root.join("presentation-defaults.json");
        let out_dir = suite_root.join("demo-icons-pack");
        write_demo_iconify_collection(&source_file);
        write_demo_iconify_acquisition_provenance(&provenance_file, true);

        run_repo_icons_contract(
            IconsCommandArgs {
                command: IconsCommandContract::Suggest(IconSuggestCommandArgs {
                    kind: IconSuggestKindContract::PresentationDefaults(
                        SuggestPresentationDefaultsArgs {
                            provenance: provenance_file,
                            out: suggestion_file.clone(),
                            report_out: None,
                        },
                    ),
                }),
            },
            &workspace_root,
        )
        .expect("repo suggestion should succeed");

        run_repo_icons_contract(
            IconsCommandArgs {
                command: IconsCommandContract::Import(IconImportCommandArgs {
                    source: IconImportSourceContract::IconifyCollection(
                        ImportIconifyCollectionArgs {
                            source: source_file,
                            common: ImportCommonArgs {
                                crate_name: "demo-icons-pack".to_string(),
                                vendor_namespace: "demo".to_string(),
                                pack_id: None,
                                path: Some(out_dir.clone()),
                                source_label: Some("demo-iconify".to_string()),
                                semantic_aliases: None,
                                presentation_defaults: Some(suggestion_file),
                                no_check: true,
                            },
                        },
                    ),
                }),
            },
            &workspace_root,
        )
        .expect("repo iconify import with suggested presentation defaults should succeed");

        cargo_check_generated_pack(&out_dir);

        let provenance = std::fs::read_to_string(out_dir.join("pack-provenance.json"))
            .expect("generated provenance json");
        let provenance: Value = serde_json::from_str(&provenance).expect("valid provenance json");
        assert_eq!(
            provenance["presentation_defaults"]["default_render_mode"],
            "original-colors"
        );
        assert_eq!(provenance["icons"][0]["render_mode"], "original-colors");
    }

    #[test]
    fn repo_svg_dir_analysis_suggestion_flows_into_imported_pack_overrides() {
        let workspace_root = repo_workspace_root();
        let suite_root = make_repo_local_dir("fretboard-svg-analysis-presentation-proof");
        let source_dir = suite_root.join("source");
        let suggestion_file = suite_root.join("presentation-defaults.json");
        let report_file = suite_root.join("presentation-defaults.report.json");
        let out_dir = suite_root.join("demo-icons-pack");
        write_demo_svg_dir_for_analysis(&source_dir);

        run_repo_icons_contract(
            IconsCommandArgs {
                command: IconsCommandContract::Suggest(IconSuggestCommandArgs {
                    kind: IconSuggestKindContract::SvgDirPresentationOverrides(
                        SuggestSvgDirPresentationOverridesArgs {
                            source: source_dir.clone(),
                            out: suggestion_file.clone(),
                            report_out: Some(report_file.clone()),
                        },
                    ),
                }),
            },
            &workspace_root,
        )
        .expect("repo svg-dir analysis suggestion should succeed");

        run_repo_icons_contract(
            IconsCommandArgs {
                command: IconsCommandContract::Import(IconImportCommandArgs {
                    source: IconImportSourceContract::SvgDir(ImportSvgDirArgs {
                        source: source_dir,
                        common: ImportCommonArgs {
                            crate_name: "demo-icons-pack".to_string(),
                            vendor_namespace: "demo".to_string(),
                            pack_id: None,
                            path: Some(out_dir.clone()),
                            source_label: Some("demo-source".to_string()),
                            semantic_aliases: None,
                            presentation_defaults: Some(suggestion_file),
                            no_check: true,
                        },
                    }),
                }),
            },
            &workspace_root,
        )
        .expect("repo svg-dir import with suggested overrides should succeed");

        cargo_check_generated_pack(&out_dir);

        let provenance = std::fs::read_to_string(out_dir.join("pack-provenance.json"))
            .expect("generated provenance json");
        let provenance: Value = serde_json::from_str(&provenance).expect("valid provenance json");
        assert!(provenance["presentation_defaults"]["default_render_mode"].is_null());
        assert_eq!(
            provenance["presentation_defaults"]["icon_overrides"][0]["icon_name"],
            "brand-logo"
        );
        assert_eq!(
            provenance["presentation_defaults"]["icon_overrides"][0]["render_mode"],
            "original-colors"
        );
        assert_eq!(provenance["icons"][0]["icon_name"], "brand-logo");
        assert_eq!(provenance["icons"][0]["render_mode"], "original-colors");
        assert_eq!(provenance["icons"][1]["icon_name"], "close");
        assert_eq!(provenance["icons"][1]["render_mode"], "mask");

        let report = std::fs::read_to_string(report_file).expect("read suggestion report");
        let report: Value = serde_json::from_str(&report).expect("valid suggestion report");
        assert_eq!(report["summary"]["suggested_override_count"], 1);
        assert_eq!(report["summary"]["parse_failure_count"], 0);
    }
}
