use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[command(arg_required_else_help = true)]
pub(crate) struct AssetsCommandArgs {
    #[command(subcommand)]
    pub target: AssetsTargetContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub(crate) enum AssetsTargetContract {
    /// Generate a JSON assets manifest.
    Manifest(AssetsManifestCommandArgs),
    /// Generate a Rust assets module.
    Rust(AssetsRustCommandArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[command(arg_required_else_help = true)]
pub(crate) struct AssetsManifestCommandArgs {
    #[command(subcommand)]
    pub action: AssetsManifestActionContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub(crate) enum AssetsManifestActionContract {
    /// Scan an assets directory and write a manifest JSON file.
    Write(AssetsManifestWriteCommandArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[command(arg_required_else_help = true)]
pub(crate) struct AssetsRustCommandArgs {
    #[command(subcommand)]
    pub action: AssetsRustActionContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub(crate) enum AssetsRustActionContract {
    /// Scan an assets directory and generate a Rust module with embedded entries.
    Write(AssetsRustWriteCommandArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[group(id = "bundle_selector", required = true, multiple = false)]
pub(crate) struct AssetBundleSelectorArgs {
    /// Use a raw bundle id.
    #[arg(long, group = "bundle_selector")]
    pub bundle: Option<String>,
    /// Use an app bundle id (`app/<name>`).
    #[arg(long = "app-bundle", group = "bundle_selector")]
    pub app_bundle: Option<String>,
    /// Use a package bundle id (`pkg/<name>`).
    #[arg(long = "package-bundle", group = "bundle_selector")]
    pub package_bundle: Option<String>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub(crate) struct AssetsManifestWriteCommandArgs {
    /// Assets source directory to scan.
    #[arg(long)]
    pub dir: PathBuf,
    /// Output manifest JSON path.
    #[arg(long)]
    pub out: PathBuf,
    #[command(flatten)]
    pub bundle: AssetBundleSelectorArgs,
    /// Overwrite an existing output file.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum AssetsRustSurfaceContract {
    Fret,
    Framework,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub(crate) struct AssetsRustWriteCommandArgs {
    /// Assets source directory to scan.
    #[arg(long)]
    pub dir: PathBuf,
    /// Output Rust module path.
    #[arg(long)]
    pub out: PathBuf,
    /// Crate root used to relativize embedded asset paths.
    #[arg(long = "crate-root")]
    pub crate_root: Option<PathBuf>,
    /// Output authoring surface.
    #[arg(long, value_enum, default_value_t = AssetsRustSurfaceContract::Fret)]
    pub surface: AssetsRustSurfaceContract,
    #[command(flatten)]
    pub bundle: AssetBundleSelectorArgs,
    /// Overwrite an existing output file.
    #[arg(long)]
    pub force: bool,
}
