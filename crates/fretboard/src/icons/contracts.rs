use std::path::PathBuf;

use clap::{ArgAction, Args, Subcommand};

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct IconsCommandArgs {
    #[command(subcommand)]
    pub command: IconsCommandContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum IconsCommandContract {
    /// Acquire pinned local icon artifacts from remote upstreams.
    Acquire(IconAcquireCommandArgs),
    /// Generate icon-pack crates from local icon sources.
    Import(IconImportCommandArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct IconAcquireCommandArgs {
    #[command(subcommand)]
    pub source: IconAcquireSourceContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum IconAcquireSourceContract {
    /// Acquire a local Iconify collection snapshot JSON file from the Iconify API.
    IconifyCollection(AcquireIconifyCollectionArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct AcquireIconifyCollectionArgs {
    /// Iconify collection prefix, for example `mdi` or `lucide`.
    #[arg(long, value_name = "PREFIX")]
    pub collection: String,
    /// Icon name to include in the emitted snapshot. Repeat this flag for multiple icons.
    #[arg(long = "icon", value_name = "NAME", action = ArgAction::Append)]
    pub icons: Vec<String>,
    /// Output path for the emitted local Iconify collection snapshot JSON file.
    #[arg(long, value_name = "FILE")]
    pub out: PathBuf,
    /// Optional output path for acquisition provenance metadata JSON.
    ///
    /// Defaults to a sibling `<snapshot>.provenance.json` file next to `--out`.
    #[arg(long, value_name = "FILE")]
    pub provenance_out: Option<PathBuf>,
    /// Iconify API base URL.
    #[arg(long, value_name = "URL", default_value = "https://api.iconify.design")]
    pub api_base_url: String,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct IconImportCommandArgs {
    #[command(subcommand)]
    pub source: IconImportSourceContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum IconImportSourceContract {
    /// Generate a pack crate from a local SVG directory.
    SvgDir(ImportSvgDirArgs),
    /// Generate a pack crate from a local Iconify collection snapshot JSON file.
    IconifyCollection(ImportIconifyCollectionArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct ImportCommonArgs {
    /// Generated crate/package name.
    #[arg(long, value_name = "NAME")]
    pub crate_name: String,
    /// Vendor namespace for generated icon IDs, for example `mdi` or `custom`.
    #[arg(long, value_name = "NAMESPACE")]
    pub vendor_namespace: String,
    /// Optional pack id recorded in `PACK_METADATA` (defaults to the crate name).
    #[arg(long, value_name = "PACK_ID")]
    pub pack_id: Option<String>,
    /// Output directory for the generated pack crate.
    #[arg(long, value_name = "DIR")]
    pub path: Option<PathBuf>,
    /// Stable source label recorded in `README.md` and `pack-provenance.json`.
    #[arg(long, value_name = "LABEL")]
    pub source_label: Option<String>,
    /// JSON file describing explicit semantic `ui.*` alias mappings for the generated pack.
    ///
    /// Expected shape:
    /// `{ "schema_version": 1, "semantic_aliases": [{ "semantic_id": "ui.search", "target_icon": "search" }] }`
    #[arg(long, value_name = "FILE")]
    pub semantic_aliases: Option<PathBuf>,
    /// JSON file describing explicit presentation defaults for generated icons.
    ///
    /// Expected shape:
    /// `{ "schema_version": 1, "default_render_mode": "mask", "icon_overrides": [{ "icon_name": "brand-logo", "render_mode": "original-colors" }] }`
    #[arg(long, value_name = "FILE")]
    pub presentation_defaults: Option<PathBuf>,
    /// Skip `cargo check --features app-integration` after generation.
    #[arg(long)]
    pub no_check: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct ImportSvgDirArgs {
    /// Source directory containing one or more SVG files.
    #[arg(long, value_name = "DIR")]
    pub source: PathBuf,
    #[command(flatten)]
    pub common: ImportCommonArgs,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct ImportIconifyCollectionArgs {
    /// Local Iconify collection snapshot JSON file.
    #[arg(long, value_name = "FILE")]
    pub source: PathBuf,
    #[command(flatten)]
    pub common: ImportCommonArgs,
}
